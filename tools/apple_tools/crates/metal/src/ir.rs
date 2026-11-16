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

/// The handle a record of one opaque `_..._t` pointer wraps: our stdlib's
/// textures and samplers, which Xcode's frontend has as builtin handle types.
pub fn wrapped_handle(body: &str) -> Option<&str> {
    regex!(r"^\{ (%struct\._\w+_t addrspace\(\d\)\*) \}$").captures(body.trim()).map(|c| c.get(1).unwrap().as_str())
}

/// LLVM numbers a clashing struct name (`struct.metal::matrix.2`) from one
/// counter for the whole module. Our stdlib's records that Xcode's headers
/// lack (helpers with a `__` name, wrapped handles) clash too: each user
/// type's number drops by those clashes numbered before it.
pub fn renumber_types(ir: &str) -> String {
    let helpers: Vec<u64> = regex!(r#"(?m)^%"?(?:struct|class|union)\.(.*?)\.(\d+)"? = type (.*)$"#)
        .captures_iter(ir)
        .filter(|c| c[1].split("::").any(|p| p.starts_with("__")) || wrapped_handle(&c[3]).is_some())
        .filter_map(|c| c[2].parse().ok())
        .collect();
    if helpers.is_empty() {
        return ir.to_string();
    }
    replace_all(ir, regex!(r#"%("?)((?:struct|class|union)\.[^"\s,*()]*?)\.(\d+)("?)([\s,*()]|$)"#), |c| {
        let n: u64 = c[3].parse().unwrap();
        // the counter is the module's, so a helper's number is its own
        let n = if helpers.contains(&n) || c[1] != c[4] { n } else { n - helpers.iter().filter(|&&h| h < n).count() as u64 };
        format!("%{}{}.{n}{}{}", &c[1], &c[2], &c[4], &c[5])
    })
}

/// Xcode's frontend freezes the value a SIMD-group or quad-group function moves
/// between threads (shuffles and broadcasts; not reductions or scans) before
/// the call. clang 15's C++ has no freeze; inside our stdlib's wrapper it would
/// freeze a `noundef` parameter, which instcombine drops, so it goes in once
/// the wrappers are inlined.
pub fn freeze_moved_values(ir: &str) -> String {
    let n = std::cell::Cell::new(0);
    replace_all(ir, regex!(r#"(?m)^(\s*)((?:%[\w.]+ = )?(?:tail )?call .*? @"?air\.(?:simd|quad)_(?:shuffle|broadcast)\w*\.[\w.]+"?)\((.*)\)(.*)$"#), |c| {
        let mut args = split_args(&c[3]);
        let Some(first) = args.first().cloned() else { return c[0].to_string() };
        let Some((head, value)) = first.rsplit_once(' ') else { return c[0].to_string() };
        let ty = regex!(r" (?:noundef|zeroext|signext)\b").replace_all(head, "");
        n.set(n.get() + 1);
        let frozen = format!("%__fz{}", n.get());
        args[0] = format!("{head} {frozen}");
        format!("{ind}{frozen} = freeze {ty} {value}\n{ind}{}({}){}", &c[2], args.join(", "), &c[4], ind = &c[1])
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

/// The function constants' globals in Xcode's order: each constant (in the source's order)
/// with its initializer right after it, and after them the program-scope constants the
/// static initializer sets (`has_opt = is_function_constant_defined(...)`).
pub fn order_constants(ir: &str, order: &[String]) -> String {
    let lines: Vec<&str> = ir.lines().collect();
    let global = |l: &str| regex!(r#"^@("?[\w.$]+"?) = "#).captures(l).map(|c| c[1].to_string());
    let set_by_init: BTreeSet<String> = regex!(r#"(?ms)^define [^\n]*@(?:_GLOBAL__sub_I_|__cxx_global_var_init)[^\n]*\{\n(.*?)\n\}"#)
        .captures_iter(ir)
        .flat_map(|c| regex!(r"\bstore [^,]+, [^,]*\* @([\w.$]+)").captures_iter(&c[1]).map(|s| s[1].to_string()).collect::<Vec<_>>())
        .collect();
    let mut block = Vec::new();
    for name in order {
        let own = format!("_ZL{}{}", name.len(), name);
        for l in &lines {
            if global(l).as_deref() == Some(own.as_str()) {
                block.push(*l);
            }
        }
        for l in &lines {
            if global(l).is_some_and(|g| regex!(r"^__msl_fcinit_\d+_(\w+)$").captures(&g).is_some_and(|c| &c[1] == name)) {
                block.push(*l);
            }
        }
    }
    let late: Vec<&str> = lines.iter().copied().filter(|l| global(l).is_some_and(|g| set_by_init.contains(&g) && !block.contains(l))).collect();
    if block.is_empty() {
        return ir.to_string();
    }
    let Some(first) = lines.iter().position(|l| block.contains(l) || late.contains(l)) else { return ir.to_string() };
    let mut out: Vec<&str> = Vec::with_capacity(lines.len());
    for (i, l) in lines.iter().enumerate() {
        if i == first {
            out.extend(&block);
            out.extend(&late);
        }
        if !block.contains(l) && !late.contains(l) {
            out.push(l);
        }
    }
    let ir = out.join("\n") + if ir.ends_with('\n') { "\n" } else { "" };
    static_inits_last(&ir)
}

/// The program-scope constants' initialisers (`__cxx_global_var_init`), and what only they
/// call, just before the static initializer that runs them: Xcode's front end emits them
/// after the file, so what they call (`air.is_function_constant_defined`) is declared last.
fn static_inits_last(ir: &str) -> String {
    let fns = crate::text::functions(ir);
    let name_of = |s: usize| regex!(r#"^define [^@\n]*@("?[\w.$]+"?)\("#).captures(&ir[s..]).map(|c| c[1].to_string()).unwrap_or_default();
    let Some(&(ss, _)) = fns.iter().find(|(s, _)| name_of(*s).starts_with("_GLOBAL__sub_I_")) else { return ir.to_string() };
    let inits: Vec<(usize, usize)> = fns.iter().copied().filter(|(s, _)| name_of(*s).starts_with("__cxx_global_var_init")).collect();
    if inits.is_empty() || inits.iter().any(|(s, _)| *s > ss) {
        return ir.to_string();
    }
    let init_text: String = inits.iter().map(|(s, e)| &ir[*s..*e]).collect();
    // declarations only the initialisers call
    let mut rest = ir.to_string();
    for (s, e) in inits.iter().rev() {
        rest.replace_range(*s..*e, "");
    }
    let mut decls = Vec::new();
    for c in regex!(r#"(?m)^(?:; Function Attrs: [^\n]*\n)?declare [^@\n]*@("?[\w.$]+"?)\([^\n]*\n"#).captures_iter(ir) {
        let used = Regex::new(&format!(r"@{}\(", regex::escape(&c[1]))).unwrap();
        if used.is_match(&init_text) && used.find_iter(&rest).count() == 1 {
            decls.push(c[0].to_string());
        }
    }
    for d in &decls {
        rest = rest.replacen(d.as_str(), "", 1);
    }
    let at = rest.find("define").and_then(|_| {
        let s = crate::text::functions(&rest).into_iter().find(|(s, _)| regex!(r#"^define [^@\n]*@_GLOBAL__sub_I_"#).is_match(&rest[*s..]))?.0;
        Some(rest[..s].rfind("\n; Function Attrs:").filter(|p| !rest[p + 1..s].contains("\n\n")).map_or(s, |p| p + 1))
    });
    let Some(at) = at else { return ir.to_string() };
    let moved: String = inits.iter().map(|(s, e)| format!("{}\n\n", &ir[*s..*e])).collect::<String>() + &decls.iter().map(|d| format!("{d}\n")).collect::<String>();
    rest.insert_str(at, &moved);
    rest
}

/// Define the constexpr samplers' state words: one global per distinct state, as
/// Xcode defines them (the first sampler with a state stands for the others).
pub fn samplers(ir: &str, words: &[u64], air: (u32, u32)) -> String {
    let first: Vec<usize> = words.iter().enumerate().map(|(k, w)| words.iter().position(|x| x == w).unwrap_or(k)).collect();
    let ir = regex!(r"(?m)^@__msl_sampler_(\d+) = external [^\n]*\n").replace_all(ir, |c: &Captures| match c[1].parse::<usize>() {
        Ok(k) if first.get(k).is_some_and(|f| *f != k) => String::new(),
        _ => c[0].to_string(),
    });
    let mut ir = regex!(r"@__msl_sampler_(\d+)\b").replace_all(&ir, |c: &Captures| match c[1].parse::<usize>() {
        Ok(k) => format!("@__msl_sampler_{}", first.get(k).copied().unwrap_or(k)),
        _ => c[0].to_string(),
    }).to_string();
    for (k, w) in words.iter().enumerate() {
        if first[k] != k {
            continue;
        }
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
pub fn prepare_flat(ir: &str, entries: &[String], stages: &[String]) -> String {
    let mut ir = replace_all(ir, regex!(r"(?m)^(define [^{]*\)[^{#]*)(#\d+)?\s*\{$"), |c| {
        let is_entry = entries.iter().any(|e| c[1].contains(&format!("@{e}(")));
        // the user's functions are inlined as Xcode's inliner decides (their stage functions always)
        let name = regex!(r#"@("?[\w.$]+"?)\("#).captures(&c[1]).map(|n| n[1].to_string()).unwrap_or_default();
        if crate::abi::is_user(&name) && !stages.contains(&name) {
            return c[0].to_string();
        }
        // the user's [[noinline]] stays a call
        let noinline = c.get(2).is_some_and(|g| regex!(r"(?m)^attributes #(\d+) = \{ (.*) \}$").captures_iter(ir).any(|a| a[1] == g.as_str()[1..] && a[2].split(' ').any(|x| x == "noinline")));
        if is_entry || noinline {
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

/// Tag each call to a user's function with the function it's written in
/// (`home` maps our layers around an entry point's body to the entry point),
/// for `nounwind_inlined_calls`.
pub fn tag_user_calls(ir: &str, home: &std::collections::HashMap<String, String>) -> String {
    let defined: BTreeSet<String> = regex!(r#"(?m)^define [^\n]*?@("?[\w.$]+"?)\("#).captures_iter(ir).map(|c| c[1].to_string()).collect();
    map_functions(ir, |f| {
        let Some(name) = regex!(r#"@("?[\w.$]+"?)\("#).captures(f).map(|c| c[1].to_string()) else { return f.to_string() };
        let at = home.get(&name).cloned().unwrap_or(name);
        replace_all(f, regex!(r#"(?m)^(\s+(?:%[\w.]+ = )?(?:tail )?call [^@\n]*@("?[\w.$]+"?)\(.*)$"#), |c| {
            if crate::abi::is_user(&c[2]) && defined.contains(&c[2]) {
                format!("{}, !msl.home !{{!\"{}\"}}", &c[1], at.trim_matches('"'))
            } else {
                c[0].to_string()
            }
        })
    })
}

/// A call inlined from a function that can't unwind can't either: newer
/// LLVMs' inliner marks such calls nounwind (Xcode's does), LLVM 15's doesn't.
/// So a tagged call (`tag_user_calls`) now in a function other than the one
/// it was written in gets nounwind; the tags go.
pub fn nounwind_inlined_calls(ir: &str) -> String {
    let tags: BTreeSet<String> = regex!(r"!msl\.home !(\d+)").captures_iter(ir).map(|c| c[1].to_string()).collect();
    let homes: std::collections::HashMap<String, String> = regex!(r#"(?m)^!(\d+) = !\{!"([^"]*)"\}$"#).captures_iter(ir).filter(|c| tags.contains(&c[1])).map(|c| (c[1].to_string(), c[2].to_string())).collect();
    let groups: std::collections::HashMap<String, String> = regex!(r"(?m)^attributes #(\d+) = \{ (.*) \}$").captures_iter(ir).map(|c| (c[1].to_string(), c[2].to_string())).collect();
    let mut next = groups.keys().filter_map(|k| k.parse::<usize>().ok()).max().map_or(0, |m| m + 1);
    let mut added: std::collections::BTreeMap<String, usize> = Default::default();
    let ir = map_functions(ir, |f| {
        let name = regex!(r#"@("?[\w.$]+"?)\("#).captures(f).map(|c| c[1].trim_matches('"').to_string()).unwrap_or_default();
        replace_all(f, regex!(r"(?m)^(.*?)(?: #(\d+))?((?:, ![\w.]+ !\d+)*), !msl\.home !(\d+)((?:, ![\w.]+ !\d+)*)$"), |c| {
            let inlined = homes.get(&c[4]).is_some_and(|h| *h != name);
            let group = match c.get(2) {
                Some(g) if inlined && !groups.get(g.as_str()).is_some_and(|a| a.split(' ').any(|x| x == "nounwind")) => {
                    let n = *added.entry(g.as_str().to_string()).or_insert_with(|| {
                        next += 1;
                        next - 1
                    });
                    format!(" #{n}")
                }
                Some(g) => format!(" #{}", g.as_str()),
                None if inlined => " nounwind".to_string(),
                None => String::new(),
            };
            format!("{}{group}{}{}", &c[1], &c[3], &c[5])
        })
    });
    let mut ir = ir;
    for (old, new) in &added {
        let attrs = &groups[old];
        // nounwind goes in LLVM's order: after the enum attributes before it, before the strings
        let words: Vec<&str> = attrs.split(' ').collect();
        let at = words.iter().position(|w| w.starts_with('"') || *w > "nounwind").unwrap_or(words.len());
        let mut w = words.clone();
        w.insert(at, "nounwind");
        ir += &format!("\nattributes #{new} = {{ {} }}", w.join(" "));
    }
    // the tags' nodes go (nothing else refers to them)
    ir = regex!(r#"(?m)^!(\d+) = !\{!"[^"]*"\}\n"#).replace_all(&ir, |c: &Captures| if tags.contains(&c[1]) { String::new() } else { c[0].to_string() }).to_string();
    ir + "\n"
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
    let pat = regex!(r"(?m)^(\s*)(?:(%[\w.]+) = )?(?:tail )?call (?:(?:spir_func|noundef|fast|nnan|ninf|nsz|arcp|contract|afn|reassoc) )*(?:i32|float|void) @__msl_atomic_(\w+?)_([usn])_(i32|f32)\((?:i32|float)\* (?:noundef )?(?:nonnull )?(%[\w.]+), (.*?)\)(?: #\d+)?(?:, ![\w.]+ !\d+)*$");
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
                    // a float result keeps the call's fast-math flag, as Xcode's call has it
                    let fast = if lt == "float" && op != "store" && regex!(r"\bcall (?:\w+ )*?fast ").is_match(&c[0]) { "fast " } else { "" };
                    format!("call {fast}{} @{name}({lt} addrspace({space})* {p}, {lt} {}, i32 {}, i32 {scope}, i32 0, i1 false)", if op == "store" { "void" } else { lt }, args[0], args[1])
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

/// Only `keep` (and our vector types' members) stay `alwaysinline`, for a
/// step inlining what Xcode's frontend has no call for: the entry points'
/// stage functions (Xcode's entry point is the user's function), vector
/// construction and conversion and a bool vector's members (Xcode's frontend
/// builds them in place). The others' `alwaysinline` becomes `"msl.ai"` until
/// `restore_always_inline`.
pub fn only_frontend_inlines(ir: &str, keep: &BTreeSet<String>) -> String {
    let groups: std::collections::HashMap<String, String> = regex!(r"(?m)^attributes #(\d+) = \{ (.*) \}$").captures_iter(ir).map(|c| (c[1].to_string(), c[2].to_string())).collect();
    let mut next = groups.keys().filter_map(|k| k.parse::<usize>().ok()).max().map_or(0, |m| m + 1);
    let mut added: std::collections::BTreeMap<String, usize> = Default::default();
    let held = |a: &str| a.split(' ').map(|w| if w == "alwaysinline" { "\"msl.ai\"" } else { w }).collect::<Vec<_>>().join(" ");
    let out = replace_all(ir, regex!(r#"(?m)^(define [^\n]*?@("?[\w.$]+"?)\([^\n]*\))([^\n]*?)(?: #(\d+))?((?: comdat(?:\([^)\n]*\))?)?(?: align \d+)?)? \{$"#), |c| {
        let name = &c[2];
        if keep.contains(name) || regex!(r"^_ZNK?5metal(?:7__vpart|7__place|8__vchain|7__widen|9__boolvec|5__vec|14__convert_each|8__unpack)").is_match(name) {
            return c[0].to_string();
        }
        let group = match c.get(4) {
            Some(g) if groups.get(g.as_str()).is_some_and(|a| a.split(' ').any(|w| w == "alwaysinline")) => {
                let n = *added.entry(g.as_str().to_string()).or_insert_with(|| {
                    next += 1;
                    next - 1
                });
                format!(" #{n}")
            }
            Some(g) => format!(" #{}", g.as_str()),
            None => String::new(),
        };
        format!("{}{}{group}{} {{", &c[1], held(&c[3]), c.get(5).map_or("", |m| m.as_str()))
    });
    let mut out = out;
    for (old, new) in &added {
        out += &format!("\nattributes #{new} = {{ {} }}", held(&groups[old]));
    }
    out + "\n"
}

/// Each stage function's entry point, its name and the label of the block before its
/// return block (None: it returns from its first block).
pub fn return_places(ir: &str, home: &std::collections::HashMap<String, String>) -> Vec<(String, String, Option<String>)> {
    let mut out = Vec::new();
    for (stage, entry) in home {
        let Some((_, body)) = crate::text::find_define(ir, stage) else { continue };
        let mut labels: Vec<String> = vec!["entry".into()];
        let mut ret_block = None;
        for l in body.lines() {
            if let Some(c) = regex!(r"^([\w.$-]+):").captures(l) {
                labels.push(c[1].to_string());
            } else if l.trim_start().starts_with("ret ") {
                ret_block = Some(labels.len() - 1);
            }
        }
        if let Some(r) = ret_block {
            out.push((entry.clone(), stage.trim_matches('"').to_string(), if r == 0 { None } else { Some(labels[r - 1].clone()) }));
        }
    }
    out.sort();
    out
}

/// The inliner moves an inlined function's return block to after all its blocks
/// (`<function>.exit`); Xcode inlines no entry point, so the return stays where the
/// user's function had it: after the block that came before it (`return_places`).
pub fn place_inlined_returns(ir: &str, returns: &[(String, String, Option<String>)]) -> String {
    let mut ir = ir.to_string();
    for (entry, stage, before) in returns {
        let Some(before) = before else { continue };
        let Some(start) = ir.find(&format!("@{entry}(")).map(|i| ir[..i].rfind('\n').map_or(0, |n| n + 1)).filter(|&l| ir[l..].starts_with("define ")) else { continue };
        let end = start + ir[start..].find("\n}\n").map(|e| e + 1).unwrap_or(ir.len() - start);
        let f = ir[start..end].to_string();
        // the blocks: (label, text)
        let mut blocks: Vec<(String, String)> = Vec::new();
        let (head, rest) = f.split_once('\n').unwrap_or((&f, ""));
        for line in rest.split_inclusive('\n') {
            match regex!(r"^([\w.$-]+):").captures(line) {
                Some(c) => blocks.push((c[1].to_string(), line.to_string())),
                None if blocks.is_empty() => blocks.push(("entry".into(), line.to_string())),
                None => blocks.last_mut().unwrap().1.push_str(line),
            }
        }
        let exit = format!("{stage}.exit");
        let after = if before == "entry" { blocks.first().map(|b| b.0.clone()) } else { Some(format!("{before}.i")) };
        let (Some(from), Some(after)) = (blocks.iter().position(|b| b.0 == exit), after) else { continue };
        let Some(to) = blocks.iter().position(|b| b.0 == after) else { continue };
        if from == to + 1 {
            continue;
        }
        let b = blocks.remove(from);
        let to = blocks.iter().position(|x| x.0 == after).unwrap();
        blocks.insert(to + 1, b);
        let body: String = blocks.into_iter().map(|b| b.1).collect();
        ir.replace_range(start..end, &format!("{head}\n{body}"));
    }
    ir
}

/// The inliner puts an inlined function's allocas first in the entry block, before the
/// entry point's own (its parameters' spills, which Xcode's entry point doesn't have):
/// the entry point's go first, so SROA promotes the user's variables in their order.
pub fn own_allocas_first(ir: &str, entries: &[String]) -> String {
    map_functions(ir, |f| {
        let head = f.lines().next().unwrap_or("");
        let Some(name) = regex!(r#"@("?[\w.$]+"?)\("#).captures(head).map(|c| c[1].trim_matches('"').to_string()) else { return f.to_string() };
        if !entries.contains(&name) {
            return f.to_string();
        }
        let lines: Vec<&str> = f.split_inclusive('\n').collect();
        // the entry block's leading allocas
        let first = lines.iter().position(|l| regex!(r"^\s+%[\w.]+ = alloca ").is_match(l));
        let Some(first) = first else { return f.to_string() };
        let end = lines[first..].iter().position(|l| !regex!(r"^\s+%[\w.]+ = alloca ").is_match(l)).map_or(lines.len(), |n| first + n);
        let (theirs, ours): (Vec<&str>, Vec<&str>) = lines[first..end].iter().copied().partition(|l| regex!(r"^\s+%[\w.]+\.i\d* = alloca ").is_match(l));
        let mut out: String = lines[..first].concat();
        out += &ours.concat();
        out += &theirs.concat();
        out += &lines[end..].concat();
        out
    })
}

/// The lifetime markers the inliner adds for an inlined function's locals (clang's
/// own carry an attribute group): Xcode's entry point is the user's function, inlined
/// nowhere, so it has none of them, and they'd keep its return block apart.
pub fn drop_inliner_lifetimes(ir: &str) -> String {
    regex!(r"(?m)^\s+call void @llvm\.lifetime\.(?:start|end)\.p0i8\(i64 -?\d+, i8\* (?:nonnull )?%[\w.]+\)\n").replace_all(ir, "").to_string()
}

/// `alwaysinline` back where `only_frontend_inlines` held it.
pub fn restore_always_inline(ir: &str) -> String {
    ir.replace("\"msl.ai\"", "alwaysinline")
}

/// Xcode's AIR alias scopes: each entry point gets a domain
/// (air-alias-scopes(<fn>)), a scope for each pointer parameter
/// (air-alias-scope-arg(<index>)), one for all its textures and one for all its
/// samplers. A load, a store or an argmemonly call whose every pointer comes from
/// those parameters (through GEPs, casts, phis and selects) gets the scopes of
/// those it uses (!alias.scope) and of all the others (!noalias); an access
/// through anything else (a global, a local, a loaded pointer) gets none. Run
/// before O2's inliner walk (as Xcode's AIRAliasScopePropagation) and again on
/// what O2 left; the parameters have no `noalias`.
pub fn xcode_alias_scopes(ir: &str, entries: &[String]) -> String {
    let groups_of: std::collections::HashMap<String, String> = regex!(r"(?m)^attributes #(\d+) = \{ (.*) \}$").captures_iter(ir).map(|c| (c[1].to_string(), c[2].to_string())).collect();
    let attrs_of = |tail: &str| -> String {
        let mut a = tail.to_string();
        for g in regex!(r"#(\d+)").captures_iter(tail) {
            a += " ";
            a += groups_of.get(&g[1].to_string()).map(String::as_str).unwrap_or("");
        }
        a.split('"').next().unwrap_or("").to_string()
    };
    // each function's own attributes, by name
    let mut fn_attrs: std::collections::HashMap<String, String> = Default::default();
    for l in ir.lines().filter(|l| l.starts_with("declare ") || l.starts_with("define ")) {
        let Some(c) = regex!(r#"@("?[\w.$]+"?)\("#).captures(l) else { continue };
        let name = c[1].to_string();
        let open = c.get(0).unwrap().end() - 1;
        let close = balanced(l.as_bytes(), open, b'(', b')');
        fn_attrs.insert(name, attrs_of(&l[close..]));
    }
    let mut next = regex!(r"(?m)^!(\d+) = ").captures_iter(ir).map(|c| c[1].parse::<usize>().unwrap()).max().unwrap_or(0) + 1;
    let mut nodes = String::new();
    let ir = map_functions(ir, |f| {
        let head = f.lines().next().unwrap_or("");
        let Some(name) = regex!(r#"@("?[\w.$]+"?)\("#).captures(head).map(|c| c[1].trim_matches('"').to_string()) else { return f.to_string() };
        if !entries.contains(&name) {
            return f.to_string();
        }
        let open = head.find(&format!("{name}(")).unwrap() + name.len();
        let close = balanced(head.as_bytes(), open, b'(', b')');
        // the groups in parameter order, and each pointer parameter's group
        let mut groups: Vec<String> = Vec::new();
        let mut param_group: std::collections::HashMap<String, usize> = Default::default();
        for (k, p) in split_args(&head[open + 1..close - 1]).iter().enumerate() {
            let Some((ty, value)) = p.rsplit_once(' ') else { continue };
            if !ty.contains('*') {
                continue;
            }
            let label = if regex!(r"%struct\._(?:texture|depth)_").is_match(ty) {
                "textures".to_string()
            } else if ty.contains("%struct._sampler_t") {
                "samplers".to_string()
            } else {
                format!("arg({k})")
            };
            let g = groups.iter().position(|x| *x == label).unwrap_or_else(|| {
                groups.push(label);
                groups.len() - 1
            });
            param_group.insert(value.to_string(), g);
        }
        if groups.is_empty() {
            return f.to_string();
        }
        let domain = next;
        next += 1;
        nodes += &format!("!{domain} = distinct !{{!{domain}, !\"air-alias-scopes({name})\"}}\n");
        let scopes: Vec<usize> = groups
            .iter()
            .map(|g| {
                let n = next;
                next += 1;
                nodes += &format!("!{n} = distinct !{{!{n}, !{domain}, !\"air-alias-scope-{g}\"}}\n");
                n
            })
            .collect();
        let mut lists: std::collections::HashMap<Vec<usize>, usize> = Default::default();
        let mut list = |ids: Vec<usize>, nodes: &mut String, next: &mut usize| -> usize {
            *lists.entry(ids.clone()).or_insert_with(|| {
                let n = *next;
                *next += 1;
                *nodes += &format!("!{n} = !{{{}}}\n", ids.iter().map(|i| format!("!{}", scopes[*i])).collect::<Vec<_>>().join(", "));
                n
            })
        };
        let defs: std::collections::HashMap<String, String> = f.lines().filter_map(|l| regex!(r"^\s+(%[\w.]+) = (.*)$").captures(l).map(|c| (c[1].to_string(), c[2].to_string()))).collect();
        // the parameters' groups a pointer comes from, or None (anything else)
        fn trace(v: &str, defs: &std::collections::HashMap<String, String>, param_group: &std::collections::HashMap<String, usize>, out: &mut BTreeSet<usize>, depth: u32) -> bool {
            if depth > 64 {
                return false;
            }
            if let Some(g) = param_group.get(v) {
                out.insert(*g);
                return true;
            }
            let Some(d) = defs.get(v) else { return false };
            if let Some(c) = regex!(r"^getelementptr (?:inbounds )?[^,]+, [^,]*?\*\s+(%[\w.]+)").captures(d) {
                return trace(&c[1], defs, param_group, out, depth + 1);
            }
            if let Some(c) = regex!(r"^(?:bitcast|addrspacecast) .*?\*\s+(%[\w.]+) to ").captures(d) {
                return trace(&c[1], defs, param_group, out, depth + 1);
            }
            if d.starts_with("phi ") {
                let incoming: Vec<String> = regex!(r"\[ ([^,\]]+), %[\w.]+ \]").captures_iter(d).map(|c| c[1].trim().to_string()).collect();
                return !incoming.is_empty() && incoming.iter().all(|x| trace(x, defs, param_group, out, depth + 1));
            }
            if d.starts_with("select ") {
                let ops = split_args(d.strip_prefix("select ").unwrap_or(""));
                return ops.len() == 3 && ops[1..].iter().all(|o| o.rsplit_once(' ').is_some_and(|(_, x)| trace(x, defs, param_group, out, depth + 1)));
            }
            false
        }
        let pointer_operand = |p: &str| -> Option<String> { p.rsplit_once(' ').filter(|(t, _)| t.contains('*')).map(|(_, v)| v.to_string()) };
        let lines: Vec<String> = f
            .lines()
            .map(|l| {
                // Xcode's parameters have no noalias
                if l.starts_with("define ") {
                    let open = l.find(&format!("{name}(")).unwrap() + name.len();
                    let close = balanced(l.as_bytes(), open, b'(', b')');
                    return format!("{}{}{}", &l[..open], l[open..close].replace(" noalias", ""), &l[close..]);
                }
                let t = l.trim();
                let mut ptrs: Vec<String> = Vec::new();
                if let Some(c) = regex!(r"^(?:%[\w.]+ = )?load (?:volatile )?(.*)$").captures(t) {
                    let args = split_args(c[1].split(", align").next().unwrap_or(""));
                    if args.len() >= 2 {
                        ptrs.push(args[1].clone());
                    }
                } else if let Some(c) = regex!(r"^store (?:volatile )?(.*)$").captures(t) {
                    let args = split_args(c[1].split(", align").next().unwrap_or(""));
                    if args.len() >= 2 {
                        ptrs.push(args[1].clone());
                    }
                } else if let Some(c) = regex!(r#"^(?:%[\w.]+ = )?(?:tail )?call [^@]*@("?[\w.$]+"?)\("#).captures(t) {
                    let open = c.get(0).unwrap().end() - 1;
                    let close = balanced(t.as_bytes(), open, b'(', b')');
                    let attrs = format!("{} {}", fn_attrs.get(&c[1].to_string()).map(String::as_str).unwrap_or(""), attrs_of(t[close..].split(", !").next().unwrap_or("")));
                    if attrs.split(' ').any(|a| a == "readnone") || !attrs.split(' ').any(|a| a == "argmemonly") {
                        return l.to_string();
                    }
                    ptrs = split_args(&t[open + 1..close - 1]).into_iter().filter(|p| p.rsplit_once(' ').is_some_and(|(ty, _)| ty.contains('*'))).collect();
                } else {
                    return l.to_string();
                }
                if ptrs.is_empty() {
                    return l.to_string();
                }
                let mut used = BTreeSet::new();
                for p in &ptrs {
                    let Some(v) = pointer_operand(p) else { return l.to_string() };
                    if p.contains('@') || !trace(&v, &defs, &param_group, &mut used, 0) {
                        return l.to_string();
                    }
                }
                let scope: Vec<usize> = (0..groups.len()).filter(|g| used.contains(g)).collect();
                let noalias: Vec<usize> = (0..groups.len()).filter(|g| !used.contains(g)).collect();
                let mut out = l.to_string();
                out += &format!(", !alias.scope !{}", list(scope, &mut nodes, &mut next));
                if !noalias.is_empty() {
                    out += &format!(", !noalias !{}", list(noalias, &mut nodes, &mut next));
                }
                out
            })
            .collect();
        lines.join("\n") + if f.ends_with('\n') { "\n" } else { "" }
    });
    ir.trim_end().to_string() + "\n" + &nodes
}

/// Xcode's alias analysis knows address space 2 is constant: nothing writes it, so its
/// loads are as invariant to its optimiser as `!invariant.load` makes them to ours. For O2
/// only (`strip_invariant_loads` after).
pub fn constant_loads_invariant(ir: &str) -> String {
    let next = regex!(r"(?m)^!(\d+) = ").captures_iter(ir).map(|c| c[1].parse::<usize>().unwrap()).max().unwrap_or(0) + 1;
    let out = regex!(r"(?m)^(\s+%[\w.]+ = load [^,]+, [^,]* addrspace\(2\)\* [^,]+, align \d+)(.*)$").replace_all(ir, |c: &Captures| format!("{}, !invariant.load !{next}{}", &c[1], &c[2]));
    format!("{}\n!{next} = !{{}}\n", out.trim_end())
}

/// Each entry point where its layers are (the first of them defined): Xcode's front end
/// emits the entry point where the user wrote it, so what its body first refers to (the
/// lifetime markers, say) is declared after it, where ours, a wrapper after the file,
/// came after its stage function's.
pub fn entries_at_stages(ir: &str, home: &std::collections::HashMap<String, String>) -> String {
    let mut ir = ir.to_string();
    let entries: std::collections::BTreeSet<&String> = home.values().collect();
    for e in entries {
        let fns = crate::text::functions(&ir);
        let name_of = |(s, _): &(usize, usize)| regex!(r#"^define [^@\n]*@("?[\w.$]+"?)\("#).captures(&ir[*s..]).map(|c| c[1].to_string());
        let Some(&(es, ee)) = fns.iter().find(|f| name_of(f).as_deref() == Some(e.as_str())) else { continue };
        let Some(&(as_, _)) = fns.iter().find(|f| name_of(f).is_some_and(|n| home.get(&n) == Some(e))) else { continue };
        if as_ > es {
            continue;
        }
        // with the attributes comment above each, if any
        let with_comment = |s: usize| -> usize {
            let prev = ir[..s.saturating_sub(1)].rfind('\n').map_or(0, |i| i + 1);
            if s > 0 && ir[prev..s].starts_with("; Function Attrs:") { prev } else { s }
        };
        let (es, as_) = (with_comment(es), with_comment(as_));
        let block = format!("{}\n\n", &ir[es..ee]);
        let mut rest = String::with_capacity(ir.len());
        rest.push_str(&ir[..es]);
        rest.push_str(ir[ee..].trim_start_matches('\n'));
        rest.insert_str(as_, &block);
        ir = rest;
    }
    ir
}

/// Xcode's front end initialises a local aggregate from constants in place, where clang
/// 15 copies it from a private constant (`@__const.f.v`): element by element (vectors
/// whole, zeros too) or, when at most a quarter of more than 16 bytes isn't zero, a
/// memset and the non-zero scalars. Done here for arrays of scalars or vectors and for
/// a struct holding one (a matrix); other copies stay.
pub fn constant_inits(ir: &str) -> String {
    let consts: std::collections::HashMap<String, String> = regex!(r#"(?m)^@(__const\.[\w.$"]+) = private unnamed_addr constant (.*), align \d+$"#).captures_iter(ir).map(|c| (c[1].to_string(), c[2].to_string())).collect();
    if consts.is_empty() {
        return ir.to_string();
    }
    let mut n = 0;
    crate::text::map_functions(ir, |f| {
        let casts: std::collections::HashMap<&str, (&str, &str)> = regex!(r"(?m)^\s+(%[\w.]+) = bitcast (.+?)\* (%[\w.]+) to i8\*$").captures_iter(f).map(|c| (c.get(1).unwrap().as_str(), (c.get(2).unwrap().as_str(), c.get(3).unwrap().as_str()))).collect();
        regex!(r"(?m)^(\s+)call void @llvm\.memcpy\.p0i8\.p0i8\.i64\(i8\* align (\d+) (%[\w.]+), i8\* align \d+ bitcast \((.+?)\* @(__const\.[\w.$]+) to i8\*\), i64 (\d+), i1 false\)$").replace_all(f, |c: &Captures| {
            let keep = c[0].to_string();
            let (ind, align, dest, ty, name, size) = (&c[1], c[2].parse::<u64>().unwrap(), &c[3], &c[4], &c[5], c[6].parse::<u64>().unwrap());
            let (Some(init), Some((cty, var))) = (consts.get(name), casts.get(dest)) else { return keep };
            if *cty != ty {
                return keep;
            }
            n += 1;
            constant_init_stores(ind, align, dest, ty, var, size, init, n).unwrap_or(keep)
        })
        .to_string()
    })
}

/// (byte size, alignment) of a scalar or vector type, as AIR lays them out.
fn scalar_layout(t: &str) -> Option<(u64, u64)> {
    let s = |e: &str| match e {
        "i1" | "i8" => Some(1),
        "half" | "i16" => Some(2),
        "float" | "i32" => Some(4),
        "double" | "i64" => Some(8),
        _ => None,
    };
    if let Some(c) = regex!(r"^<(\d+) x (\w+)>$").captures(t) {
        let bytes = c[1].parse::<u64>().ok()? * s(&c[2])?;
        let size = bytes.next_power_of_two();
        return Some((size, size));
    }
    s(t).map(|b| (b, b))
}

/// The stores for `constant_inits`: None when the initialiser isn't one of its shapes.
#[allow(clippy::too_many_arguments)]
fn constant_init_stores(ind: &str, align: u64, dest: &str, ty: &str, var: &str, size: u64, init: &str, n: usize) -> Option<String> {
    // the array: the struct's one field, or the object itself
    let (array_ty, array_val, field) = match regex!(r"^(\[\d+ x [^\]]+\]) (.*)$").captures(init) {
        Some(c) if init.starts_with('[') => (c[1].to_string(), c[2].to_string(), false),
        _ => {
            let c = regex!(r"^\S+ \{ (\[\d+ x [^\]]+\]) (.*) \}$").captures(init)?;
            (c[1].to_string(), c[2].to_string(), true)
        }
    };
    let a = regex!(r"^\[(\d+) x (.+)\]$").captures(&array_ty)?;
    let (count, elem) = (a[1].parse::<usize>().ok()?, a[2].to_string());
    let (esize, ealign) = scalar_layout(&elem)?;
    let lane_ty = regex!(r"^<\d+ x (\w+)>$").captures(&elem).map(|c| c[1].to_string());
    let (lane_size, _) = scalar_layout(lane_ty.as_deref().unwrap_or(&elem))?;
    // each element's value, and its lanes' values
    let values: Vec<String> = if array_val == "zeroinitializer" {
        vec!["zeroinitializer".into(); count]
    } else {
        let inner = array_val.strip_prefix('[')?.strip_suffix(']')?;
        crate::text::split_args(inner).into_iter().map(|v| v.strip_prefix(&format!("{elem} ")).map(str::to_string)).collect::<Option<Vec<_>>>()?
    };
    if values.len() != count {
        return None;
    }
    let lanes = |v: &str| -> Option<Vec<String>> {
        match &lane_ty {
            None => Some(vec![v.to_string()]),
            Some(l) => {
                let k: usize = regex!(r"^<(\d+) x").captures(&elem)?[1].parse().ok()?;
                if v == "zeroinitializer" || v == "undef" || v == "poison" {
                    return Some(vec![v.to_string(); k]);
                }
                crate::text::split_args(v.strip_prefix('<')?.strip_suffix('>')?).into_iter().map(|x| x.strip_prefix(&format!("{l} ")).map(str::to_string)).collect()
            }
        }
    };
    let zero = |x: &str| matches!(x, "zeroinitializer" | "0" | "false" | "0.000000e+00" | "0xH0000" | "undef" | "poison");
    let mut all: Vec<Vec<String>> = Vec::new();
    for v in &values {
        all.push(lanes(v)?);
    }
    let nonzero: u64 = all.iter().flatten().filter(|x| !zero(x)).count() as u64 * lane_size;
    let p = format!("%__ci{n}");
    let mut out = String::new();
    let base = if field {
        out += &format!("{ind}{p}.f = getelementptr inbounds {ty}, {ty}* {var}, i32 0, i32 0\n");
        format!("{p}.f")
    } else {
        var.to_string()
    };
    if size > 16 && nonzero * 4 <= size {
        out += &format!("{ind}call void @llvm.memset.p0i8.i64(i8* align {align} {dest}, i8 0, i64 {size}, i1 false)\n");
        for (i, ls) in all.iter().enumerate() {
            for (j, x) in ls.iter().enumerate() {
                if zero(x) {
                    continue;
                }
                let off = i as u64 * esize + j as u64 * lane_size;
                let a = if off == 0 { align } else { align.min(1 << off.trailing_zeros()) };
                let scalar = lane_ty.clone().unwrap_or(elem.clone());
                out += &format!("{ind}{p}.e{i} = getelementptr inbounds {array_ty}, {array_ty}* {base}, i32 0, i32 {i}\n");
                let target = if lane_ty.is_some() {
                    out += &format!("{ind}{p}.e{i}.{j} = getelementptr inbounds {elem}, {elem}* {p}.e{i}, i32 0, i32 {j}\n");
                    format!("{p}.e{i}.{j}")
                } else {
                    format!("{p}.e{i}")
                };
                out += &format!("{ind}store {scalar} {x}, {scalar}* {target}, align {a}\n");
            }
        }
    } else {
        out += &format!("{ind}{p}.b = bitcast {array_ty}* {base} to {elem}*\n");
        for (i, v) in values.iter().enumerate() {
            out += &format!("{ind}{p}.e{i} = getelementptr {elem}, {elem}* {p}.b, i64 {i}\n{ind}store {elem} {v}, {elem}* {p}.e{i}, align {ealign}\n");
        }
    }
    Some(out.trim_end_matches('\n').to_string())
}

/// Codegen is flat, and the entry points' parameters give the address spaces back, but a
/// pointer loaded from memory has none to infer: a struct's pointer and reference fields
/// (an argument buffer's `device float *data`, `constant uint &n`) get their declared space
/// in the struct's type, and each read of one through its field's address is read in that
/// space and cast to flat, for infer-address-spaces to carry on (as Xcode's types have it).
pub fn field_address_spaces(ir: &str, records: &std::collections::HashMap<String, Vec<crate::ast::Param>>) -> String {
    // struct name -> (field index -> space)
    let mut spaced: std::collections::HashMap<String, Vec<(usize, u32)>> = Default::default();
    let mut out = ir.to_string();
    for c in regex!(r#"(?m)^%("?struct\.([^"\s]+)"?) = type \{ (.*) \}$"#).captures_iter(ir) {
        let Some(fields) = records.get(&c[2]) else { continue };
        let elems = crate::text::split_args(&c[3]);
        if elems.len() != fields.len() {
            continue;
        }
        let mut spaces = Vec::new();
        for (i, (f, e)) in fields.iter().zip(&elems).enumerate() {
            let Some(a) = regex!(r"address_space\((\d)\)\)\)[^*&]*[*&]\s*$").captures(&f.ty) else { continue };
            let n: u32 = a[1].parse().unwrap();
            if matches!(n, 1..=3) && e.ends_with('*') && !e.contains("addrspace") && e.matches('*').count() == 1 {
                spaces.push((i, n));
            }
        }
        if spaces.is_empty() {
            continue;
        }
        let new_elems: Vec<String> = elems.iter().enumerate().map(|(i, e)| match spaces.iter().find(|s| s.0 == i) {
            Some((_, n)) => format!("{} addrspace({n})*", &e[..e.len() - 1]),
            None => e.clone(),
        }).collect();
        out = out.replace(&c[0], &format!("%{} = type {{ {} }}", &c[1], new_elems.join(", ")));
        spaced.insert(c[1].to_string(), spaces);
    }
    if spaced.is_empty() {
        return out;
    }
    crate::text::map_functions(&out, |f| {
        let mut fields: std::collections::HashMap<String, u32> = Default::default();
        let mut lines = Vec::new();
        for line in f.lines() {
            // the field's address: kept under a name of its own, the old name a flat view of it
            if let Some(g) = regex!(r#"^(\s+)(%[\w.]+) = getelementptr inbounds (%"?struct\.[^"\s,]+"?), [^,]+, i32 0, i32 (\d+)$"#).captures(line) {
                let st = g[3].trim_start_matches('%').to_string();
                if let Some((_, n)) = spaced.get(&st).and_then(|v| v.iter().find(|s| s.0 == g[4].parse::<usize>().unwrap())) {
                    fields.insert(g[2].to_string(), *n);
                    lines.push(line.replacen(&format!("{} = ", &g[2]), &format!("{}.as = ", &g[2]), 1));
                    continue;
                }
            }
            if let Some(l) = regex!(r"^(\s+)(%[\w.]+) = load ([^,]+)\*, [^,]+\*\* (%[\w.]+)(, .*)$").captures(line) {
                if let Some(n) = fields.get(&l[4]) {
                    let t = &l[3];
                    lines.push(format!("{}{}.as = load {t} addrspace({n})*, {t} addrspace({n})** {}.as{}", &l[1], &l[2], &l[4], &l[5]));
                    lines.push(format!("{}{} = addrspacecast {t} addrspace({n})* {}.as to {t}*", &l[1], &l[2], &l[2]));
                    continue;
                }
            }
            lines.push(line.to_string());
        }
        // a flat view of each field address, right after it, for its other uses
        let mut res = Vec::new();
        for line in lines {
            let view = regex!(r#"^(\s+)(%[\w.]+)\.as = getelementptr inbounds (%"?struct\.[^"\s,]+"?), [^,]+, i32 0, i32 (\d+)$"#).captures(&line).and_then(|g| {
                let n = fields.get(&g[2])?;
                let st = g[3].trim_start_matches('%');
                let i: usize = g[4].parse().ok()?;
                let ty = regex!(r#"(?m)^%"?struct\.[^"\s]+"? = type \{ (.*) \}$"#).captures_iter(&out).find(|c| c[0].starts_with(&format!("%{st} ")))
                    .map(|c| crate::text::split_args(&c[1]))?.get(i)?.clone();
                let flat = ty.replace(&format!(" addrspace({n})*"), "*");
                Some(format!("{}{} = bitcast {ty}* {}.as to {flat}*", &g[1], &g[2], &g[2]))
            });
            res.push(line);
            if let Some(v) = view {
                res.push(v);
            }
        }
        res.join("\n") + if f.ends_with('\n') { "\n" } else { "" }
    })
}

/// Constant buffers are immutable to Xcode's alias analysis, so its attribute inference
/// ignores loads from them (not from program-scope constants, which count as reads): for
/// O2 each such load's TBAA tag is a copy marked immutable (TBAA's fourth operand), which
/// LLVM's own analysis honours, and an untagged one gets an immutable tag of a type of its
/// own. A copy of a buffer (a memcpy) keeps its tags: one tag covers both its ends, and
/// InstCombine deletes a copy into what it takes for constant memory.
/// `restore_constant_tbaa` puts the original tags back.
pub fn constant_tbaa(ir: &str) -> String {
    let nodes: std::collections::HashMap<String, String> = regex!(r"(?m)^!(\d+) = !\{(![0-9]+, ![0-9]+, i64 \d+)\}$").captures_iter(ir).map(|c| (c[1].to_string(), c[2].to_string())).collect();
    let mut next = regex!(r"(?m)^!(\d+) = ").captures_iter(ir).map(|c| c[1].parse::<usize>().unwrap()).max().unwrap_or(0) + 1;
    let mut made: std::collections::BTreeMap<String, usize> = Default::default();
    let untagged = next;
    let mut any_untagged = false;
    next += 3;
    let out = crate::text::map_functions(ir, |f| {
        let defs: std::collections::HashMap<&str, &str> = regex!(r"(?m)^\s+(%[\w.]+) = (?:getelementptr|bitcast|addrspacecast) (.*)$").captures_iter(f).map(|c| (c.get(1).unwrap().as_str(), c.get(2).unwrap().as_str())).collect();
        // what a pointer is based on, through GEPs and casts (a global's name, or with one in it)
        let base = |p: &str| {
            let mut p = p.to_string();
            for _ in 0..64 {
                if p.contains('@') {
                    break;
                }
                let Some(d) = defs.get(p.as_str()) else { break };
                let Some(c) = regex!(r"\* ([%@][\w.$]+)").captures(d) else { return if d.contains('@') { "@".to_string() } else { p } };
                p = c[1].to_string();
            }
            p
        };
        regex!(r"(?m)^(\s+%[\w.]+ = load [^,]+, [^,]* addrspace\(2\)\* ([^,]+)[^\n]*?)$").replace_all(f, |c: &Captures| {
            let line = &c[1];
            if base(c[2].trim()).contains('@') {
                return line.to_string();
            }
            if let Some(t) = regex!(r", !tbaa !(\d+)").captures(line) {
                let Some(_) = nodes.get(&t[1]) else { return line.to_string() };
                let n = *made.entry(t[1].to_string()).or_insert_with(|| {
                    next += 1;
                    next - 1
                });
                return line.replacen(&t[0], &format!(", !tbaa !{n}"), 1);
            }
            any_untagged = true;
            format!("{line}, !tbaa !{untagged}")
        })
        .to_string()
    });
    let mut out = out.trim_end().to_string() + "\n";
    for (old, new) in &made {
        out += &format!("!{new} = !{{{}, i64 1}}\n", nodes[old]);
    }
    if any_untagged {
        let root = regex!(r#"(?m)^!(\d+) = !\{!"Simple C\+\+ TBAA"\}$"#).captures(&out).map(|c| c[1].to_string());
        let root = root.unwrap_or_else(|| {
            out += &format!("!{} = !{{!\"Simple C++ TBAA\"}}\n", untagged + 2);
            (untagged + 2).to_string()
        });
        out += &format!("!{untagged} = !{{!{t}, !{t}, i64 0, i64 1}}\n!{t} = !{{!\"msl.untagged\", !{root}, i64 0}}\n", t = untagged + 1);
    }
    out
}

/// `constant_tbaa`'s tags back to mutable ones, by content (opt renumbers metadata): a
/// load tagged with an immutable copy gets the original tag where it is still there, and
/// a copy whose original is gone becomes it; the untagged lose their tag.
pub fn restore_constant_tbaa(ir: &str) -> String {
    let untagged_type = regex!(r#"(?m)^!(\d+) = !\{!"msl\.untagged", ![0-9]+, i64 0\}\n"#).captures(ir).map(|c| (c[0].to_string(), c[1].to_string()));
    let mut ir = ir.to_string();
    if let Some((line, t)) = &untagged_type {
        let tags: Vec<String> = regex!(r"(?m)^!(\d+) = !\{!(\d+), !(\d+), i64 0, i64 1\}$").captures_iter(&ir).filter(|c| c[2] == *t && c[3] == *t).map(|c| c[1].to_string()).collect();
        for tag in &tags {
            ir = regex!(r", !tbaa !(\d+)").replace_all(&ir, |c: &Captures| if c[1] == *tag { String::new() } else { c[0].to_string() }).to_string();
            ir = ir.replace(&format!("!{tag} = !{{!{t}, !{t}, i64 0, i64 1}}\n"), "");
        }
        ir = ir.replace(line.as_str(), "");
    }
    let plain: std::collections::HashMap<String, String> = regex!(r"(?m)^!(\d+) = !\{(![0-9]+, ![0-9]+, i64 \d+)\}$").captures_iter(&ir).map(|c| (c[2].to_string(), c[1].to_string())).collect();
    let mut map = std::collections::HashMap::new();
    for c in regex!(r"(?m)^!(\d+) = !\{(![0-9]+, ![0-9]+, i64 \d+), i64 1\}$").captures_iter(&ir) {
        if let Some(p) = plain.get(&c[2]) {
            map.insert(c[1].to_string(), p.clone());
        }
    }
    let ir = regex!(r"!tbaa !(\d+)").replace_all(&ir, |c: &Captures| match map.get(&c[1]) {
        Some(o) => format!("!tbaa !{o}"),
        None => c[0].to_string(),
    });
    regex!(r"(?m)^!(\d+) = !\{(![0-9]+, ![0-9]+, i64 \d+), i64 1\}\n").replace_all(&ir, |c: &Captures| if map.contains_key(&c[1]) { String::new() } else { format!("!{} = !{{{}}}\n", &c[1], &c[2]) }).to_string()
}

/// `constant_loads_invariant`'s marks go (Xcode's modules have none).
pub fn strip_invariant_loads(ir: &str) -> String {
    regex!(r", !invariant\.load !\d+").replace_all(ir, "").to_string()
}

/// The scopes O2 carried (`xcode_alias_scopes` writes them again from what O2 left).
pub fn strip_alias_scopes(ir: &str) -> String {
    regex!(r", !(?:alias\.scope|noalias) !\d+").replace_all(ir, "").to_string()
}

/// The inliner's scope declarations: Xcode's modules have none (and they would
/// keep opt from inferring argmemonly).
pub fn drop_scope_decls(ir: &str) -> String {
    let ir = regex!(r"(?m)^\s*(?:tail )?call void @llvm\.experimental\.noalias\.scope\.decl\(metadata !\d+\)(?: #\d+)?\n").replace_all(ir, "");
    regex!(r"(?m)^(?:; Function Attrs: .*\n)?declare void @llvm\.experimental\.noalias\.scope\.decl\(metadata\).*\n").replace_all(&ir, "").to_string()
}

/// Xcode's handles are types of their own to TBAA: the handle a texture or sampler
/// holds is `__metal_texture_2d_t`, `__metal_sampler_t` (the AIR type's name), where ours,
/// a pointer, is `any pointer`; a copy of the object (a sampler passed by value) has
/// the handle's type in its !tbaa.struct too.
pub fn handle_tbaa(ir: &str) -> String {
    // the C++ class -> the handle it holds
    let handles: std::collections::HashMap<String, String> = regex!(r#"(?m)^%"struct\.metal::(\w+?)(?:\.\d+)?" = type \{ %struct\.(_\w+_t) addrspace\(\d\)\* \}$"#).captures_iter(ir).map(|c| (c[1].to_string(), c[2].to_string())).collect();
    let (Some(any), Some(char_id)) = (
        regex!(r#"(?m)^!(\d+) = !\{!"any pointer", !\d+, i64 0\}$"#).captures(ir).map(|c| c[1].to_string()),
        regex!(r#"(?m)^!(\d+) = !\{!"omnipotent char", !\d+, i64 0\}$"#).captures(ir).map(|c| c[1].to_string()),
    ) else {
        return ir.to_string();
    };
    if handles.is_empty() {
        return ir.to_string();
    }
    let mut next = regex!(r"(?m)^!(\d+) = ").captures_iter(ir).map(|c| c[1].parse::<usize>().unwrap()).max().unwrap_or(0) + 1;
    let mut nodes = String::new();
    // handle -> (scalar type node, access tag node), for each handle the module's classes hold
    let mut made: std::collections::BTreeMap<String, (usize, usize)> = Default::default();
    for h in handles.values() {
        made.entry(h.clone()).or_insert_with(|| {
            let (t, g) = (next, next + 1);
            next += 2;
            nodes += &format!("!{t} = !{{!\"__metal{h}\", !{char_id}, i64 0}}\n!{g} = !{{!{t}, !{t}, i64 0}}\n");
            (t, g)
        });
    }
    // the classes' struct-path nodes: their one field becomes the handle's type
    let mut classes: std::collections::HashMap<String, String> = Default::default();
    let mut out = String::with_capacity(ir.len());
    for line in ir.lines() {
        if let Some(c) = Regex::new(&format!(r#"^!(\d+) = !\{{!"_ZTSN5metal(\d+)(\w+)", !{any}, i64 0\}}$"#)).unwrap().captures(line) {
            let len: usize = c[2].parse().unwrap_or(0);
            if let Some(h) = c[3].get(..len).and_then(|class| handles.get(class)) {
                let t = made[h].0;
                classes.insert(c[1].to_string(), h.clone());
                out += &line.replacen(&format!("!{any}, i64 0}}"), &format!("!{t}, i64 0}}"), 1);
                out.push('\n');
                continue;
            }
        }
        out += line;
        out.push('\n');
    }
    // access tags through them
    let out = Regex::new(&format!(r"(?m)^!(\d+) = !\{{!(\d+), !{any}, i64 0\}}$")).unwrap().replace_all(&out, |c: &Captures| match classes.get(&c[2]) {
        Some(h) => format!("!{} = !{{!{}, !{}, i64 0}}", &c[1], &c[2], made[h].0),
        None => c[0].to_string(),
    });
    // copies of the objects: a !tbaa.struct of the handle's type
    let out = crate::text::map_functions(&out, |f| {
        let casts: std::collections::HashMap<&str, &str> = regex!(r#"(?m)^\s+(%[\w.]+) = bitcast %"struct\.metal::(\w+?)(?:\.\d+)?"\* "#).captures_iter(f).map(|c| (c.get(1).unwrap().as_str(), c.get(2).unwrap().as_str())).collect();
        regex!(r"(?m)^(\s+(?:tail )?call void @llvm\.memcpy\.p0i8\.p0i8\.i64\(i8\* [^%\n]*(%[\w.]+), i8\* [^%\n]*(%[\w.]+), i64 8, [^)]*\)), !tbaa\.struct !\d+$").replace_all(f, |c: &Captures| {
            let class = casts.get(&c[2]).or(casts.get(&c[3]));
            match class.and_then(|k| handles.get(*k)) {
                Some(h) => {
                    let g = made[h].1;
                    let n = next;
                    next += 1;
                    nodes += &format!("!{n} = !{{i64 0, i64 8, !{g}}}\n");
                    format!("{}, !tbaa.struct !{n}", &c[1])
                }
                None => c[0].to_string(),
            }
        })
        .to_string()
    });
    out.trim_end().to_string() + "\n" + &nodes
}

/// clang 15's TBAA gives a vector access the type `omnipotent char`; Xcode's
/// gives it the element's scalar type (float4 -> "float") through a pointer or an
/// array element (a matrix's column), but keeps `omnipotent char` for a struct's
/// vector field read as such (`c.texel_size`); a vector field of a struct copy's
/// !tbaa.struct has the element type too. `structs`: each record's struct-copy fields
/// (offset, size, the element type of a vector field).
pub fn vector_tbaa(ir: &str, structs: &[Vec<(u32, u32, Option<String>)>]) -> String {
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
    // the function's struct field addresses (a GEP into a struct by field numbers alone)
    let mut fields: BTreeSet<String> = BTreeSet::new();
    let lines: Vec<String> = ir
        .lines()
        .map(|line| {
            if line.starts_with("define ") {
                fields.clear();
            }
            if let Some(g) = regex!(r#"^\s+(%[\w.]+) = getelementptr inbounds (?:%struct\.[\w.]+|%"struct\.[^"]+"), [^,]+, (?:i32 \d+(?:, )?)+$"#).captures(line) {
                fields.insert(g[1].to_string());
            }
            let Some(c) = access.captures(line) else { return line.to_string() };
            if regex!(r"<\d+ x \w+>(?: addrspace\(\d+\))?\* (%[\w.]+), align").captures(line).is_some_and(|p| fields.contains(&p[1])) {
                return line.to_string();
            }
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
    let mut ir = lines.join("\n") + "\n";
    // struct copies: a node that is exactly one record's fields
    let tbaa_struct = regex!(r"(?m)^!(\d+) = !\{((?:i64 \d+, i64 \d+, !\d+(?:, )?)+)\}$");
    let mut edits = Vec::new();
    for c in tbaa_struct.captures_iter(&ir) {
        let items: Vec<(u32, u32, String)> = regex!(r"i64 (\d+), i64 (\d+), !(\d+)").captures_iter(&c[2]).map(|f| (f[1].parse().unwrap(), f[2].parse().unwrap(), f[3].to_string())).collect();
        let matching: Vec<&Vec<(u32, u32, Option<String>)>> = structs.iter().filter(|r| r.len() == items.len() && r.iter().zip(&items).all(|(a, b)| a.0 == b.0 && a.1 == b.1)).collect();
        let Some(r) = matching.first() else { continue };
        if matching.iter().any(|m| m.iter().map(|x| &x.2).ne(r.iter().map(|x| &x.2))) {
            continue;
        }
        let mut changed = false;
        let mut parts = Vec::new();
        for ((off, size, tag), (_, _, elem)) in items.iter().zip(r.iter()) {
            let name = match elem.as_deref() {
                Some("float") => "float",
                Some("half") => "half",
                Some("int") | Some("uint") => "int",
                Some("short") | Some("ushort") => "short",
                Some("long") | Some("ulong") => "long",
                _ => "",
            };
            if !name.is_empty() && char_tag.contains(tag) {
                let t = *tags.entry(name).or_insert_with(|| {
                    let (t, g) = (next, next + 1);
                    next += 2;
                    nodes += &format!("!{t} = !{{!\"{name}\", !{char_id}, i64 0}}\n!{g} = !{{!{t}, !{t}, i64 0}}\n");
                    g
                });
                parts.push(format!("i64 {off}, i64 {size}, !{t}"));
                changed = true;
            } else {
                parts.push(format!("i64 {off}, i64 {size}, !{tag}"));
            }
        }
        if changed {
            edits.push((c[0].to_string(), format!("!{} = !{{{}}}", &c[1], parts.join(", "))));
        }
    }
    // struct type nodes: a vector field's type is its element's (a char field stays char, so only
    // where one record's layout has a vector at each such offset)
    let desc = regex!(r#"(?m)^!(\d+) = !\{!"(_ZTS[^"]*)", ((?:!\d+, i64 \d+(?:, )?)+)\}$"#);
    for c in desc.captures_iter(&ir) {
        let fields: Vec<(String, u32)> = regex!(r"!(\d+), i64 (\d+)").captures_iter(&c[3]).map(|f| (f[1].to_string(), f[2].parse().unwrap())).collect();
        if !fields.iter().any(|(t, _)| *t == char_id) {
            continue;
        }
        let elems = |r: &Vec<(u32, u32, Option<String>)>| -> Option<Vec<Option<String>>> {
            fields.iter().map(|(t, off)| {
                let f = r.iter().find(|x| x.0 == *off)?;
                Some(if *t == char_id { Some(f.2.clone()?) } else { None })
            }).collect()
        };
        let found: Vec<Vec<Option<String>>> = structs.iter().filter_map(elems).collect();
        let Some(first) = found.first() else { continue };
        if found.iter().any(|f| f != first) {
            continue;
        }
        let mut parts = Vec::new();
        for ((t, off), elem) in fields.iter().zip(first) {
            let name = match elem.as_deref() {
                Some("float") => "float",
                Some("half") => "half",
                Some("int") | Some("uint") => "int",
                Some("short") | Some("ushort") => "short",
                Some("long") | Some("ulong") => "long",
                _ => "",
            };
            let ty = if name.is_empty() {
                t.clone()
            } else if let Some(n) = Regex::new(&format!(r#"(?m)^!(\d+) = !\{{!"{name}", !{char_id}, i64 0\}}$"#)).unwrap().captures(&ir) {
                n[1].to_string()
            } else {
                let g = *tags.entry(name).or_insert_with(|| {
                    let (t, g) = (next, next + 1);
                    next += 2;
                    nodes += &format!("!{t} = !{{!\"{name}\", !{char_id}, i64 0}}\n!{g} = !{{!{t}, !{t}, i64 0}}\n");
                    g
                });
                (g - 1).to_string()
            };
            parts.push(format!("!{ty}, i64 {off}"));
        }
        edits.push((c[0].to_string(), format!("!{} = !{{!\"{}\", {}}}", &c[1], &c[2], parts.join(", "))));
    }
    for (a, b) in edits {
        ir = ir.replacen(&a, &b, 1);
    }
    ir + &nodes
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
                user = !regex!(r"@\x22?(?:_ZN5metal|_ZNK5metal|_ZZN5metal|__msl|_ZN11__air_entry)").is_match(l);
            }
            // a float the user's code takes as a bool: clang compares it with 0 (%tobool), Xcode converts it
            if user_scalars && user {
                if let Some(c) = regex!(r"^(\s+)(%tobool[\w.]*) = fcmp (?:fast )?une (float|half) (%[\w.]+), 0\.000000e\+00$").captures(l) {
                    let sn = name(&c[3]).unwrap();
                    let f = format!("air.convert.u.i1.f.{sn}");
                    let decl = format!("declare i1 @{f}({})", &c[3]);
                    if !decls.contains(&decl) {
                        decls.push(decl);
                    }
                    return format!("{}{} = tail call i1 @{f}({} {})", &c[1], &c[2], &c[3], &c[4]);
                }
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_frontends_layers_inline_first() {
        let ir = "define internal void @_ZL1kv() alwaysinline #0 {\nentry:\n  ret void\n}\ndefine linkonce_odr void @_ZNK5metal7__vpartIDv4_fLi0EfLi4EE3putEf() #0 {\nentry:\n  ret void\n}\ndefine linkonce_odr void @_ZN5metal5fractEf() #0 comdat {\nentry:\n  ret void\n}\ndefine linkonce_odr void @_ZNK5metal9texture2dIfE5widthEv() #0 comdat align 2 {\nentry:\n  ret void\n}\ndefine internal void @_ZL1hv() alwaysinline #1 {\nentry:\n  ret void\n}\nattributes #0 = { alwaysinline nounwind }\nattributes #1 = { nounwind }\n";
        let out = only_frontend_inlines(ir, &["_ZL1kv".to_string()].into());
        assert!(out.contains("@_ZL1kv() alwaysinline #0 {"));
        assert!(out.contains("3putEf() #0 {"));
        assert!(out.contains("5fractEf() #2 comdat {"));
        assert!(out.contains("5widthEv() #2 comdat align 2 {"));
        assert!(out.contains("@_ZL1hv() \"msl.ai\" #1 {"));
        assert!(out.contains("attributes #2 = { \"msl.ai\" nounwind }"));
        assert_eq!(restore_always_inline(&out).matches("alwaysinline").count(), 4);
    }

    #[test]
    fn xcode_scopes_follow_each_access_to_its_parameters() {
        let ir = "define void @k(float addrspace(1)* noalias %a, %struct._texture_2d_t addrspace(1)* noalias %t, float addrspace(2)* noalias %c, i32 %i) #0 {\nentry:\n  %p = getelementptr inbounds float, float addrspace(2)* %c, i64 1\n  %v = load float, float addrspace(2)* %p, align 4, !tbaa !1\n  %w = tail call i32 @air.get_width_texture_2d(%struct._texture_2d_t addrspace(1)* nocapture readonly %t, i32 0) #1\n  %g = load float, float addrspace(2)* @g, align 4\n  store float %v, float addrspace(1)* %a, align 4\n  ret void\n}\ndeclare i32 @air.get_width_texture_2d(%struct._texture_2d_t addrspace(1)* nocapture readonly, i32) argmemonly nounwind readonly\nattributes #0 = { nounwind }\nattributes #1 = { argmemonly nounwind readonly }\n!1 = !{!\"x\"}\n";
        let out = xcode_alias_scopes(ir, &["k".to_string()]);
        assert!(out.contains("define void @k(float addrspace(1)* %a, %struct._texture_2d_t addrspace(1)* %t, float addrspace(2)* %c, i32 %i) #0 {"));
        assert!(out.contains("!2 = distinct !{!2, !\"air-alias-scopes(k)\"}"));
        assert!(out.contains("!3 = distinct !{!3, !2, !\"air-alias-scope-arg(0)\"}"));
        assert!(out.contains("!4 = distinct !{!4, !2, !\"air-alias-scope-textures\"}"));
        assert!(out.contains("!5 = distinct !{!5, !2, !\"air-alias-scope-arg(2)\"}"));
        // the constant buffer's load: its own scope, the others' noalias
        assert!(out.contains("%v = load float, float addrspace(2)* %p, align 4, !tbaa !1, !alias.scope !6, !noalias !7"));
        assert!(out.contains("!6 = !{!5}") && out.contains("!7 = !{!3, !4}"));
        assert!(out.contains("i32 0) #1, !alias.scope !8, !noalias !9"));
        // a global: nothing
        assert!(out.contains("%g = load float, float addrspace(2)* @g, align 4\n"));
        assert!(out.contains("store float %v, float addrspace(1)* %a, align 4, !alias.scope !10, !noalias !11"));
    }

    #[test]
    fn calls_inlined_from_elsewhere_are_nounwind() {
        let ir = "define float @_Z1gf(float %x) #0 {\n  %y = call float @_Z1hf(float %x) #1\n  ret float %y\n}\ndefine float @_Z1hf(float %x) #0 {\n  ret float %x\n}\ndefine float @_ZL1kf(float %x) #0 {\n  %y = call float @_Z1gf(float %x) #1\n  ret float %y\n}\nattributes #0 = { nounwind }\nattributes #1 = { convergent nobuiltin \"no-builtins\" }\n";
        let home: std::collections::HashMap<String, String> = [("_ZL1kf".to_string(), "k".to_string())].into_iter().collect();
        let tagged = tag_user_calls(ir, &home);
        assert!(tagged.contains("call float @_Z1hf(float %x) #1, !msl.home !{!\"_Z1gf\"}"));
        assert!(tagged.contains("call float @_Z1gf(float %x) #1, !msl.home !{!\"k\"}"));
        // after O2: g inlined into the entry point k, its call to h with it; k's own call stays as written
        let after = "define float @k(float %x) #0 {\n  %y = call float @_Z1hf(float %x) #1, !msl.home !5\n  %z = call float @_Z1gf(float %x) #1, !msl.home !6\n  ret float %y\n}\nattributes #0 = { nounwind }\nattributes #1 = { convergent nobuiltin \"no-builtins\" }\n!5 = !{!\"_Z1gf\"}\n!6 = !{!\"k\"}\n";
        let out = nounwind_inlined_calls(after);
        assert!(out.contains("%y = call float @_Z1hf(float %x) #2\n"));
        assert!(out.contains("%z = call float @_Z1gf(float %x) #1\n"));
        assert!(out.contains("attributes #2 = { convergent nobuiltin nounwind \"no-builtins\" }"));
        assert!(!out.contains("msl.home") && !out.contains("!5 ="));
        // other single-string nodes stay
        assert!(nounwind_inlined_calls(&format!("{after}!7 = !{{!\"Simple C++ TBAA\"}}\n")).contains("!7 = !{!\"Simple C++ TBAA\"}"));
    }

    #[test]
    fn bools_convert_as_xcode_converts_them() {
        let ir = "define spir_func float @_Z1fbf(i1 %b, float %x) {\n  %conv = uitofp i1 %b to float\n  %tobool = fcmp une float %x, 0.000000e+00\n  %cmp = fcmp une float %x, 0.000000e+00\n  ret float %conv\n}\n";
        let out = user_scalar_conversions(ir);
        assert!(out.contains("%conv = tail call fast float @air.convert.f.f32.u.i1(i1 %b)"));
        assert!(out.contains("%tobool = tail call i1 @air.convert.u.i1.f.f32(float %x)"));
        // a comparison the user wrote stays one
        assert!(out.contains("%cmp = fcmp une float %x, 0.000000e+00"));
    }

    #[test]
    fn moved_values_are_frozen() {
        let ir = "define float @f(float %x, i16 %l) {\n  %a = call spir_func noundef float @air.simd_shuffle.f32(float noundef %x, i16 noundef zeroext %l) #3\n  %b = call spir_func noundef float @air.simd_sum.f32(float noundef %a) #3\n  ret float %b\n}\n";
        let out = freeze_moved_values(ir);
        assert!(out.contains("  %__fz1 = freeze float %x\n  %a = call spir_func noundef float @air.simd_shuffle.f32(float noundef %__fz1, i16 noundef zeroext %l) #3\n"));
        assert!(out.contains("@air.simd_sum.f32(float noundef %a)"));
        assert!(!out.contains("%__fz2"));
    }

    #[test]
    fn helper_type_clashes_do_not_number_user_types() {
        let ir = "%\"struct.metal::matrix\" = type { [3 x <3 x float>] }\n%\"struct.metal::__vbuild\" = type { <2 x float>, i32 }\n%\"struct.metal::__vbuild.0\" = type { <4 x float>, i32 }\n%struct.S.1 = type { i32 }\n%\"struct.metal::texture2d.3\" = type { %struct._texture_2d_t addrspace(1)* }\n%\"struct.metal::matrix.4\" = type { [4 x <2 x float>] }\n%\"struct.metal::matrix.2\" = type { [4 x <4 x float>] }\n%struct.S = type { float }\ndefine void @f(%\"struct.metal::matrix.2\"* %m, %struct.S.1* %s) {\n  ret void\n}\n";
        let out = renumber_types(ir);
        assert!(out.contains("%\"struct.metal::matrix.1\" = type { [4 x <4 x float>] }"));
        assert!(out.contains("define void @f(%\"struct.metal::matrix.1\"* %m, %struct.S.0* %s)"));
        assert!(out.contains("%struct.S = type { float }"));
        assert!(out.contains("%\"struct.metal::__vbuild.0\" = type"));
        assert!(out.contains("%\"struct.metal::texture2d.3\" = type"));
        assert!(out.contains("%\"struct.metal::matrix.2\" = type { [4 x <2 x float>] }"));
    }

    #[test]
    fn conversions_become_air_convert() {
        let ir = "define void @f(i32 %a, <2 x half> %h, float %s) {\n  %x = sitofp i32 %a to float\n  %y = fpext <2 x half> %h to <2 x float>\n  %z = fpext float %s to double\n  ret void\n}\n";
        let out = air_conversions(ir);
        assert!(out.contains("%x = tail call fast float @air.convert.f.f32.s.i32(i32 %a)"));
        assert!(out.contains("%y = tail call fast <2 x float> @air.convert.f.v2f32.f.v2f16(<2 x half> %h)"));
        assert!(out.contains("%z = fpext float %s to double"));
        // scalars declared first
        assert!(out.find("declare float @air.convert.f.f32.s.i32").unwrap() < out.find("declare <2 x float>").unwrap());
    }

    #[test]
    fn struct_type_vector_fields_get_element_types() {
        let ir = "!3 = !{!\"omnipotent char\", !4, i64 0}\n!4 = !{!\"Simple C++ TBAA\"}\n!5 = !{!3, !3, i64 0}\n!6 = !{!\"float\", !3, i64 0}\n!7 = !{!\"_ZTS1S\", !6, i64 0, !3, i64 16}\n!8 = !{!\"_ZTS1T\", !3, i64 0}\n";
        // S { float a; float4 b; } has a vector at 16; T { char c; } has none
        let out = vector_tbaa(ir, &[vec![(0, 4, None), (16, 16, Some("float".into()))]]);
        assert!(out.contains("!7 = !{!\"_ZTS1S\", !6, i64 0, !6, i64 16}"));
        assert!(out.contains("!8 = !{!\"_ZTS1T\", !3, i64 0}"));
    }

    #[test]
    fn function_constants_each_with_its_initializer() {
        let ir = "@_ZL7has_opt = internal addrspace(2) global i8 0, align 1\n@__msl_fcinit_8_c_opt = linkonce_odr hidden addrspace(2) externally_initialized constant i8 undef, align 1\n@_ZL1a = internal addrspace(2) global i32 undef, align 4\n@_ZL5c_opt = internal addrspace(2) global i8 undef, align 1\n@__msl_fcinit_0_a = linkonce_odr hidden addrspace(2) externally_initialized constant i32 undef, align 4\n\ndefine internal void @__cxx_global_var_init() {\n  %d = call i1 @air.is_function_constant_defined(i8 addrspace(2)* @__msl_fcinit_8_c_opt)\n  store i8 0, i8* @_ZL7has_opt, align 1\n  ret void\n}\n\ndeclare i1 @air.is_function_constant_defined(i8 addrspace(2)*)\n\ndefine void @k() {\n  ret void\n}\n\ndefine internal void @_GLOBAL__sub_I_x() {\n  call void @__cxx_global_var_init()\n  ret void\n}\n";
        let out = order_constants(ir, &["a".to_string(), "c_opt".to_string()]);
        assert!(out.starts_with("@_ZL1a = internal addrspace(2) global i32 undef, align 4\n@__msl_fcinit_0_a = linkonce_odr hidden addrspace(2) externally_initialized constant i32 undef, align 4\n@_ZL5c_opt = internal addrspace(2) global i8 undef, align 1\n@__msl_fcinit_8_c_opt = linkonce_odr hidden addrspace(2) externally_initialized constant i8 undef, align 1\n@_ZL7has_opt = internal addrspace(2) global i8 0, align 1\n"), "{out}");
        let (k, init, decl, ctor) = (out.find("@k(").unwrap(), out.find("@__cxx_global_var_init() {").unwrap(), out.find("declare i1 @air.is_function_constant_defined").unwrap(), out.find("@_GLOBAL__sub_I_x").unwrap());
        assert!(k < init && init < decl && decl < ctor, "{out}");
    }

    #[test]
    fn an_entry_point_stands_where_its_stage_function_does() {
        let ir = "define void @a() {\n  ret void\n}\n\n; Function Attrs: x\ndefine internal void @s() {\n  call void @llvm.lifetime.start.p0i8(i64 4, i8* null)\n  ret void\n}\n\ndeclare void @llvm.lifetime.start.p0i8(i64, i8*)\n\n; Function Attrs: y\ndefine void @e() {\n  call void @s()\n  ret void\n}\n";
        let home = [("s".to_string(), "e".to_string())].into_iter().collect();
        let out = entries_at_stages(ir, &home);
        assert_eq!(out, "define void @a() {\n  ret void\n}\n\n; Function Attrs: y\ndefine void @e() {\n  call void @s()\n  ret void\n}\n\n; Function Attrs: x\ndefine internal void @s() {\n  call void @llvm.lifetime.start.p0i8(i64 4, i8* null)\n  ret void\n}\n\ndeclare void @llvm.lifetime.start.p0i8(i64, i8*)\n\n");
    }

    #[test]
    fn constant_locals_are_initialised_in_place() {
        let ir = "@__const.f.m = private unnamed_addr constant %M { [4 x <2 x float>] [<2 x float> <float 0.000000e+00, float 1.000000e+00>, <2 x float> <float 1.000000e+00, float 1.000000e+00>, <2 x float> zeroinitializer, <2 x float> <float 1.000000e+00, float 0.000000e+00>] }, align 8\n@__const.g.w = private unnamed_addr constant [12 x float] [float 0.000000e+00, float 0.000000e+00, float 0.000000e+00, float 0.000000e+00, float 0.000000e+00, float 3.000000e+00, float 0.000000e+00, float 0.000000e+00, float 0.000000e+00, float 0.000000e+00, float 0.000000e+00, float 0.000000e+00], align 4\ndefine void @f() {\n  %m = alloca %M, align 8\n  %0 = bitcast %M* %m to i8*\n  call void @llvm.memcpy.p0i8.p0i8.i64(i8* align 8 %0, i8* align 8 bitcast (%M* @__const.f.m to i8*), i64 32, i1 false)\n  ret void\n}\ndefine void @g() {\n  %w = alloca [12 x float], align 4\n  %0 = bitcast [12 x float]* %w to i8*\n  call void @llvm.memcpy.p0i8.p0i8.i64(i8* align 4 %0, i8* align 4 bitcast ([12 x float]* @__const.g.w to i8*), i64 48, i1 false)\n  ret void\n}\n";
        let out = constant_inits(ir);
        assert!(!out.contains("call void @llvm.memcpy"), "{out}");
        assert!(out.contains("  %__ci1.f = getelementptr inbounds %M, %M* %m, i32 0, i32 0\n  %__ci1.b = bitcast [4 x <2 x float>]* %__ci1.f to <2 x float>*\n  %__ci1.e0 = getelementptr <2 x float>, <2 x float>* %__ci1.b, i64 0\n  store <2 x float> <float 0.000000e+00, float 1.000000e+00>, <2 x float>* %__ci1.e0, align 8\n"), "{out}");
        assert!(out.contains("store <2 x float> zeroinitializer, <2 x float>* %__ci1.e2, align 8"), "{out}");
        assert!(out.contains("  call void @llvm.memset.p0i8.i64(i8* align 4 %0, i8 0, i64 48, i1 false)\n  %__ci2.e5 = getelementptr inbounds [12 x float], [12 x float]* %w, i32 0, i32 5\n  store float 3.000000e+00, float* %__ci2.e5, align 4\n"), "{out}");
    }

    #[test]
    fn handles_are_tbaa_types_of_their_own() {
        let ir = "%\"struct.metal::sampler\" = type { %struct._sampler_t addrspace(2)* }\ndefine void @f(%\"struct.metal::sampler\"* %a, %\"struct.metal::sampler\"* %b) {\n  %x = bitcast %\"struct.metal::sampler\"* %a to i8*\n  %y = bitcast %\"struct.metal::sampler\"* %b to i8*\n  call void @llvm.memcpy.p0i8.p0i8.i64(i8* align 8 %x, i8* align 8 %y, i64 8, i1 false), !tbaa.struct !9\n  ret void\n}\n!3 = !{!4, !4, i64 0}\n!4 = !{!\"any pointer\", !5, i64 0}\n!5 = !{!\"omnipotent char\", !6, i64 0}\n!6 = !{!\"Simple C++ TBAA\"}\n!9 = !{i64 0, i64 8, !3}\n!13 = !{!\"_ZTSN5metal7samplerE\", !4, i64 0}\n!21 = !{!13, !4, i64 0}\n";
        let out = handle_tbaa(ir);
        assert!(out.contains("!22 = !{!\"__metal_sampler_t\", !5, i64 0}\n!23 = !{!22, !22, i64 0}"), "{out}");
        assert!(out.contains("!13 = !{!\"_ZTSN5metal7samplerE\", !22, i64 0}"), "{out}");
        assert!(out.contains("!21 = !{!13, !22, i64 0}"), "{out}");
        assert!(out.contains("i64 8, i1 false), !tbaa.struct !24") && out.contains("!24 = !{i64 0, i64 8, !23}"), "{out}");
    }

    #[test]
    fn struct_pointer_fields_get_their_address_spaces() {
        let ir = "%struct.Args = type { float*, i32*, float }\ndefine void @f(%struct.Args* %a) {\n  %d = getelementptr inbounds %struct.Args, %struct.Args* %a, i32 0, i32 0\n  %p = load float*, float** %d, align 8, !tbaa !1\n  %v = load float, float* %p, align 4\n  ret void\n}\n";
        let field = |ty: &str| crate::ast::Param { name: String::new(), ty: ty.into(), desugared: ty.into(), attrs: vec![], stage_in: false };
        let records = [("Args".to_string(), vec![field("__attribute__((address_space(1))) float *"), field("__attribute__((address_space(2))) uint &"), field("float")])].into_iter().collect();
        let out = field_address_spaces(ir, &records);
        assert!(out.contains("%struct.Args = type { float addrspace(1)*, i32 addrspace(2)*, float }"), "{out}");
        assert!(out.contains("  %d.as = getelementptr inbounds %struct.Args, %struct.Args* %a, i32 0, i32 0\n  %d = bitcast float addrspace(1)** %d.as to float**\n"), "{out}");
        assert!(out.contains("  %p.as = load float addrspace(1)*, float addrspace(1)** %d.as, align 8, !tbaa !1\n  %p = addrspacecast float addrspace(1)* %p.as to float*\n"), "{out}");
    }

    #[test]
    fn a_struct_vector_field_keeps_the_char_tag() {
        let ir = "define void @f(%struct.S* %s, <4 x float>* %p) {\n  %f = getelementptr inbounds %struct.S, %struct.S* %s, i32 0, i32 1\n  %a = load <4 x float>, <4 x float>* %f, align 16, !tbaa !5\n  %b = load <4 x float>, <4 x float>* %p, align 16, !tbaa !5\n  ret void\n}\n!3 = !{!\"omnipotent char\", !4, i64 0}\n!4 = !{!\"Simple C++ TBAA\"}\n!5 = !{!3, !3, i64 0}\n";
        let out = vector_tbaa(ir, &[]);
        assert!(out.contains("%a = load <4 x float>, <4 x float>* %f, align 16, !tbaa !5"), "{out}");
        assert!(out.contains("%b = load <4 x float>, <4 x float>* %p, align 16, !tbaa !7"), "{out}");
        assert!(out.contains("!6 = !{!\"float\", !3, i64 0}"), "{out}");
    }

    #[test]
    fn struct_copy_vector_fields_get_element_tags() {
        let ir = "  %v = load <2 x float>, <2 x float>* %p, align 16, !tbaa.struct !1\n!1 = !{i64 0, i64 8, !2, i64 16, i64 16, !2}\n!2 = !{!3, !3, i64 0}\n!3 = !{!\"omnipotent char\", !4, i64 0}\n!4 = !{!\"Simple C++ TBAA\"}\n";
        let out = vector_tbaa(ir, &[vec![(0, 8, Some("float".into())), (16, 16, Some("float".into()))]]);
        assert!(out.contains("!1 = !{i64 0, i64 8, !6, i64 16, i64 16, !6}"));
        assert!(out.contains("!5 = !{!\"float\", !3, i64 0}"));
    }

    #[test]
    fn packed_vectors_are_bare_arrays() {
        let ir = "%\"struct.metal::__packed\" = type { [3 x float] }\n%struct.V = type { %\"struct.metal::__packed\", float }\n@g = internal addrspace(2) constant %\"struct.metal::__packed\" { [3 x float] [float 1.0, float 2.0, float 3.0] }, align 4\ndefine void @f(%struct.V addrspace(1)* %p, %\"struct.metal::__packed\" addrspace(1)* %q) {\n  %a = getelementptr inbounds %struct.V, %struct.V addrspace(1)* %p, i64 %i, i32 0, i32 0, i64 2\n  %b = getelementptr inbounds %\"struct.metal::__packed\", %\"struct.metal::__packed\" addrspace(1)* %q, i64 1, i32 0, i64 0\n  ret void\n}\n";
        let out = unwrap_packed(ir);
        assert!(out.contains("%struct.V = type { [3 x float], float }"));
        assert!(out.contains("@g = internal addrspace(2) constant [3 x float] [float 1.0, float 2.0, float 3.0], align 4"));
        assert!(out.contains("%a = getelementptr inbounds %struct.V, %struct.V addrspace(1)* %p, i64 %i, i32 0, i64 2"));
        assert!(out.contains("%b = getelementptr inbounds [3 x float], [3 x float] addrspace(1)* %q, i64 1, i64 0"));
        assert!(!out.contains("__packed"));
    }

    #[test]
    fn constant_loads_are_invariant_during_o2_only() {
        let ir = "define void @f(float addrspace(2)* %c, float addrspace(1)* %d) {\n  %a = load float, float addrspace(2)* %c, align 4, !tbaa !1\n  %b = load float, float addrspace(1)* %d, align 4, !tbaa !1\n  ret void\n}\n!1 = !{!\"x\"}\n";
        let out = constant_loads_invariant(ir);
        assert!(out.contains("%a = load float, float addrspace(2)* %c, align 4, !invariant.load !2, !tbaa !1\n"));
        assert!(out.contains("%b = load float, float addrspace(1)* %d, align 4, !tbaa !1\n"));
        assert!(out.ends_with("!2 = !{}\n"));
        assert!(strip_invariant_loads(&out).contains("%a = load float, float addrspace(2)* %c, align 4, !tbaa !1\n"));
    }

    #[test]
    fn an_inlined_return_goes_back_to_its_place() {
        let stage = "define internal void @_ZL1fv() {\nentry:\n  br label %for.cond\nfor.cond:\n  br i1 %c, label %for.body, label %for.cond.cleanup\nfor.cond.cleanup:\n  ret void\nfor.body:\n  br label %for.cond\n}\n";
        let home: std::collections::HashMap<String, String> = [("_ZL1fv".to_string(), "k".to_string())].into();
        let places = return_places(stage, &home);
        assert_eq!(places, vec![("k".to_string(), "_ZL1fv".to_string(), Some("for.cond".to_string()))]);
        let inlined = "define void @k() {\nentry:\n  %x.i = alloca i32, align 4\n  %y = alloca i32, align 4\n  call void @llvm.lifetime.start.p0i8(i64 4, i8* %0)\n  br label %for.cond.i\nfor.cond.i:\n  br i1 %c, label %for.body.i, label %_ZL1fv.exit\nfor.body.i:\n  br label %for.cond.i\n_ZL1fv.exit:\n  ret void\n}\n";
        let out = own_allocas_first(&place_inlined_returns(&drop_inliner_lifetimes(inlined), &places), &["k".to_string()]);
        assert_eq!(out, "define void @k() {\nentry:\n  %y = alloca i32, align 4\n  %x.i = alloca i32, align 4\n  br label %for.cond.i\nfor.cond.i:\n  br i1 %c, label %for.body.i, label %_ZL1fv.exit\n_ZL1fv.exit:\n  ret void\nfor.body.i:\n  br label %for.cond.i\n}\n");
    }

    #[test]
    fn samplers_of_one_state_share_a_global() {
        let ir = "@__msl_sampler_0 = external addrspace(2) global [2 x i64], align 8\n@__msl_sampler_1 = external addrspace(2) global [2 x i64], align 8\n@__msl_sampler_2 = external addrspace(2) global [2 x i64], align 8\ndefine void @f() {\n  %a = load i64, i64 addrspace(2)* getelementptr ([2 x i64], [2 x i64] addrspace(2)* @__msl_sampler_1, i64 0, i64 0)\n  %b = load i64, i64 addrspace(2)* getelementptr ([2 x i64], [2 x i64] addrspace(2)* @__msl_sampler_2, i64 0, i64 0)\n  ret void\n}\n";
        let out = samplers(ir, &[5, 7, 5], (2, 8));
        assert!(out.contains("@__msl_sampler_0 = internal addrspace(2) constant [2 x i64] [i64 5, i64 0], align 8\n@__msl_sampler_1 = internal addrspace(2) constant [2 x i64] [i64 7, i64 0], align 8\ndefine"));
        assert!(out.contains("@__msl_sampler_1, i64 0, i64 0)\n  %b = load i64, i64 addrspace(2)* getelementptr ([2 x i64], [2 x i64] addrspace(2)* @__msl_sampler_0,"));
    }

    #[test]
    fn a_packed_element_is_read_with_no_tbaa() {
        let ir = "%\"struct.metal::__packed\" = type { [3 x float] }\n%struct.V = type { %\"struct.metal::__packed\", float }\ndefine void @f(%struct.V addrspace(1)* %p) {\n  %a = getelementptr inbounds %struct.V, %struct.V addrspace(1)* %p, i64 %i, i32 0, i32 0, i64 2\n  %x = load float, float addrspace(1)* %a, align 4, !tbaa !3, !alias.scope !5\n  %b = getelementptr inbounds %struct.V, %struct.V addrspace(1)* %p, i64 %i, i32 1\n  %y = load float, float addrspace(1)* %b, align 4, !tbaa !3\n  ret void\n}\ndefine void @g(float addrspace(1)* %a) {\n  %x = load float, float addrspace(1)* %a, align 4, !tbaa !3\n  ret void\n}\n";
        let out = unwrap_packed(ir);
        assert!(out.contains("%x = load float, float addrspace(1)* %a, align 4, !alias.scope !5\n"));
        assert!(out.contains("%y = load float, float addrspace(1)* %b, align 4, !tbaa !3\n"));
        assert!(out.contains("define void @g(float addrspace(1)* %a) {\n  %x = load float, float addrspace(1)* %a, align 4, !tbaa !3\n"));
    }

}

/// Xcode's front end declares a scalar conversion as it emits the function that
/// first makes it, so its declaration follows that function's definition.
pub fn place_scalar_conversions(ir: &str) -> String {
    let decl = regex!(r"(?m)^(?:; Function Attrs: .*\n)?declare [^@\n]*@(air\.convert\.[a-z]\.[a-z]+\d+\.[a-z]\.[a-z]+\d+)\(.*\n\n?");
    let decls: Vec<(String, String)> = decl.captures_iter(ir).map(|c| (c[1].to_string(), c[0].trim_end().to_string() + "\n\n")).collect();
    if decls.is_empty() {
        return ir.to_string();
    }
    let mut out = decl.replace_all(ir, "").to_string();
    // each function's end, in order, and what it calls
    let mut placed: Vec<&str> = Vec::new();
    let mut inserts: Vec<(usize, String)> = Vec::new();
    for (start, end) in crate::text::functions(&out) {
        let body = &out[start..end];
        let mut here: Vec<(usize, &(String, String))> = decls.iter().filter(|(n, _)| !placed.contains(&n.as_str())).filter_map(|d| body.find(&format!("@{}(", d.0)).map(|i| (i, d))).collect();
        here.sort_by_key(|x| x.0);
        if here.is_empty() {
            continue;
        }
        let text: String = here.iter().map(|(_, d)| format!("\n{}", d.1.trim_end())).collect::<String>() + "\n";
        placed.extend(here.iter().map(|(_, d)| d.0.as_str()));
        inserts.push((end, text));
    }
    for (at, text) in inserts.into_iter().rev() {
        out.insert_str(at, &text);
    }
    out
}

/// Xcode's splats insert into poison (instcombine leaves ours on undef when it
/// forms one from an element insert).
pub fn splat_poison(ir: &str) -> String {
    let lines: Vec<&str> = ir.lines().collect();
    let mut out: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    for i in 0..lines.len().saturating_sub(1) {
        let Some(c) = regex!(r"^(\s+(%[\w.]+) = insertelement (<\d+ x \w+>)) undef, (.*, i(?:32|64) 0)$").captures(lines[i]) else { continue };
        let shuffle = format!("shufflevector {} {}, {} ", &c[3], &c[2], &c[3]);
        if lines[i + 1].contains(&shuffle) && lines[i + 1].ends_with("zeroinitializer") {
            out[i] = format!("{} poison, {}", &c[1], &c[4]);
        }
    }
    out.join("\n") + if ir.ends_with('\n') { "\n" } else { "" }
}

/// A scalar access split from a struct copy keeps no !tbaa.struct in Xcode's
/// modules (a packed vector's element, say).
pub fn scalar_tbaa_struct(ir: &str) -> String {
    regex!(r"(?m)^(\s+(?:%[\w.]+ = load (?:float|half|double|i\d+), |store (?:float|half|double|i\d+) ).*?), !tbaa\.struct !\d+")
        .replace_all(ir, "$1")
        .to_string()
}

/// Xcode lays out a packed vector (packed_float3 ...) as a bare array; ours is a
/// struct holding one (`metal::__packed`). Each such struct type becomes its
/// array: the index stepping into it goes from GEPs and extract/insertvalue,
/// a constant of it becomes the array constant, and its name the array type.
pub fn unwrap_packed(ir: &str) -> String {
    let defs: std::collections::HashMap<String, String> = regex!(r#"(?m)^(%[\w.]+|%"[^"]+") = type (.*)$"#).captures_iter(ir).map(|c| (c[1].to_string(), c[2].to_string())).collect();
    let packed: std::collections::HashMap<String, String> = defs
        .iter()
        .filter(|(n, _)| regex!(r#"^%"struct\.metal::__packed(?:\.\d+)?"$"#).is_match(n))
        .filter_map(|(n, b)| regex!(r"^\{ (\[\d+ x [^\]]+\]) \}$").captures(b).map(|c| (n.clone(), c[1].to_string())))
        .collect();
    if packed.is_empty() {
        return ir.to_string();
    }
    let fields = |t: &str| -> Vec<String> {
        let body = defs.get(t).map(String::as_str).unwrap_or(t);
        let inner = body.trim().trim_start_matches('<').trim_end_matches('>');
        inner.strip_prefix('{').and_then(|x| x.strip_suffix('}')).map(|x| split_args(x).into_iter().map(|f| f.trim().to_string()).collect()).unwrap_or_default()
    };
    let element = |t: &str| -> Option<String> { regex!(r"^[\[<]\d+ x (.*)[\]>]$").captures(t.trim()).map(|c| c[1].to_string()) };
    // walk the indices from `ty`, dropping the one into a packed struct
    let walk = |ty: &str, idx: &[String]| -> Vec<String> {
        let mut cur = ty.trim().to_string();
        let mut out = Vec::new();
        for i in idx {
            if let Some(a) = packed.get(&cur) {
                cur = a.clone();
                continue;
            }
            out.push(i.clone());
            let k = i.trim().rsplit(' ').next().and_then(|x| x.parse::<usize>().ok());
            cur = if cur.starts_with('[') || cur.starts_with('<') && !cur.starts_with("<{") {
                element(&cur).unwrap_or_default()
            } else {
                k.and_then(|k| fields(&cur).get(k).cloned()).unwrap_or_default()
            };
        }
        out
    };
    // whether the indices step past a packed struct into its array's element
    let into_element = |ty: &str, idx: &[String]| -> bool {
        let mut cur = ty.trim().to_string();
        for (n, i) in idx.iter().enumerate() {
            if let Some(a) = packed.get(&cur) {
                return n + 1 < idx.len() && !a.is_empty();
            }
            let k = i.trim().rsplit(' ').next().and_then(|x| x.parse::<usize>().ok());
            cur = if cur.starts_with('[') || cur.starts_with('<') && !cur.starts_with("<{") {
                element(&cur).unwrap_or_default()
            } else {
                k.and_then(|k| fields(&cur).get(k).cloned()).unwrap_or_default()
            };
        }
        false
    };
    // (function, pointer) of each packed element: Xcode reads one with no TBAA
    let mut elements: BTreeSet<(usize, String)> = BTreeSet::new();
    let mut function = 0;
    let gep = regex!(r"getelementptr (inbounds )?(\(?)");
    let mut out = String::with_capacity(ir.len());
    for line in ir.lines() {
        let mut l = line.to_string();
        if l.starts_with("define ") {
            function += 1;
        }
        if let Some(c) = regex!(r"^\s+(%[\w.]+) = getelementptr (?:inbounds )?(.*)$").captures(&l) {
            let args = split_args(c[2].split(", !").next().unwrap());
            if args.len() >= 3 && into_element(args[0].trim(), &args[3..]) {
                elements.insert((function, c[1].to_string()));
            }
        }
        // instruction and constant-expression GEPs
        let mut from = 0;
        while let Some(m) = gep.find_at(&l.clone(), from) {
            let start = m.end();
            let (args_s, end) = if l[m.start()..m.end()].ends_with('(') {
                let close = balanced(l.as_bytes(), m.end() - 1, b'(', b')');
                (l[start..close - 1].to_string(), close - 1)
            } else {
                let tail = l[start..].split(", !").next().unwrap().to_string();
                (tail.clone(), start + tail.len())
            };
            let args = split_args(&args_s);
            if args.len() >= 3 {
                let ty = args[0].trim().to_string();
                let idx: Vec<String> = args[2..].to_vec();
                let kept = walk(&ty, &idx[1..]);
                if kept.len() + 1 != idx.len() {
                    let mut new_args = vec![args[0].trim().to_string(), args[1].trim().to_string(), idx[0].trim().to_string()];
                    new_args.extend(kept.iter().map(|x| x.trim().to_string()));
                    l = format!("{}{}{}", &l[..start], new_args.join(", "), &l[end..]);
                }
            }
            from = start;
        }
        if let Some(c) = regex!(r"^(\s+%[\w.]+ = (?:extractvalue|insertvalue) )(.*)$").captures(&l) {
            let args = split_args(&c[2]);
            let is_insert = l.contains("insertvalue");
            let n = if is_insert { 2 } else { 1 };
            if args.len() > n {
                let agg = args[0].trim();
                let ty = agg.rsplit_once(' ').map(|x| x.0).unwrap_or(agg);
                let kept = walk(ty, &args[n..]);
                if kept.len() != args.len() - n {
                    let mut new_args: Vec<String> = args[..n].iter().map(|x| x.trim().to_string()).collect();
                    new_args.extend(kept.iter().map(|x| x.trim().to_string()));
                    l = format!("{}{}", &c[1], new_args.join(", "));
                }
            }
        }
        out.push_str(&l);
        out.push('\n');
    }
    if !elements.is_empty() {
        let mut function = 0;
        out = out
            .lines()
            .map(|l| {
                if l.starts_with("define ") {
                    function += 1;
                }
                match regex!(r"^(\s+%[\w.]+ = load [^,]+, [^,]*\* (%[\w.]+), align \d+), !tbaa !\d+(.*)$").captures(l) {
                    Some(c) if elements.contains(&(function, c[2].to_string())) => format!("{}{}\n", &c[1], &c[3]),
                    _ => format!("{l}\n"),
                }
            })
            .collect();
    }
    // constants: `%"...__packed" { [N x T] [...] }` -> `[N x T] [...]`
    let mut names: Vec<&String> = packed.keys().collect();
    names.sort_by_key(|n| std::cmp::Reverse(n.len()));
    for n in &names {
        let pat = format!("{n} {{ ");
        while let Some(i) = out.find(&pat) {
            let open = i + n.len() + 1;
            let close = balanced(out.as_bytes(), open, b'{', b'}');
            let inner = out[open + 1..close - 1].trim().to_string();
            out.replace_range(i..close, &inner);
        }
    }
    // TBAA: a struct's packed member is its element type, as in Xcode's modules
    let char_id = regex!(r#"(?m)^!(\d+) = !\{!"omnipotent char", !\d+, i64 0\}$"#).captures(&out).map(|c| c[1].to_string());
    if let Some(char_id) = char_id {
        let nodes: Vec<(String, String)> = regex!(r#"(?m)^!(\d+) = !\{!"_ZTSN5metal8__packedI(\w+?)Li\dEEE", !\d+, i64 0\}$"#).captures_iter(&out).map(|c| (c[1].to_string(), c[2].to_string())).collect();
        let mut next = regex!(r"(?m)^!(\d+) = ").captures_iter(&out).map(|c| c[1].parse::<usize>().unwrap()).max().unwrap_or(0) + 1;
        let mut extra = String::new();
        for (id, mangled) in nodes {
            let name = match mangled.as_str() {
                "f" => "float",
                "Dh" | "DF16_" => "half",
                "i" | "j" => "int",
                "s" | "t" => "short",
                "l" | "m" => "long",
                _ => continue,
            };
            let existing = Regex::new(&format!(r#"(?m)^!(\d+) = !\{{!"{name}", !{char_id}, i64 0\}}$"#)).unwrap().captures(&out).map(|c| c[1].to_string());
            let elem = existing.unwrap_or_else(|| {
                let n = next;
                next += 1;
                extra += &format!("!{n} = !{{!\"{name}\", !{char_id}, i64 0}}\n");
                n.to_string()
            });
            let re = Regex::new(&format!(r"(?m)^(!\d+ = !\{{!\x22_ZTS.*?)!{id}\b")).unwrap();
            loop {
                let next_out = re.replace_all(&out, format!("${{1}}!{elem}").as_str()).to_string();
                if next_out == out {
                    break;
                }
                out = next_out;
            }
        }
        out += &extra;
    }
    // the types themselves
    for n in names {
        out = regex!(r"(?m)^%.*= type .*$").replace_all(&out, |c: &regex::Captures| if c[0].starts_with(&format!("{n} = type")) { String::new() } else { c[0].to_string() }).to_string();
        out = out.replace(n.as_str(), &packed[n]);
    }
    out
}
