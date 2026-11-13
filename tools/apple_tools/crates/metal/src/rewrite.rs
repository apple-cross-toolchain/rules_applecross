//! The source rewrite: what clang 15 compiling C++ can't take from MSL as is.
//! Runs on preprocessed text (the stdlib included), once for the AST pass and
//! once for codegen.

use crate::text::{at, balanced, before, is_word, replace_all, replace_where, split_args};
use regex::{Captures, Regex};
use std::collections::BTreeSet;
use std::sync::LazyLock;

pub const AS1: &str = "__attribute__((address_space(1)))";
pub const AS2: &str = "__attribute__((address_space(2)))";
pub const AS3: &str = "__attribute__((address_space(3)))";

/// A `template [[host_name("n")]] kernel F_t f<args>;` instantiation.
#[derive(Clone, Debug)]
pub struct Instance {
    pub host_name: String,
    pub template: String,
    pub args: String,
}

#[derive(Clone, Debug)]
pub struct FunctionConstant {
    pub index: u32,
    pub name: String,
}

/// What the rewrite learns about the source.
#[derive(Default, Debug)]
pub struct Rewritten {
    pub code: String,
    pub instances: Vec<Instance>,
    /// In source order.
    pub constants: Vec<FunctionConstant>,
    /// Constants tested with is_function_constant_defined.
    pub checked: BTreeSet<String>,
    /// Constexpr sampler states, in source order.
    pub samplers: Vec<u64>,
}

macro_rules! regex {
    ($r:expr) => {{
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new($r).unwrap());
        &*RE
    }};
}

pub fn rewrite(src: &str, ast_mode: bool, air: (u32, u32)) -> Result<Rewritten, String> {
    let mut r = Rewritten::default();
    let mut s = attributes(src);
    s = host_names(&s, &mut r.instances);
    s = replace_all(&s, regex!(r"\b__msl_stage_(kernel|vertex|fragment)\b"), |c| {
        format!("static __MSL_ATTR(\"msl.stage:{}\") __attribute__((always_inline))", &c[1])
    });
    s = function_constants(&s, &mut r);
    s = program_scope_constants(&s);
    s = replace_all(&s, regex!(r"\b((?:bool|char|uchar|short|ushort|int|uint|long|ulong|half|float)[234])\s*\("), |c| {
        format!("metal::__vec<{}>(", &c[1])
    });
    s = vector_casts(&s);
    s = lazy_vector_args(&s);
    s = replace_where(
        &s,
        regex!(r"\b((?:float|half)[234]x[234])\s*\("),
        |s, c| {
            let b = before(s, c.get(0).unwrap().start());
            !is_word(b) && b != b':'
        },
        |c| format!("metal::__mat<metal::{}>(", &c[1]),
    );
    // The AST pass sees attributes as annotations; codegen doesn't see them.
    s = replace_all(&s, regex!(r#"__MSL_ATTR\(("(?:[^"\\]|\\.)*")\)"#), |c| {
        if ast_mode {
            format!("__attribute__((annotate({})))", &c[1])
        } else {
            String::new()
        }
    });
    // MSL has no double: unsuffixed floating literals are float.
    s = replace_where(
        &s,
        regex!(r"(\d+\.\d*(?:[eE][-+]?\d+)?|\.\d+(?:[eE][-+]?\d+)?|\d+[eE][-+]?\d+)"),
        |s, c| {
            let m = c.get(0).unwrap();
            let (b, a) = (before(s, m.start()), at(s, m.end()));
            !is_word(b) && b != b'.' && !is_word(a) && a != b'.'
        },
        |c| format!("{}f", &c[1]),
    );
    // Half literals: 0.5h
    s = replace_where(
        &s,
        regex!(r"(\d+\.\d*|\.\d+|\d+)((?:[eE][-+]?\d+)?)[hH]\b"),
        |s, c| {
            let b = before(s, c.get(0).unwrap().start());
            !is_word(b) && b != b'.'
        },
        |c| format!("((half){}{})", &c[1], &c[2]),
    );
    r.code = samplers(&s, air, &mut r.samplers)?;
    Ok(r)
}

/// `[[a, b(c)]]` -> `__MSL_ATTR("msl:a") __MSL_ATTR("msl:b(c)")`.
fn attributes(src: &str) -> String {
    replace_all(src, regex!(r"\[\[\s*((?:[^\[\]]|\[[^\[\]]*\])*?)\s*\]\]"), |c| {
        // the preprocessor expanded address-space words inside attributes
        let mut body = replace_all(&c[1], regex!(r"__attribute__\(\(address_space\((\d)\)\)\)"), |a| {
            match &a[1] {
                "1" => "device",
                "2" => "constant",
                _ => "threadgroup",
            }
            .to_string()
        });
        if body.trim_start().starts_with('(') {
            // [[threadgroup(0)]] with `threadgroup` expanded to nothing
            body = format!("threadgroup{}", body.trim_start());
        }
        let mut parts = Vec::new();
        let (mut depth, mut cur) = (0i32, String::new());
        for ch in body.chars() {
            if ch == ',' && depth == 0 {
                parts.push(cur.trim().to_string());
                cur.clear();
                continue;
            }
            match ch {
                '(' | '<' => depth += 1,
                ')' | '>' => depth -= 1,
                _ => {}
            }
            cur.push(ch);
        }
        parts.push(cur.trim().to_string());
        parts
            .iter()
            .filter(|p| !p.is_empty())
            .map(|p| format!("__MSL_ATTR(\"msl:{}\")", p.replace('"', "\\\"")))
            .collect::<Vec<_>>()
            .join(" ")
    })
}

/// `template [[host_name("n")]] kernel F_t f<args>;` -> an explicit instantiation;
/// the entry point named n calls f<args> (llama.cpp and MLX declare kernels this way).
fn host_names(src: &str, out: &mut Vec<Instance>) -> String {
    let pat = regex!(r#"\btemplate\s+__MSL_ATTR\("msl:host_name\(\\"([^"\\]+)\\"\)"\)\s+__msl_stage_(?:kernel|vertex|fragment)\s+([\w:]+)\s+([\w:]+)\s*<"#);
    let b = src.as_bytes();
    let (mut res, mut pos) = (String::new(), 0);
    for c in pat.captures_iter(src) {
        let m = c.get(0).unwrap();
        if m.start() < pos {
            continue;
        }
        let (mut j, mut d) = (m.end(), 1i32);
        while j < b.len() && d > 0 {
            match b[j] {
                b'<' | b'(' => d += 1,
                b'>' | b')' => d -= 1,
                _ => {}
            }
            j += 1;
        }
        let rest = &src[j..];
        let semi = rest.len() - rest.trim_start().len();
        if !rest[semi..].starts_with(';') {
            continue;
        }
        let args = src[m.end()..j - 1].to_string();
        out.push(Instance { host_name: c[1].to_string(), template: c[3].to_string(), args: args.clone() });
        res.push_str(&src[pos..m.start()]);
        res.push_str(&format!("template {} {}<{}>;", &c[2], &c[3], args));
        pos = j + semi + 1;
    }
    res.push_str(&src[pos..]);
    res
}

/// Codegen's `constant` (`__msl_constant`): `static` at program scope, where MSL's
/// variables are constants (and so internal, as Xcode's are), else nothing.
fn program_scope_constants(src: &str) -> String {
    const MARK: &str = "__msl_constant";
    if !src.contains(MARK) {
        return src.to_string();
    }
    let b = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    // braces that open a scope (true) or only group declarations (namespace, extern "C")
    let mut braces: Vec<bool> = Vec::new();
    let (mut parens, mut i, mut last) = (0i32, 0, 0);
    while i < b.len() {
        match b[i] {
            b'"' => {
                i += 1;
                while i < b.len() && b[i] != b'"' {
                    i += if b[i] == b'\\' { 2 } else { 1 };
                }
            }
            b'(' => parens += 1,
            b')' => parens -= 1,
            b'{' => {
                let head = src[src[..i].rfind(|c: char| c == ';' || c == '}' || c == '{').map_or(0, |x| x + 1)..i].trim();
                braces.push(!regex!(r#"^(?:inline\s+)?namespace\b|^extern\s*"C""#).is_match(head));
            }
            b'}' => {
                braces.pop();
            }
            _ if src[i..].starts_with(MARK) && (i == 0 || !is_word(b[i - 1])) && !is_word(at(src, i + MARK.len())) => {
                out.push_str(&src[last..i]);
                let program_scope = parens == 0 && !braces.iter().any(|s| *s);
                // static rather than const: internal as Xcode's, without clang's invariant.start after a dynamic initialiser
                let already = regex!(r"\bstatic\s*$").is_match(&out);
                out.push_str(if program_scope && !already { "static" } else { "" });
                i += MARK.len();
                last = i;
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    out.push_str(&src[last..]);
    out
}

/// `constant T x [[function_constant(i)]];` becomes an internal global copied at
/// static-init time from an external initializer the IR pass turns into AIR's.
fn function_constants(src: &str, r: &mut Rewritten) -> String {
    let fc = regex!(r#"(?m)^[ \t]*(?:(?:__attribute__\(\(address_space\(2\)\)\)|__msl_constant)\s*)?([A-Za-z_][\w:<>, ]*?)\s+(\w+)\s*__MSL_ATTR\("msl:function_constant\((\d+)\)"\)\s*;"#);
    for c in fc.captures_iter(src) {
        r.constants.push(FunctionConstant { index: c[3].parse().unwrap(), name: c[2].to_string() });
    }
    let s = replace_all(src, fc, |c| {
        format!(
            "extern \"C\" {AS2} {t} __msl_fcinit_{i}_{n}; static {AS2} {t} {n} = __msl_fcinit_{i}_{n};",
            t = &c[1],
            i = &c[3],
            n = &c[2]
        )
    });
    let defined = regex!(r"\bis_function_constant_defined\s*\(\s*(\w+)\s*\)");
    for c in defined.captures_iter(&s) {
        r.checked.insert(c[1].to_string());
    }
    let consts = r.constants.clone();
    replace_all(&s, defined, |c| match consts.iter().find(|k| k.name == c[1]) {
        Some(k) => format!("metal::__air_is_function_constant_defined(&__msl_fcinit_{}_{})", k.index, k.name),
        None => c[0].to_string(),
    })
}

/// End of the unary expression starting at `i`: unary operators, a primary
/// (identifier, number, parenthesised), then postfix [..] (..) .x ->x.
pub fn operand_end(src: &str, mut i: usize) -> Option<usize> {
    let b = src.as_bytes();
    let n = b.len();
    let skip = |mut i: usize| {
        while i < n && (b[i] == b' ' || b[i] == b'\t') {
            i += 1;
        }
        i
    };
    i = skip(i);
    while i < n && b"-+!~*&".contains(&b[i]) && !(src[i..].starts_with("++") || src[i..].starts_with("--")) {
        i = skip(i + 1);
    }
    if i < n && b[i] == b'(' {
        let j = balanced(b, i, b'(', b')');
        let k = skip(j);
        // (type) followed by an operand is itself a cast: its operand is part of ours
        if regex!(r"^\(\s*[\w:]+(?:\s*<[^()]*>)?\s*\)$").is_match(&src[i..j]) && k < n && (b[k].is_ascii_alphanumeric() || b[k] == b'_' || b[k] == b'(') {
            return operand_end(src, k);
        }
        i = j;
    } else {
        let m = regex!(r"^(?:(?:::)?[A-Za-z_]\w*(?:::[A-Za-z_]\w*)*|[0-9][\w.]*(?:[eE][-+]?\d+)?\w*)").find(&src[i..])?;
        i += m.end();
    }
    while i < n {
        if b[i] == b'[' {
            i = balanced(b, i, b'[', b']');
        } else if b[i] == b'(' {
            i = balanced(b, i, b'(', b')');
        } else if b[i] == b'.' && i + 1 < n && (b[i + 1].is_ascii_alphabetic() || b[i + 1] == b'_') {
            i += 1 + regex!(r"^\w+").find(&src[i + 1..]).unwrap().end();
        } else if src[i..].starts_with("->") {
            i += 2 + regex!(r"^\s*\w+").find(&src[i + 2..]).map(|m| m.end()).unwrap_or(0);
        } else {
            break;
        }
    }
    Some(i)
}

/// MSL's (float4)h converts element by element; clang would reinterpret.
/// (T)expr becomes metal::__vec<T>(expr) (or __mat for matrices).
fn vector_casts(src: &str) -> String {
    let cast = regex!(r"\(\s*((?:metal::)?(?:(?:bool|char|uchar|short|ushort|int|uint|long|ulong|half|float)[234](?:x[234])?))\s*\)");
    let b = src.as_bytes();
    let (mut out, mut pos) = (String::new(), 0);
    for c in cast.captures_iter(src) {
        let m = c.get(0).unwrap();
        if m.start() < pos {
            continue;
        }
        // followed by an operand
        let mut k = m.end();
        while k < b.len() && b[k].is_ascii_whitespace() {
            k += 1;
        }
        if k >= b.len() || !(is_word(b[k]) || b"(!~+-*&.".contains(&b[k])) {
            continue;
        }
        // not a call or a declaration's parameter list
        let mut p = m.start();
        while p > 0 && b[p - 1].is_ascii_whitespace() {
            p -= 1;
        }
        if p > 0 && (is_word(b[p - 1]) || b")]>".contains(&b[p - 1])) {
            continue;
        }
        let Some(end) = operand_end(src, m.end()) else { continue };
        let t = c[1].replace("metal::", "");
        let f = if t.len() >= 3 && t[t.len() - 3..].contains('x') {
            format!("metal::__mat<metal::{t}>")
        } else {
            format!("metal::__vec<{t}>")
        };
        out.push_str(&src[pos..m.start()]);
        out.push_str(&format!("{f}({})", src[m.end()..end].trim()));
        pos = end;
    }
    out.push_str(&src[pos..]);
    out
}

/// For each byte, whether it is in a function body (block scope).
fn block_scope(src: &str) -> Vec<bool> {
    let b = src.as_bytes();
    let mut inside = vec![false; b.len() + 1];
    let mut stack: Vec<bool> = Vec::new();
    for i in 0..b.len() {
        match b[i] {
            b'{' => {
                let parent = *stack.last().unwrap_or(&false);
                // a function's body follows its parameter list (and qualifiers)
                let head = src[..i].trim_end();
                let head = regex!(r"(?:\s*\b(?:const|noexcept|override|mutable|volatile)\b)*$").replace(head, "");
                stack.push(parent || head.ends_with(')'));
            }
            b'}' => {
                stack.pop();
            }
            _ => {}
        }
        inside[i + 1] = *stack.last().unwrap_or(&false);
    }
    inside
}

/// `__vec<T>(a, b, ...)` in a function -> a statement expression putting each
/// argument as it is evaluated, as Xcode builds a vector.
fn lazy_vector_args(src: &str) -> String {
    let scope = block_scope(src);
    let call = regex!(r"metal::__vec<(\w+)>\(");
    let (mut out, mut pos) = (String::new(), 0);
    for c in call.captures_iter(src) {
        let m = c.get(0).unwrap();
        if m.start() < pos {
            continue;
        }
        let open = m.end() - 1;
        let close = balanced(src.as_bytes(), open, b'(', b')');
        let args = split_args(&src[open + 1..close - 1]);
        // a statement expression needs a function body (and isn't constant)
        let stmt_start = src[..m.start()].rfind(|ch: char| ch == ';' || ch == '{' || ch == '}').map_or(0, |x| x + 1);
        if args.len() < 2 || !scope[m.start()] || src[stmt_start..m.start()].contains("constexpr") {
            continue;
        }
        out.push_str(&src[pos..m.start()]);
        // one number literal per element: a constant vector, as Xcode's frontend makes it
        let n: usize = c[1].chars().last().and_then(|d| d.to_digit(10)).unwrap_or(0) as usize;
        if args.len() == n && args.iter().all(|a| regex!(r"^[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?[fFhHuUlL]*$").is_match(a.trim())) {
            let elem = &c[1][..c[1].len() - 1];
            let elems: Vec<String> = args.iter().map(|a| format!("({elem}){}", a.trim())).collect();
            out.push_str(&format!("{}{{{}}}", &c[1], elems.join(", ")));
            pos = close;
            continue;
        }
        let puts: String = args.iter().map(|a| format!(" __vb.put({});", lazy_vector_args(a.trim()))).collect();
        out.push_str(&format!("({{ metal::__vbuild<{}> __vb; __vb.i = 0;{puts} __vb.v; }})", &c[1]));
        pos = close;
    }
    out.push_str(&src[pos..]);
    out
}

// ---------------------------------------------------------------- constexpr samplers

fn half_bits(x: f32) -> u64 {
    // round to nearest even, as numpy/struct 'e' does for these values
    let b = x.to_bits();
    let sign = ((b >> 16) & 0x8000) as u64;
    let exp = ((b >> 23) & 0xff) as i32 - 127 + 15;
    let man = b & 0x7fffff;
    if exp >= 31 {
        return sign | 0x7c00;
    }
    if exp <= 0 {
        return sign; // values here are never subnormal
    }
    let mut h = ((exp as u32) << 10) | (man >> 13);
    let rest = man & 0x1fff;
    if rest > 0x1000 || (rest == 0x1000 && h & 1 == 1) {
        h += 1;
    }
    sign | h as u64
}

/// The sampler state word Xcode writes (FORMAT.md, "Constexpr samplers").
pub fn sampler_state(args: &str) -> Result<u64, String> {
    let (mut s, mut t, mut r, mut mag, mut min, mut mip, mut coord, mut cmp, mut aniso, mut border) = (1u64, 1u64, 1u64, 0u64, 0u64, 0u64, 0u64, 8u64, 1u64, 0u64);
    let (mut lodmin, mut lodmax) = (0.0f32, 65504.0f32);
    let address = |v: &str| -> Result<u64, String> {
        Ok(match v {
            "clamp_to_zero" | "clamp_to_border" => 0,
            "clamp_to_edge" => 1,
            "repeat" => 2,
            "mirrored_repeat" => 3,
            _ => return Err(format!("unsupported address mode {v}")),
        })
    };
    let filter = |v: &str| -> Result<u64, String> {
        Ok(match v {
            "nearest" => 0,
            "linear" => 1,
            _ => return Err(format!("unsupported filter {v}")),
        })
    };
    for a in split_args(args) {
        let a = regex!(r"\s*::\s*").replace_all(&a.replace("metal::", ""), "::").trim().to_string();
        if let Some(c) = regex!(r"^(\w+)::(\w+)$").captures(&a) {
            let v = &c[2];
            match &c[1] {
                "address" => {
                    s = address(v)?;
                    t = s;
                    r = s;
                }
                "s_address" => s = address(v)?,
                "t_address" => t = address(v)?,
                "r_address" => r = address(v)?,
                "filter" => {
                    mag = filter(v)?;
                    min = mag;
                }
                "mag_filter" => mag = filter(v)?,
                "min_filter" => min = filter(v)?,
                "mip_filter" => {
                    mip = match v {
                        "none" => 0,
                        "nearest" => 1,
                        "linear" => 2,
                        _ => return Err(format!("unsupported mip filter {v}")),
                    }
                }
                "coord" => coord = if v == "pixel" { 1 } else { 0 },
                "compare_func" => {
                    cmp = ["none", "less", "less_equal", "greater", "greater_equal", "equal", "not_equal", "always", "never"]
                        .iter()
                        .position(|x| *x == v)
                        .ok_or_else(|| format!("unsupported compare_func {v}"))? as u64
                }
                "border_color" => {
                    border = ["transparent_black", "opaque_black", "opaque_white"]
                        .iter()
                        .position(|x| *x == v)
                        .ok_or_else(|| format!("unsupported border_color {v}"))? as u64
                }
                k => return Err(format!("unsupported sampler argument {k}")),
            }
        } else if let Some(c) = regex!(r"^max_anisotropy\((\d+)\)$").captures(&a) {
            aniso = c[1].parse().unwrap();
        } else if let Some(c) = regex!(r"^lod_clamp\(([^,]+),([^)]+)\)$").captures(&a) {
            let f = |x: &str| x.trim().trim_end_matches(['f', 'F']).parse::<f32>().map_err(|e| e.to_string());
            lodmin = f(&c[1])?;
            lodmax = f(&c[2])?;
        } else {
            return Err(format!("unsupported sampler argument {a}"));
        }
    }
    Ok(s | t << 3 | r << 6 | mag << 9 | min << 11 | mip << 13 | coord << 15 | cmp << 16 | (aniso - 1) << 20 | half_bits(lodmin) << 24 | half_bits(lodmax) << 40 | border << 56)
}

/// `constexpr sampler s(args);` refers to an address-space-2 state the IR pass defines.
fn samplers(src: &str, air: (u32, u32), out: &mut Vec<u64>) -> Result<String, String> {
    let mut decls = Vec::new();
    let mut err = None;
    let mut one = |c: &Captures| -> String {
        let k = out.len();
        let word = match sampler_state(c.get(2).map(|m| m.as_str()).unwrap_or("")) {
            Ok(w) => w,
            Err(e) => {
                err = Some(e);
                0
            }
        };
        out.push(word);
        decls.push(format!("extern \"C\" {AS2} unsigned long __msl_sampler_{k}{};", if air >= (2, 7) { "[2]" } else { "" }));
        format!("const metal::sampler {} = {{({AS2} _sampler_t *)&__msl_sampler_{k}}};", &c[1])
    };
    let mut s = src.to_string();
    for re in [
        regex!(r"\bconstexpr\s+sampler\s+(\w+)\s*\(((?:[^()]|\([^()]*\))*)\)\s*;"),
        regex!(r"\bconstexpr\s+(?:metal::)?sampler\s+(\w+)\s*=\s*(?:metal::)?sampler\s*\(((?:[^()]|\([^()]*\))*)\)\s*;"),
        regex!(r"\bconstexpr\s+(?:metal::)?sampler\s+(\w+)\s*()(?:\{\s*\})?\s*;"),
    ] {
        s = replace_all(&s, re, &mut one);
    }
    if let Some(e) = err {
        return Err(e);
    }
    if decls.is_empty() {
        Ok(s)
    } else {
        Ok(decls.join("\n") + "\n" + &s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sampler_words_match_xcode() {
        assert_eq!(sampler_state("").unwrap(), 0x7bff0000080049);
        assert_eq!(sampler_state("coord::pixel").unwrap(), 0x7bff0000088049);
        assert_eq!(sampler_state("address::repeat, filter::linear, mip_filter::linear").unwrap(), 34901797601036946);
        assert_eq!(sampler_state("lod_clamp(2.5f, 8.0f)").unwrap(), 0x0048004100080049);
        assert_eq!(sampler_state("max_anisotropy(16)").unwrap(), 34901797616746569);
    }

    #[test]
    fn casts_and_literals() {
        let r = rewrite("float4 a = (float4)h + float2(0.5, 1.0).xyxy; half b = 0.5h;", false, (2, 9)).unwrap();
        assert!(r.code.contains("metal::__vec<float4>(h)"), "{}", r.code);
        assert!(r.code.contains("metal::__vec<float2>(0.5f, 1.0f)"), "{}", r.code);
        assert!(r.code.contains("((half)0.5)"), "{}", r.code);
    }

    #[test]
    fn program_scope_constants_are_static() {
        let src = "__msl_constant float k = 2;\nnamespace n { __msl_constant int j = 1; }\nvoid f(__msl_constant float *p) { __msl_constant float *q = p; }\nstatic __msl_constant bool b = true;";
        let out = program_scope_constants(src);
        assert!(out.starts_with("static float k = 2;"));
        assert!(out.contains("namespace n { static int j = 1; }"));
        assert!(out.contains("void f( float *p) {  float *q = p; }"));
        assert!(out.contains("static  bool b = true;"));
    }

    #[test]
    fn vectors_built_in_functions_put_each_argument() {
        let src = "void f(float a) { float4 v = metal::__vec<float4>(g(a), a, 0, 1); }\nfloat4 c = metal::__vec<float4>(1, 2, 3, 4);";
        let out = lazy_vector_args(src);
        assert!(out.contains("({ metal::__vbuild<float4> __vb; __vb.i = 0; __vb.put(g(a)); __vb.put(a); __vb.put(0); __vb.put(1); __vb.v; })"));
        // number literals alone: a constant
        assert!(lazy_vector_args("void f() { x = metal::__vec<float3>(1.0, 0.0, -1.5f); }").contains("x = float3{(float)1.0, (float)0.0, (float)-1.5f};"));
        assert!(out.contains("float4 c = metal::__vec<float4>(1, 2, 3, 4);"));
    }
}
