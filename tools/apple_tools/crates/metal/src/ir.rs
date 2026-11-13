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

/// The functions a module defines as `convergent`.
pub fn convergent_functions(ir: &str) -> BTreeSet<String> {
    let groups: BTreeSet<String> = regex!(r"(?m)^attributes #(\d+) = \{ (.*) \}$").captures_iter(ir).filter(|c| c[2].split(' ').any(|a| a == "convergent")).map(|c| c[1].to_string()).collect();
    regex!(r#"(?m)^define [^\n]*?@("?[\w.$]+"?)\([^\n]*\)[^\n]*? #(\d+)"#).captures_iter(ir).filter(|c| groups.contains(&c[2])).map(|c| c[1].to_string()).collect()
}

/// The convergent functions that call a function the module defines: those
/// Xcode's single attribute inference leaves convergent (its frontend's calls
/// all are), whatever a later inference finds.
pub fn convergent_callers(ir: &str) -> BTreeSet<String> {
    let defined: BTreeSet<String> = regex!(r#"(?m)^define [^\n]*?@("?[\w.$]+"?)\("#).captures_iter(ir).map(|c| c[1].to_string()).collect();
    let convergent = convergent_functions(ir);
    let mut out = BTreeSet::new();
    map_functions(ir, |f| {
        let name = regex!(r#"@("?[\w.$]+"?)\("#).captures(f).map(|c| c[1].to_string()).unwrap_or_default();
        if convergent.contains(&name) && regex!(r#"\bcall [^@\n]*@("?[\w.$]+"?)\("#).captures_iter(f).any(|c| defined.contains(&c[1])) {
            out.insert(name);
        }
        f.to_string()
    });
    out
}

/// Xcode infers function attributes once, while its frontend's call sites are
/// all still convergent, so a function calling another keeps `convergent`.
/// Our second O2 infers them again after InstCombine has taken that off the
/// call sites. So a function convergent after the first O2 keeps it, and so do
/// the calls to it (InstCombine leaves a call convergent if its callee is).
pub fn keep_convergent(ir: &str, was: &BTreeSet<String>) -> String {
    let groups: std::collections::HashMap<String, String> = regex!(r"(?m)^attributes #(\d+) = \{ (.*) \}$").captures_iter(ir).map(|c| (c[1].to_string(), c[2].to_string())).collect();
    let lost: BTreeSet<String> = was.difference(&convergent_functions(ir)).cloned().collect();
    if lost.is_empty() {
        return ir.to_string();
    }
    let mut next = groups.keys().filter_map(|k| k.parse::<usize>().ok()).max().map_or(0, |m| m + 1);
    let mut added: std::collections::BTreeMap<String, usize> = Default::default();
    let with_convergent = |g: &str| -> String {
        if groups.get(g).is_some_and(|a| a.split(' ').any(|x| x == "convergent")) {
            return g.to_string();
        }
        added
            .entry(g.to_string())
            .or_insert_with(|| {
                next += 1;
                next - 1
            })
            .to_string()
    };
    let lines: Vec<String> = {
        let mut with_convergent = with_convergent;
        ir.lines()
        .map(|l| {
            let callee = if l.starts_with("define ") {
                regex!(r#"@("?[\w.$]+"?)\("#).captures(l).map(|c| c[1].to_string())
            } else {
                regex!(r#"^\s+(?:%[\w.]+ = )?(?:tail )?call [^@]*@("?[\w.$]+"?)\("#).captures(l).map(|c| c[1].to_string())
            };
            match callee {
                Some(f) if lost.contains(&f) => regex!(r"#(\d+)( \{|,|$)").replace(l, |c: &Captures| format!("#{}{}", with_convergent(&c[1]), &c[2])).to_string(),
                _ => l.to_string(),
            }
        })
        .collect()
    };
    let mut ir = lines.join("\n");
    for (old, new) in &added {
        ir += &format!("\nattributes #{new} = {{ convergent {} }}", groups[old]);
    }
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
            // Xcode leaves an access through a global (a constexpr sampler, a function constant), a call
            // without pointers, or one to thread memory (no address space), unannotated
            let thread_only = |t: &str| t.contains('*') && !t.contains("addrspace(");
            match regex!(r"\bcall [^@]*@[\w.$\x22%]+\((.*)\)").captures(line) {
                Some(args) if args[1].contains('@') || !args[1].contains('*') || thread_only(&args[1]) => return out,
                None if line.contains('@') => return out,
                None if thread_only(line.split(", !").next().unwrap_or("")) => return out,
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
/// gives it the element's scalar type (float4 -> "float"), for a vector field of
/// a struct copy's !tbaa.struct too. `structs`: each record's struct-copy fields
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


/// Xcode's GPU target runs LLVM's SpeculativeExecution (branches diverge), as
/// our pipeline now does: an if-then block's cheap instructions go up to the
/// branch, and InstCombine sinks those used only there back to the top of the
/// block. LLVM 15 sinks them back in their order; Xcode's InstCombine moves
/// each to the front once all its users are in the block. So the then-block's
/// leading run of such instructions is put in Xcode's order.
pub fn speculate_and_sink(ir: &str) -> String {
    map_functions(ir, |f| {
        let lines: Vec<&str> = f.lines().collect();
        // blocks: (label, first line, last line inclusive)
        let mut blocks: Vec<(String, usize, usize)> = Vec::new();
        for (i, l) in lines.iter().enumerate() {
            if i == 0 {
                continue;
            }
            if let Some(c) = regex!(r"^([-\w.$]+):").captures(l) {
                if let Some(b) = blocks.last_mut() {
                    b.2 = i - 1;
                }
                blocks.push((c[1].to_string(), i + 1, i));
            } else if blocks.is_empty() && !l.trim().is_empty() && *l != "}" {
                blocks.push((String::new(), i, i));
            }
            if *l == "}" {
                if let Some(b) = blocks.last_mut() {
                    b.2 = i - 1;
                }
            }
        }
        let term = |b: &(String, usize, usize)| -> &str {
            (b.1..=b.2).rev().map(|i| lines[i].trim()).find(|l| !l.is_empty()).unwrap_or("")
        };
        let preds = |label: &str| -> Vec<usize> {
            let re = Regex::new(&format!(r"label %{}\b", regex::escape(label))).unwrap();
            blocks.iter().enumerate().filter(|(_, b)| re.is_match(term(b))).map(|(k, _)| k).collect()
        };
        let mut new_lines: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
        for (bi, b) in blocks.iter().enumerate() {
            if b.0.is_empty() {
                continue;
            }
            // a triangle's then-block: one predecessor ending in a conditional branch to it and to our successor
            let p = preds(&b.0);
            if p.len() != 1 {
                continue;
            }
            let pt = term(&blocks[p[0]]);
            let Some(pc) = regex!(r"^br i1 [^,]+, label %([-\w.$]+), label %([-\w.$]+)(?:, !.*)?$").captures(pt) else { continue };
            let Some(succ) = regex!(r"^br label %([-\w.$]+)(?:, !.*)?$").captures(term(b)).map(|c| c[1].to_string()) else { continue };
            let other = if pc[1] == b.0 { pc[2].to_string() } else { pc[1].to_string() };
            if other != succ || blocks.iter().position(|x| x.0 == b.0) != Some(bi) {
                continue;
            }
            // the leading run of speculatable instructions, in order
            let body: Vec<usize> = (b.1..=b.2).filter(|&i| !lines[i].trim().is_empty() && !lines[i].trim_start().starts_with("br ")).collect();
            let op_of = |i: usize| regex!(r"^%[\w.]+ = (\w+)").captures(lines[i].trim()).map(|c| c[1].to_string()).unwrap_or_default();
            let cheap = |i: usize| {
                matches!(op_of(i).as_str(), "getelementptr" | "zext" | "sext" | "trunc" | "bitcast" | "addrspacecast" | "add" | "sub" | "mul" | "shl" | "lshr" | "ashr" | "and" | "or" | "xor" | "icmp" | "select" | "fadd" | "fsub" | "fmul" | "fdiv" | "fneg" | "fcmp" | "extractelement" | "insertelement" | "shufflevector")
            };
            // SpeculativeExecution hoists nothing from a block that would leave more than 5 of its
            // instructions behind (its terminator among them) or whose speculation costs more than 7:
            // a speculatable instruction counts unless an operand of it from the block stays behind
            let mut behind: Vec<String> = Vec::new();
            let (mut cost, mut left) = (0, 1);
            for &i in &body {
                let def = regex!(r"^\s+(%[\w.]+) = ").captures(lines[i]).map(|c| c[1].to_string()).unwrap_or_default();
                let rhs = lines[i].split_once(" = ").map_or(lines[i], |x| x.1);
                if cheap(i) && !regex!(r"%[\w.]+").find_iter(rhs).any(|o| behind.iter().any(|d| d == o.as_str())) {
                    cost += if matches!(op_of(i).as_str(), "bitcast" | "addrspacecast") { 0 } else { 1 };
                } else {
                    left += 1;
                    behind.push(def);
                }
            }
            if cost > 7 || left > 5 {
                continue;
            }
            let mut hoisted: Vec<usize> = Vec::new();
            for &i in &body {
                if !cheap(i) {
                    break;
                }
                hoisted.push(i);
            }
            if hoisted.len() < 2 {
                continue;
            }
            // sink back: forward, each to the front once its users are all in the block
            let mut front: Vec<usize> = Vec::new();
            let mut pending: Vec<usize> = hoisted.clone();
            let users_back = |i: usize, pending: &Vec<usize>| -> bool {
                let Some(d) = regex!(r"^\s+(%[\w.]+) = ").captures(lines[i]).map(|c| c[1].to_string()) else { return true };
                let re = Regex::new(&format!(r"{}\b", regex::escape(&d))).unwrap();
                !pending.iter().any(|&j| j != i && re.is_match(lines[j].split_once(" = ").map_or(lines[j], |x| x.1)))
            };
            let mut work: Vec<usize> = hoisted.iter().rev().cloned().collect();
            while let Some(i) = work.pop() {
                if !pending.contains(&i) || !users_back(i, &pending) {
                    continue;
                }
                pending.retain(|&j| j != i);
                front.insert(0, i);
                // its operands among the hoisted come up next, the last first (InstCombine's worklist is a stack)
                let rhs = lines[i].split_once(" = ").map_or(lines[i], |x| x.1);
                for op in regex!(r"%[\w.]+").find_iter(rhs) {
                    if let Some(&j) = pending.iter().find(|&&j| regex!(r"^\s+(%[\w.]+) = ").captures(lines[j]).is_some_and(|c| &c[1] == op.as_str())) {
                        work.push(j);
                    }
                }
            }
            if !pending.is_empty() {
                continue;
            }
            let rest: Vec<usize> = body.iter().filter(|i| !hoisted.contains(i)).cloned().collect();
            let order: Vec<usize> = front.into_iter().chain(rest).collect();
            for (slot, src) in body.iter().zip(order) {
                new_lines[*slot] = lines[src].to_string();
            }
        }
        let mut out = new_lines.join("\n");
        if f.ends_with('\n') {
            out.push('\n');
        }
        out
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn convergent_after_the_first_inference_stays() {
        let first = "define float @f(float %x) #0 {\n  ret float %x\n}\ndefine float @g(float %x) #0 {\n  %y = call float @f(float %x) #1\n  ret float %y\n}\nattributes #0 = { convergent nounwind }\nattributes #1 = { convergent nobuiltin }\n";
        assert_eq!(convergent_functions(first).into_iter().collect::<Vec<_>>(), vec!["f".to_string(), "g".to_string()]);
        // only g calls a function the module defines
        assert_eq!(convergent_callers(first).into_iter().collect::<Vec<_>>(), vec!["g".to_string()]);
        let second = "define float @f(float %x) #0 {\n  ret float %x\n}\ndefine float @g(float %x) #2 {\n  %y = call float @f(float %x) #1, !noalias !3\n  ret float %y\n}\nattributes #0 = { nounwind }\nattributes #1 = { nobuiltin }\nattributes #2 = { convergent nounwind }\n";
        let out = keep_convergent(second, &["f".to_string()].into_iter().collect());
        assert!(out.contains("define float @f(float %x) #3 {"));
        assert!(out.contains("call float @f(float %x) #4, !noalias !3"));
        assert!(out.contains("attributes #3 = { convergent nounwind }"));
        assert!(out.contains("attributes #4 = { convergent nobuiltin }"));
        assert!(out.contains("define float @g(float %x) #2 {"));
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
    fn then_block_sinks_in_xcode_order() {
        let ir = "define void @k(float* %o, float* %a) {\n  br i1 %c, label %then, label %end\n\nthen:\n  %i = zext i32 %n to i64\n  %pa = getelementptr inbounds float, float* %a, i64 %i\n  %po = getelementptr inbounds float, float* %o, i64 %i\n  %v = load float, float* %pa, align 4\n  store float %v, float* %po, align 4\n  br label %end\n\nend:\n  ret void\n}\n";
        let out = speculate_and_sink(ir);
        let order: Vec<&str> = out.lines().filter(|l| l.contains(" = ") || l.trim_start().starts_with("store")).map(|l| l.trim().split(' ').next().unwrap()).collect();
        assert_eq!(order, vec!["%i", "%po", "%pa", "%v", "store"]);
    }

    #[test]
    fn a_sunk_instructions_last_operand_sinks_first() {
        let ir = "define float @f(float %m, float %a, float %b) {\nentry:\n  br i1 %c, label %then, label %end\n\nthen:\n  %mul9 = fmul fast float %m, %a\n  %add = fadd fast float %mul9, 1.000000e+00\n  %mul10 = fmul fast float %m, %b\n  %add11 = fadd fast float %mul10, 1.000000e+00\n  %div = fdiv fast float %add, %add11\n  br label %end\n\nend:\n  %p = phi float [ %div, %then ], [ %m, %entry ]\n  ret float %p\n}\n";
        let order = |ir: &str| -> Vec<String> { ir.lines().filter(|l| l.contains(" = ") && !l.contains("phi")).map(|l| l.trim().split(' ').next().unwrap().to_string()).collect() };
        assert_eq!(order(&speculate_and_sink(ir)), vec!["%mul9", "%add", "%mul10", "%add11", "%div"]);
    }

    #[test]
    fn a_block_over_speculations_budget_stays_as_it_is() {
        // eight speculatable instructions cost more than 7: nothing is hoisted, so nothing moves
        let ir = "define float @f(float %m, float %a, float %b, <3 x float> %v) {\nentry:\n  br i1 %c, label %then, label %end\n\nthen:\n  %mul10 = fmul fast float %m, %b\n  %add11 = fadd fast float %mul10, 1.000000e+00\n  %mul9 = fmul fast float %m, %a\n  %add = fadd fast float %mul9, 1.000000e+00\n  %div = fdiv fast float %add, %add11\n  %ins = insertelement <3 x float> poison, float %div, i64 0\n  %spl = shufflevector <3 x float> %ins, <3 x float> poison, <3 x i32> zeroinitializer\n  %mul14 = fmul fast <3 x float> %spl, %v\n  br label %end\n\nend:\n  %p = phi <3 x float> [ %mul14, %then ], [ %v, %entry ]\n  ret float %m\n}\n";
        assert_eq!(speculate_and_sink(ir), ir);
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
    fn alias_scopes_named_as_xcode() {
        let ir = "  %v = load float, float addrspace(1)* %p, align 4, !alias.scope !3, !noalias !5\n!1 = distinct !{!1, !\"__msl_entry.k\"}\n!2 = distinct !{!2, !1, !\"__msl_entry.k: %__air_arg_1\"}\n!3 = !{!2}\n!4 = distinct !{!4, !1, !\"__msl_entry.k: %__air_tex_0\"}\n!5 = !{!4}\n";
        let out = alias_scopes(ir);
        assert!(out.contains("!\"air-alias-scopes(k)\""));
        assert!(out.contains("!\"air-alias-scope-arg(1)\""));
        assert!(out.contains("!\"air-alias-scope-textures\""));
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
    let gep = regex!(r"getelementptr (inbounds )?(\(?)");
    let mut out = String::with_capacity(ir.len());
    for line in ir.lines() {
        let mut l = line.to_string();
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
