//! Small text helpers shared by the source rewrite and the IR passes.

use regex::Regex;
use std::sync::LazyLock;

/// Split a comma-separated list at depth zero of ()[]{}<>.
pub fn split_args(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut cur = String::new();
    for ch in s.chars() {
        if ch == ',' && depth == 0 {
            out.push(cur.trim().to_string());
            cur.clear();
            continue;
        }
        match ch {
            '(' | '[' | '{' | '<' => depth += 1,
            ')' | ']' | '}' | '>' => depth -= 1,
            _ => {}
        }
        cur.push(ch);
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

pub fn is_word(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// The byte before `i` (or 0).
pub fn before(s: &str, i: usize) -> u8 {
    if i == 0 {
        0
    } else {
        s.as_bytes()[i - 1]
    }
}

/// The byte at `i` (or 0).
pub fn at(s: &str, i: usize) -> u8 {
    *s.as_bytes().get(i).unwrap_or(&0)
}

/// Replace every match of `re` for which `keep` accepts the match, using `f`.
/// (The regex crate has no lookaround; callers check context here.)
pub fn replace_where(
    s: &str,
    re: &Regex,
    keep: impl Fn(&str, &regex::Captures) -> bool,
    mut f: impl FnMut(&regex::Captures) -> String,
) -> String {
    let mut out = String::with_capacity(s.len());
    let mut pos = 0;
    for c in re.captures_iter(s) {
        let m = c.get(0).unwrap();
        if !keep(s, &c) {
            continue;
        }
        out.push_str(&s[pos..m.start()]);
        out.push_str(&f(&c));
        pos = m.end();
    }
    out.push_str(&s[pos..]);
    out
}

/// Replace every match using `f`.
pub fn replace_all(s: &str, re: &Regex, f: impl FnMut(&regex::Captures) -> String) -> String {
    replace_where(s, re, |_, _| true, f)
}

/// Index just past the parenthesis matching the one at `i`.
pub fn balanced(s: &[u8], mut i: usize, open: u8, close: u8) -> usize {
    let mut d = 0i32;
    while i < s.len() {
        if s[i] == open {
            d += 1;
        } else if s[i] == close {
            d -= 1;
        }
        i += 1;
        if d == 0 {
            return i;
        }
    }
    i
}

/// Each `define ... { ... }` of an LLVM module, as (start, end) byte ranges (end past the `}`).
pub fn functions(ir: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut pos = 0;
    while let Some(off) = ir[pos..].find("\ndefine ") {
        let start = pos + off + 1;
        let end = match ir[start..].find("\n}") {
            Some(e) => start + e + 2,
            None => break,
        };
        out.push((start, end));
        pos = end;
    }
    if ir.starts_with("define ") {
        if let Some(e) = ir.find("\n}") {
            out.insert(0, (0, e + 2));
        }
    }
    out
}

/// Apply `f` to the text of each function body, leaving the rest.
pub fn map_functions(ir: &str, mut f: impl FnMut(&str) -> String) -> String {
    let mut out = String::with_capacity(ir.len());
    let mut pos = 0;
    for (s, e) in functions(ir) {
        out.push_str(&ir[pos..s]);
        out.push_str(&f(&ir[s..e]));
        pos = e;
    }
    out.push_str(&ir[pos..]);
    out
}

/// Number every function's local values as an unnamed module would: the
/// arguments, then each block and each value-producing instruction in order (an
/// unnamed entry block takes a number too). Xcode's modules keep no local names.
pub fn discard_value_names(ir: &str) -> String {
    static VALUE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"%([-\w$.]+|"[^"]*")"#).unwrap());
    static LABEL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^([-\w$.]+|"[^"]*"):"#).unwrap());
    static DEF: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^\s+%([-\w$.]+|"[^"]*") = "#).unwrap());
    map_functions(ir, |f| {
        let open = f.find('(').unwrap();
        let close = balanced(f.as_bytes(), open, b'(', b')');
        let mut names = std::collections::HashMap::new();
        let mut n = 0usize;
        // unnamed values already numbered: an unnamed entry block's number follows them
        let mut unnamed = 0usize;
        for a in split_args(&f[open + 1..close - 1]) {
            if let Some(c) = VALUE.captures_iter(a.trim()).last() {
                if a.trim().ends_with(&c[0]) {
                    names.insert(c[1].to_string(), n);
                    unnamed += c[1].bytes().all(|b| b.is_ascii_digit()) as usize;
                }
            }
            n += 1;
        }
        let body = f[close..].find("{\n").map(|i| close + i + 2).unwrap_or(f.len());
        let mut first = true;
        for line in f[body..].lines() {
            if let Some(c) = LABEL.captures(line) {
                names.insert(c[1].to_string(), n);
                n += 1;
            } else if first && !line.trim().is_empty() {
                names.insert(unnamed.to_string(), n); // the unnamed entry block
                n += 1;
            }
            if !line.trim().is_empty() {
                first = false;
            }
            if let Some(c) = DEF.captures(line) {
                names.insert(c[1].to_string(), n);
                n += 1;
            }
        }
        let rename = |t: &str| -> String { VALUE.replace_all(t, |c: &regex::Captures| names.get(&c[1]).map_or(c[0].to_string(), |k| format!("%{k}"))).to_string() };
        let mut out = rename(&f[..body]);
        for line in f[body..].split_inclusive('\n') {
            match LABEL.captures(line) {
                Some(c) => out += &format!("{}:\n", names[&c[1]]),
                None => out += &rename(line),
            }
        }
        out
    })
}

/// The parameter list and body of `define ... @name(...)`.
pub fn find_define<'a>(ir: &'a str, name: &str) -> Option<(&'a str, &'a str)> {
    let needle = format!("@{}(", name);
    let mut pos = 0;
    while let Some(off) = ir[pos..].find(&needle) {
        let i = pos + off;
        let line_start = ir[..i].rfind('\n').map(|x| x + 1).unwrap_or(0);
        if ir[line_start..].starts_with("define ") {
            let open = i + needle.len() - 1;
            let close = balanced(ir.as_bytes(), open, b'(', b')');
            let sig = &ir[open + 1..close - 1];
            let body_start = ir[close..].find('{')? + close;
            let body_end = ir[body_start..].find("\n}\n").map(|e| body_start + e)?;
            return Some((sig, &ir[body_start..body_end]));
        }
        pos = i + needle.len();
    }
    None
}

pub fn quote(s: &str) -> String {
    format!("!\"{}\"", s.replace("__msl_kw_", ""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_at_depth_zero() {
        assert_eq!(split_args("a, b<c, d>, (e, f)"), vec!["a", "b<c, d>", "(e, f)"]);
    }

    #[test]
    fn finds_defines() {
        let ir = "x\ndefine void @f(i32 %a, <2 x i32> %b) #0 {\n  ret void\n}\n";
        let (sig, body) = find_define(ir, "f").unwrap();
        assert_eq!(sig, "i32 %a, <2 x i32> %b");
        assert!(body.contains("ret void"));
    }
}
