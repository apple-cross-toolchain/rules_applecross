//! The source rewrite: what clang 15 compiling C++ can't take from MSL as is.
//! Runs on preprocessed text (the stdlib included), once for the AST pass and
//! once for codegen.

use crate::text::{at, balanced, before, cast_keyword_before, is_word, replace_all, replace_where, split_source_args as split_args};
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

/// `ast_mode`: the AST pass's source (address spaces as attributes, attributes as
/// annotations); `mangle`: the one of those for Xcode's mangled names.
pub fn rewrite(src: &str, ast_mode: bool, mangle: bool, air: (u32, u32)) -> Result<Rewritten, String> {
    let mut r = Rewritten::default();
    let mut s = attributes(src);
    // the AST pass reads instances' signatures alone: their bodies, in address spaces our
    // stdlib's flat functions don't take, would make the specializations invalid
    s = host_names(&s, &mut r.instances, !(ast_mode && !mangle));
    s = threadgroup_locals(&s, ast_mode);
    s = replace_all(&s, regex!(r"\b__msl_stage_(kernel|vertex|fragment)\b"), |c| {
        format!("static __MSL_ATTR(\"msl.stage:{}\") __attribute__((always_inline, used))", &c[1])
    });
    s = function_constants(&s, &mut r)?;
    s = program_scope_constants(&s);
    s = direct_init(&s);
    // (not a declarator's parentheses: `thread float2 (*b)[2]`)
    s = replace_where(
        &s,
        regex!(r"\b((?:bool|char|uchar|short|ushort|int|uint|long|ulong|half|float)[234])\s*\("),
        |s, c| !matches!(s[c.get(0).unwrap().end()..].trim_start().as_bytes().first(), Some(b'*' | b'&')),
        |c| format!("metal::__vec<{}>(", &c[1]),
    );
    s = vector_casts(&s);
    s = typed_casts(&s);
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
        // (and codegen's marker: `[[threadgroup(0)]]`)
        let body = c[1].replace("__msl_threadgroup", "threadgroup");
        let mut body = replace_all(&body, regex!(r"__attribute__\(\(address_space\((\d)\)\)\)"), |a| {
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
        let attrs = parts
            .iter()
            .filter(|p| !p.is_empty())
            .map(|p| format!("__MSL_ATTR(\"msl:{}\")", joined_literals(p).replace('"', "\\\"")))
            .collect::<Vec<_>>()
            .join(" ");
        // set apart from the name it follows (`uint3 tgpig[[threadgroup_position_in_grid]]`)
        format!(" {attrs}")
    })
}

/// Adjacent string literals as one (`host_name("k_" "q4_0")`, built by a macro,
/// reaches us preprocessed): the compiler's own concatenation, which an attribute
/// read as text doesn't get.
fn joined_literals(p: &str) -> String {
    let b = p.as_bytes();
    let (mut out, mut i, mut open) = (String::new(), 0, false);
    while i < b.len() {
        let c = b[i];
        if open && c == b'\\' {
            // the escaped character, whatever its width
            out.push('\\');
            i += 1;
            if let Some(ch) = p[i..].chars().next() {
                out.push(ch);
                i += ch.len_utf8();
            }
            continue;
        }
        if c == b'"' && open {
            let next = i + 1 + p[i + 1..].len() - p[i + 1..].trim_start().len();
            if b.get(next) == Some(&b'"') {
                i = next + 1;
                continue;
            }
        }
        if c == b'"' {
            open = !open;
        }
        let ch = p[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// `template [[host_name("n")]] kernel F_t f<args>;` -> an explicit instantiation;
/// the entry point named n calls f<args> (llama.cpp and MLX declare kernels this way).
fn host_names(src: &str, out: &mut Vec<Instance>, bodies: bool) -> String {
    let pat = regex!(r#"\btemplate\s+__MSL_ATTR\("msl:host_name\(\s*\\"([^"\\]+)\\"\s*\)"\)\s+__msl_stage_(?:kernel|vertex|fragment)\s+([\w:]+)\s+([\w:]+)\s*<"#);
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
        // `F_t f<args>;`, or with the signature written out: `void f<args>(params);`
        let mut params = String::new();
        let mut semi = semi;
        if rest[semi..].starts_with('(') {
            let open = j + semi;
            let close = balanced(b, open, b'(', b')');
            let after = &src[close..];
            let s2 = after.len() - after.trim_start().len();
            if !after[s2..].starts_with(';') {
                continue;
            }
            params = src[open..close].to_string();
            semi = close - j + s2;
        } else if !rest[semi..].starts_with(';') {
            continue;
        }
        let args = src[m.end()..j - 1].to_string();
        out.push(Instance { host_name: c[1].to_string(), template: c[3].to_string(), args: args.clone() });
        res.push_str(&src[pos..m.start()]);
        // the static_assert names the specialization for ast::program, which
        // clang resolves (typedefs, enumerators, expressions); it makes no code
        // without bodies, no instantiation: the static_assert's operand isn't evaluated
        let (ty, f) = (&c[2], &c[3]);
        if bodies {
            res.push_str(&format!("template {ty} {f}<{args}>{params}; "));
        }
        // the address as the declared function type, where one is (`F_t f<args>;`), to pick among overloads
        let address = if params.is_empty() { format!("static_cast<{ty} *>(&{f}<{args}>)") } else { format!("&{f}<{args}>") };
        res.push_str(&format!("static_assert(sizeof({address}) > 0, \"msl.instance:{}\");", &c[1]));
        pos = j + semi + 1;
    }
    res.push_str(&src[pos..]);
    res
}

/// Codegen's `constant` (`__msl_constant`): `static` at program scope, where MSL's
/// variables are constants (and so internal, as Xcode's are), else `const`.
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
                // elsewhere read-only (MSL's constant memory is): a pointer to it takes a constant array
                out.push_str(if program_scope { if already { "" } else { "static" } } else { "const" });
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

/// A kernel's threadgroup variables (`threadgroup float shmem[N];`), one per threadgroup:
/// statics, as Xcode has them (`@_ZZ<kernel>E5shmem = internal addrspace(3) global`).
/// Codegen has no address spaces, so its are marked by a section the IR pass
/// (ir::threadgroup_statics) moves to address space 3; elsewhere (pointers,
/// parameters) the word goes. The AST pass has the address space, which an
/// automatic variable may not have.
fn threadgroup_locals(src: &str, ast_mode: bool) -> String {
    const MARK: &str = "__msl_threadgroup";
    let decl = regex!(r"([;{}]\s*)(?:__msl_threadgroup|__attribute__\(\(address_space\(3\)\)\))(\s+[A-Za-z_][\w:]*(?:\s*<[^;{}()]*>)?\s+[A-Za-z_]\w*\s*[\[;])");
    if ast_mode {
        return decl.replace_all(src, "${1}static __attribute__((address_space(3)))${2}").to_string();
    }
    if !src.contains(MARK) {
        return src.to_string();
    }
    let s = decl.replace_all(src, "${1}static __attribute__((section(\"msl.threadgroup\")))${2}");
    replace_where(&s, regex!(r"__msl_threadgroup\s*"), |s, c| !is_word(before(s, c.get(0).unwrap().start())), |_| String::new())
}

/// `constant T x [[function_constant(i)]];` becomes an internal global copied at
/// static-init time from an external initializer the IR pass turns into AIR's.
/// The index may be an expression, its macros expanded (`[[function_constant(FC_FLASH_ATTN_EXT + 22)]]`
/// arrives as `300 + 22`).
fn function_constants(src: &str, r: &mut Rewritten) -> Result<String, String> {
    let fc = regex!(r#"(?m)^[ \t]*(?:(?:__attribute__\(\(address_space\(2\)\)\)|__msl_constant)\s*)?([A-Za-z_][\w:<>, ]*?)\s+(\w+)\s*__MSL_ATTR\("msl:function_constant\(((?:[^"\\]|\\.)*)\)"\)\s*;"#);
    let index = |e: &str| integer_expr(e).and_then(|i| u32::try_from(i).ok()).ok_or_else(|| format!("function_constant({e}): an index we can't evaluate"));
    for c in fc.captures_iter(src) {
        r.constants.push(FunctionConstant { index: index(&c[3])?, name: c[2].to_string() });
    }
    let s = replace_all(src, fc, |c| {
        format!(
            "extern \"C\" {AS2} {t} __msl_fcinit_{i}_{n}; static {AS2} {t} {n} = __msl_fcinit_{i}_{n};",
            t = &c[1],
            i = index(&c[3]).unwrap_or_default(),
            n = &c[2]
        )
    });
    let defined = regex!(r"\bis_function_constant_defined\s*\(\s*(\w+)\s*\)");
    for c in defined.captures_iter(&s) {
        r.checked.insert(c[1].to_string());
    }
    let consts = r.constants.clone();
    Ok(replace_all(&s, defined, |c| match consts.iter().find(|k| k.name == c[1]) {
        Some(k) => format!("metal::__air_is_function_constant_defined(&__msl_fcinit_{}_{})", k.index, k.name),
        None => c[0].to_string(),
    }))
}

/// An integer constant expression of literals (`300 + 22`, `(1 << 4) | 2`), or None.
fn integer_expr(e: &str) -> Option<i64> {
    let toks: Vec<String> = regex!(r"0[xX][0-9a-fA-F]+[uUlL]*|\d+[uUlL]*|<<|>>|[-+*/%&|^~()]|\S").find_iter(e).map(|m| m.as_str().to_string()).collect();
    // precedence climbing: | ^ & (<< >>) (+ -) (* / %)
    fn bin(t: &[String], i: &mut usize, level: usize) -> Option<i64> {
        const LEVELS: [&[&str]; 6] = [&["|"], &["^"], &["&"], &["<<", ">>"], &["+", "-"], &["*", "/", "%"]];
        if level == LEVELS.len() {
            return unary(t, i);
        }
        let mut v = bin(t, i, level + 1)?;
        while let Some(op) = t.get(*i).filter(|o| LEVELS[level].contains(&o.as_str())) {
            let op = op.clone();
            *i += 1;
            let r = bin(t, i, level + 1)?;
            v = match op.as_str() {
                "|" => v | r,
                "^" => v ^ r,
                "&" => v & r,
                "<<" => v.checked_shl(u32::try_from(r).ok()?)?,
                ">>" => v.checked_shr(u32::try_from(r).ok()?)?,
                "+" => v.checked_add(r)?,
                "-" => v.checked_sub(r)?,
                "*" => v.checked_mul(r)?,
                "/" => v.checked_div(r)?,
                _ => v.checked_rem(r)?,
            };
        }
        Some(v)
    }
    fn unary(t: &[String], i: &mut usize) -> Option<i64> {
        let tok = t.get(*i)?.clone();
        *i += 1;
        match tok.as_str() {
            "-" => unary(t, i)?.checked_neg(),
            "+" => unary(t, i),
            "~" => Some(!unary(t, i)?),
            "(" => {
                let v = bin(t, i, 0)?;
                (t.get(*i)? == ")").then(|| *i += 1)?;
                Some(v)
            }
            lit => {
                let digits = lit.trim_end_matches(['u', 'U', 'l', 'L']);
                match digits.strip_prefix("0x").or_else(|| digits.strip_prefix("0X")) {
                    Some(h) => i64::from_str_radix(h, 16).ok(),
                    None if digits.len() > 1 && digits.starts_with('0') => i64::from_str_radix(&digits[1..], 8).ok(),
                    None => digits.parse().ok(),
                }
            }
        }
    }
    let mut i = 0;
    let v = bin(&toks, &mut i, 0)?;
    (i == toks.len()).then_some(v)
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
/// The spans of preprocessed text that come from our own headers (by clang's line markers:
/// `# 1 "<scratch>/include/metal_stdlib"`), which the user's rewrites leave alone.
fn our_header_spans(src: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut open: Option<usize> = None;
    for m in regex!(r#"(?m)^# \d+ "([^"]*)""#).captures_iter(src) {
        let at = m.get(0).unwrap().start();
        let ours = regex!(r"/include/(?:metal_stdlib|__metal_\w+\.h|simd/|TargetConditionals\.h)").is_match(&m[1]);
        match (ours, open) {
            (true, None) => open = Some(at),
            (false, Some(s)) => {
                spans.push((s, at));
                open = None;
            }
            _ => {}
        }
    }
    if let Some(s) = open {
        spans.push((s, src.len()));
    }
    spans
}

fn in_spans(spans: &[(usize, usize)], i: usize) -> bool {
    spans.iter().any(|(s, e)| *s <= i && i < *e)
}

/// `(T) x` where T is a template's type parameter or a typedef's name (`(q4_t) q4[i]`):
/// `metal::__cast<T>(x)`, which converts a vector element by element as MSL does.
fn typed_casts(src: &str) -> String {
    let mut names: BTreeSet<String> = BTreeSet::new();
    for c in regex!(r"\btemplate\s*<([^<>]*)>").captures_iter(src) {
        for p in regex!(r"\b(?:typename|class)\s+([A-Za-z_]\w*)").captures_iter(&c[1]) {
            names.insert(p[1].to_string());
        }
    }
    // (a typedef's or alias's name only when it names a vector or a matrix)
    let vector = regex!(r"\b(?:bool|char|uchar|short|ushort|int|uint|long|ulong|half|float|bfloat)[234](?:x[234])?\b|ext_vector_type|__vector<|\bmatrix<|\bvec<");
    for c in regex!(r"\btypedef\s+([^;{}()]*?)\b([A-Za-z_]\w*)\s*;").captures_iter(src) {
        if vector.is_match(&c[1]) {
            names.insert(c[2].to_string());
        }
    }
    for c in regex!(r"\busing\s+([A-Za-z_]\w*)\s*=\s*([^;]*);").captures_iter(src) {
        if vector.is_match(&c[2]) {
            names.insert(c[1].to_string());
        }
    }
    // (MSL's own vector and matrix names have rewrites of their own)
    names.retain(|n| !regex!(r"^(?:bool|char|uchar|short|ushort|int|uint|long|ulong|half|float|bfloat)[234](?:x[234])?$").is_match(n));
    if names.is_empty() {
        return src.to_string();
    }
    let ours = our_header_spans(src);
    let b = src.as_bytes();
    let (mut out, mut pos) = (String::new(), 0);
    for c in regex!(r"\(\s*([A-Za-z_]\w*)\s*\)").captures_iter(src) {
        let m = c.get(0).unwrap();
        if m.start() < pos || !names.contains(&c[1]) || in_spans(&ours, m.start()) {
            continue;
        }
        // a cast: followed by an operand, and not a call's or a declaration's parentheses
        let mut k = m.end();
        while k < b.len() && b[k].is_ascii_whitespace() {
            k += 1;
        }
        if k >= b.len() || !(is_word(b[k]) || b"(!~+-*&.".contains(&b[k])) {
            continue;
        }
        let mut p = m.start();
        while p > 0 && b[p - 1].is_ascii_whitespace() {
            p -= 1;
        }
        if p > 0 && (is_word(b[p - 1]) || b")]>".contains(&b[p - 1])) && !(cast_keyword_before(src, p) && !in_spans(&ours, p)) {
            continue;
        }
        let Some(end) = operand_end(src, m.end()) else { continue };
        out.push_str(&src[pos..m.start()]);
        out.push_str(&format!("metal::__cast<{}>({})", &c[1], src[m.end()..end].trim()));
        pos = end;
    }
    out.push_str(&src[pos..]);
    // and the functional form, `T(x)` with one argument (not a call through a member, a
    // qualified name or a declaration's type)
    let src = out;
    let ours = our_header_spans(&src);
    let b = src.as_bytes();
    let (mut out, mut pos) = (String::new(), 0);
    for c in regex!(r"\b([A-Za-z_]\w*)\(").captures_iter(&src) {
        let m = c.get(0).unwrap();
        if m.start() < pos || !names.contains(&c[1]) || in_spans(&ours, m.start()) {
            continue;
        }
        let mut p = m.start();
        while p > 0 && b[p - 1].is_ascii_whitespace() {
            p -= 1;
        }
        if p > 0 && (is_word(b[p - 1]) || b".>:~#".contains(&b[p - 1])) {
            continue;
        }
        let open = m.end() - 1;
        let close = balanced(b, open, b'(', b')');
        let inner = &src[open + 1..close - 1];
        if inner.trim().is_empty() || split_args(inner).len() != 1 {
            continue;
        }
        out.push_str(&src[pos..m.start()]);
        out.push_str(&format!("metal::__cast<{}>({})", &c[1], inner.trim()));
        pos = close;
    }
    out.push_str(&src[pos..]);
    out
}

fn vector_casts(src: &str) -> String {
    let ours = our_header_spans(src);
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
        if p > 0 && (is_word(b[p - 1]) || b")]>".contains(&b[p - 1])) && !(cast_keyword_before(src, p) && !in_spans(&ours, p)) {
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
        let puts: String = args.iter().enumerate().map(|(i, a)| format!(" auto __vb{} = __vb{i}.put({});", i + 1, lazy_vector_args(a.trim()))).collect();
        out.push_str(&format!("({{ metal::__vpart<{}, 0> __vb0;{puts} __vb{}.v; }})", &c[1], args.len()));
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

/// A vector or matrix declared with its constructor's arguments (`float4x4 m(s);`) is
/// initialised from the constructor as written (`float4x4 m = float4x4(s);`); a function
/// declaration (parameters a type and a name each, or none) is left alone.
fn direct_init(src: &str) -> String {
    let b = src.as_bytes();
    let (mut out, mut pos) = (String::new(), 0);
    for c in regex!(r"\b((?:bool|char|uchar|short|ushort|int|uint|long|ulong|half|float)[234](?:x[234])?)\s+([A-Za-z_]\w*)\s*\(").captures_iter(src) {
        let m = c.get(0).unwrap();
        if m.start() < pos || b"<,:".contains(&before(src, m.start())) || is_word(before(src, m.start())) || c[2].starts_with("__") {
            continue;
        }
        // nor a typedef
        let stmt = src[..m.start()].rfind([';', '{', '}']).map_or(0, |i| i + 1);
        if regex!(r"\btypedef\b").is_match(&src[stmt..m.start()]) {
            continue;
        }
        let open = m.end() - 1;
        let close = balanced(b, open, b'(', b')');
        let args = split_args(&src[open + 1..close - 1]);
        let declaration = args.iter().all(|a| a.trim().is_empty())
            || args.iter().any(|a| a.contains("[[") || regex!(r"^\s*(?:(?:const|constant|device|thread|threadgroup|volatile)\s+)*[\w:]+(?:<[^()]*>)?\s*[&*]*\s*[A-Za-z_]\w*\s*(?:=.*)?$").is_match(a) && !regex!(r"^\s*[A-Za-z_]\w*\s*$").is_match(a));
        let after = src[close..].trim_start();
        if declaration || !(after.starts_with(';') || after.starts_with(',')) {
            continue;
        }
        let t = &c[1];
        let f = if t.contains('x') && t.len() > 3 && t.as_bytes()[t.len() - 2] == b'x' { format!("metal::__mat<metal::{t}>") } else { format!("metal::__vec<{t}>") };
        out.push_str(&src[pos..m.start()]);
        out.push_str(&format!("{t} {} = {f}({})", &c[2], &src[open + 1..close - 1]));
        pos = close;
    }
    out.push_str(&src[pos..]);
    out
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
        let r = rewrite("float4 a = (float4)h + float2(0.5, 1.0).xyxy; half b = 0.5h;", false, false, (2, 9)).unwrap();
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
        assert!(out.contains("void f(const float *p) { const float *q = p; }"), "{out}");
        assert!(out.contains("static  bool b = true;"));
    }

    #[test]
    fn vectors_built_in_functions_put_each_argument() {
        let src = "void f(float a) { float4 v = metal::__vec<float4>(g(a), a, 0, 1); }\nfloat4 c = metal::__vec<float4>(1, 2, 3, 4);";
        let out = lazy_vector_args(src);
        assert!(out.contains("({ metal::__vpart<float4, 0> __vb0; auto __vb1 = __vb0.put(g(a)); auto __vb2 = __vb1.put(a); auto __vb3 = __vb2.put(0); auto __vb4 = __vb3.put(1); __vb4.v; })"));
        // number literals alone: a constant
        assert!(lazy_vector_args("void f() { x = metal::__vec<float3>(1.0, 0.0, -1.5f); }").contains("x = float3{(float)1.0, (float)0.0, (float)-1.5f};"));
        assert!(out.contains("float4 c = metal::__vec<float4>(1, 2, 3, 4);"));
    }

    #[test]
    fn an_attribute_right_after_a_name_stays_apart_from_it() {
        let out = attributes("kernel void k(uint3 tgpig[[threadgroup_position_in_grid]], ushort3 t [[thread_position_in_threadgroup]])");
        assert!(out.contains("uint3 tgpig __MSL_ATTR(\"msl:threadgroup_position_in_grid\"),"), "{out}");
        assert!(!direct_init(&out).contains("__vec"), "{out}");
    }

    #[test]
    fn function_constant_indices_are_evaluated() {
        assert_eq!(integer_expr("300 + 22"), Some(322));
        assert_eq!(integer_expr("(1 << 4) | 0x2u"), Some(18));
        assert_eq!(integer_expr("2 * (3 + 4) - 010"), Some(6));
        assert_eq!(integer_expr("kIndex + 1"), None);
        assert_eq!(integer_expr("1 +"), None);
        let mut r = Rewritten::default();
        let out = function_constants("constant int nsg __MSL_ATTR(\"msl:function_constant(300 + 22)\");\n", &mut r).unwrap();
        assert_eq!(r.constants[0].index, 322);
        assert!(out.contains("__msl_fcinit_322_nsg"), "{out}");
        assert!(function_constants("constant int n __MSL_ATTR(\"msl:function_constant(kIndex)\");\n", &mut Rewritten::default()).is_err());
    }

    #[test]
    fn threadgroup_locals_are_statics() {
        let tg = "__attribute__((address_space(3)))";
        let ast = threadgroup_locals(&format!("void k({tg} float *p) {{\n    {tg} float shmem[N];\n    {tg} uint hits;\n    {tg} float *q = p;\n}}\n"), true);
        assert!(ast.contains(&format!("{{\n    static {tg} float shmem[N];\n    static {tg} uint hits;\n    {tg} float *q = p;")), "{ast}");
        let code = threadgroup_locals("void k(__msl_threadgroup float *p) {\n    __msl_threadgroup float shmem[N];\n    __msl_threadgroup half4 sq4 [8];\n    __msl_threadgroup float4 *w = (__msl_threadgroup float4 *) shmem;\n}\n", false);
        assert_eq!(code, "void k(float *p) {\n    static __attribute__((section(\"msl.threadgroup\"))) float shmem[N];\n    static __attribute__((section(\"msl.threadgroup\"))) half4 sq4 [8];\n    float4 *w = (float4 *) shmem;\n}\n");
        assert!(attributes("float *p [[__msl_threadgroup(0)]]").contains("msl:threadgroup(0)"));
    }

    #[test]
    fn a_host_name_built_from_adjacent_literals_is_one_name() {
        let out = attributes("template [[host_name(\"k_\" \"q4_0\"  \"_f32\")]] kernel f_t f<1>;");
        assert!(out.contains(r#"__MSL_ATTR("msl:host_name(\"k_q4_0_f32\")")"#), "{out}");
        assert_eq!(joined_literals(r#"a("x\"" "y", "z")"#), r#"a("x\"y", "z")"#);
    }

    #[test]
    fn declarations_with_constructor_arguments() {
        let out = direct_init("float4x4 m(s);\nfloat2x2 n(s, 1, 2, s), k;\nfloat4 v(1.0f);\nfloat4x4 helper(float4x4 a, float4x4 b);\nfloat4 f();\ntypedef float float2 __attribute__((ext_vector_type(2)));");
        assert!(out.contains("float4x4 m = metal::__mat<metal::float4x4>(s);"), "{out}");
        assert!(out.contains("float2x2 n = metal::__mat<metal::float2x2>(s, 1, 2, s), k;"), "{out}");
        assert!(out.contains("float4 v = metal::__vec<float4>(1.0f);"), "{out}");
        assert!(out.contains("float4x4 helper(float4x4 a, float4x4 b);"), "{out}");
        assert!(out.contains("float4 f();"), "{out}");
        assert!(out.contains("typedef float float2 __attribute__((ext_vector_type(2)));"), "{out}");
    }

}
