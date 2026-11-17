//! Xcode's mangled names for the user's functions. Codegen compiles flat, so
//! its names lack the address spaces of pointer and reference parameters;
//! Xcode's carry them as vendor qualifiers (device: U9MTLdevice, constant:
//! U11MTLconstant, which is const too, threadgroup: U14MTLthreadgroup, an
//! explicit thread: U9MTLthread). The
//! AST pass's source has the address spaces, so clang mangles them there
//! (U3AS<n>): each function's name in both, paired, gives the renaming.

use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

macro_rules! regex {
    ($r:expr) => {{
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new($r).unwrap());
        &*RE
    }};
}

/// Each function declaration's (name, mangled name, type) in a JSON AST dump, in order.
pub fn decls(json: &str) -> Vec<(String, String, String)> {
    regex!(r#""kind": "(?:FunctionDecl|CXXMethodDecl)",[\s\S]*?"name": "([^"]*)",\s*"mangledName": "([^"]*)",\s*"type": \{\s*"qualType": "((?:[^"\\]|\\.)*)""#)
        .captures_iter(json)
        .map(|c| (c[1].to_string(), c[2].to_string(), c[3].replace("\\\"", "\"")))
        .collect()
}

/// The address space of each parameter of a function type that is a pointer or
/// reference into one (`float (__attribute__((address_space(1))) float *, uint)`).
pub fn param_spaces(fn_type: &str) -> Vec<Option<u32>> {
    let Some(open) = fn_type.rfind(')').and_then(|close| {
        let b = fn_type.as_bytes();
        let mut depth = 0;
        (0..=close).rev().find(|&i| {
            match b[i] {
                b')' => depth += 1,
                b'(' => depth -= 1,
                _ => {}
            }
            depth == 0
        })
    }) else {
        return vec![];
    };
    let params = &fn_type[open + 1..fn_type.rfind(')').unwrap()];
    if params.trim().is_empty() || params.trim() == "void" {
        return vec![];
    }
    crate::text::split_args(params)
        .iter()
        .map(|p| if p.contains('*') || p.contains('&') { regex!(r"address_space\((\d)\)").captures(p).and_then(|c| c[1].parse().ok()) } else { None })
        .collect()
}

/// A user function as Xcode has it: its name, and its parameters' address spaces.
#[derive(Clone, Debug)]
pub struct Xcode {
    pub name: String,
    pub spaces: Vec<Option<u32>>,
}

/// Flat name -> Xcode's name and parameter address spaces, for functions with
/// pointers or references into an address space, or a half in their name.
pub fn user_functions(with_spaces: &[(String, String, String)], flat: &[(String, String)]) -> HashMap<String, Xcode> {
    let mut seen: HashMap<&str, usize> = HashMap::new();
    let mut by_key: HashMap<(String, usize), (&str, &str)> = HashMap::new();
    for (n, m, t) in with_spaces {
        let k = seen.entry(n).or_insert(0);
        by_key.insert((n.clone(), *k), (m, t));
        *k += 1;
    }
    let mut seen: HashMap<&str, usize> = HashMap::new();
    let mut out = HashMap::new();
    for (n, m) in flat {
        let k = seen.entry(n).or_insert(0);
        if let Some((a, t)) = by_key.get(&(n.clone(), *k)) {
            if a.contains("U3AS") || a.contains("DF16_") {
                out.insert(m.clone(), Xcode { name: xcode_spelling(a), spaces: param_spaces(t) });
            }
        }
        *k += 1;
    }
    out
}

/// Xcode's name -> the indices of a free function's generic parameters: pointers and
/// references with no address space (MSL's generic one, which Xcode specializes).
pub fn generic_params(with_spaces: &[(String, String, String)]) -> HashMap<String, Vec<usize>> {
    let mut out = HashMap::new();
    for (_, m, t) in with_spaces {
        // free functions only (a method's object is another matter)
        if !regex!(r"^_Z\d").is_match(m) {
            continue;
        }
        let params = generic_indices(t);
        if !params.is_empty() {
            out.insert(xcode_spelling(m), params);
        }
    }
    out
}

fn generic_indices(fn_type: &str) -> Vec<usize> {
    let spaces = param_spaces(fn_type);
    let Some(open) = fn_type.find('(') else { return vec![] };
    let close = fn_type.rfind(')').unwrap_or(fn_type.len());
    let params = &fn_type[open + 1..close];
    if params.trim().is_empty() || params.trim() == "void" {
        return vec![];
    }
    crate::text::split_args(params)
        .iter()
        .enumerate()
        .filter(|(i, p)| (p.contains('*') || p.contains('&')) && !p.contains('(') && spaces.get(*i).copied().flatten().is_none())
        .map(|(i, _)| i)
        .collect()
}

/// clang's address-space qualifiers as Xcode spells them (constant implies const),
/// and half: our `_Float16` is `DF16_`, Xcode's half `Dh` (builtin types both, never
/// substitution candidates, so nothing else renumbers).
pub fn xcode_spelling(mangled: &str) -> String {
    let s = regex!(r"U3AS2(K?)").replace_all(mangled, "U11MTLconstantK");
    s.replace("U3AS1", "U9MTLdevice").replace("U3AS3", "U14MTLthreadgroup").replace("U3AS9", "U9MTLthread").replace("DF16_", "Dh")
}

/// The functions of `ir` renamed.
pub fn apply(ir: &str, renames: &HashMap<String, String>) -> String {
    if renames.is_empty() {
        return ir.to_string();
    }
    // functions and their comdats, and their statics (`_ZZ<function>E<name>`; a kernel's
    // without the L kernel_statics took from it)
    regex!(r"([@$])(_Z[\w.$]+)")
        .replace_all(ir, |c: &regex::Captures| {
            if let Some(n) = renames.get(&c[2]) {
                return format!("{}{n}", &c[1]);
            }
            let Some(inner) = c[2].strip_prefix("_ZZ") else { return c[0].to_string() };
            for (k, v) in renames {
                let (enc, to) = (&k[2..], &v[2..]);
                for (enc, to) in [(enc, to), (enc.strip_prefix('L').unwrap_or(enc), to.strip_prefix('L').unwrap_or(to))] {
                    if let Some(rest) = inner.strip_prefix(enc).filter(|r| r.starts_with('E')) {
                        return format!("{}_ZZ{to}{rest}", &c[1]);
                    }
                }
            }
            c[0].to_string()
        })
        .to_string()
}

/// A kernel's statics (its threadgroup variables) named as Xcode names them, in a
/// function that isn't static (`_ZZ9tg_reduce...E6shared`): we make the stage
/// functions (`stages`, `_ZL...`) static.
pub fn kernel_statics(ir: &str, stages: &[String]) -> String {
    let mut out = ir.to_string();
    for s in stages {
        if let Some(enc) = s.strip_prefix("_ZL") {
            out = out.replace(&format!("@_ZZL{enc}E"), &format!("@_ZZ{enc}E"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_kernels_statics_follow_its_name() {
        let ir = "@_ZZL1kPfE6shared = internal addrspace(3) global [16 x float] undef\n@_ZZL1hPfE1t = internal global i32 0\n";
        let ir = kernel_statics(ir, &["_ZL1kPf".to_string()]);
        assert!(ir.starts_with("@_ZZ1kPfE6shared = "));
        let renames: HashMap<String, String> = [("_ZL1kPf".to_string(), "_ZL1kPU9MTLdevicef".to_string()), ("_ZL1hPf".to_string(), "_ZL1hPU9MTLdevicef".to_string())].into();
        let out = apply(&ir, &renames);
        assert!(out.contains("@_ZZ1kPU9MTLdevicefE6shared = "), "{out}");
        // a static helper's statics keep its L
        assert!(out.contains("@_ZZL1hPU9MTLdevicefE1t = "), "{out}");
    }

    #[test]
    fn spells_address_spaces_as_xcode() {
        assert_eq!(xcode_spelling("_Z1fPU3AS1fRU3AS2K1SPU3AS3fPU3AS2f"), "_Z1fPU9MTLdevicefRU11MTLconstantK1SPU14MTLthreadgroupfPU11MTLconstantKf");
        assert_eq!(xcode_spelling("_Z2g5RU3AS9K1S"), "_Z2g5RU9MTLthreadK1S");
        assert_eq!(xcode_spelling("_Z3lumDv3_DF16_"), "_Z3lumDv3_Dh");
    }

    #[test]
    fn generic_parameters_are_those_with_no_space() {
        assert_eq!(generic_indices("void (float &)"), vec![0]);
        assert_eq!(generic_indices("float (__attribute__((address_space(9))) float &, const S &, float)"), vec![1]);
        assert!(generic_indices("void (__attribute__((address_space(1))) float *)").is_empty());
    }

    #[test]
    fn parameter_address_spaces() {
        assert_eq!(param_spaces("float (__attribute__((address_space(1))) float *, uint)"), vec![Some(1), None]);
        assert_eq!(param_spaces("void (const __attribute__((address_space(2))) S &, float (*)(int))"), vec![Some(2), None]);
    }

    #[test]
    fn pairs_declarations_by_name_and_order() {
        let a = vec![("f".to_string(), "_Z1fPU3AS1f".to_string(), "float (__attribute__((address_space(1))) float *)".to_string()), ("f".to_string(), "_Z1fi".to_string(), "float (int)".to_string())];
        let b = vec![("f".to_string(), "_Z1fPf".to_string()), ("f".to_string(), "_Z1fi".to_string())];
        let r = user_functions(&a, &b);
        assert_eq!(r.get("_Z1fPf").map(|x| x.name.as_str()), Some("_Z1fPU9MTLdevicef"));
        assert_eq!(r.get("_Z1fPf").map(|x| x.spaces.clone()), Some(vec![Some(1)]));
        assert_eq!(r.len(), 1);
    }
}
