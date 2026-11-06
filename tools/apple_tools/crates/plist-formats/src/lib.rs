pub mod ascii;

use plist::Value;
use std::io::Cursor;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlistError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("plist error: {0}")]
    Plist(#[from] plist::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("ASCII parse error: {0}")]
    AsciiParse(String),
    #[error("unknown format")]
    UnknownFormat,
    #[error("invalid key path: {0}")]
    InvalidKeyPath(String),
    #[error("unsupported type: {0}")]
    UnsupportedType(String),
}

/// Plist formats supported by this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlistFormat {
    Xml,
    Binary,
    Ascii,
    Json,
    Raw,
    Swift,
    Objc,
}

impl PlistFormat {
    pub fn name(&self) -> &'static str {
        match self {
            PlistFormat::Xml => "xml1",
            PlistFormat::Binary => "binary1",
            PlistFormat::Ascii => "openstep1",
            PlistFormat::Json => "json",
            PlistFormat::Raw => "raw",
            PlistFormat::Swift => "swift",
            PlistFormat::Objc => "objc",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "xml1" => Some(PlistFormat::Xml),
            "binary1" => Some(PlistFormat::Binary),
            "openstep1" | "ascii1" => Some(PlistFormat::Ascii),
            "json" => Some(PlistFormat::Json),
            "raw" => Some(PlistFormat::Raw),
            "swift" => Some(PlistFormat::Swift),
            "objc" => Some(PlistFormat::Objc),
            _ => None,
        }
    }
}

/// Object type names (matching Apple's PlistBuddy types).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    String,
    Dictionary,
    Array,
    Boolean,
    Real,
    Integer,
    Date,
    Data,
}

impl ObjectType {
    pub fn parse(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "string" => Some(ObjectType::String),
            "dictionary" | "dict" => Some(ObjectType::Dictionary),
            "array" => Some(ObjectType::Array),
            "bool" | "boolean" => Some(ObjectType::Boolean),
            "real" | "float" => Some(ObjectType::Real),
            "integer" | "int" => Some(ObjectType::Integer),
            "date" => Some(ObjectType::Date),
            "data" => Some(ObjectType::Data),
            _ => None,
        }
    }
}

/// Detect the format of a plist file from its contents.
pub fn identify_format(data: &[u8]) -> Option<PlistFormat> {
    if data.len() >= 8 && &data[0..8] == b"bplist00" {
        return Some(PlistFormat::Binary);
    }

    let trimmed = skip_whitespace_and_bom(data);

    if trimmed.starts_with(b"<?xml")
        || trimmed.starts_with(b"<plist")
        || trimmed.starts_with(b"<!DOCTYPE")
    {
        return Some(PlistFormat::Xml);
    }

    // JSON starts with { or [ and uses : for key-value separation
    if !trimmed.is_empty() && (trimmed[0] == b'{' || trimmed[0] == b'[') {
        return Some(PlistFormat::Json);
    }

    // ASCII/OpenStep format
    if !trimmed.is_empty() {
        let first = trimmed[0];
        if first == b'"' || first.is_ascii_alphanumeric() || first == b'(' {
            return Some(PlistFormat::Ascii);
        }
    }

    None
}

fn skip_whitespace_and_bom(data: &[u8]) -> &[u8] {
    let mut start = 0;
    if data.len() >= 3 && data[0] == 0xEF && data[1] == 0xBB && data[2] == 0xBF {
        start = 3;
    }
    while start < data.len() && data[start].is_ascii_whitespace() {
        start += 1;
    }
    &data[start..]
}

/// Deserialize plist data in any supported format.
pub fn deserialize(data: &[u8]) -> Result<(Value, PlistFormat), PlistError> {
    let format = identify_format(data).ok_or(PlistError::UnknownFormat)?;
    let value = deserialize_with_format(data, format)?;
    Ok((value, format))
}

/// Deserialize plist data with a known format.
pub fn deserialize_with_format(data: &[u8], format: PlistFormat) -> Result<Value, PlistError> {
    match format {
        PlistFormat::Xml | PlistFormat::Binary => {
            let value = Value::from_reader(Cursor::new(data))?;
            Ok(value)
        }
        PlistFormat::Ascii => {
            let text =
                std::str::from_utf8(data).map_err(|e| PlistError::AsciiParse(e.to_string()))?;
            ascii::parse(text).map_err(|e| PlistError::AsciiParse(e.to_string()))
        }
        PlistFormat::Json => {
            let json_value: serde_json::Value = serde_json::from_slice(data)?;
            Ok(json_to_plist(json_value))
        }
        PlistFormat::Raw | PlistFormat::Swift | PlistFormat::Objc => Err(PlistError::UnknownFormat),
    }
}

/// Serialize a plist value to bytes in the given format.
pub fn serialize(value: &Value, format: PlistFormat) -> Result<Vec<u8>, PlistError> {
    match format {
        PlistFormat::Xml => {
            let mut buf = Vec::new();
            value.to_writer_xml(&mut buf)?;
            Ok(buf)
        }
        PlistFormat::Binary => {
            let mut buf = Vec::new();
            value.to_writer_binary(&mut buf)?;
            Ok(buf)
        }
        PlistFormat::Ascii => Ok(ascii::write(value).into_bytes()),
        PlistFormat::Json => {
            let json_value = plist_to_json(value);
            let buf = serde_json::to_vec_pretty(&json_value)?;
            Ok(buf)
        }
        PlistFormat::Raw => Ok(serialize_raw(value)),
        PlistFormat::Swift => Ok(serialize_swift(value, true)),
        PlistFormat::Objc => Ok(serialize_objc(value, true)),
    }
}

/// Serialize a plist value to JSON with sorted dictionary keys (for -r flag).
pub fn serialize_json_sorted(value: &Value) -> Result<Vec<u8>, PlistError> {
    let json_value = plist_to_json_sorted(value);
    let buf = serde_json::to_vec_pretty(&json_value)?;
    Ok(buf)
}

fn serialize_raw(value: &Value) -> Vec<u8> {
    let s = match value {
        Value::Boolean(b) => {
            if *b {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Value::Integer(i) => {
            if let Some(n) = i.as_signed() {
                n.to_string()
            } else if let Some(n) = i.as_unsigned() {
                n.to_string()
            } else {
                "0".to_string()
            }
        }
        Value::Real(f) => f.to_string(),
        Value::String(s) => s.clone(),
        Value::Date(d) => d.to_xml_format(),
        Value::Data(d) => base64_encode(d),
        Value::Array(arr) => arr.len().to_string(),
        Value::Dictionary(dict) => {
            let mut keys: Vec<&str> = dict.keys().map(|k| k.as_str()).collect();
            keys.sort();
            let mut result = String::new();
            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    result.push('\n');
                }
                result.push_str(key);
            }
            result
        }
        _ => String::new(),
    };
    let mut bytes = s.into_bytes();
    bytes.push(b'\n');
    bytes
}

fn format_swift_literal(value: &Value, indent: usize, pretty: bool) -> String {
    match value {
        Value::Dictionary(dictionary) => {
            if dictionary.is_empty() {
                return "[:]".to_string();
            }
            let mut out = String::new();
            out.push('[');
            if pretty {
                out.push('\n');
            }
            for (i, (key, val)) in dictionary.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                    if pretty {
                        out.push('\n');
                    }
                }
                if pretty {
                    out.push_str(&" ".repeat(indent + 2));
                }
                out.push_str(&format!(
                    "\"{}\": {}",
                    key.replace('\\', "\\\\").replace('"', "\\\""),
                    format_swift_literal(val, indent + 2, pretty)
                ));
            }
            if pretty {
                out.push('\n');
                out.push_str(&" ".repeat(indent));
            }
            out.push(']');
            out
        }
        Value::Array(array) => {
            let mut out = String::new();
            out.push('[');
            if pretty && !array.is_empty() {
                out.push('\n');
            }
            for (i, val) in array.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                    if pretty {
                        out.push('\n');
                    }
                }
                if pretty {
                    out.push_str(&" ".repeat(indent + 2));
                }
                out.push_str(&format_swift_literal(val, indent + 2, pretty));
            }
            if pretty && !array.is_empty() {
                out.push('\n');
                out.push_str(&" ".repeat(indent));
            }
            out.push(']');
            out
        }
        Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        Value::Boolean(b) => if *b { "true" } else { "false" }.to_string(),
        Value::Integer(i) => {
            if let Some(n) = i.as_signed() {
                n.to_string()
            } else if let Some(n) = i.as_unsigned() {
                n.to_string()
            } else {
                "0".to_string()
            }
        }
        Value::Real(f) => f.to_string(),
        Value::Data(d) => format!("Data(base64Encoded: \"{}\")!", base64_encode(d)),
        Value::Date(d) => format!("\"{}\"", d.to_xml_format()),
        Value::Uid(u) => u.get().to_string(),
        _ => "nil".to_string(),
    }
}

fn format_objc_literal(value: &Value, indent: usize, pretty: bool) -> String {
    match value {
        Value::Dictionary(dictionary) => {
            if dictionary.is_empty() {
                return "@{}".to_string();
            }
            let mut out = String::new();
            out.push_str("@{");
            if pretty {
                out.push('\n');
            }
            for (i, (key, val)) in dictionary.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                    if pretty {
                        out.push('\n');
                    }
                }
                if pretty {
                    out.push_str(&" ".repeat(indent + 2));
                }
                out.push_str(&format!(
                    "@\"{}\": {}",
                    key.replace('\\', "\\\\").replace('"', "\\\""),
                    format_objc_literal(val, indent + 2, pretty)
                ));
            }
            if pretty {
                out.push('\n');
                out.push_str(&" ".repeat(indent));
            }
            out.push('}');
            out
        }
        Value::Array(array) => {
            let mut out = String::new();
            out.push_str("@[");
            if pretty && !array.is_empty() {
                out.push('\n');
            }
            for (i, val) in array.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                    if pretty {
                        out.push('\n');
                    }
                }
                if pretty {
                    out.push_str(&" ".repeat(indent + 2));
                }
                out.push_str(&format_objc_literal(val, indent + 2, pretty));
            }
            if pretty && !array.is_empty() {
                out.push('\n');
                out.push_str(&" ".repeat(indent));
            }
            out.push(']');
            out
        }
        Value::String(s) => format!("@\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        Value::Boolean(b) => if *b { "@YES" } else { "@NO" }.to_string(),
        Value::Integer(i) => {
            if let Some(n) = i.as_signed() {
                format!("@({n})")
            } else if let Some(n) = i.as_unsigned() {
                format!("@({n})")
            } else {
                "@(0)".to_string()
            }
        }
        Value::Real(f) => format!("@({f})"),
        Value::Data(d) => format!(
            "[NSData dataWithBase64EncodedString:@\"{}\" options:0]",
            base64_encode(d)
        ),
        Value::Date(d) => format!("@\"{}\"", d.to_xml_format()),
        Value::Uid(u) => format!("@({})", u.get()),
        _ => "[NSNull null]".to_string(),
    }
}

fn serialize_swift(value: &Value, pretty: bool) -> Vec<u8> {
    let mut s = format_swift_literal(value, 0, pretty);
    s.push('\n');
    s.into_bytes()
}

fn serialize_objc(value: &Value, pretty: bool) -> Vec<u8> {
    let mut s = format_objc_literal(value, 0, pretty);
    s.push('\n');
    s.into_bytes()
}

fn plist_to_json_sorted(value: &Value) -> serde_json::Value {
    match value {
        Value::Boolean(b) => serde_json::Value::Bool(*b),
        Value::Integer(i) => {
            if let Some(n) = i.as_signed() {
                serde_json::Value::Number(serde_json::Number::from(n))
            } else if let Some(n) = i.as_unsigned() {
                serde_json::Value::Number(serde_json::Number::from(n))
            } else {
                serde_json::Value::Null
            }
        }
        Value::Real(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::Data(d) => serde_json::Value::String(base64_encode(d)),
        Value::Date(d) => serde_json::Value::String(d.to_xml_format()),
        Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(plist_to_json_sorted).collect())
        }
        Value::Dictionary(dict) => {
            // Use BTreeMap for sorted keys
            let mut map = serde_json::Map::new();
            let mut keys: Vec<&str> = dict.keys().map(|k| k.as_str()).collect();
            keys.sort();
            for k in keys {
                if let Some(v) = dict.get(k) {
                    map.insert(k.to_string(), plist_to_json_sorted(v));
                }
            }
            serde_json::Value::Object(map)
        }
        Value::Uid(_) => serde_json::Value::Null,
        _ => serde_json::Value::Null,
    }
}

fn json_to_plist(json: serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::String(String::new()),
        serde_json::Value::Bool(b) => Value::Boolean(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Integer(i.into())
            } else if let Some(f) = n.as_f64() {
                Value::Real(f)
            } else {
                Value::String(n.to_string())
            }
        }
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => Value::Array(arr.into_iter().map(json_to_plist).collect()),
        serde_json::Value::Object(obj) => {
            let mut dict = plist::Dictionary::new();
            for (k, v) in obj {
                dict.insert(k, json_to_plist(v));
            }
            Value::Dictionary(dict)
        }
    }
}

fn plist_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Boolean(b) => serde_json::Value::Bool(*b),
        Value::Integer(i) => {
            if let Some(n) = i.as_signed() {
                serde_json::Value::Number(serde_json::Number::from(n))
            } else if let Some(n) = i.as_unsigned() {
                serde_json::Value::Number(serde_json::Number::from(n))
            } else {
                serde_json::Value::Null
            }
        }
        Value::Real(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::Data(d) => serde_json::Value::String(base64_encode(d)),
        Value::Date(d) => serde_json::Value::String(d.to_xml_format()),
        Value::Array(arr) => serde_json::Value::Array(arr.iter().map(plist_to_json).collect()),
        Value::Dictionary(dict) => {
            let mut map = serde_json::Map::new();
            for (k, v) in dict.iter() {
                map.insert(k.clone(), plist_to_json(v));
            }
            serde_json::Value::Object(map)
        }
        Value::Uid(_) => serde_json::Value::Null,
        _ => serde_json::Value::Null,
    }
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

// --- Key path navigation for PlistBuddy ---

/// Parse a PlistBuddy-style key path (`:key:subkey:0:` syntax).
pub fn parse_key_path(path: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut prev = 0;

    loop {
        let pos = path[prev..].find(':');
        match pos {
            Some(p) => {
                let end = prev + p;
                if !(prev == 0 && end == 0) {
                    keys.push(path[prev..end].to_string());
                }
                prev = end + 1;
            }
            None => {
                if prev < path.len() {
                    keys.push(path[prev..].to_string());
                }
                break;
            }
        }
    }

    keys
}

/// Navigate to a value at a key path, returning a reference.
pub fn get_at_key_path<'a>(value: &'a Value, keys: &[String]) -> Option<&'a Value> {
    let mut current = value;
    for key in keys {
        match current {
            Value::Dictionary(dict) => {
                current = dict.get(key.as_str())?;
            }
            Value::Array(arr) => {
                let idx: usize = key.parse().ok()?;
                current = arr.get(idx)?;
            }
            _ => return None,
        }
    }
    Some(current)
}

/// Navigate to a value at a key path, returning a mutable reference.
pub fn get_at_key_path_mut<'a>(value: &'a mut Value, keys: &[String]) -> Option<&'a mut Value> {
    let mut current = value;
    for key in keys {
        current = match current {
            Value::Dictionary(dict) => dict.get_mut(key.as_str())?,
            Value::Array(arr) => {
                let idx: usize = key.parse().ok()?;
                arr.get_mut(idx)?
            }
            _ => return None,
        };
    }
    Some(current)
}

/// Parse a boolean string value. Case-insensitive, accepts YES/NO, true/false, 1/0.
pub fn parse_bool(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "yes" | "true" | "1" => Some(true),
        "no" | "false" | "0" => Some(false),
        _ => None,
    }
}

/// Decode a base64-encoded string into bytes.
pub fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    const DECODE: [u8; 256] = {
        let mut table = [0xFFu8; 256];
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut i = 0;
        while i < 64 {
            table[chars[i] as usize] = i as u8;
            i += 1;
        }
        table
    };

    let input: Vec<u8> = input.bytes().filter(|&b| !b.is_ascii_whitespace()).collect();
    let mut result = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf = 0u32;
    let mut bits = 0u32;

    for &byte in &input {
        if byte == b'=' {
            break;
        }
        let val = DECODE[byte as usize];
        if val == 0xFF {
            return Err(format!("invalid base64 character: {}", byte as char));
        }
        buf = (buf << 6) | val as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            result.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }

    Ok(result)
}

/// Create a new plist Value from a type and string value.
pub fn create_value(obj_type: ObjectType, value_str: &str) -> Option<Value> {
    match obj_type {
        ObjectType::String => Some(Value::String(value_str.to_string())),
        ObjectType::Dictionary => Some(Value::Dictionary(plist::Dictionary::new())),
        ObjectType::Array => Some(Value::Array(Vec::new())),
        ObjectType::Boolean => parse_bool(value_str).map(Value::Boolean),
        ObjectType::Real => value_str.parse::<f64>().ok().map(Value::Real),
        ObjectType::Integer => value_str.parse::<i64>().ok().map(|i| Value::Integer(i.into())),
        ObjectType::Date => {
            plist::Date::from_xml_format(value_str)
                .ok()
                .map(Value::Date)
        }
        ObjectType::Data => {
            base64_decode(value_str).ok().map(Value::Data)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identify_xml() {
        let data = b"<?xml version=\"1.0\"?><plist><dict/></plist>";
        assert_eq!(identify_format(data), Some(PlistFormat::Xml));
    }

    #[test]
    fn test_identify_binary() {
        let mut data = vec![0u8; 32];
        data[..8].copy_from_slice(b"bplist00");
        assert_eq!(identify_format(&data), Some(PlistFormat::Binary));
    }

    #[test]
    fn test_identify_json() {
        let data = b"{ \"key\": \"value\" }";
        assert_eq!(identify_format(data), Some(PlistFormat::Json));
    }

    #[test]
    fn test_key_path_parsing() {
        assert_eq!(parse_key_path(":key:subkey"), vec!["key", "subkey"]);
        assert_eq!(parse_key_path("key"), vec!["key"]);
        assert_eq!(parse_key_path(":key:0:name"), vec!["key", "0", "name"]);
    }

    #[test]
    fn test_xml_roundtrip() {
        let mut dict = plist::Dictionary::new();
        dict.insert("key".to_string(), Value::String("value".to_string()));
        let value = Value::Dictionary(dict);

        let data = serialize(&value, PlistFormat::Xml).unwrap();
        let (value2, fmt) = deserialize(&data).unwrap();
        assert_eq!(fmt, PlistFormat::Xml);

        if let Value::Dictionary(d) = value2 {
            assert_eq!(d.get("key"), Some(&Value::String("value".to_string())));
        } else {
            panic!("expected dictionary");
        }
    }

    #[test]
    fn test_json_roundtrip() {
        let mut dict = plist::Dictionary::new();
        dict.insert("key".to_string(), Value::String("value".to_string()));
        dict.insert("num".to_string(), Value::Integer(42.into()));
        let value = Value::Dictionary(dict);

        let data = serialize(&value, PlistFormat::Json).unwrap();
        let value2 = deserialize_with_format(&data, PlistFormat::Json).unwrap();

        if let Value::Dictionary(d) = value2 {
            assert_eq!(d.get("key"), Some(&Value::String("value".to_string())));
        } else {
            panic!("expected dictionary");
        }
    }
}
