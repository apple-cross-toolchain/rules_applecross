//! Xcode's declarations of the AIR intrinsics and the Metal library functions
//! it calls (`___metal_fract_v2float`): their parameter and function
//! attributes, which clang 15 doesn't know (it gives every parameter `noundef`
//! and no memory attributes). Learned from Xcode's output for our probe shaders
//! (data/air_intrinsics.tsv: name, return type, parameters, function attributes;
//! one line per signature, as signatures change between AIR versions).
//!
//! Applied before opt, so attribute inference sees them, and again at the end.

use crate::text::split_args;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

macro_rules! regex {
    ($r:expr) => {{
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new($r).unwrap());
        &*RE
    }};
}

const ATTR: &str = r#"(?:noalias|nocapture|noundef|readonly|writeonly|readnone|nonnull|zeroext|signext|inreg|immarg|returned|align \d+|dereferenceable(?:_or_null)?\(\d+\))"#;

struct Decl {
    /// Each parameter's attributes ("" for none).
    params: Vec<String>,
    fn_attrs: String,
}

static TABLE: LazyLock<HashMap<&'static str, Vec<Decl>>> = LazyLock::new(|| {
    let strip = Regex::new(&format!(r"\s+{ATTR}")).unwrap();
    let mut m: HashMap<&str, Vec<Decl>> = HashMap::new();
    for line in include_str!("../data/air_intrinsics.tsv").lines() {
        let f: Vec<&str> = line.split('\t').collect();
        let params = if f[2].is_empty() { vec![] } else { split_args(f[2]) };
        let params = params
            .iter()
            .map(|p| {
                let ty = strip.replace_all(p, "");
                p[ty.len()..].trim().to_string()
            })
            .collect();
        m.entry(f[0]).or_default().push(Decl { params, fn_attrs: f[3].to_string() });
    }
    m
});

fn lookup(name: &str, arity: usize) -> Option<&'static Decl> {
    TABLE.get(name)?.iter().find(|d| d.params.len() == arity)
}

fn strip_attrs(s: &str) -> String {
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(&format!(r"\s+{ATTR}")).unwrap());
    RE.replace_all(s, "").to_string()
}

fn with_attrs(ty: &str, attrs: &str, value: Option<&str>) -> String {
    let mut s = ty.to_string();
    if !attrs.is_empty() {
        s += " ";
        s += attrs;
    }
    if let Some(v) = value {
        s += " ";
        s += v;
    }
    s
}

/// Give every air.* and ___metal_* declaration and call Xcode's attributes. Idempotent.
pub fn apply(ir: &str) -> String {
    // declarations: parameter types are the call sites' argument types too
    let mut types: HashMap<String, Vec<String>> = HashMap::new();
    let decl = regex!(r"(?m)^declare (.*?) @(air\.[\w.]+|___metal_\w+)\((.*)\)(.*)$");
    let ir = decl
        .replace_all(ir, |c: &regex::Captures| {
            let (ret, name, params) = (&c[1], &c[2], &c[3]);
            let tys: Vec<String> = if params.is_empty() { vec![] } else { split_args(params).iter().map(|p| strip_attrs(p.trim())).collect() };
            let Some(d) = lookup(name, tys.len()) else { return c[0].to_string() };
            types.insert(name.to_string(), tys.clone());
            let ps: Vec<String> = tys.iter().zip(&d.params).map(|(t, a)| with_attrs(t, a, None)).collect();
            let ret = regex!(r"\b(?:noundef|zeroext|signext) ").replace_all(ret, "");
            format!("declare {ret} @{name}({}) local_unnamed_addr {}", ps.join(", "), d.fn_attrs)
        })
        .to_string();
    // calls: each argument is its declared type, Xcode's attributes, then the value
    let call = regex!(r"(?m)^(\s*(?:%[\w.]+ = )?(?:tail )?call (?:(?:noundef|zeroext|signext|fast|spir_func) )*.*? @(air\.[\w.]+|___metal_\w+))\((.*)\)((?: #\d+| [a-z][a-z ]*)?)$");
    call.replace_all(&ir, |c: &regex::Captures| {
        let (head, name, args) = (&c[1], &c[2], &c[3]);
        let (Some(tys), Some(d)) = (types.get(name), lookup(name, types.get(name).map_or(usize::MAX, Vec::len))) else { return c[0].to_string() };
        let args = if args.is_empty() { vec![] } else { split_args(args) };
        if args.len() != tys.len() {
            return c[0].to_string();
        }
        let mut out = Vec::new();
        for ((a, t), at) in args.iter().zip(tys).zip(&d.params) {
            let a = a.trim();
            let Some(rest) = a.strip_prefix(t.as_str()) else { return c[0].to_string() };
            let v = strip_attrs(rest).trim().to_string();
            out.push(with_attrs(t, at, Some(&v)));
        }
        let head = regex!(r"\b(?:noundef|zeroext|signext) ").replace_all(head, "");
        // the call site carries the declaration's attributes but those opt infers; nothing in Metal unwinds
        let mut site: Vec<&str> = d.fn_attrs.split(' ').filter(|a| !matches!(*a, "mustprogress" | "nofree" | "nosync")).collect();
        if !site.contains(&"nounwind") {
            site.push("nounwind");
        }
        format!("{head}({}) {}", out.join(", "), site.join(" "))
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declarations_and_calls_get_xcodes_attributes() {
        let ir = "declare spir_func noundef float @air.fast_sin.f32(float noundef) #1\ndefine void @f(float %x) {\n  %y = tail call fast spir_func noundef float @air.fast_sin.f32(float noundef %x) #2\n  ret void\n}\n";
        let out = apply(ir);
        assert!(out.contains("declare spir_func float @air.fast_sin.f32(float) local_unnamed_addr mustprogress nofree nosync nounwind readnone willreturn"));
        assert!(out.contains("call fast spir_func float @air.fast_sin.f32(float %x) nounwind readnone willreturn"));
        assert_eq!(apply(&out), out);
    }

    #[test]
    fn library_functions_are_convergent_calls_that_never_unwind() {
        let ir = "declare spir_func noundef <2 x float> @___metal_fract_v2float(<2 x float> noundef, i32 noundef) #1\ndefine void @f(<2 x float> %x) {\n  %y = tail call fast spir_func noundef <2 x float> @___metal_fract_v2float(<2 x float> noundef %x, i32 noundef 0) #2\n  ret void\n}\n";
        let out = apply(ir);
        assert!(out.contains("declare spir_func <2 x float> @___metal_fract_v2float(<2 x float>, i32) local_unnamed_addr convergent\n"));
        assert!(out.contains("call fast spir_func <2 x float> @___metal_fract_v2float(<2 x float> %x, i32 0) convergent nounwind"));
        assert_eq!(apply(&out), out);
    }
}
