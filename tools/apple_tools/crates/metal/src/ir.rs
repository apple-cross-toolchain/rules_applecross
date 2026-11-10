//! Passes over LLVM IR text between clang and AIR.

use crate::ast::Function;
use crate::text::{map_functions, replace_all, split_args};
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
    let pat = regex!(r"(?m)^(\s*)(?:(%[\w.]+) = )?(?:tail )?call (?:(?:spir_func|noundef|fast) )*(?:i32|float|void) @__msl_atomic_(\w+?)_([usn])_(i32|f32)\((?:i32|float)\* (?:noundef )?(?:nonnull )?(%[\w.]+), (.*?)\)(?: #\d+)?$");
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
    let call_re = regex!(r#"(?m)^(\s*)(%[\w.]+ = )?((?:tail )?call [^@]*)@"(air\.[^"]*%AS%[^"]*)"\((.*)\)(?: #\d+)?$"#);
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
