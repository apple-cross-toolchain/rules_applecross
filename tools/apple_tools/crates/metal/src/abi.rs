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

/// A parameter's type, attributes and name.
fn parts(p: &str) -> (String, String, String) {
    let p = p.trim();
    let (head, name) = p.rsplit_once(' ').unwrap_or((p, ""));
    // the type ends where its attributes begin
    let attrs_at = regex!(r#" (?:noundef|nonnull|nocapture|readonly|writeonly|readnone|noalias|byval\(|sret\(|align \d|dereferenceable|zeroext|signext|"[^"]*")"#).find(head).map(|m| m.start()).unwrap_or(head.len());
    (head[..attrs_at].to_string(), head[attrs_at..].trim().to_string(), name.to_string())
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
                if tys.get(st).is_some_and(|b| fields(b).iter().all(|f| scalar_or_vector(f))) {
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
            if regex!(r#"^%"struct\.metal::(?:texture\w*|depth\w*|sampler)(?:\.\d+)?"$"#).is_match(&st) && fs.len() == 1 {
                out.push(Param::Handle(st, fs[0].clone()));
            } else if !fs.is_empty() && fs.iter().all(|f| scalar_or_vector(f)) {
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

/// Lower the user's functions' signatures and calls (before optimisation).
pub fn lower(ir: &str, xcode: &HashMap<String, Xcode>, keep_flat: &[String]) -> String {
    let tys = types(ir);
    // what each function becomes
    let mut plans: HashMap<String, Lowering> = HashMap::new();
    for c in regex!(r#"(?m)^define [^@\n]*?(void|\S+) @("?[\w.$]+"?)\((.*)\)[^\n]*\{$"#).captures_iter(ir) {
        let name = c[2].to_string();
        if !is_user(&name) || keep_flat.contains(&name) {
            continue;
        }
        let l = plan(&c[3], &c[1] == "void", xcode.get(name.trim_matches('"')), &tys);
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
                    format!("{pre}{indent}{r} = {head}{st} @{}({}){}\n{indent}store {st} {r}, {st}* {ptr}", &c[4], new_args.join(", "), &c[6])
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

#[cfg(test)]
mod tests {
    use super::*;

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

