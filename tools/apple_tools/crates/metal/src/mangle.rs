//! Xcode's mangled names for the user's functions. Codegen compiles flat, so
//! its names lack the address spaces of pointer and reference parameters;
//! Xcode's carry them as vendor qualifiers (device: U9MTLdevice, constant:
//! U11MTLconstant, which is const too, threadgroup: U14MTLthreadgroup). The
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

/// Each function declaration's (name, mangled name), in order.
pub fn names(json: &str) -> Vec<(String, String)> {
    decls(json).into_iter().map(|(n, m, _)| (n, m)).collect()
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
/// pointers or references into an address space.
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
            if a.contains("U3AS") {
                out.insert(m.clone(), Xcode { name: xcode_spelling(a), spaces: param_spaces(t) });
            }
        }
        *k += 1;
    }
    out
}

/// clang's address-space qualifiers as Xcode spells them (constant implies const).
pub fn xcode_spelling(mangled: &str) -> String {
    let s = regex!(r"U3AS2(K?)").replace_all(mangled, "U11MTLconstantK");
    s.replace("U3AS1", "U9MTLdevice").replace("U3AS3", "U14MTLthreadgroup")
}

/// Flat name -> Xcode's, for the functions whose names differ.
pub fn renames(with_spaces: &[(String, String)], flat: &[(String, String)]) -> HashMap<String, String> {
    // pair the k-th declaration of each name in both
    let mut seen: HashMap<&str, usize> = HashMap::new();
    let mut by_key: HashMap<(String, usize), &str> = HashMap::new();
    for (n, m) in with_spaces {
        let k = seen.entry(n).or_insert(0);
        by_key.insert((n.clone(), *k), m);
        *k += 1;
    }
    let mut seen: HashMap<&str, usize> = HashMap::new();
    let mut out = HashMap::new();
    for (n, m) in flat {
        let k = seen.entry(n).or_insert(0);
        if let Some(a) = by_key.get(&(n.clone(), *k)) {
            if a.contains("U3AS") {
                out.insert(m.clone(), xcode_spelling(a));
            }
        }
        *k += 1;
    }
    out
}

/// The functions of `ir` renamed.
pub fn apply(ir: &str, renames: &HashMap<String, String>) -> String {
    if renames.is_empty() {
        return ir.to_string();
    }
    // functions and their comdats
    regex!(r"([@$])(_Z[\w.$]+)")
        .replace_all(ir, |c: &regex::Captures| match renames.get(&c[2]) {
            Some(n) => format!("{}{n}", &c[1]),
            None => c[0].to_string(),
        })
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spells_address_spaces_as_xcode() {
        assert_eq!(xcode_spelling("_Z1fPU3AS1fRU3AS2K1SPU3AS3fPU3AS2f"), "_Z1fPU9MTLdevicefRU11MTLconstantK1SPU14MTLthreadgroupfPU11MTLconstantKf");
    }

    #[test]
    fn parameter_address_spaces() {
        assert_eq!(param_spaces("float (__attribute__((address_space(1))) float *, uint)"), vec![Some(1), None]);
        assert_eq!(param_spaces("void (const __attribute__((address_space(2))) S &, float (*)(int))"), vec![Some(2), None]);
    }

    #[test]
    fn pairs_declarations_by_name_and_order() {
        let a = vec![("f".to_string(), "_Z1fPU3AS1f".to_string()), ("f".to_string(), "_Z1fi".to_string())];
        let b = vec![("f".to_string(), "_Z1fPf".to_string()), ("f".to_string(), "_Z1fi".to_string())];
        let r = renames(&a, &b);
        assert_eq!(r.get("_Z1fPf").map(String::as_str), Some("_Z1fPU9MTLdevicef"));
        assert_eq!(r.len(), 1);
    }
}
