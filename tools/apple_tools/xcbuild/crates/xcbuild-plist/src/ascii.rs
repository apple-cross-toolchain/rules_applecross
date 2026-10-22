//! ASCII/OpenStep plist format parser and writer.
//!
//! Ported from Libraries/plist/Sources/Format/ASCIIParser.cpp and ASCIIWriter.cpp.

use plist::Value;
use std::fmt;

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {}",
            self.line, self.column, self.message
        )
    }
}

impl std::error::Error for ParseError {}

struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Lexer {
            input: input.as_bytes(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let c = self.input.get(self.pos).copied()?;
        self.pos += 1;
        if c == b'\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(c)
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(c) if c.is_ascii_whitespace() => {
                    self.advance();
                }
                Some(b'/') => {
                    if self.pos + 1 < self.input.len() {
                        match self.input[self.pos + 1] {
                            b'/' => {
                                // Line comment
                                self.advance();
                                self.advance();
                                while let Some(c) = self.peek() {
                                    if c == b'\n' {
                                        break;
                                    }
                                    self.advance();
                                }
                            }
                            b'*' => {
                                // Block comment
                                self.advance();
                                self.advance();
                                loop {
                                    match self.advance() {
                                        Some(b'*') => {
                                            if self.peek() == Some(b'/') {
                                                self.advance();
                                                break;
                                            }
                                        }
                                        None => break,
                                        _ => {}
                                    }
                                }
                            }
                            _ => break,
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn error(&self, message: &str) -> ParseError {
        ParseError {
            message: message.to_string(),
            line: self.line,
            column: self.column,
        }
    }
}

/// Parse an ASCII/OpenStep plist string into a plist Value.
pub fn parse(input: &str) -> Result<Value, ParseError> {
    let mut lexer = Lexer::new(input);
    lexer.skip_whitespace_and_comments();
    let value = parse_value(&mut lexer)?;
    Ok(value)
}

fn parse_value(lexer: &mut Lexer) -> Result<Value, ParseError> {
    lexer.skip_whitespace_and_comments();

    match lexer.peek() {
        Some(b'{') => parse_dictionary(lexer),
        Some(b'(') => parse_array(lexer),
        Some(b'"') => parse_quoted_string(lexer).map(Value::String),
        Some(b'<') => parse_data(lexer),
        Some(c) if is_unquoted_char(c) => parse_unquoted_string(lexer).map(Value::String),
        Some(c) => Err(lexer.error(&format!("unexpected character '{}'", c as char))),
        None => Err(lexer.error("unexpected end of input")),
    }
}

fn parse_dictionary(lexer: &mut Lexer) -> Result<Value, ParseError> {
    assert_eq!(lexer.advance(), Some(b'{'));

    let mut dict = plist::Dictionary::new();

    loop {
        lexer.skip_whitespace_and_comments();

        match lexer.peek() {
            Some(b'}') => {
                lexer.advance();
                return Ok(Value::Dictionary(dict));
            }
            None => return Err(lexer.error("unexpected end of dictionary")),
            _ => {}
        }

        // Parse key
        let key = parse_string_value(lexer)?;

        lexer.skip_whitespace_and_comments();

        // Expect = or ;
        match lexer.peek() {
            Some(b'=') => {
                lexer.advance();
                lexer.skip_whitespace_and_comments();
                let value = parse_value(lexer)?;
                dict.insert(key, value);

                lexer.skip_whitespace_and_comments();
                if lexer.peek() == Some(b';') {
                    lexer.advance();
                }
            }
            Some(b';') => {
                // Key without value (shorthand for key = key)
                dict.insert(key.clone(), Value::String(key));
                lexer.advance();
            }
            _ => {
                return Err(lexer.error("expected '=' or ';' in dictionary"));
            }
        }
    }
}

fn parse_array(lexer: &mut Lexer) -> Result<Value, ParseError> {
    assert_eq!(lexer.advance(), Some(b'('));

    let mut arr = Vec::new();

    loop {
        lexer.skip_whitespace_and_comments();

        match lexer.peek() {
            Some(b')') => {
                lexer.advance();
                return Ok(Value::Array(arr));
            }
            None => return Err(lexer.error("unexpected end of array")),
            _ => {}
        }

        let value = parse_value(lexer)?;
        arr.push(value);

        lexer.skip_whitespace_and_comments();
        if lexer.peek() == Some(b',') {
            lexer.advance();
        }
    }
}

fn parse_string_value(lexer: &mut Lexer) -> Result<String, ParseError> {
    lexer.skip_whitespace_and_comments();
    match lexer.peek() {
        Some(b'"') => parse_quoted_string(lexer),
        Some(c) if is_unquoted_char(c) => parse_unquoted_string(lexer),
        _ => Err(lexer.error("expected string")),
    }
}

fn parse_quoted_string(lexer: &mut Lexer) -> Result<String, ParseError> {
    assert_eq!(lexer.advance(), Some(b'"'));

    let mut s = String::new();

    loop {
        match lexer.advance() {
            Some(b'"') => return Ok(s),
            Some(b'\\') => {
                match lexer.advance() {
                    Some(b'n') => s.push('\n'),
                    Some(b'r') => s.push('\r'),
                    Some(b't') => s.push('\t'),
                    Some(b'\\') => s.push('\\'),
                    Some(b'"') => s.push('"'),
                    Some(b'a') => s.push('\x07'),
                    Some(b'b') => s.push('\x08'),
                    Some(b'f') => s.push('\x0C'),
                    Some(b'v') => s.push('\x0B'),
                    Some(b'0') => s.push('\0'),
                    Some(b'U') | Some(b'u') => {
                        // Unicode escape: \Uxxxx
                        let mut hex = String::new();
                        for _ in 0..4 {
                            match lexer.advance() {
                                Some(c) if (c as char).is_ascii_hexdigit() => {
                                    hex.push(c as char);
                                }
                                _ => return Err(lexer.error("invalid unicode escape")),
                            }
                        }
                        if let Ok(code) = u32::from_str_radix(&hex, 16) {
                            if let Some(ch) = char::from_u32(code) {
                                s.push(ch);
                            }
                        }
                    }
                    Some(c) if (c as char).is_ascii_digit() => {
                        // Octal escape
                        let mut oct = String::new();
                        oct.push(c as char);
                        for _ in 0..2 {
                            if let Some(c) = lexer.peek() {
                                if (c as char).is_ascii_digit() && c < b'8' {
                                    oct.push(c as char);
                                    lexer.advance();
                                } else {
                                    break;
                                }
                            }
                        }
                        if let Ok(code) = u32::from_str_radix(&oct, 8) {
                            if let Some(ch) = char::from_u32(code) {
                                s.push(ch);
                            }
                        }
                    }
                    Some(c) => s.push(c as char),
                    None => return Err(lexer.error("unexpected end of string escape")),
                }
            }
            Some(c) => s.push(c as char),
            None => return Err(lexer.error("unterminated string")),
        }
    }
}

fn parse_unquoted_string(lexer: &mut Lexer) -> Result<String, ParseError> {
    let mut s = String::new();
    while let Some(c) = lexer.peek() {
        if is_unquoted_char(c) {
            s.push(c as char);
            lexer.advance();
        } else {
            break;
        }
    }
    if s.is_empty() {
        return Err(lexer.error("expected unquoted string"));
    }
    Ok(s)
}

fn parse_data(lexer: &mut Lexer) -> Result<Value, ParseError> {
    assert_eq!(lexer.advance(), Some(b'<'));

    let mut hex = String::new();
    loop {
        lexer.skip_whitespace_and_comments();
        match lexer.peek() {
            Some(b'>') => {
                lexer.advance();
                break;
            }
            Some(c) if (c as char).is_ascii_hexdigit() => {
                hex.push(c as char);
                lexer.advance();
            }
            Some(c) => return Err(lexer.error(&format!("invalid hex character '{}'", c as char))),
            None => return Err(lexer.error("unterminated data")),
        }
    }

    // Pad with trailing 0 if odd length
    if hex.len() % 2 != 0 {
        hex.push('0');
    }

    let mut bytes = Vec::new();
    for i in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[i..i + 2], 16)
            .map_err(|_| lexer.error("invalid hex data"))?;
        bytes.push(byte);
    }

    Ok(Value::Data(bytes))
}

fn is_unquoted_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'$' || c == b'/' || c == b':' || c == b'.' || c == b'-'
}

// --- ASCII Writer ---

/// Write a plist Value in ASCII/OpenStep format.
pub fn write(value: &Value) -> String {
    let mut output = String::new();
    write_value(&mut output, value, 0);
    output.push('\n');
    output
}

fn write_value(output: &mut String, value: &Value, indent: usize) {
    match value {
        Value::Dictionary(dict) => write_dictionary(output, dict, indent),
        Value::Array(arr) => write_array(output, arr, indent),
        Value::String(s) => write_string(output, s),
        Value::Integer(i) => {
            if let Some(n) = i.as_signed() {
                output.push_str(&n.to_string());
            } else if let Some(n) = i.as_unsigned() {
                output.push_str(&n.to_string());
            }
        }
        Value::Real(f) => output.push_str(&f.to_string()),
        Value::Boolean(b) => output.push_str(if *b { "1" } else { "0" }),
        Value::Data(d) => write_data(output, d),
        Value::Date(d) => write_string(output, &d.to_xml_format()),
        Value::Uid(u) => output.push_str(&u.get().to_string()),
        _ => output.push_str("\"\""),
    }
}

fn write_dictionary(output: &mut String, dict: &plist::Dictionary, indent: usize) {
    output.push_str("{\n");
    let child_indent = indent + 1;
    for (key, value) in dict.iter() {
        for _ in 0..child_indent {
            output.push_str("\t");
        }
        write_string(output, key);
        output.push_str(" = ");
        write_value(output, value, child_indent);
        output.push_str(";\n");
    }
    for _ in 0..indent {
        output.push_str("\t");
    }
    output.push('}');
}

fn write_array(output: &mut String, arr: &[Value], indent: usize) {
    output.push_str("(\n");
    let child_indent = indent + 1;
    for (i, value) in arr.iter().enumerate() {
        for _ in 0..child_indent {
            output.push_str("\t");
        }
        write_value(output, value, child_indent);
        if i + 1 < arr.len() {
            output.push(',');
        }
        output.push('\n');
    }
    for _ in 0..indent {
        output.push_str("\t");
    }
    output.push(')');
}

fn write_string(output: &mut String, s: &str) {
    if !s.is_empty() && s.bytes().all(is_unquoted_char) {
        output.push_str(s);
    } else {
        output.push('"');
        for c in s.chars() {
            match c {
                '"' => output.push_str("\\\""),
                '\\' => output.push_str("\\\\"),
                '\n' => output.push_str("\\n"),
                '\r' => output.push_str("\\r"),
                '\t' => output.push_str("\\t"),
                '\x07' => output.push_str("\\a"),
                '\x08' => output.push_str("\\b"),
                '\x0C' => output.push_str("\\f"),
                '\x0B' => output.push_str("\\v"),
                _ => output.push(c),
            }
        }
        output.push('"');
    }
}

fn write_data(output: &mut String, data: &[u8]) {
    output.push('<');
    for (i, byte) in data.iter().enumerate() {
        output.push_str(&format!("{:02x}", byte));
        if i + 1 < data.len() && (i + 1) % 4 == 0 {
            output.push(' ');
        }
    }
    output.push('>');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_dict() {
        let input = r#"{ key = value; "quoted key" = "quoted value"; }"#;
        let result = parse(input).unwrap();
        if let Value::Dictionary(d) = result {
            assert_eq!(d.get("key"), Some(&Value::String("value".to_string())));
            assert_eq!(
                d.get("quoted key"),
                Some(&Value::String("quoted value".to_string()))
            );
        } else {
            panic!("expected dictionary");
        }
    }

    #[test]
    fn test_parse_array() {
        let input = r#"(one, two, three)"#;
        let result = parse(input).unwrap();
        if let Value::Array(a) = result {
            assert_eq!(a.len(), 3);
            assert_eq!(a[0], Value::String("one".to_string()));
        } else {
            panic!("expected array");
        }
    }

    #[test]
    fn test_parse_nested() {
        let input = r#"{
            key = {
                nested = value;
            };
            arr = (1, 2, 3);
        }"#;
        let result = parse(input).unwrap();
        if let Value::Dictionary(d) = result {
            assert!(d.get("key").is_some());
            assert!(d.get("arr").is_some());
        } else {
            panic!("expected dictionary");
        }
    }

    #[test]
    fn test_parse_comments() {
        let input = r#"{
            // line comment
            key = value; /* block comment */
        }"#;
        let result = parse(input).unwrap();
        if let Value::Dictionary(d) = result {
            assert_eq!(d.get("key"), Some(&Value::String("value".to_string())));
        } else {
            panic!("expected dictionary");
        }
    }

    #[test]
    fn test_parse_data() {
        let input = r#"{ data = <0123 4567>; }"#;
        let result = parse(input).unwrap();
        if let Value::Dictionary(d) = result {
            if let Some(Value::Data(data)) = d.get("data") {
                assert_eq!(data, &[0x01, 0x23, 0x45, 0x67]);
            } else {
                panic!("expected data");
            }
        } else {
            panic!("expected dictionary");
        }
    }

    #[test]
    fn test_parse_escape_sequences() {
        let input = r#"{ key = "hello\nworld\t\"quoted\""; }"#;
        let result = parse(input).unwrap();
        if let Value::Dictionary(d) = result {
            assert_eq!(
                d.get("key"),
                Some(&Value::String("hello\nworld\t\"quoted\"".to_string()))
            );
        } else {
            panic!("expected dictionary");
        }
    }

    #[test]
    fn test_write_roundtrip() {
        let mut dict = plist::Dictionary::new();
        dict.insert("key".to_string(), Value::String("value".to_string()));
        dict.insert(
            "nested".to_string(),
            Value::Dictionary({
                let mut d = plist::Dictionary::new();
                d.insert("inner".to_string(), Value::String("data".to_string()));
                d
            }),
        );
        let value = Value::Dictionary(dict);

        let written = write(&value);
        let parsed = parse(&written).unwrap();

        if let Value::Dictionary(d) = parsed {
            assert_eq!(d.get("key"), Some(&Value::String("value".to_string())));
        } else {
            panic!("expected dictionary");
        }
    }
}
