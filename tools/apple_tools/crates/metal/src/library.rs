//! `metallib`: link AIR modules into a metallib. Everything comes from each
//! module itself (its text form, from llvm-dis): the function list from
//! !air.kernel/vertex/fragment, and the public metadata (VATT/VATY, CNST) from
//! its descriptors and function constants.

use crate::container::{block, tagged, wrap, Function, Library};
use crate::target::Target;
use crate::text::{find_define, split_args};
use crate::types::mtl_data_type;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

macro_rules! regex {
    ($r:expr) => {{
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new($r).unwrap());
        &*RE
    }};
}

struct Module {
    nodes: HashMap<String, String>,
    named: HashMap<String, Vec<String>>,
}

impl Module {
    fn parse(ir: &str) -> Module {
        let nodes = regex!(r"(?m)^(![0-9]+) = (?:distinct )?(!\{.*\})$").captures_iter(ir).map(|c| (c[1].to_string(), c[2].to_string())).collect();
        let named = regex!(r"(?m)^!([a-z][\w.]*) = !\{(.*)\}$")
            .captures_iter(ir)
            .map(|c| (c[1].to_string(), regex!(r"![0-9]+").find_iter(&c[2]).map(|m| m.as_str().to_string()).collect()))
            .collect();
        Module { nodes, named }
    }
    fn node(&self, r: &str) -> &str {
        self.nodes.get(r).map(String::as_str).unwrap_or("")
    }
    /// The elements of `!{a, b, ...}`.
    fn items(&self, r: &str) -> Vec<String> {
        let n = self.node(r);
        split_args(n.strip_prefix("!{").and_then(|x| x.strip_suffix('}')).unwrap_or(""))
    }
}

fn unquote(s: &str) -> &str {
    s.trim().trim_start_matches('!').trim_matches('"')
}

/// One function's public metadata, as Xcode's metallib writes it.
fn public(ir: &str, m: &Module, name: &str, stage: &str, descs: &[String]) -> Vec<u8> {
    let (sig, body) = find_define(ir, name).unwrap_or(("", ""));
    let pnames: Vec<String> = split_args(sig).iter().map(|a| a.rsplit(' ').next().unwrap().to_string()).collect();
    let used = |k: usize| pnames.get(k).map(|n| Regex::new(&format!(r"{}\b", regex::escape(n))).unwrap().is_match(body)).unwrap_or(true);
    let mut tags = Vec::new();
    if stage == "vertex" {
        let (mut vatt, mut vaty, mut count) = (Vec::new(), Vec::new(), 0u16);
        for d in descs {
            let items = m.items(d);
            if items.get(1).map(|x| unquote(x)) != Some("air.vertex_input") {
                continue;
            }
            let arg: usize = items[0].trim_start_matches("i32 ").parse().unwrap_or(0);
            let loc: u16 = items[3].trim_start_matches("i32 ").parse().unwrap_or(0);
            let field = |key: &str| items.iter().position(|x| unquote(x) == key).and_then(|i| items.get(i + 1)).map(|x| unquote(x).to_string()).unwrap_or_default();
            vatt.extend_from_slice(field("air.arg_name").as_bytes());
            vatt.push(0);
            vatt.extend_from_slice(&(loc | if used(arg) { 0x8000 } else { 0 }).to_le_bytes());
            vaty.push(mtl_data_type(&field("air.arg_type_name")));
            count += 1;
        }
        if count > 0 {
            let mut a = count.to_le_bytes().to_vec();
            a.extend(vatt);
            let mut t = count.to_le_bytes().to_vec();
            t.extend(vaty);
            tags.push(tagged(b"VATT", &a));
            tags.push(tagged(b"VATY", &t));
        }
    }
    // function constants this function reads, in index order
    let mut cnst = Vec::new();
    for r in m.named.get("air.function_constants").into_iter().flatten() {
        let items = m.items(r);
        if items.len() < 4 {
            continue;
        }
        let init = regex!(r"@([\w.]+)").captures(&items[0]).map(|c| c[1].to_string()).unwrap_or_default();
        let (ty, cname) = (unquote(&items[1]).to_string(), unquote(&items[2]).to_string());
        let idx: u16 = items[3].trim_start_matches("i32 ").parse().unwrap_or(0);
        let copy = Regex::new(&format!(r"@_ZL{}{}\b", cname.len(), regex::escape(&cname))).unwrap();
        if !copy.is_match(body) {
            continue;
        }
        let checked = ir.lines().any(|l| l.contains("air.is_function_constant_defined(") && l.contains(&format!("@{init}")));
        cnst.push((idx, cname, ty, checked));
    }
    if !cnst.is_empty() {
        cnst.sort_by_key(|c| c.0);
        let mut data = (cnst.len() as u16).to_le_bytes().to_vec();
        for (idx, cname, ty, checked) in cnst {
            data.extend_from_slice(cname.as_bytes());
            data.push(0);
            data.push(mtl_data_type(&ty));
            data.extend_from_slice(&idx.to_le_bytes());
            data.push(if checked { 0 } else { 1 });
        }
        tags.push(tagged(b"CNST", &data));
    }
    block(&tags)
}

/// One function's module, as metallib writes it: the function and what it
/// refers to, its stage list naming it alone, and its name as the source file.
pub fn extract(ir: &str, name: &str) -> String {
    // top-level entities: (symbol defined, text including its attribute comment)
    let mut header = Vec::new();
    let mut entities: Vec<(Option<String>, String)> = Vec::new();
    let mut named = Vec::new();
    let mut rest = Vec::new();
    let mut lines = ir.lines().peekable();
    let mut comment = String::new();
    while let Some(l) = lines.next() {
        if l.starts_with("; Function Attrs:") {
            comment = format!("{l}\n");
        } else if l.starts_with("define ") {
            let mut t = std::mem::take(&mut comment) + l + "\n";
            for b in lines.by_ref() {
                t += b;
                t += "\n";
                if b == "}" {
                    break;
                }
            }
            entities.push((regex!(r"@([-\w.$]+|\x22[^\x22]*\x22)\(").captures(l).map(|c| c[1].to_string()), t));
        } else if l.starts_with("declare ") {
            entities.push((regex!(r"@([-\w.$]+|\x22[^\x22]*\x22)\(").captures(l).map(|c| c[1].to_string()), std::mem::take(&mut comment) + l + "\n"));
        } else if l.starts_with('@') {
            entities.push((regex!(r"^@([-\w.$]+|\x22[^\x22]*\x22) =").captures(l).map(|c| c[1].to_string()), format!("{l}\n")));
        } else if regex!(r"^![a-z]").is_match(l) {
            named.push(l.to_string());
        } else if l.starts_with('!') || l.starts_with("attributes ") {
            rest.push(l.to_string());
        } else if l.starts_with("source_filename") {
            header.push(format!("source_filename = \"{name}\""));
        } else if !l.starts_with("; ModuleID") {
            header.push(l.to_string());
        }
    }
    // the stage list naming this function alone; no source file; Xcode's order
    let m = Module::parse(ir);
    let mut keep_named = Vec::new();
    for key in ["llvm.module.flags", "llvm.ident", "air.version", "air.language_version", "air.compile_options"] {
        keep_named.extend(named.iter().filter(|l| l.starts_with(&format!("!{key} ="))).cloned());
    }
    for stage in ["kernel", "vertex", "fragment"] {
        for r in m.named.get(&format!("air.{stage}")).into_iter().flatten() {
            if regex!(r"@([\w.$]+)").captures(m.node(r)).map(|c| c[1] == *name).unwrap_or(false) {
                keep_named.push(format!("!air.{stage} = !{{{r}}}"));
            }
        }
    }
    for l in &named {
        let key = l.split(' ').next().unwrap_or("");
        if !keep_named.iter().any(|k| k.split(' ').next() == Some(key)) && !regex!(r"^!(air\.(kernel|vertex|fragment|source_file_name|version|language_version|compile_options)|llvm\.(module\.flags|ident)) ").is_match(l) {
            keep_named.push(l.clone());
        }
    }
    // other entry points go, then whatever nothing kept refers to
    let entries: Vec<String> = ["kernel", "vertex", "fragment"].iter().flat_map(|st| m.named.get(&format!("air.{st}")).cloned().unwrap_or_default()).filter_map(|r| regex!(r"@([\w.$]+)").captures(m.node(&r)).map(|c| c[1].to_string())).collect();
    let mut kept: Vec<bool> = entities.iter().map(|(sym, _)| sym.as_deref().map(|s| s == name || !entries.iter().any(|e| e == s)).unwrap_or(true)).collect();
    // what the code kept refers to (metadata naming a global doesn't keep it)
    loop {
        let text: String = entities.iter().zip(&kept).filter(|(_, k)| **k).map(|(e, _)| e.1.as_str()).collect::<String>();
        let mut changed = false;
        for (i, (sym, t)) in entities.iter().enumerate() {
            let Some(sym) = sym else { continue };
            // llvm.global_ctors runs the function constants' initialisers: nothing refers to it
            if !kept[i] || sym == name || sym.starts_with("llvm.") {
                continue;
            }
            let re = Regex::new(&format!(r"@{}(?:[^-\w.$]|$)", regex::escape(sym))).unwrap();
            let own = re.find_iter(t).count();
            if re.find_iter(&text).count() <= own {
                kept[i] = false;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    // named metadata keeps the entries whose globals the code kept (a sampler it uses), and goes if none
    let globals_kept: Vec<&str> = entities.iter().zip(&kept).filter(|(e, k)| **k && e.0.is_some()).map(|(e, _)| e.0.as_deref().unwrap()).collect();
    let names_kept = |r: &str| -> bool {
        let mut seen = std::collections::HashSet::new();
        let mut todo = vec![r.to_string()];
        while let Some(r) = todo.pop() {
            if !seen.insert(r.clone()) {
                continue;
            }
            let n = m.node(&r);
            if regex!(r"@([-\w.$]+)").captures_iter(n).any(|c| !globals_kept.contains(&&c[1])) {
                return false;
            }
            todo.extend(regex!(r"![0-9]+").find_iter(n).map(|x| x.as_str().to_string()));
        }
        true
    };
    let keep_named: Vec<String> = keep_named
        .into_iter()
        .filter_map(|l| {
            let c = regex!(r"^(![\w.]+) = !\{(.*)\}$").captures(&l)?;
            let refs: Vec<&str> = regex!(r"![0-9]+").find_iter(&c[2]).map(|x| x.as_str()).filter(|r| names_kept(r)).collect();
            (!refs.is_empty() || c[2].is_empty()).then(|| format!("{} = !{{{}}}", &c[1], refs.join(", ")))
        })
        .collect();
    let mut out = header.join("\n") + "\n\n";
    // metallib declares what the function calls in reverse order of first use
    let body: String = entities.iter().zip(&kept).filter(|(e, k)| **k && e.1.starts_with("define") || **k && e.1.contains("\ndefine ")).map(|(e, _)| e.1.as_str()).collect();
    let first_use = |sym: &str| body.find(&format!("@{sym}(")).unwrap_or(usize::MAX);
    let mut decls: Vec<&(Option<String>, String)> = entities.iter().zip(&kept).filter(|(e, k)| **k && e.1.lines().any(|l| l.starts_with("declare "))).map(|(e, _)| e).collect();
    decls.sort_by_key(|e| std::cmp::Reverse(first_use(e.0.as_deref().unwrap_or(""))));
    // functions: static initialisers (function constants), then the rest in order
    let defines: Vec<&String> = entities.iter().zip(&kept).filter(|(e, k)| **k && e.1.lines().any(|l| l.starts_with("define "))).map(|(e, _)| &e.1).collect();
    let (init, other): (Vec<&String>, Vec<&String>) = defines.iter().partition(|t| t.contains("section \"air.static_init\""));
    let functions: Vec<&String> = init.into_iter().chain(other).collect();
    // globals: in reverse order of discovery through those functions (the unreferenced, as llvm.global_ctors, last)
    let globals: Vec<&(Option<String>, String)> = entities.iter().zip(&kept).filter(|(e, k)| **k && e.1.starts_with('@')).map(|(e, _)| e).collect();
    let text: String = functions.iter().map(|t| t.as_str()).collect();
    let found = |g: &str| regex!(r"@([-\w.$]+)").find_iter(&text).position(|m| &m.as_str()[1..] == g).unwrap_or(usize::MAX);
    let mut globals = globals;
    globals.sort_by_key(|g| std::cmp::Reverse(found(g.0.as_deref().unwrap_or(""))));
    for (_, t) in globals {
        // metallib internalises the function constants' initialisers
        out += &t.replace("linkonce_odr hidden local_unnamed_addr ", "internal unnamed_addr ");
        out += "\n";
    }
    for t in functions {
        out += t;
        out += "\n";
    }
    for (_, t) in decls {
        out += t;
        out += "\n";
    }
    out += &rest.iter().filter(|l| l.starts_with("attributes ")).cloned().collect::<Vec<_>>().join("\n");
    out += "\n\n";
    out += &keep_named.join("\n");
    out += "\n\n";
    // numbered metadata naming what went (another entry point's descriptor), and what refers to that
    let gone: Vec<Regex> = entities.iter().zip(&kept).filter(|(e, k)| !**k && e.0.is_some()).map(|(e, _)| Regex::new(&format!(r"@{}(?:[^-\w.$]|$)", regex::escape(e.0.as_ref().unwrap()))).unwrap()).collect();
    let mut nodes: Vec<&String> = rest.iter().filter(|l| l.starts_with('!')).collect();
    let mut dropped: Vec<String> = Vec::new();
    loop {
        let before = nodes.len();
        nodes.retain(|l| {
            let id = l.split(' ').next().unwrap_or("").to_string();
            let refs_gone = gone.iter().any(|g| g.is_match(l)) || regex!(r"![0-9]+").find_iter(&l[id.len()..]).any(|x| dropped.contains(&x.as_str().to_string()));
            if refs_gone {
                dropped.push(id);
            }
            !refs_gone
        });
        if nodes.len() == before {
            break;
        }
    }
    // metallib marks each function constant the function keeps as used
    let fc: Vec<String> = keep_named.iter().filter(|l| l.starts_with("!air.function_constants ")).flat_map(|l| regex!(r"![0-9]+").find_iter(l).map(|x| x.as_str().to_string()).collect::<Vec<_>>()).collect();
    let nodes: Vec<String> = nodes
        .iter()
        .map(|l| match l.split_once(" = ") {
            Some((id, n)) if fc.iter().any(|f| f == id) => format!("{id} = {}, i1 true}}", n.trim_end_matches('}')),
            _ => l.to_string(),
        })
        .collect();
    out += &nodes.join("\n");
    out + "\n"
}

/// `ir`: each module's text (llvm-dis); `assemble`: one function's module text -> its bitcode.
pub fn link(modules: &[String], assemble: &dyn Fn(&str) -> Result<Vec<u8>, String>) -> Result<Vec<u8>, String> {
    let mut lib = Library { header: [0; 6], modules: Vec::new(), functions: Vec::new() };
    for (mi, ir) in modules.iter().enumerate() {
        let triple = regex!(r#"target triple = "([^"]*)""#).captures(ir).ok_or("a module without a target triple")?[1].to_string();
        let target = Target::from_triple(&triple)?;
        if mi == 0 {
            lib.header = target.header();
        }
        let (air, lang) = target.versions();
        let m = Module::parse(ir);
        // files in order; within one, kernels, then vertex, then fragment functions (as Xcode orders them)
        for (stage, kind) in [("kernel", 2u8), ("vertex", 0), ("fragment", 1)] {
            for r in m.named.get(&format!("air.{stage}")).into_iter().flatten() {
                let items = m.items(r);
                let name = regex!(r"@([\w.]+)").captures(items.first().map(String::as_str).unwrap_or("")).map(|c| c[1].to_string()).ok_or("a stage function without a name")?;
                let descs = items.get(2).map(|d| m.items(d)).unwrap_or_default();
                // each function gets a module of its own
                lib.modules.push(wrap(&assemble(&extract(ir, &name))?));
                lib.functions.push(Function { name: name.clone(), kind, module: lib.modules.len() - 1, air, lang, public: public(ir, &m, &name, stage, &descs) });
            }
        }
    }
    Ok(lib.write())
}
