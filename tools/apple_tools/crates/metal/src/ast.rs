//! Signatures, structs and typedefs from clang's text AST dump (`-Xclang -ast-dump`).

use crate::rewrite::Instance;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

macro_rules! regex {
    ($r:expr) => {{
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new($r).unwrap());
        &*RE
    }};
}

pub struct Node {
    pub kind: String,
    pub rest: String,
    pub kids: Vec<Node>,
}

/// The dump's tree, from its `|-`/`` `-`` prefixes.
pub fn tree(text: &str) -> Node {
    let mut stack: Vec<(i64, Node)> = vec![(-1, Node { kind: "root".into(), rest: String::new(), kids: vec![] })];
    for line in text.lines() {
        let Some(c) = regex!(r"^([| `-]*)([A-Za-z]+)(.*)$").captures(line) else { continue };
        let depth = c[1].len() as i64;
        while stack.last().unwrap().0 >= depth {
            let (_, done) = stack.pop().unwrap();
            stack.last_mut().unwrap().1.kids.push(done);
        }
        stack.push((depth, Node { kind: c[2].to_string(), rest: c[3].to_string(), kids: vec![] }));
    }
    while stack.len() > 1 {
        let (_, done) = stack.pop().unwrap();
        stack.last_mut().unwrap().1.kids.push(done);
    }
    stack.pop().unwrap().1
}

/// (name, type, desugared type) of a declaration line.
pub fn decl(rest: &str) -> Option<(String, String, String)> {
    let c = regex!(r"(?:\s(?:col|line):\d+(?::\d+)?)\s(?:(?:used|referenced|invalid|implicit|constexpr|static|cinit|callinit|listinit) )*(\w*) ?'([^']*)'(?::'([^']*)')?").captures(rest)?;
    let t = c[2].to_string();
    Some((c[1].to_string(), t.clone(), c.get(3).map(|m| m.as_str().to_string()).unwrap_or(t)))
}

fn annotations(n: &Node) -> Vec<String> {
    n.kids
        .iter()
        .filter(|k| k.kind == "AnnotateAttr")
        .filter_map(|k| regex!(r#""(.*)"$"#).captures(&k.rest).map(|c| c[1].replace("\\\"", "\"")))
        .collect()
}

fn msl_attrs(n: &Node) -> Vec<String> {
    annotations(n).into_iter().filter_map(|a| a.strip_prefix("msl:").map(str::to_string)).collect()
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: String,
    pub ty: String,
    pub desugared: String,
    pub attrs: Vec<String>,
    /// A field of a [[stage_in]] struct (after flattening).
    pub stage_in: bool,
}

#[derive(Clone, Debug)]
pub struct Function {
    /// The AIR name: the function's, or its [[host_name]].
    pub name: String,
    pub stage: String,
    /// The function type, e.g. `VOut (VIn, constant float2 &)`.
    pub ty: String,
    pub params: Vec<Param>,
    pub attrs: Vec<String>,
    pub ns: String,
    /// What the entry point calls (the name, or `f<args>` for an instantiation).
    pub call: String,
}

#[derive(Default)]
pub struct Program {
    pub functions: Vec<Function>,
    pub records: HashMap<String, Vec<Param>>,
    pub typedefs: HashMap<String, String>,
    /// Function constants' declared types, by name.
    pub constant_types: HashMap<String, String>,
    /// Function constants' namespaces (`metalpetal::`), by name.
    pub constant_namespaces: HashMap<String, String>,
    /// [[host_name]] instances whose specialization clang didn't resolve (an error, not a kernel left out).
    pub unresolved: Vec<String>,
    /// Local arrays of pointers into an address space (`device const half *pm[NQ];`), by name:
    /// the space, or None where variables of that name differ.
    pub pointer_arrays: HashMap<String, Option<u32>>,
}

/// A stage function's buffers, textures and samplers written without an index (llama.cpp's
/// `device const char * src0,`) get one, as Xcode's assign them: in parameter order, each
/// kind counting on from 0 past the indices written out.
pub fn implicit_resources(program: &mut Program) {
    for f in program.functions.iter_mut() {
        let explicit = |kind: &str, ps: &[Param]| -> std::collections::BTreeSet<u32> {
            ps.iter().filter_map(|p| p.attrs.first()).filter_map(|a| regex!(r"^(\w+)\((\d+)").captures(a).filter(|c| &c[1] == kind).map(|c| c[2].parse().unwrap())).collect()
        };
        let mut used: std::collections::HashMap<&str, std::collections::BTreeSet<u32>> = ["buffer", "texture", "sampler"].into_iter().map(|k| (k, explicit(k, &f.params))).collect();
        for p in f.params.iter_mut() {
            if !p.attrs.is_empty() || p.stage_in {
                continue;
            }
            let t = format!("{} {}", p.ty, p.desugared);
            let kind = if regex!(r"\b(?:texture|depth)\w*<|_texture_\w+_t\b").is_match(&t) {
                "texture"
            } else if regex!(r"\bsampler\b").is_match(&t) && !t.contains('*') {
                "sampler"
            } else if regex!(r"[*&]\s*$").is_match(p.ty.trim_end()) && regex!(r"address_space\((?:1|2)\)|\b(?:device|constant)\b").is_match(&p.ty) {
                "buffer"
            } else {
                continue;
            };
            let set = used.get_mut(kind).unwrap();
            let n = (0..).find(|n| !set.contains(n)).unwrap();
            set.insert(n);
            p.attrs.insert(0, format!("{kind}({n})"));
        }
    }
}

fn params(n: &Node) -> Vec<Param> {
    n.kids
        .iter()
        .filter(|p| p.kind == "ParmVarDecl")
        .filter_map(|p| {
            let (name, ty, desugared) = decl(&p.rest)?;
            Some(Param { name, ty, desugared, attrs: msl_attrs(p), stage_in: false })
        })
        .collect()
}

fn stage_of(n: &Node) -> Option<String> {
    annotations(n).into_iter().find_map(|a| a.strip_prefix("msl.stage:").map(str::to_string))
}

pub fn program(text: &str, instances: &[Instance]) -> Program {
    let mut p = Program::default();
    let mut last_anon: Option<Vec<Param>> = None;
    let root = tree(text);
    let mut specs = HashMap::new();
    instance_specs(&root, &mut specs);
    walk(&root, "", instances, &specs, &mut p, &mut last_anon);
    pointer_arrays(&root, &mut p.pointer_arrays);
    p.unresolved = instances.iter().filter(|i| !p.functions.iter().any(|f| f.name == i.host_name)).map(|i| i.host_name.clone()).collect();
    p
}

/// Every local array of pointers into an address space, in any function's body (a
/// template's as declared: `half *[NQ]`).
fn pointer_arrays(n: &Node, out: &mut HashMap<String, Option<u32>>) {
    for k in &n.kids {
        if k.kind == "VarDecl" {
            if let Some((name, ty, _)) = decl(&k.rest) {
                if let Some(c) = regex!(r"address_space\((\d)\)\)\)[^*\[\]()]*\*\s*(?:\[[^\[\]]+\])+$").captures(&ty) {
                    let space: u32 = c[1].parse().unwrap();
                    let e = out.entry(name).or_insert(Some(space));
                    if *e != Some(space) {
                        *e = None;
                    }
                }
            }
        }
        pointer_arrays(k, out);
    }
}

/// Each instance's specialization (host name -> its FunctionDecl's address), from
/// the `static_assert(sizeof(&f<args>) > 0, "msl.instance:<host name>")` the
/// rewrite puts after it. Specializations are listed in the order clang made
/// them, which a `decltype(f<args>)` ahead of the instances changes.
fn instance_specs(n: &Node, out: &mut HashMap<String, String>) {
    fn find<'a>(n: &'a Node, f: &dyn Fn(&'a Node) -> Option<&'a str>) -> Option<&'a str> {
        f(n).or_else(|| n.kids.iter().find_map(|k| find(k, f)))
    }
    for k in &n.kids {
        if k.kind == "StaticAssertDecl" {
            let name = find(k, &|n| (n.kind == "StringLiteral").then(|| regex!(r#""msl\.instance:([^"]*)"$"#).captures(&n.rest)?.get(1).map(|m| m.as_str())).flatten());
            let addr = find(k, &|n| (n.kind == "DeclRefExpr").then(|| regex!(r" Function (0x[0-9a-f]+) ").captures(&n.rest)?.get(1).map(|m| m.as_str())).flatten());
            if let (Some(name), Some(addr)) = (name, addr) {
                out.insert(name.to_string(), addr.to_string());
            }
        } else {
            instance_specs(k, out);
        }
    }
}

fn address(n: &Node) -> Option<&str> {
    n.rest.split_whitespace().next()
}

fn walk(n: &Node, ns: &str, instances: &[Instance], specs: &HashMap<String, String>, p: &mut Program, last_anon: &mut Option<Vec<Param>>) {
    for k in &n.kids {
        match k.kind.as_str() {
            "TranslationUnitDecl" | "LinkageSpecDecl" => walk(k, ns, instances, specs, p, last_anon),
            "NamespaceDecl" => {
                let inner = match regex!(r" (\w+)$").captures(&k.rest) {
                    Some(c) => format!("{ns}{}::", &c[1]),
                    None => ns.to_string(),
                };
                walk(k, &inner, instances, specs, p, last_anon)
            }
            "VarDecl" if k.rest.contains("__msl_fcinit_") => {
                if let Some((vn, vt, _)) = decl(&k.rest) {
                    if let Some(c) = regex!(r"^__msl_fcinit_\d+_(\w+)$").captures(&vn) {
                        p.constant_types.insert(c[1].to_string(), vt);
                        p.constant_namespaces.insert(c[1].to_string(), ns.to_string());
                    }
                }
            }
            "CXXRecordDecl" if k.rest.contains(" definition") => {
                let fields: Vec<Param> = k
                    .kids
                    .iter()
                    .filter(|f| f.kind == "FieldDecl")
                    .filter_map(|f| {
                        let (name, ty, desugared) = decl(&f.rest)?;
                        Some(Param { name, ty, desugared, attrs: msl_attrs(f), stage_in: false })
                    })
                    .collect();
                if let Some(c) = regex!(r" (?:struct|class) (\w+) definition").captures(&k.rest) {
                    p.records.insert(format!("{ns}{}", &c[1]), fields.clone());
                    p.records.entry(c[1].to_string()).or_insert(fields);
                } else if regex!(r" (?:struct|class) definition").is_match(&k.rest) {
                    *last_anon = Some(fields);
                }
            }
            "TypedefDecl" | "TypeAliasDecl" => {
                if let Some((tn, tt, td)) = decl(&k.rest) {
                    if !tn.is_empty() {
                        if last_anon.is_some() && (tt.starts_with(&format!("struct {tn}")) || tt.starts_with(&format!("class {tn}")) || tt.contains("(unnamed")) {
                            let f = last_anon.take().unwrap();
                            p.records.insert(format!("{ns}{tn}"), f.clone());
                            p.records.entry(tn.clone()).or_insert(f);
                        }
                        p.typedefs.insert(format!("{ns}{tn}"), td.clone());
                        p.typedefs.entry(tn).or_insert(td);
                    }
                }
            }
            "FunctionTemplateDecl" => {
                let decls: Vec<&Node> = k.kids.iter().filter(|c| c.kind == "FunctionDecl").collect();
                let Some(pattern) = decls.first() else { continue };
                let Some(stage) = stage_of(pattern) else { continue };
                let Some((tname, _, _)) = decl(&pattern.rest) else { continue };
                let mine = instances.iter().filter(|i| i.template.rsplit("::").next() == Some(tname.as_str()));
                for inst in mine {
                    let Some(spec) = specs.get(&inst.host_name).and_then(|a| decls[1..].iter().find(|d| address(d) == Some(a.as_str()))) else { continue };
                    let Some((_, ty, _)) = decl(&spec.rest) else { continue };
                    p.functions.push(Function {
                        name: inst.host_name.clone(),
                        stage: stage.clone(),
                        ty,
                        params: params(spec),
                        attrs: msl_attrs(pattern),
                        ns: ns.to_string(),
                        call: format!("{}<{}>", inst.template, inst.args),
                    });
                }
            }
            "FunctionDecl" => {
                let Some(stage) = stage_of(k) else { continue };
                let Some((name, ty, _)) = decl(&k.rest) else { continue };
                p.functions.push(Function { name: name.clone(), stage, ty, params: params(k), attrs: msl_attrs(k), ns: ns.to_string(), call: name });
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instances_take_their_own_specialization_whatever_the_order_clang_made_them() {
        // `typedef decltype(f<64, half>)` made f<64, half> before the instance f<128, float>
        let dump = r#"TranslationUnitDecl 0x10 <<invalid sloc>> <invalid sloc>
|-FunctionTemplateDecl 0x1 <t.cpp:4:1, col:76> col:56 f
| |-FunctionDecl 0x2 <col:51, col:76> col:56 f 'void (T *)'
| | `-AnnotateAttr 0x3 <col:1> "msl.stage:kernel"
| |-FunctionDecl 0x4 <col:51, col:76> col:56 referenced f 'void (half *)'
| | `-ParmVarDecl 0x5 <col:10> col:16 p 'half *'
| `-FunctionDecl 0x6 <col:51, col:76> col:56 f 'void (float *)'
|   `-ParmVarDecl 0x7 <col:10> col:16 p 'float *'
`-StaticAssertDecl 0x8 <line:9:1, col:60> col:1
  |-BinaryOperator 0x9 <col:15, col:40> 'bool' '>'
  | |-UnaryExprOrTypeTraitExpr 0xa <col:15, col:36> 'unsigned long' sizeof
  | | `-UnaryOperator 0xb <col:22, col:35> 'void (*)(float *)' prefix '&' cannot overflow
  | |   `-DeclRefExpr 0xc <col:23, col:35> 'void (float *)' lvalue Function 0x6 'f' 'void (float *)' (FunctionTemplate 0x1 'f') non_odr_use_unevaluated
  | `-IntegerLiteral 0xd <col:40> 'int' 0
  `-StringLiteral 0xe <col:43> 'const char[19]' lvalue "msl.instance:f_128"
"#;
        let inst = Instance { host_name: "f_128".into(), template: "f".into(), args: "128, float".into() };
        let p = program(dump, &[inst]);
        assert_eq!(p.functions.len(), 1);
        assert_eq!(p.functions[0].name, "f_128");
        assert_eq!(p.functions[0].call, "f<128, float>");
        assert_eq!(p.functions[0].params[0].ty, "float *");
    }
}
