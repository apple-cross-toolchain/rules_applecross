//! The user's functions as Xcode calls them. Codegen is flat (spir64's ABI):
//! pointers in no address space, textures and structs by value through byval
//! pointers, structs returned through sret. Xcode keeps the functions its
//! inliner doesn't inline as calls with AIR's ABI: a pointer or reference in its
//! address space, a texture or sampler as its handle, a struct by value
//! expanded into its fields, a struct returned directly. Each function's
//! signature becomes that, with a prologue rebuilding the flat values its body
//! uses, and each call site converts its arguments; address-space inference and
//! SROA then clean both up.

use crate::mangle::Xcode;
use crate::text::{balanced, map_functions, split_args};
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

macro_rules! regex {
    ($r:expr) => {{
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new($r).unwrap());
        &*RE
    }};
}

/// Whether a function (by its IR name) is the user's, not our stdlib's or the driver's.
pub fn is_user(name: &str) -> bool {
    let n = name.trim_start_matches('"');
    n.starts_with("_Z") && !regex!(r"^_Z[NKZ]*5metal|^_ZN11__air_entry|^_ZZN5metal|^_GLOBAL__|^__cxx_global|__msl").is_match(n)
}

/// The functions the entry points call: the user's stage functions, always inlined.
pub fn stage_functions(ir: &str, entries: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for e in entries {
        if let Some((_, body)) = crate::text::find_define(ir, e) {
            for c in regex!(r#"\bcall [^@]*@("?[\w.$]+"?)\("#).captures_iter(body) {
                let n = c[1].to_string();
                if is_user(&n) && !out.contains(&n) {
                    out.push(n);
                }
            }
        }
    }
    out
}

/// How one parameter changes.
#[derive(Clone, Debug)]
enum Param {
    Same,
    /// A pointer into an address space: (pointee, space).
    Space(String, u32),
    /// A handle struct passed by value: (struct, handle type).
    Handle(String, String),
    /// A struct passed by value, expanded: (struct, field types).
    Expand(String, Vec<String>),
}

struct Lowering {
    params: Vec<Param>,
    /// The struct returned through sret, returned directly instead.
    sret: Option<String>,
}

fn types(ir: &str) -> HashMap<String, String> {
    regex!(r#"(?m)^(%[\w.]+|%"[^"]+") = type (\{.*\})$"#).captures_iter(ir).map(|c| (c[1].to_string(), c[2].to_string())).collect()
}

fn fields(body: &str) -> Vec<String> {
    body.trim().strip_prefix('{').and_then(|x| x.strip_suffix('}')).map(|x| split_args(x).iter().map(|f| f.trim().to_string()).collect()).unwrap_or_default()
}

/// A parameter's type, attributes and name (or an argument's value, which may be
/// a constant expression: `getelementptr inbounds ([16 x float], ...)`).
pub(crate) fn parts(p: &str) -> (String, String, String) {
    let p = p.trim();
    let ty_end = type_end(p);
    let mut rest = &p[ty_end..];
    let mut attrs_end = 0;
    while let Some(m) = regex!(r#"^\s+(?:noundef|nonnull|nocapture|readonly|writeonly|readnone|noalias|zeroext|signext|inreg|immarg|returned|align \d+|dereferenceable(?:_or_null)?\(\d+\)|(?:byval|sret|elementtype)\([^()]*\)|"[^"]*")"#).find(rest) {
        attrs_end += m.end();
        rest = &rest[m.end()..];
    }
    let after = &p[ty_end..];
    (p[..ty_end].to_string(), after[..attrs_end].trim().to_string(), after[attrs_end..].trim().to_string())
}

/// Where the LLVM type at the start of `s` ends: a primitive, a named, vector, array
/// or struct type, then any `addrspace(n)` and `*`s (and a function type's parameters).
fn type_end(s: &str) -> usize {
    let b = s.as_bytes();
    let mut i = match b.first() {
        Some(b'<') => balanced(b, 0, b'<', b'>'),
        Some(b'[') => balanced(b, 0, b'[', b']'),
        Some(b'{') => balanced(b, 0, b'{', b'}'),
        Some(b'%') if b.get(1) == Some(&b'"') => s[2..].find('"').map(|q| q + 3).unwrap_or(s.len()),
        _ => regex!(r"^%?[\w.:$-]+").find(s).map(|m| m.end()).unwrap_or(0),
    };
    loop {
        if let Some(m) = regex!(r"^ addrspace\(\d+\)").find(&s[i..]) {
            i += m.end();
        } else if b.get(i) == Some(&b'*') {
            i += 1;
        } else if s[i..].starts_with(" (") && s[balanced(b, i + 1, b'(', b')')..].starts_with('*') {
            i = balanced(b, i + 1, b'(', b')');
        } else {
            return i;
        }
    }
}

fn plan(sig: &str, ret_void: bool, xcode: Option<&Xcode>, tys: &HashMap<String, String>) -> Lowering {
    let params = split_args(sig);
    let mut out = Vec::new();
    let mut sret = None;
    let mut k = 0;
    for (i, p) in params.iter().enumerate() {
        let (ty, attrs, _) = parts(p);
        if i == 0 && ret_void && attrs.contains("sret(") {
            if let Some(st) = ty.strip_suffix('*') {
                if tys.get(st).is_some_and(|b| fields(b).iter().all(|f| value_field(f))) {
                    sret = Some(st.to_string());
                    out.push(Param::Same);
                    continue;
                }
            }
        }
        let space = xcode.and_then(|x| x.spaces.get(k).cloned().flatten());
        k += 1;
        if attrs.contains("byval(") {
            let st = ty.trim_end_matches('*').to_string();
            let fs = tys.get(&st).map(|b| fields(b)).unwrap_or_default();
            if let Some(h) = tys.get(&st).and_then(|b| crate::ir::wrapped_handle(b)) {
                out.push(Param::Handle(st, h.to_string()));
            } else if !fs.is_empty() && fs.iter().all(|f| value_field(f)) {
                out.push(Param::Expand(st, fs));
            } else {
                out.push(Param::Same);
            }
            continue;
        }
        match (space, ty.strip_suffix('*')) {
            (Some(s @ 1..=3), Some(pointee)) if !pointee.contains("addrspace") => out.push(Param::Space(pointee.to_string(), s)),
            _ => out.push(Param::Same),
        }
    }
    Lowering { params: out, sret }
}

fn scalar_or_vector(t: &str) -> bool {
    regex!(r"^(?:half|float|double|i\d+|<\d+ x (?:half|float|double|i\d+)>)$").is_match(t.trim())
}

/// A field Xcode passes and returns as a value: a scalar, a vector or an array of them
/// (a matrix's columns, `[2 x <2 x float>]`; padding, `[12 x i8]`; probes/p55_aggregate_abi).
fn value_field(t: &str) -> bool {
    match regex!(r"^\[\d+ x (.*)\]$").captures(t.trim()) {
        Some(c) => value_field(c.get(1).unwrap().as_str()),
        None => scalar_or_vector(t),
    }
}

/// Lower the user's functions' signatures and calls (before optimisation).
pub fn lower(ir: &str, xcode: &HashMap<String, Xcode>, keep_flat: &[String]) -> String {
    let tys = types(ir);
    // what each function becomes
    let mut plans: HashMap<String, Lowering> = HashMap::new();
    for c in regex!(r#"(?m)^define [^@\n]*?(void|\S+) @("?[\w.$]+"?)\((.*)\)[^\n]*\{$"#).captures_iter(ir) {
        let name = c[2].to_string();
        let mut l = plan(&c[3], &c[1] == "void", xcode.get(name.trim_matches('"')), &tys);
        // a stage function stays flat but takes its textures and samplers as handles, stored in
        // their objects first thing, as Xcode's entry point (the user's function) has them; and so
        // does the stdlib (Xcode's passes a sampler or texture object as its handle)
        if keep_flat.contains(&name) || !is_user(&name) {
            l.sret = None;
            for p in l.params.iter_mut() {
                if !matches!(p, Param::Handle(..)) {
                    *p = Param::Same;
                }
            }
        }
        if l.sret.is_some() || l.params.iter().any(|p| !matches!(p, Param::Same)) {
            plans.insert(name, l);
        }
    }
    if plans.is_empty() {
        return ir.to_string();
    }
    let mut n = 0usize;
    let mut fresh = |s: &str| {
        n += 1;
        format!("%__abi{n}.{s}")
    };
    // call sites first (they are in function bodies, the signatures on define lines)
    let call = regex!(r#"(?m)^(\s+)((?:%[\w.]+ = )?(?:tail )?call [^@\n]*?)(void|\S+) @("?[\w.$]+"?)\((.*)\)(.*)$"#);
    let out = map_functions(ir, |f| {
        call.replace_all(f, |c: &regex::Captures| {
            let Some(l) = plans.get(&c[4]) else { return c[0].to_string() };
            let indent = &c[1];
            let args = split_args(&c[5]);
            if args.len() != l.params.len() {
                return c[0].to_string();
            }
            let mut pre = String::new();
            let mut new_args = Vec::new();
            let mut sret_ptr = None;
            for (a, p) in args.iter().zip(&l.params) {
                let (ty, attrs, v) = parts(a);
                match p {
                    Param::Same if attrs.contains("sret(") && l.sret.is_some() => sret_ptr = Some(v),
                    Param::Same => new_args.push(a.trim().to_string()),
                    Param::Space(pt, s) => {
                        let x = fresh("as");
                        pre += &format!("{indent}{x} = addrspacecast {ty} {v} to {pt} addrspace({s})*\n");
                        new_args.push(format!("{pt} addrspace({s})* {} {x}", attrs.replace("nonnull ", "")).replace("  ", " "));
                    }
                    Param::Handle(st, h) => {
                        let (g, x) = (fresh("hp"), fresh("h"));
                        pre += &format!("{indent}{g} = getelementptr inbounds {st}, {st}* {v}, i32 0, i32 0\n{indent}{x} = load {h}, {h}* {g}, align 8\n");
                        new_args.push(format!("{h} {x}"));
                    }
                    Param::Expand(st, fs) => {
                        for (i, ft) in fs.iter().enumerate() {
                            let (g, x) = (fresh("fp"), fresh("f"));
                            pre += &format!("{indent}{g} = getelementptr inbounds {st}, {st}* {v}, i32 0, i32 {i}\n{indent}{x} = load {ft}, {ft}* {g}\n");
                            new_args.push(format!("{ft} {x}"));
                        }
                    }
                }
            }
            let head = &c[2];
            match (&l.sret, sret_ptr) {
                (Some(st), Some(ptr)) => {
                    let r = fresh("ret");
                    let head = regex!(r"^%[\w.]+ = ").replace(head, "");
                    // stored field by field, as Xcode's frontend stores a returned struct
                    // (a subscript of a returned matrix's column is two extractvalues; probes/p55_aggregate_abi)
                    let mut store = String::new();
                    for (i, ft) in tys.get(st).map(|b| fields(b)).unwrap_or_default().iter().enumerate() {
                        let (x, g) = (fresh("rf"), fresh("rp"));
                        store += &format!("\n{indent}{x} = extractvalue {st} {r}, {i}\n{indent}{g} = getelementptr inbounds {st}, {st}* {ptr}, i32 0, i32 {i}\n{indent}store {ft} {x}, {ft}* {g}");
                    }
                    format!("{pre}{indent}{r} = {head}{st} @{}({}){}{store}", &c[4], new_args.join(", "), &c[6])
                }
                _ => format!("{pre}{indent}{head}{} @{}({}){}", &c[3], &c[4], new_args.join(", "), &c[6]),
            }
        })
        .to_string()
    });
    // signatures, with a prologue rebuilding the flat values
    let mut result = String::with_capacity(out.len());
    let mut pos = 0;
    let def = regex!(r#"(?m)^(define [^@\n]*?)(void|\S+) @("?[\w.$]+"?)\("#);
    for c in def.captures_iter(&out.clone()) {
        let Some(l) = plans.get(&c[3]) else { continue };
        let m = c.get(0).unwrap();
        let open = m.end() - 1;
        let close = balanced(out.as_bytes(), open, b'(', b')');
        let brace = close + out[close..].find("{\n").unwrap() + 2;
        let params = split_args(&out[open + 1..close - 1]);
        let mut sig = Vec::new();
        let mut prologue = String::new();
        let mut sret_name = None;
        for (p, how) in params.iter().zip(&l.params) {
            let (ty, attrs, v) = parts(p);
            match how {
                Param::Same if attrs.contains("sret(") && l.sret.is_some() => {
                    prologue += &format!("  {v} = alloca {}\n", l.sret.as_ref().unwrap());
                    sret_name = Some(v);
                }
                Param::Same => sig.push(p.trim().to_string()),
                Param::Space(pt, s) => {
                    let x = format!("{v}.as");
                    // a pointer into an address space keeps its attributes but nonnull
                    sig.push(format!("{pt} addrspace({s})* {} {x}", attrs.replace("nonnull ", "")).replace("  ", " "));
                    prologue += &format!("  {v} = addrspacecast {pt} addrspace({s})* {x} to {ty}\n");
                }
                Param::Handle(st, h) => {
                    let x = format!("{v}.h");
                    sig.push(format!("{h} {x}"));
                    prologue += &format!("  {v} = alloca {st}\n  {v}.p = getelementptr inbounds {st}, {st}* {v}, i32 0, i32 0\n  store {h} {x}, {h}* {v}.p\n");
                }
                Param::Expand(st, fs) => {
                    prologue += &format!("  {v} = alloca {st}\n");
                    for (i, ft) in fs.iter().enumerate() {
                        let x = format!("{v}.{i}");
                        sig.push(format!("{ft} {x}"));
                        prologue += &format!("  {v}.p{i} = getelementptr inbounds {st}, {st}* {v}, i32 0, i32 {i}\n  store {ft} {x}, {ft}* {v}.p{i}\n");
                    }
                }
            }
            let _ = ty;
        }
        let ret = match &l.sret {
            Some(st) => st.clone(),
            None => c[2].to_string(),
        };
        let mut body = out[brace..].split_once("\n}\n").map(|x| x.0.to_string()).unwrap_or_default();
        // the prologue goes after the entry block's label, if it has one
        let (label, rest) = match body.split_once('\n') {
            Some((first, rest)) if regex!(r"^[\w.$-]+:").is_match(first) => (format!("{first}\n"), rest.to_string()),
            _ => (String::new(), body.clone()),
        };
        body = format!("{label}{prologue}{rest}");
        if let (Some(st), Some(v)) = (&l.sret, &sret_name) {
            let mut k = 0;
            body = regex!(r"(?m)^(\s+)ret void$")
                .replace_all(&body, |r: &regex::Captures| {
                    k += 1;
                    format!("{0}%__abiret{k} = load {st}, {st}* {v}\n{0}ret {st} %__abiret{k}", &r[1])
                })
                .to_string();
        }
        let head = &c[1];
        result.push_str(&out[pos..m.start()]);
        result.push_str(&format!("{head}{ret} @{}({}){}{}\n}}\n", &c[3], sig.join(", "), &out[close..brace], body));
        pos = brace + out[brace..].find("\n}\n").map(|x| x + 3).unwrap_or(out.len() - brace);
    }
    result.push_str(&out[pos..]);
    result
}

/// Xcode's modules have no comdats.
pub fn strip_comdats(ir: &str) -> String {
    let ir = regex!(r"(?m)^\$[^\n]* = comdat any\n\n?").replace_all(ir, "");
    regex!(r"(?m)^(define .*?) comdat(?:\(\$[^)]*\))?(.*)$").replace_all(&ir, "$1$2").to_string()
}

/// Xcode's SpecializeAddressSpaces, for a user function with generic parameters
/// (pointers and references with no address space, `generic`'s indices): the
/// function keeps address space 8 (Metal's generic one) there, and each call
/// goes to an internal copy for its argument's address space,
/// `<name>.specialized.p<space>`. A generic parameter isn't `nonnull`. The bodies
/// of the generic function and of any copy for a space other than 0 reach their
/// parameter through a cast for InferAddressSpaces to fold.
pub fn specialize_generic(ir: &str, generic: &HashMap<String, Vec<usize>>) -> String {
    let mut ir = ir.to_string();
    let mut clones = String::new();
    let mut names: Vec<&String> = generic.keys().collect();
    names.sort();
    for name in names {
        let indices = &generic[name];
        let Some((sig, _)) = crate::text::find_define(&ir, name) else { continue };
        let params = split_args(sig);
        if indices.iter().any(|&i| i >= params.len()) {
            continue;
        }
        // each call's space for the generic parameters (one each), the call rewritten to its copy
        let mut spaces: Vec<u32> = Vec::new();
        let call = Regex::new(&format!(r#"(?m)^(\s+(?:%[\w.]+ = )?(?:tail )?call [^@\n]*@){}\((.*)$"#, regex::escape(name))).unwrap();
        ir = call
            .replace_all(&ir, |c: &regex::Captures| {
                let open = c.get(2).unwrap().start() - 1;
                let line = &c[0];
                let rel = open - c.get(0).unwrap().start();
                let close = balanced(line.as_bytes(), rel, b'(', b')');
                let args = split_args(&line[rel + 1..close - 1]);
                let space = indices.iter().filter_map(|&i| args.get(i)).map(|a| regex!(r"addrspace\((\d+)\)").captures(a.rsplit_once(' ').map(|x| x.0).unwrap_or(a)).and_then(|m| m[1].parse::<u32>().ok()).unwrap_or(0)).max().unwrap_or(0);
                if !spaces.contains(&space) {
                    spaces.push(space);
                }
                format!("{}{name}.specialized.p{space}({}", &c[1], &c[2])
            })
            .to_string();
        spaces.sort();
        let Some(start) = ir.find(&format!("@{name}(")).map(|i| ir[..i].rfind('\n').map_or(0, |n| n + 1)).filter(|&l| ir[l..].starts_with("define ")) else { continue };
        let end = start + ir[start..].find("\n}\n").map(|e| e + 3).unwrap_or(ir.len() - start);
        let original = ir[start..end].to_string();
        // the function with its generic parameters in `space`, through casts to the flat ones
        let retyped = |text: &str, space: u32, rename: Option<&str>| -> String {
            let head_end = text.find('\n').unwrap_or(text.len());
            let (head, body) = text.split_at(head_end);
            let open = head.find(&format!("@{name}(")).unwrap() + name.len() + 1;
            let close = balanced(head.as_bytes(), open, b'(', b')');
            let ps = split_args(&head[open + 1..close - 1]);
            let mut casts = String::new();
            let new: Vec<String> = ps
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let p = p.trim();
                    if !indices.contains(&i) {
                        return p.to_string();
                    }
                    let p = p.replace(" nonnull", "");
                    let Some((ty_attrs, v)) = p.rsplit_once(' ') else { return p };
                    let ty = ty_attrs.split(' ').next().unwrap_or("");
                    if space == 0 || !ty.ends_with('*') {
                        return p.to_string();
                    }
                    let pointee = &ty[..ty.len() - 1];
                    casts += &format!("\n  {v} = addrspacecast {pointee} addrspace({space})* {v}.generic to {ty}");
                    format!("{pointee} addrspace({space})*{} {v}.generic", &ty_attrs[ty.len()..])
                })
                .collect();
            let mut h = format!("{}{}{}", &head[..open + 1], new.join(", "), &head[close - 1..]);
            if let Some(r) = rename {
                h = h.replacen(&format!("@{name}("), &format!("@{r}("), 1);
                h = regex!(r"^define (?:(?:dso_local|linkonce_odr|weak_odr|hidden) )*").replace(&h, "define internal ").to_string();
            }
            // the casts go first in the entry block
            let body = match body.strip_prefix("\n").and_then(|b| b.split_once('\n')) {
                Some((first, rest)) if regex!(r"^[\w.$-]+:").is_match(first) => format!("\n{first}{casts}\n{rest}"),
                _ => format!("{casts}{body}"),
            };
            format!("{h}{body}")
        };
        for space in &spaces {
            clones += "\n";
            clones += &retyped(&original, *space, Some(&format!("{name}.specialized.p{space}")));
        }
        ir.replace_range(start..end, &retyped(&original, 8, None));
    }
    if clones.is_empty() {
        return ir;
    }
    // the copies after the module's functions and declarations
    let at = regex!(r"(?m)^(?:attributes #|!)").find(&ir).map(|m| m.start()).unwrap_or(ir.len());
    ir.insert_str(at, &format!("{}\n", clones.trim_start()));
    ir
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_argument_is_its_type_attributes_and_value() {
        assert_eq!(parts("float* noundef %src"), ("float*".into(), "noundef".into(), "%src".into()));
        assert_eq!(parts(r#"%"struct.metal::S"* noundef nonnull align 4 dereferenceable(8) %s"#), (r#"%"struct.metal::S"*"#.into(), "noundef nonnull align 4 dereferenceable(8)".into(), "%s".into()));
        assert_eq!(parts("%struct.R* sret(%struct.R) align 4 %agg"), ("%struct.R*".into(), "sret(%struct.R) align 4".into(), "%agg".into()));
        assert_eq!(parts("<4 x float> addrspace(1)* %v"), ("<4 x float> addrspace(1)*".into(), String::new(), "%v".into()));
        // a constant expression, spaces and parentheses included
        let gep = "getelementptr inbounds ([16 x float], [16 x float]* addrspacecast ([16 x float] addrspace(2)* @k to [16 x float]*), i64 0, i64 0)";
        assert_eq!(parts(&format!("float* noundef {gep}")), ("float*".into(), "noundef".into(), gep.into()));
    }

    #[test]
    fn generic_parameters_specialize_by_space() {
        let ir = "define dso_local spir_func void @_Z2g2Rf(float* nocapture noundef nonnull align 4 dereferenceable(4) %x) local_unnamed_addr #0 {\nentry:\n  %v = load float, float* %x, align 4\n  store float %v, float* %x, align 4\n  ret void\n}\n\ndefine void @tk(float* %p) #1 {\nentry:\n  call spir_func void @_Z2g2Rf(float* noundef nonnull align 4 dereferenceable(4) %p) #5\n  ret void\n}\nattributes #0 = { nounwind }\n";
        let g: HashMap<String, Vec<usize>> = [("_Z2g2Rf".to_string(), vec![0])].into();
        let out = specialize_generic(ir, &g);
        assert!(out.contains("define dso_local spir_func void @_Z2g2Rf(float addrspace(8)* nocapture noundef align 4 dereferenceable(4) %x.generic) local_unnamed_addr #0 {\nentry:\n  %x = addrspacecast float addrspace(8)* %x.generic to float*\n"), "{out}");
        assert!(out.contains("call spir_func void @_Z2g2Rf.specialized.p0(float* noundef nonnull align 4 dereferenceable(4) %p) #5"));
        assert!(out.contains("define internal spir_func void @_Z2g2Rf.specialized.p0(float* nocapture noundef align 4 dereferenceable(4) %x) local_unnamed_addr #0 {\nentry:\n  %v = load float, float* %x"));
        assert!(out.find("@_Z2g2Rf.specialized.p0(float* nocapture").unwrap() < out.find("attributes #0").unwrap());
    }

    #[test]
    fn pointers_and_textures_lowered_with_their_calls() {
        let ir = "%\"struct.metal::texture2d\" = type { %struct._texture_2d_t addrspace(1)* }\ndefine spir_func float @_Z1fPfN5metal9texture2dE(float* noundef %p, %\"struct.metal::texture2d\"* noundef byval(%\"struct.metal::texture2d\") align 8 %t) #0 {\nbb:\n  %v = load float, float* %p, align 4\n  ret float %v\n}\n\ndefine spir_func void @k(float* %q, %\"struct.metal::texture2d\"* %tt) #0 {\nbb:\n  %r = call spir_func float @_Z1fPfN5metal9texture2dE(float* noundef %q, %\"struct.metal::texture2d\"* noundef byval(%\"struct.metal::texture2d\") align 8 %tt)\n  ret void\n}\n";
        let x: HashMap<String, Xcode> = [("_Z1fPfN5metal9texture2dE".to_string(), Xcode { name: "n".into(), spaces: vec![Some(1), None] })].into();
        let out = lower(ir, &x, &[]);
        assert!(out.contains("define spir_func float @_Z1fPfN5metal9texture2dE(float addrspace(1)* noundef %p.as, %struct._texture_2d_t addrspace(1)* %t.h) #0 {"));
        assert!(out.contains("  %p = addrspacecast float addrspace(1)* %p.as to float*"));
        assert!(out.contains("  store %struct._texture_2d_t addrspace(1)* %t.h, %struct._texture_2d_t addrspace(1)** %t.p"));
        assert!(out.contains("addrspacecast float* %q to float addrspace(1)*"));
        assert!(out.contains("call spir_func float @_Z1fPfN5metal9texture2dE(float addrspace(1)* noundef %__abi1.as, %struct._texture_2d_t addrspace(1)* %__abi3.h)"));
    }
}

