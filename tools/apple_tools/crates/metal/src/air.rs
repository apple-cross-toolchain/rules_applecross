//! The last step: optimised spir64 IR -> an AIR module (triple, datalayout,
//! parameter attributes and the AIR metadata).

use crate::ast::Program;
use crate::descriptors::{Descriptors, Md};
use crate::ir::ConstantEntry;
use crate::target::Target;
use crate::text::{find_define, quote as s, replace_all, split_args};
use crate::types::{itanium, Types};
use regex::Regex;
use std::sync::LazyLock;

macro_rules! regex {
    ($r:expr) => {{
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new($r).unwrap());
        &*RE
    }};
}

/// The Metal front end we match: its llvm.ident and its bitcode's producer.
pub const PRODUCER: &str = "32023.922.4";

pub const DATALAYOUT: &str = "e-p:64:64:64-i1:8:8-i8:8:8-i16:16:16-i32:32:32-i64:64:64-f32:32:32-f64:64:64-v16:16:16-v24:32:32-v32:32:32-v48:64:64-v64:64:64-v96:128:128-v128:128:128-v192:256:256-v256:256:256-v512:512:512-v1024:1024:1024-n8:16:32";

pub struct Inputs<'a, 'b> {
    pub types: &'b Types<'a>,
    pub program: &'a Program,
    pub target: &'b Target,
    pub source_path: &'b str,
    /// The source as named on the command line (the module's source_filename).
    pub source_name: &'b str,
    /// Function constants the module declares, and their source order.
    pub constants: &'b [ConstantEntry],
    pub constant_order: &'b [String],
    pub samplers: usize,
}

pub fn to_air(ir: &str, x: &Inputs) -> Result<String, String> {
    let (air, lang) = x.target.versions();
    let mut ir = regex!(r#"target datalayout = ".*""#).replace(ir, format!("target datalayout = \"{DATALAYOUT}\"").as_str()).to_string();
    ir = regex!(r#"target triple = ".*""#).replace(&ir, format!("target triple = \"{}\"", x.target.triple()).as_str()).to_string();
    ir = regex!(r#"(?m)^source_filename = ".*""#).replace(&ir, format!("source_filename = \"{}\"", x.source_name).as_str()).to_string();
    ir = ir.replace("dso_local ", "").replace("spir_func ", "").replace("spir_kernel ", "");
    // Xcode gives return values no noundef
    ir = regex!(r"(?m)^(define (?:linkonce_odr |internal |weak_odr )?(?:local_unnamed_addr )?)noundef ").replace_all(&ir, "$1").to_string();
    ir = regex!(r"(call (?:fast )?)noundef ").replace_all(&ir, "$1").to_string();
    // TBAA names MSL's half as such
    ir = ir.replace("!\"_Float16\"", "!\"half\"");
    // clang's own named metadata goes; its numbered nodes (TBAA etc.) are renumbered after ours
    ir = regex!(r"(?m)^!(llvm\.module\.flags|llvm\.ident|opencl\.\w+) = .*\n").replace_all(&ir, "").to_string();
    let mut d = Descriptors { types: x.types, air, md: Md::default() };
    let mut flags = vec![
        d.md.add("!{i32 2, !\"SDK Version\", [2 x i32] [i32 27, i32 2]}".into()),
        d.md.add("!{i32 1, !\"wchar_size\", i32 4}".into()),
        d.md.add("!{i32 7, !\"frame-pointer\", i32 2}".into()),
    ];
    if air >= (2, 3) {
        for (k, v) in [("device_buffers", 31), ("constant_buffers", 31), ("threadgroup_buffers", 31), ("textures", 128), ("read_write_textures", 8), ("samplers", 16)] {
            flags.push(d.md.add(format!("!{{i32 7, !\"air.max_{k}\", i32 {v}}}")));
        }
    }
    let mut stages: Vec<(&str, Vec<String>)> = vec![("kernel", vec![]), ("vertex", vec![]), ("fragment", vec![])];
    for f in &x.program.functions {
        let (sig, body) = find_define(&ir, &f.name).ok_or_else(|| format!("no function {} in the IR", f.name))?;
        let (sig, body) = (sig.to_string(), body.to_string());
        let params = x.types.flat_params(f);
        let names: Vec<String> = split_args(&sig).iter().map(|a| a.rsplit(' ').next().unwrap().to_string()).collect();
        let used: Vec<bool> = names.iter().map(|n| Regex::new(&format!(r"{}\b", regex::escape(n))).unwrap().is_match(&body)).collect();
        let mut descs = Vec::new();
        for (k, p) in params.iter().enumerate() {
            let u = *used.get(k).unwrap_or(&true);
            descs.push(if p.stage_in { d.stage_in(k, p, &f.stage, u) } else { d.argument(k, p, u)? });
        }
        let def = Regex::new(&format!(r"(?m)^define ([^@]*?) ?@{}\(", regex::escape(&f.name))).unwrap();
        let fty = regex!(r"^(?:(?:dso_local|local_unnamed_addr|noundef|zeroext|signext) )*").replace(&def.captures(&ir).unwrap()[1], "").to_string();
        let argtys: Vec<String> = split_args(&sig)
            .iter()
            .map(|a| {
                let a = regex!(r#"\s+(?:noalias|nocapture|noundef|readonly|writeonly|readnone|nonnull|zeroext|signext|inreg|immarg|returned|"[^"]*"|align \d+|dereferenceable(?:_or_null)?\(\d+\))"#).replace_all(a, "");
                a.rsplit_once(" %").map(|(t, _)| t.to_string()).unwrap_or(a.to_string())
            })
            .collect();
        let mut extra = String::new();
        let mut fn_attrs = String::new();
        for a in &f.attrs {
            if let Some(c) = regex!(r"^max_total_threads_per_threadgroup\((\d+)\)").captures(a) {
                extra += &format!(", {}", d.md.add(format!("!{{!\"air.max_work_group_size\", i32 {}}}", &c[1])));
                fn_attrs += &format!(" \"max-work-group-size\"=\"{}\"", &c[1]);
            }
        }
        let outs = d.outputs(f);
        let args = d.md.list(&descs);
        let node = d.md.add(format!("!{{{fty} ({})* @{}, {outs}, {args}{extra}}}", argtys.join(", "), f.name));
        stages.iter_mut().find(|(s, _)| *s == f.stage).unwrap().1.push(node);
        // buffers and threadgroup memory get "air-buffer-no-alias"; stage_in values and textures and samplers no noundef
        let mut args = split_args(&sig);
        for (k, p) in params.iter().enumerate() {
            if k >= args.len() {
                break;
            }
            if p.attrs.first().map(|a| a.starts_with("buffer(") || a.starts_with("threadgroup(")).unwrap_or(false) && !args[k].contains("\"air-buffer-no-alias\"") {
                args[k] = regex!(r" (%[\w.]+)$").replace(&args[k], " \"air-buffer-no-alias\" $1").to_string();
            }
            if p.stage_in || regex!(r"%struct\._(?:texture|depth)_\w+ addrspace|%struct\._sampler_t addrspace").is_match(&args[k]) {
                args[k] = args[k].replace(" noundef", "");
            }
            // no extension attributes on an entry point's parameters
            args[k] = args[k].replace(" zeroext", "").replace(" signext", "");
        }
        ir = ir.replacen(&format!("@{}({})", f.name, sig), &format!("@{}({})", f.name, args.join(", ")), 1);
        if !fn_attrs.is_empty() {
            let def = Regex::new(&format!(r"(?m)^(define [^@]*@{}\(.*\).*?)( \{{)$", regex::escape(&f.name))).unwrap();
            ir = def.replace(&ir, |c: &regex::Captures| format!("{}{fn_attrs}{}", &c[1], &c[2])).to_string();
        }
    }
    // function constants: name the initializers as Xcode does
    let mut fcnodes = Vec::new();
    let mut constants: Vec<&ConstantEntry> = x.constants.iter().collect();
    constants.sort_by_key(|e| x.constant_order.iter().position(|n| *n == e.1).unwrap_or(usize::MAX));
    for (idx, name, lty) in constants {
        let g = format!("__msl_fcinit_{idx}_{name}");
        let refs = Regex::new(&format!(r"@{}\b", regex::escape(&g))).unwrap();
        if refs.find_iter(&ir).count() <= 1 {
            // only its definition: unused
            ir = Regex::new(&format!(r"(?m)^@{} = .*\n", regex::escape(&g))).unwrap().replace_all(&ir, "").to_string();
            continue;
        }
        let mty = x.types.msl_name(x.program.constant_types.get(name).map(String::as_str).unwrap_or(""));
        let new = format!("_Z{}{}.MTL_FC_INIT_{idx}_{}", name.len(), name, itanium(&mty));
        ir = refs.replace_all(&ir, format!("@{new}").as_str()).to_string();
        fcnodes.push(d.md.add(format!("!{{{lty} addrspace(2)* @{new}, {}, {}, i32 {idx}}}", s(&mty), s(name))));
    }
    // constexpr samplers
    let mut snodes = Vec::new();
    let mut live = 0;
    for k in 0..x.samplers {
        let re = Regex::new(&format!(r"@__msl_sampler_{k}\b")).unwrap();
        if !re.is_match(&ir) {
            continue;
        }
        let name = if live == 0 { "__air_sampler_state".to_string() } else { format!("__air_sampler_state.{live}") };
        live += 1;
        ir = re.replace_all(&ir, format!("@{name}").as_str()).to_string();
        let tre = Regex::new(&format!(r"(?m)^@{} = internal addrspace\(2\) constant (\[2 x i64\]|i64) ", regex::escape(&name))).unwrap();
        let ty = tre.captures(&ir).map(|c| c[1].to_string()).unwrap_or("i64".into());
        snodes.push(d.md.add(format!("!{{!\"air.sampler_state\", {ty} addrspace(2)* @{name}}}")));
    }
    let opts = ["air.compile.denorms_disable", "air.compile.fast_math_enable", "air.compile.framebuffer_fetch_enable"].map(|o| d.md.add(format!("!{{{}}}", s(o))));
    let ident = d.md.add(format!("!{{!\"Apple metal version {} (metalfe-{PRODUCER})\"}}", PRODUCER.rsplit_once('.').unwrap().0));
    let vers = d.md.add(format!("!{{i32 2, i32 {}, i32 0}}", air.1));
    let langv = d.md.add(format!("!{{!\"Metal\", i32 {}, i32 {}, i32 0}}", lang.0, lang.1));
    let srcn = d.md.add(format!("!{{{}}}", s(x.source_path)));
    // renumber clang's nodes past ours
    let base = d.md.nodes.len();
    ir = replace_all(&ir, regex!(r"!(\d+)\b"), |c| format!("!{}", c[1].parse::<usize>().unwrap() + base));
    let mut named = vec![format!("!llvm.module.flags = !{{{}}}", flags.join(", "))];
    for (st, nodes) in &stages {
        if !nodes.is_empty() {
            named.push(format!("!air.{st} = !{{{}}}", nodes.join(", ")));
        }
    }
    named.push(format!("!air.compile_options = !{{{}}}", opts.join(", ")));
    if !fcnodes.is_empty() {
        named.push(format!("!air.function_constants = !{{{}}}", fcnodes.join(", ")));
    }
    if !snodes.is_empty() {
        named.push(format!("!air.sampler_states = !{{{}}}", snodes.join(", ")));
    }
    named.push(format!("!llvm.ident = !{{{ident}}}"));
    named.push(format!("!air.version = !{{{vers}}}"));
    named.push(format!("!air.language_version = !{{{langv}}}"));
    named.push(format!("!air.source_file_name = !{{{srcn}}}"));
    // Xcode's front end builds a stage function's packed return on undef
    ir = regex!(r"(insertvalue <\{[^}]*\}>) poison,").replace_all(&ir, "$1 undef,").to_string();
    let defined: Vec<String> = regex!(r#"(?m)^define [^@\n]*@("?[\w.$]+"?)\("#).captures_iter(&ir).map(|c| c[1].to_string()).collect();
    ir = constant_reads(&ir, &defined.iter().map(String::as_str).collect::<Vec<_>>());
    ir = after_constant_reads(&ir);
    // the static initialiser of function constants: Xcode's front end gives it no vector width
    let inits: Vec<String> = regex!(r#"(?m)^define [^@]*@([\w.$]+)\(.*section "air\.static_init""#).captures_iter(&ir).map(|c| c[1].to_string()).collect();
    for f in inits {
        ir = edit_attrs(&ir, &f, |a| regex!(r#""min-legal-vector-width"="\d+""#).replace(a, "\"min-legal-vector-width\"=\"0\"").to_string());
    }
    let md: String = d.md.nodes.iter().enumerate().map(|(i, b)| format!("!{i} = {b}\n")).collect();
    Ok(format!("{}\n\n{}\n\n{}", ir.trim_end(), named.join("\n"), md))
}

/// Give `name`'s definition its own copy of its attribute group, edited.
fn edit_attrs(ir: &str, name: &str, edit: impl Fn(&str) -> String) -> String {
    let def = Regex::new(&format!(r"(?m)^(define .*?@{}\(.*\)[^#\n]*#)(\d+)", regex::escape(name))).unwrap();
    let Some(c) = def.captures(ir) else { return ir.to_string() };
    let gre = Regex::new(&format!(r"(?m)^attributes #{} = \{{ (.*) \}}$", &c[2])).unwrap();
    let Some(g) = gre.captures(ir) else { return ir.to_string() };
    let new_attrs = edit(&g[1]);
    if new_attrs == g[1] {
        return ir.to_string();
    }
    let n = regex!(r"(?m)^attributes #(\d+) = ").captures_iter(ir).map(|c| c[1].parse::<usize>().unwrap()).max().map_or(0, |m| m + 1);
    let ir = def.replace(ir, |c: &regex::Captures| format!("{}{n}", &c[1])).to_string();
    ir.trim_end().to_string() + &format!("\nattributes #{n} = {{ {new_attrs} }}\n")
}

/// Each function's "min-legal-vector-width" as clang 15 sets it in Xcode's front
/// end, before inlining merges them: its vector parameters and return value and
/// the vectors its calls pass and return. Our own stdlib helpers (metal::__*),
/// which Xcode's front end has no calls to, don't count.
pub fn vector_widths(ir: &str) -> String {
    fn direct(t: &str) -> usize {
        let t = t.trim();
        // a vector, not a pointer to one
        if regex!(r"^<\d+ x \w+>\s*(?:\*|addrspace)").is_match(t) {
            return 0;
        }
        regex!(r"^<(\d+) x (half|float|double|bfloat|i\d+)>")
            .captures(t)
            .map(|c| {
                let bits = match &c[2] {
                    "half" | "bfloat" => 16,
                    "float" => 32,
                    "double" => 64,
                    i => i[1..].parse().unwrap_or(0),
                };
                c[1].parse::<usize>().unwrap_or(0) * bits
            })
            .unwrap_or(0)
    }
    let helper = regex!(r#"^"?_ZNK?5metal\d+__"#);
    let mut widths: Vec<(usize, usize)> = Vec::new(); // (define line start, width)
    for (start, end) in crate::text::functions(ir) {
        let f = &ir[start..end];
        let head = f.lines().next().unwrap_or("");
        let Some(at) = head.find(" @") else { continue };
        // our helpers have no counterpart to merge into what inlines them
        if helper.is_match(&head[at + 2..]) {
            widths.push((start, 0));
            continue;
        }
        let open = at + head[at..].find('(').unwrap_or(0);
        let close = crate::text::balanced(head.as_bytes(), open, b'(', b')');
        let mut w = direct(regex!(r"(<\d+ x \w+>)$").captures(&head[..at]).map(|c| c[1].to_string()).unwrap_or_default().as_str());
        for a in split_args(&head[open + 1..close - 1]) {
            w = w.max(direct(&a));
        }
        for l in f.lines().skip(1) {
            let Some(c) = regex!(r#"\bcall (?:[a-z_]+ )*(<\d+ x \w+>|[^ @]+) @("?[^("]+"?)\((.*)\)"#).captures(l) else { continue };
            if helper.is_match(&c[2]) {
                continue;
            }
            w = w.max(direct(&c[1]));
            for a in split_args(&c[3]) {
                w = w.max(direct(&a));
            }
        }
        widths.push((start, w));
    }
    // one attribute group per (group, width)
    let groups: std::collections::HashMap<String, String> = regex!(r"(?m)^attributes #(\d+) = \{ (.*) \}$").captures_iter(ir).map(|c| (c[1].to_string(), c[2].to_string())).collect();
    let mut next = groups.keys().filter_map(|k| k.parse::<usize>().ok()).max().map_or(0, |m| m + 1);
    let mut made: std::collections::HashMap<(String, usize), usize> = Default::default();
    let mut new_groups = String::new();
    let mut out = String::with_capacity(ir.len());
    let mut pos = 0;
    for (start, w) in widths {
        let line_end = start + ir[start..].find('\n').unwrap_or(ir.len() - start);
        let line = &ir[start..line_end];
        let Some(c) = regex!(r"#(\d+)[^#]*$").captures(line) else { continue };
        let Some(attrs) = groups.get(&c[1]) else { continue };
        if !attrs.contains("\"min-legal-vector-width\"") {
            continue;
        }
        let id = *made.entry((c[1].to_string(), w)).or_insert_with(|| {
            let n = next;
            next += 1;
            let a = regex!(r#""min-legal-vector-width"="\d+""#).replace(attrs, format!("\"min-legal-vector-width\"=\"{w}\"").as_str());
            new_groups += &format!("attributes #{n} = {{ {a} }}\n");
            n
        });
        let m = c.get(1).unwrap();
        out.push_str(&ir[pos..start + m.start()]);
        out.push_str(&id.to_string());
        pos = start + m.end();
    }
    out.push_str(&ir[pos..]);
    out.trim_end().to_string() + "\n" + &new_groups
}

/// A call to a function that reads no memory carries no alias scopes (the
/// inliner adds none), and Xcode's LLVM gives lifetime markers mustprogress.
fn after_constant_reads(ir: &str) -> String {
    let groups: std::collections::HashMap<String, String> = regex!(r"(?m)^attributes #(\d+) = \{ (.*) \}$").captures_iter(ir).map(|c| (c[1].to_string(), c[2].to_string())).collect();
    let readnone: Vec<String> = regex!(r#"(?m)^define [^@\n]*@("?[\w.$]+"?)\(.*\)[^#\n]*#(\d+)"#)
        .captures_iter(ir)
        .filter(|c| groups.get(&c[2]).is_some_and(|g| g.split(' ').any(|a| a == "readnone")))
        .map(|c| c[1].to_string())
        .collect();
    let ir = regex!(r#"(?m)^(\s+(?:%[\w.]+ = )?(?:tail )?call [^@\n]*@("?[\w.$]+"?)\(.*?)(, !alias\.scope !\d+)?(, !noalias !\d+)?$"#)
        .replace_all(ir, |c: &regex::Captures| if readnone.iter().any(|r| *r == c[2]) { c[1].to_string() } else { c[0].to_string() })
        .to_string();
    let lifetime: Vec<String> = regex!(r"(?m)^declare void @llvm\.lifetime\.\w+\.p0i8\(.*\) #(\d+)$").captures_iter(&ir).map(|c| c[1].to_string()).collect();
    regex!(r"(?m)^attributes #(\d+) = \{ argmemonly (nocallback .*)\}$")
        .replace_all(&ir, |c: &regex::Captures| if lifetime.contains(&c[1].to_string()) { format!("attributes #{} = {{ argmemonly mustprogress {}}}", &c[1], &c[2]) } else { c[0].to_string() })
        .to_string()
}

/// Xcode's alias analysis knows address space 2 is constant: a function that
/// only reads it reads no memory.
fn constant_reads(ir: &str, entries: &[&str]) -> String {
    let mut ir = ir.to_string();
    let readnone: Vec<String> = regex!(r"(?m)^declare .*? @([\w.]+)\(.*\).*\breadnone\b").captures_iter(&ir).map(|c| c[1].to_string()).collect();
    for e in entries {
        let Some((_, body)) = find_define(&ir, e) else { continue };
        let reads_other = body.lines().any(|l| {
            let l = l.trim();
            l.starts_with("store ")
                || regex!(r"= load ").is_match(l) && !regex!(r"= load [^,]*, [^,]* addrspace\(2\)\*").is_match(l)
                || regex!(r"\bcall [^@]*@([\w.]+)\(").captures(l).map(|c| !readnone.iter().any(|r| *r == c[1])).unwrap_or(false)
        });
        if reads_other {
            continue;
        }
        ir = edit_attrs(&ir, e, |a| {
            if !regex!(r"\breadonly\b").is_match(a) {
                return a.to_string();
            }
            a.replace("argmemonly ", "").replace("readonly", "readnone")
        });
    }
    ir
}
