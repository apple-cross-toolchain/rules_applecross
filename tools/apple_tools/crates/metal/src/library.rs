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

/// `ir`: each module's text (llvm-dis); `bitcode`: the matching `.air` contents.
pub fn link(modules: &[(String, Vec<u8>)]) -> Result<Vec<u8>, String> {
    let mut lib = Library { header: [0; 6], modules: Vec::new(), functions: Vec::new() };
    for (mi, (ir, bitcode)) in modules.iter().enumerate() {
        let triple = regex!(r#"target triple = "([^"]*)""#).captures(ir).ok_or("a module without a target triple")?[1].to_string();
        let target = Target::from_triple(&triple)?;
        if mi == 0 {
            lib.header = target.header();
        }
        let (air, lang) = target.versions();
        lib.modules.push(wrap(bitcode));
        let m = Module::parse(ir);
        // files in order; within one, kernels, then vertex, then fragment functions (as Xcode orders them)
        for (stage, kind) in [("kernel", 2u8), ("vertex", 0), ("fragment", 1)] {
            for r in m.named.get(&format!("air.{stage}")).into_iter().flatten() {
                let items = m.items(r);
                let name = regex!(r"@([\w.]+)").captures(items.first().map(String::as_str).unwrap_or("")).map(|c| c[1].to_string()).ok_or("a stage function without a name")?;
                let descs = items.get(2).map(|d| m.items(d)).unwrap_or_default();
                lib.functions.push(Function { name: name.clone(), kind, module: mi, air, lang, public: public(ir, &m, &name, stage, &descs) });
            }
        }
    }
    Ok(lib.write())
}
