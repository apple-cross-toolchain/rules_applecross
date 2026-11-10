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
    walk(&tree(text), "", instances, &mut p, &mut last_anon);
    p
}

fn walk(n: &Node, ns: &str, instances: &[Instance], p: &mut Program, last_anon: &mut Option<Vec<Param>>) {
    for k in &n.kids {
        match k.kind.as_str() {
            "TranslationUnitDecl" | "LinkageSpecDecl" => walk(k, ns, instances, p, last_anon),
            "NamespaceDecl" => {
                let inner = match regex!(r" (\w+)$").captures(&k.rest) {
                    Some(c) => format!("{ns}{}::", &c[1]),
                    None => ns.to_string(),
                };
                walk(k, &inner, instances, p, last_anon)
            }
            "VarDecl" if k.rest.contains("__msl_fcinit_") => {
                if let Some((vn, vt, _)) = decl(&k.rest) {
                    if let Some(c) = regex!(r"^__msl_fcinit_\d+_(\w+)$").captures(&vn) {
                        p.constant_types.insert(c[1].to_string(), vt);
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
                let mine: Vec<&Instance> = instances.iter().filter(|i| i.template.rsplit("::").next() == Some(tname.as_str())).collect();
                for (spec, inst) in decls[1..].iter().zip(mine) {
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
