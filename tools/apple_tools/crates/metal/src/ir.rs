//! Passes over LLVM IR text between clang and AIR.

use crate::ast::Function;
use crate::text::{balanced, map_functions, replace_all, split_args};
use crate::types::Types;
use regex::{Captures, Regex};
use std::collections::BTreeSet;
use std::sync::LazyLock;

macro_rules! regex {
    ($r:expr) => {{
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new($r).unwrap());
        &*RE
    }};
}

/// AIR functions return small structs directly; spir64 lowers them to sret.
fn return_type(cname: &str) -> Option<&'static str> {
    let r = regex!(r"(__ret_\w+)").captures(cname)?;
    Some(match &r[1] {
        "__ret_v4f32" => "{ <4 x float>, i8 }",
        "__ret_v4f16" => "{ <4 x half>, i8 }",
        "__ret_f32" => "{ float, i8 }",
        "__ret_v4u32" | "__ret_v4i32" => "{ <4 x i32>, i8 }",
        _ => return None,
    })
}

/// Calls to air.* functions through sret become direct calls whose result is stored.
pub fn direct_returns(ir: &str) -> String {
    let ir = replace_all(ir, regex!(r#"declare (?:dso_local )?(?:spir_func )?void (@"?air\.[\w.]+"?)\((%"?struct\.[\w:.]*__ret_\w+"?\*) sret\([^)]*\)(?: align \d+)?(?:, )?"#), |c| {
        format!("declare {} {}(", return_type(&c[2]).unwrap_or("void"), &c[1])
    });
    let n = std::cell::Cell::new(0);
    replace_all(&ir, regex!(r#"(?m)^(\s*)(?:tail )?call (?:spir_func )?void (@"?air\.[\w.]+"?)\((%"?struct\.[\w:.]*__ret_\w+"?\*) sret\([^)]*\)(?: align \d+)? (%[\w.]+)(?:, )?(.*)$"#), |c| {
        n.set(n.get() + 1);
        let k = n.get();
        let t = return_type(&c[3]).unwrap_or("void");
        let ind = &c[1];
        format!(
            "{ind}%__ret{k} = call {t} {}({}\n{ind}%__retp{k} = bitcast {} {} to {t}*\n{ind}store {t} %__ret{k}, {t}* %__retp{k}",
            &c[2], &c[5], &c[3], &c[4]
        )
    })
}

/// spir64 returns the packed output struct through sret; AIR returns it directly.
/// Runs on instnamer'd IR, so dropping the sret parameter renumbers nothing.
pub fn direct_struct_returns(ir: &str, types: &Types, functions: &[Function]) -> Result<String, String> {
    let mut ir = ir.to_string();
    for f in functions {
        if types.ret_fields(f).is_none() {
            continue;
        }
        let pat = Regex::new(&format!(
            r#"(?m)^define (?:dso_local )?(?:spir_func )?void @{n}\((%"?struct\.[^*]*__ret_{n}"?)\* noalias sret\([^)]*\) align (\d+) (%[\w.]+)(?:, )?(.*)$"#,
            n = regex::escape(&f.name)
        ))
        .unwrap();
        let c = pat.captures(&ir).ok_or_else(|| format!("no sret entry for {}", f.name))?;
        let (sty, align, sret, rest) = (c[1].to_string(), c[2].to_string(), c[3].to_string(), c[4].to_string());
        let start = c.get(0).unwrap().start();
        let tre = Regex::new(&format!(r"(?m)^{} = type <?(\{{.*\}})>?$", regex::escape(&sty))).unwrap();
        let lit = format!("<{}>", &tre.captures(&ir).ok_or("no return struct type")?[1]);
        let line_end = start + ir[start..].find('\n').unwrap();
        let end = start + ir[start..].find("\n}\n").unwrap();
        let mut body = replace_all(&ir[line_end..end], regex!(r"(?m)^(\s*)ret void$"), |r| {
            let i = &r[1];
            format!("{i}%__rv.p = bitcast {sty}* {sret} to {lit}*\n{i}%__rv = load {lit}, {lit}* %__rv.p, align {align}\n{i}ret {lit} %__rv")
        });
        let alloca = format!("\n  {sret} = alloca {sty}, align {align}");
        body = match regex!(r"^\n([\w.]+:[^\n]*)").captures(&body) {
            Some(lm) => {
                let l = lm.get(0).unwrap().end();
                format!("\n{}{}{}", &lm[1], alloca, &body[l..])
            }
            None => alloca + &body,
        };
        ir = format!("{}define {lit} @{}({rest}{body}{}", &ir[..start], f.name, &ir[end..]);
    }
    Ok(ir)
}

/// (index, name, LLVM type) of each function constant the module declares.
pub type ConstantEntry = (u32, String, String);

/// Give function constants AIR's shape: initializer globals, static init.
pub fn function_constants(ir: &str) -> (String, Vec<ConstantEntry>) {
    let mut ir = ir.to_string();
    let mut entries = Vec::new();
    let caps: Vec<(String, u32, String, String, String)> = regex!(r"(?m)^@__msl_fcinit_(\d+)_(\w+) = external (?:dso_local )?addrspace\(2\) global (.*?), align (\d+)$")
        .captures_iter(&ir)
        .map(|c| (c[0].to_string(), c[1].parse().unwrap(), c[2].to_string(), c[3].to_string(), c[4].to_string()))
        .collect();
    for (whole, idx, name, lty, align) in caps {
        ir = ir.replace(
            &whole,
            &format!("@__msl_fcinit_{idx}_{name} = linkonce_odr hidden addrspace(2) externally_initialized constant {lty} undef, section \"air.fc_initializer\", align {align}"),
        );
        entries.push((idx, name, lty));
    }
    if !entries.is_empty() {
        ir = replace_all(&ir, regex!(r"(?m)^(define internal (?:spir_func )?void @_GLOBAL__sub_I_[^{]*?)(\s*\{)$"), |c| format!("{} section \"air.static_init\"{}", &c[1], &c[2]));
        for (_, name, _) in &entries {
            let re = Regex::new(&format!(r"(?m)^(@_ZL{}{} = internal (?:unnamed_addr )?addrspace\(2\) global (?:<[^>]*>|\[[^\]]*\]|\S+)) .*?(, align \d+)$", name.len(), regex::escape(name))).unwrap();
            ir = replace_all(&ir, &re, |c| format!("{} undef{}", &c[1], &c[2]));
        }
    }
    (ir, entries)
}

/// Define the constexpr samplers' state words.
pub fn samplers(ir: &str, words: &[u64], air: (u32, u32)) -> String {
    let mut ir = ir.to_string();
    for (k, w) in words.iter().enumerate() {
        let init = if air >= (2, 7) { format!("[2 x i64] [i64 {}, i64 0]", *w as i64) } else { format!("i64 {}", (*w | 1 << 63) as i64) };
        let re = Regex::new(&format!(r"(?m)^@__msl_sampler_{k} = external (?:dso_local )?addrspace\(2\) global .*$")).unwrap();
        ir = replace_all(&ir, &re, |_| format!("@__msl_sampler_{k} = internal addrspace(2) constant {init}, align 8"));
    }
    ir
}

fn global_type(rest: &str) -> &str {
    let mut depth = 0i32;
    for (i, ch) in rest.char_indices() {
        match ch {
            '<' | '[' | '{' | '(' => depth += 1,
            '>' | ']' | '}' | ')' => depth -= 1,
            ' ' if depth == 0 => return &rest[..i],
            _ => {}
        }
    }
    rest
}

/// Before opt: inline everything into the entry points (address spaces are
/// inferred within one function), and put program-scope constants in address space 2.
pub fn prepare_flat(ir: &str, entries: &[String]) -> String {
    let mut ir = replace_all(ir, regex!(r"(?m)^(define [^{]*\)[^{#]*)(#\d+)?\s*\{$"), |c| {
        let is_entry = entries.iter().any(|e| c[1].contains(&format!("@{e}(")));
        if is_entry {
            c[0].to_string()
        } else {
            format!("{}alwaysinline {} {{", &c[1], c.get(2).map(|m| m.as_str()).unwrap_or(""))
        }
    });
    let globals: Vec<(String, String, String, String, String, String, String)> = regex!(r"(?m)^@([\w.$]+) = ((?:internal |private |dso_local )*)((?:unnamed_addr |local_unnamed_addr )*)(constant|global) (.*?), align (\d+)$")
        .captures_iter(&ir)
        .map(|c| (c[0].to_string(), c[1].to_string(), c[2].to_string(), c[3].to_string(), c[4].to_string(), c[5].to_string(), c[6].to_string()))
        .collect();
    for (whole, name, link, ua, kind, rest, align) in globals {
        if name.starts_with("llvm.") || name.starts_with("__msl_") || rest.starts_with("addrspace(") {
            continue;
        }
        let ty = global_type(&rest).to_string();
        ir = ir.replace(&whole, &format!("@{name} = {link}{ua}addrspace(2) {kind} {rest}, align {align}"));
        let re = Regex::new(&format!(r"{}\*((?: (?:noundef|nonnull|readonly|nocapture|align \d+|dereferenceable\(\d+\)))*) @{}\b", regex::escape(&ty), regex::escape(&name))).unwrap();
        ir = replace_all(&ir, &re, |u| format!("{ty}*{} addrspacecast ({ty} addrspace(2)* @{name} to {ty}*)", &u[1]));
    }
    ir
}

/// Follow GEPs, bitcasts and casts back to the address space a flat pointer came from.
pub fn address_space_of(body: &str, v: &str, depth: u32) -> Option<String> {
    if depth > 32 || !v.starts_with('%') {
        return None;
    }
    let re = Regex::new(&format!(r"(?m)^\s*{} = (.*)$", regex::escape(v))).unwrap();
    let rhs = re.captures(body)?[1].to_string();
    if let Some(c) = regex!(r"^addrspacecast .*? addrspace\((\d)\)\* (%[\w.]+) to ").captures(&rhs) {
        return Some(c[1].to_string());
    }
    if let Some(c) = regex!(r"^getelementptr (?:inbounds )?.*?, .*?\* (%[\w.]+)").captures(&rhs).or_else(|| regex!(r"^bitcast .*?\* (%[\w.]+) to ").captures(&rhs)) {
        return address_space_of(body, &c[1], depth + 1);
    }
    if let Some(c) = regex!(r"^(?:phi|select) .*?(%[\w.]+)").captures(&rhs) {
        return address_space_of(body, &c[1], depth + 1);
    }
    None
}

/// `__msl_atomic_<op>_<u|s|n>_<i32|f32>` placeholders -> `air.atomic.<global|local>.<op>[.<u|s>].<i32|f32>`
/// on the pointer's address space (scope 2 device, 1 threadgroup). Observed shapes: ops (p, v, order,
/// scope, 0, false); load (p, order, scope, 0, false); cmpxchg.weak (p, expected*, desired, order,
/// order, scope, 0, false).
pub fn resolve_atomics(ir: &str) -> Result<String, String> {
    let mut decls: BTreeSet<(String, String, String, String)> = BTreeSet::new();
    let mut err = None;
    let mut n = 0;
    let pat = regex!(r"(?m)^(\s*)(?:(%[\w.]+) = )?(?:tail )?call (?:(?:spir_func|noundef|fast) )*(?:i32|float|void) @__msl_atomic_(\w+?)_([usn])_(i32|f32)\((?:i32|float)\* (?:noundef )?(?:nonnull )?(%[\w.]+), (.*?)\)(?: #\d+)?(?:, ![\w.]+ !\d+)*$");
    let out = map_functions(ir, |body| {
        replace_all(body, pat, |c: &Captures| {
            let (indent, op, sg, ty, ptr) = (&c[1], &c[3], &c[4], &c[5], &c[6]);
            let lt = if ty == "f32" { "float" } else { "i32" };
            let args: Vec<String> = split_args(&c[7]).iter().map(|x| regex!(r"^(?:i32|float)\*? (?:noundef )?(?:nonnull )?").replace(x.trim(), "").to_string()).collect();
            let space = address_space_of(body, ptr, 0);
            let (kind, scope) = match space.as_deref() {
                Some("1") => ("global", 2),
                Some("3") => ("local", 1),
                _ => {
                    err = Some(format!("atomic on a pointer of unknown address space: {ptr}"));
                    return c[0].to_string();
                }
            };
            let space = space.unwrap();
            n += 1;
            let p = format!("%__atomp{n}");
            let pre = format!("{indent}{p} = addrspacecast {lt}* {ptr} to {lt} addrspace({space})*\n");
            let res = c.get(2).map(|m| format!("{} = ", m.as_str())).unwrap_or_default();
            let call = if op == "cmpxchg" {
                let name = format!("air.atomic.{kind}.cmpxchg.weak.i32");
                decls.insert((name.clone(), space.clone(), op.to_string(), lt.to_string()));
                format!("call i32 @{name}(i32 addrspace({space})* {p}, i32* {}, i32 {}, i32 {}, i32 {}, i32 {scope}, i32 0, i1 false)", args[0], args[1], args[2], args[3])
            } else {
                let name = format!("air.atomic.{kind}.{op}{}.{ty}", if sg == "n" { String::new() } else { format!(".{sg}") });
                decls.insert((name.clone(), space.clone(), op.to_string(), lt.to_string()));
                if op == "load" {
                    format!("call {lt} @{name}({lt} addrspace({space})* {p}, i32 {}, i32 {scope}, i32 0, i1 false)", args[0])
                } else {
                    format!("call {} @{name}({lt} addrspace({space})* {p}, {lt} {}, i32 {}, i32 {scope}, i32 0, i1 false)", if op == "store" { "void" } else { lt }, args[0], args[1])
                }
            };
            format!("{pre}{indent}{res}{call}")
        })
    });
    if let Some(e) = err {
        return Err(e);
    }
    let mut out = regex!(r"(?m)^declare [^\n]*@__msl_atomic_\w+\([^\n]*\n").replace_all(&out, "").to_string();
    for (name, space, op, lt) in decls {
        out += &match op.as_str() {
            "load" => format!("\ndeclare {lt} @{name}({lt} addrspace({space})*, i32, i32, i32, i1)\n"),
            "cmpxchg" => format!("\ndeclare i32 @{name}(i32 addrspace({space})*, i32*, i32, i32, i32, i32, i32, i1)\n"),
            _ => format!("\ndeclare {} @{name}({lt} addrspace({space})*, {lt}, i32, i32, i32, i1)\n", if op == "store" { "void" } else { &lt }),
        };
    }
    Ok(out)
}

/// Calls to `@"air....%AS%..."` name the address space of their first flat pointer
/// argument (simdgroup matrix load/store .p1f32 / .p3f32): trace it, cast the
/// pointer and fill the name in.
pub fn resolve_as_markers(ir: &str) -> Result<String, String> {
    let decls: Vec<(String, String)> = regex!(r#"(?m)^declare [^\n]*@"(air\.[^"]*%AS%[^"]*)"\([^\n]*$"#).captures_iter(ir).map(|c| (c[1].to_string(), c[0].to_string())).collect();
    if decls.is_empty() {
        return Ok(ir.to_string());
    }
    let mut used: BTreeSet<(String, String, usize, String)> = BTreeSet::new();
    let mut err = None;
    let mut n = 0;
    let call_re = regex!(r#"(?m)^(\s*)(%[\w.]+ = )?((?:tail )?call [^@]*)@"(air\.[^"]*%AS%[^"]*)"\((.*)\)(?: #\d+)?(?:, ![\w.]+ !\d+)*$"#);
    let ptr_re = regex!(r"^(.*?)\* (?:(?:noundef|nonnull|nocapture|readonly|writeonly|align \d+) )*(%[\w.]+)$");
    let mut out = map_functions(ir, |body| {
        replace_all(body, call_re, |c: &Captures| {
            let (indent, lhs, head, name) = (&c[1], c.get(2).map(|m| m.as_str()).unwrap_or(""), &c[3], &c[4]);
            let mut parts = split_args(&c[5]);
            for k in 0..parts.len() {
                let Some(pm) = ptr_re.captures(&parts[k]) else { continue };
                if pm[1].contains("addrspace") {
                    continue;
                }
                let Some(space) = address_space_of(body, &pm[2], 0) else {
                    err = Some(format!("{name} on a pointer of unknown address space"));
                    return c[0].to_string();
                };
                n += 1;
                let ty = pm[1].trim().to_string();
                let cast = format!("{indent}%__asp{n} = addrspacecast {ty}* {} to {ty} addrspace({space})*\n", &pm[2]);
                parts[k] = format!("{ty} addrspace({space})* %__asp{n}");
                let real = name.replace("%AS%", &space);
                used.insert((name.to_string(), real.clone(), k, space));
                return format!("{cast}{indent}{lhs}{head}@\"{real}\"({})", parts.join(", "));
            }
            err = Some(format!("{name}: no pointer argument"));
            c[0].to_string()
        })
    });
    if let Some(e) = err {
        return Err(e);
    }
    for (_, d) in &decls {
        out = out.replace(&format!("{d}\n"), "");
    }
    for (name, real, k, space) in used {
        let d = &decls.iter().find(|(n, _)| *n == name).unwrap().1;
        let (head, rest) = d.split_once(&format!("@\"{name}\"(")).unwrap();
        let mut params = split_args(&rest[..rest.rfind(')').unwrap()]);
        if let Some(star) = params[k].find('*') {
            params[k] = format!("{} addrspace({space})*{}", &params[k][..star], &params[k][star + 1..]);
        }
        out += &format!("\n{head}@\"{real}\"({})\n", params.join(", "));
    }
    Ok(out)
}

/// Xcode's scoped-alias metadata comes from inlining an entry point whose
/// resource parameters are noalias. So each entry point becomes a call to an
/// internal copy with noalias pointer parameters (named for their kind and
/// index, which `alias_scopes` reads back), and opt's inliner adds the scopes.
pub fn noalias_entries(ir: &str, entries: &[String]) -> String {
    let mut ir = ir.to_string();
    for e in entries {
        let needle = format!("@{e}(");
        let Some(at) = ir.match_indices(&needle).map(|(i, _)| i).find(|&i| ir[..i].rfind('\n').map_or(0, |x| x + 1) == ir[..i].rfind("define ").unwrap_or(usize::MAX)) else { continue };
        let line = ir[..at].rfind('\n').map_or(0, |x| x + 1);
        let open = at + needle.len() - 1;
        let close = balanced(ir.as_bytes(), open, b'(', b')');
        let brace = close + ir[close..].find('{').unwrap();
        let end = brace + ir[brace..].find("\n}\n").unwrap() + 3;
        let (prefix, sig, tail, body) = (&ir[line..at], &ir[open + 1..close - 1], &ir[close..brace], &ir[brace..end]);
        let ret = regex!(r"\b(?:define|dso_local|internal|spir_func|spir_kernel|noundef|zeroext|signext|local_unnamed_addr) ").replace_all(prefix, "").trim().to_string();
        let mut inner_params = Vec::new();
        let mut call_args = Vec::new();
        let mut body = body.to_string();
        for (k, p) in split_args(sig).iter().enumerate() {
            let p = p.trim();
            let (head, name) = p.rsplit_once(' ').unwrap();
            let ty = regex!(r#"\s+(?:noalias|nocapture|noundef|readonly|writeonly|readnone|nonnull|zeroext|signext|"[^"]*"|align \d+|dereferenceable(?:_or_null)?\(\d+\))"#).replace_all(head, "").to_string();
            call_args.push(format!("{ty} {name}"));
            if !ty.ends_with('*') {
                inner_params.push(p.to_string());
                continue;
            }
            let kind = if ty.contains("_sampler_t") {
                "smp"
            } else if regex!(r"%struct\._(?:texture|depth)_").is_match(&ty) {
                "tex"
            } else {
                "arg"
            };
            let new = format!("%__air_{kind}_{k}");
            body = Regex::new(&format!(r"{}\b", regex::escape(name))).unwrap().replace_all(&body, new.as_str()).to_string();
            inner_params.push(format!("{head} noalias {new}"));
        }
        let cc = if prefix.contains("spir_func ") { "spir_func " } else { "" };
        let call = format!("call {cc}{ret} @__msl_entry.{e}({})", call_args.join(", "));
        let outer_body = if ret == "void" { format!("{{\nentry:\n  {call}\n  ret void\n}}\n") } else { format!("{{\nentry:\n  %r = {call}\n  ret {ret} %r\n}}\n") };
        let inner_prefix = prefix.replacen("define ", "define internal ", 1).replace("dso_local ", "");
        let inner_tail = regex!(r"(#\d+)\s*$").replace(tail, "alwaysinline $1 ");
        let inner = format!("{inner_prefix}@__msl_entry.{e}({}){inner_tail}{body}", inner_params.join(", "));
        let outer = format!("{prefix}@{e}({sig}){tail}{outer_body}");
        ir = format!("{}{outer}\n{inner}{}", &ir[..line], &ir[end..]);
    }
    ir
}

/// The inliner's scope declarations: Xcode's modules have none (and they would
/// keep opt from inferring argmemonly).
pub fn drop_scope_decls(ir: &str) -> String {
    let ir = regex!(r"(?m)^\s*(?:tail )?call void @llvm\.experimental\.noalias\.scope\.decl\(metadata !\d+\)(?: #\d+)?\n").replace_all(ir, "");
    regex!(r"(?m)^(?:; Function Attrs: .*\n)?declare void @llvm\.experimental\.noalias\.scope\.decl\(metadata\).*\n").replace_all(&ir, "").to_string()
}

/// After inlining: name the scopes as Xcode does (air-alias-scopes(<fn>),
/// air-alias-scope-arg(<k>)), one scope for all textures and one for all
/// samplers, and an access's noalias list without its own scopes.
pub fn alias_scopes(ir: &str) -> String {
    let dom = regex!(r#"(?m)^!(\d+) = distinct !\{!\d+, !"__msl_entry\.([\w.$]+)"\}$"#);
    let scope = regex!(r#"(?m)^!(\d+) = distinct !\{!\d+, !(\d+), !"__msl_entry\.([\w.$]+): %__air_(tex|smp|arg)_(\d+)"\}$"#);
    let mut ir = replace_all(ir, dom, |c| format!("!{} = distinct !{{!{}, !\"air-alias-scopes({})\"}}", &c[1], &c[1], &c[2]));
    // each scope node -> the node standing for its group
    let mut rep: std::collections::HashMap<String, String> = Default::default();
    let mut group_node: std::collections::HashMap<(String, String), String> = Default::default();
    for c in scope.captures_iter(&ir.clone()) {
        let label = match &c[4] {
            "tex" => "textures".to_string(),
            "smp" => "samplers".to_string(),
            _ => format!("arg({})", &c[5]),
        };
        let key = (c[2].to_string(), label.clone());
        match group_node.get(&key) {
            Some(n) => {
                rep.insert(c[1].to_string(), n.clone());
            }
            None => {
                group_node.insert(key, c[1].to_string());
                rep.insert(c[1].to_string(), c[1].to_string());
                ir = ir.replace(&c[0], &format!("!{} = distinct !{{!{}, !{}, !\"air-alias-scope-{label}\"}}", &c[1], &c[1], &c[2]));
            }
        }
    }
    if rep.is_empty() {
        return ir;
    }
    let lists: std::collections::HashMap<String, Vec<String>> = regex!(r"(?m)^!(\d+) = !\{(!\d+(?:, !\d+)*)\}$")
        .captures_iter(&ir)
        .map(|c| (c[1].to_string(), c[2].split(", ").map(|x| x[1..].to_string()).collect()))
        .collect();
    let mut next = regex!(r"(?m)^!(\d+) = ").captures_iter(&ir).map(|c| c[1].parse::<usize>().unwrap()).max().unwrap_or(0) + 1;
    let mut made: std::collections::HashMap<Vec<String>, usize> = Default::default();
    let mut new_nodes = String::new();
    let mut node = |ids: Vec<String>| -> usize {
        *made.entry(ids.clone()).or_insert_with(|| {
            let n = next;
            next += 1;
            new_nodes += &format!("!{n} = !{{{}}}\n", ids.iter().map(|i| format!("!{i}")).collect::<Vec<_>>().join(", "));
            n
        })
    };
    let map = |l: &str| -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        // scopes of other inlined functions' noalias parameters go
        for i in lists.get(l).into_iter().flatten() {
            let Some(r) = rep.get(i).cloned() else { continue };
            if !out.contains(&r) {
                out.push(r);
            }
        }
        out
    };
    let sc = regex!(r", !alias\.scope !(\d+)");
    let na = regex!(r", !noalias !(\d+)");
    let lines: Vec<String> = ir
        .lines()
        .map(|line| {
            let (a, n) = (sc.captures(line), na.captures(line));
            if a.is_none() && n.is_none() {
                return line.to_string();
            }
            let mut out = na.replace(&sc.replace(line, ""), "").to_string();
            // Xcode leaves an access through a global (a constexpr sampler, a function constant), or a call without pointers, unannotated
            match regex!(r"\bcall [^@]*@[\w.$\x22%]+\((.*)\)").captures(line) {
                Some(args) if args[1].contains('@') || !args[1].contains('*') => return out,
                None if line.contains('@') => return out,
                _ => {}
            }
            let s = a.map(|c| map(&c[1])).unwrap_or_default();
            let n: Vec<String> = n.map(|c| map(&c[1])).unwrap_or_default().into_iter().filter(|x| !s.contains(x)).collect();
            if !s.is_empty() {
                out += &format!(", !alias.scope !{}", node(s));
            }
            if !n.is_empty() {
                out += &format!(", !noalias !{}", node(n));
            }
            out
        })
        .collect();
    let ir = lines.join("\n");
    ir.trim_end().to_string() + "\n" + &new_nodes
}

/// clang 15's TBAA gives a vector access the type `omnipotent char`; Xcode's
/// gives it the element's scalar type (float4 -> "float").
pub fn vector_tbaa(ir: &str) -> String {
    let node = regex!(r#"(?m)^!(\d+) = !\{!"omnipotent char", !(\d+), i64 0\}$"#);
    let Some(c) = node.captures(ir) else { return ir.to_string() };
    let char_id = c[1].to_string();
    let char_tag: BTreeSet<String> = Regex::new(&format!(r"(?m)^!(\d+) = !\{{!{char_id}, !{char_id}, i64 0\}}$")).unwrap().captures_iter(ir).map(|c| c[1].to_string()).collect();
    if char_tag.is_empty() {
        return ir.to_string();
    }
    let mut next = regex!(r"(?m)^!(\d+) = ").captures_iter(ir).map(|c| c[1].parse::<usize>().unwrap()).max().unwrap_or(0) + 1;
    let mut tags: std::collections::HashMap<&str, usize> = Default::default();
    let mut nodes = String::new();
    let access = regex!(r"^\s+(?:(?:%[\w.]+ = )?load (?:volatile )?<\d+ x (\w+)>, |store (?:volatile )?<\d+ x (\w+)> ).*, !tbaa !(\d+)");
    let lines: Vec<String> = ir
        .lines()
        .map(|line| {
            let Some(c) = access.captures(line) else { return line.to_string() };
            let elem = c.get(1).or(c.get(2)).unwrap().as_str();
            let name = match elem {
                "float" => "float",
                "half" => "half",
                "i32" => "int",
                "i16" => "short",
                "i64" => "long",
                _ => return line.to_string(),
            };
            if !char_tag.contains(&c[3]) {
                return line.to_string();
            }
            let tag = *tags.entry(name).or_insert_with(|| {
                let (t, g) = (next, next + 1);
                next += 2;
                nodes += &format!("!{t} = !{{!\"{name}\", !{char_id}, i64 0}}\n!{g} = !{{!{t}, !{t}, i64 0}}\n");
                g
            });
            line.replacen(&format!("!tbaa !{}", &c[3]), &format!("!tbaa !{tag}"), 1)
        })
        .collect();
    lines.join("\n") + "\n" + &nodes
}

/// Xcode lowers conversions between integers and floats, and between float
/// vectors, to air.convert calls after optimisation (a scalar fpext or fptrunc
/// stays); the declarations follow the module's others.
pub fn air_conversions(ir: &str) -> String {
    convert(ir, false)
}

/// What Xcode's front end converts as it emits the user's code: scalars, in the
/// user's own functions (not our stdlib's, whose conversions fold once inlined).
pub fn user_scalar_conversions(ir: &str) -> String {
    convert(ir, true)
}

fn convert(ir: &str, user_scalars: bool) -> String {
    fn name(t: &str) -> Option<String> {
        let (n, e) = match regex!(r"^<(\d+) x (\w+)>$").captures(t) {
            Some(c) => (Some(c[1].to_string()), c[2].to_string()),
            None => (None, t.to_string()),
        };
        let e = match e.as_str() {
            "float" => "f32".to_string(),
            "half" => "f16".to_string(),
            "bfloat" => "bf16".to_string(),
            i if regex!(r"^i\d+$").is_match(i) => i.to_string(),
            _ => return None,
        };
        Some(n.map(|n| format!("v{n}{e}")).unwrap_or(e))
    }
    let conv = regex!(r"^(\s+)(%[\w.]+) = (sitofp|uitofp|fptosi|fptoui|fpext|fptrunc) (<\d+ x \w+>|\w+) (%[\w.]+) to (<\d+ x \w+>|\w+)$");
    let mut decls: Vec<String> = Vec::new();
    let mut user = !user_scalars;
    let lines: Vec<String> = ir
        .lines()
        .map(|l| {
            if user_scalars && l.starts_with("define ") {
                user = !regex!(r"@(?:_ZN5metal|_ZNK5metal|_ZZN5metal|__msl|_ZN11__air_entry)").is_match(l);
            }
            let Some(c) = conv.captures(l) else { return l.to_string() };
            let (op, src, v, dst) = (&c[3], &c[4], &c[5], &c[6]);
            if matches!(op, "fpext" | "fptrunc") && !src.starts_with('<') || !user || user_scalars && (src.starts_with('<') || dst.starts_with('<')) {
                return l.to_string();
            }
            let (Some(sn), Some(dn)) = (name(src), name(dst)) else { return l.to_string() };
            let (sk, dk) = match op {
                "sitofp" => ("s", "f"),
                "uitofp" => ("u", "f"),
                "fptosi" => ("f", "s"),
                "fptoui" => ("f", "u"),
                _ => ("f", "f"),
            };
            // a bool converts as the int it promotes to
            if op == "uitofp" && src == "i1" {
                let f = format!("air.convert.{dk}.{dn}.s.i32");
                let decl = format!("declare {dst} @{f}(i32)");
                if !decls.contains(&decl) {
                    decls.push(decl);
                }
                return format!("{0}{1}.i = zext i1 {v} to i32\n{0}{1} = tail call fast {dst} @{f}(i32 {1}.i)", &c[1], &c[2]);
            }
            let f = format!("air.convert.{dk}.{dn}.{sk}.{sn}");
            let decl = format!("declare {dst} @{f}({src})");
            if !decls.contains(&decl) {
                decls.push(decl);
            }
            let fast = if dk == "f" { "fast " } else { "" };
            format!("{}{} = tail call {fast}{dst} @{f}({src} {v})", &c[1], &c[2])
        })
        .collect();
    let mut ir = lines.join("\n") + "\n";
    let existing: BTreeSet<String> = regex!(r"(?m)^declare .*? @([\w.]+)\(").captures_iter(&ir).map(|c| c[1].to_string()).collect();
    let mut new: Vec<&String> = decls.iter().filter(|d| !existing.contains(regex!(r"@([\w.]+)\(").captures(d).unwrap().get(1).unwrap().as_str())).collect();
    // Xcode's front end converts scalars itself; vectors come later
    new.sort_by_key(|d| d.starts_with("declare <"));
    if !new.is_empty() {
        // after the last function
        let at = ir.rfind("\n}\n").map(|i| i + 3).into_iter().chain(ir.rfind("\ndeclare ").map(|i| i + 1 + ir[i + 1..].find('\n').unwrap() + 1)).max().unwrap_or(ir.len());
        let text: String = new.iter().map(|d| format!("\n{d}\n")).collect();
        ir.insert_str(at, &text);
    }
    ir
}
