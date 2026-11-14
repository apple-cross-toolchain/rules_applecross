//! `metal -c`: MSL -> an AIR module with clang, opt and llvm-as from LLVM 15
//! (the last release with typed pointers, which AIR uses).
//!
//! 1. Preprocess with our metal_stdlib (macros may spell types and attributes).
//! 2. Rewrite what clang can't take as C++ (crate::rewrite).
//! 3. Read signatures, structs and typedefs from clang's AST dump.
//! 4. Append extern "C" entry points with AIR's signatures (crate::entry).
//! 5. Compile flat (no address spaces) to IR; fix calling conventions, function
//!    constants and samplers; inline everything; opt -O2; resolve
//!    address-space-dependent calls; infer address spaces; opt -O2.
//! 6. Add the AIR metadata (crate::air) and write bitcode with llvm-as.

use crate::air::{to_air, Inputs};
use crate::intrinsics;
use crate::entry::entry_points;
use crate::rewrite::rewrite;
use crate::target::Target;
use crate::types::{record_layouts, Types};
use crate::{ast, ir};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Our MSL headers, written to a private directory for each run.
const HEADERS: &[(&str, &str)] = &[
    ("metal_stdlib", include_str!("../include/metal_stdlib")),
    ("__metal_math.h", include_str!("../include/__metal_math.h")),
    ("__metal_textures.h", include_str!("../include/__metal_textures.h")),
    ("__metal_simd.h", include_str!("../include/__metal_simd.h")),
    ("TargetConditionals.h", include_str!("../include/TargetConditionals.h")),
    ("simd/simd.h", include_str!("../include/simd/simd.h")),
];
/// MSL's other standard header names: all of it is in metal_stdlib.
const ALIASES: &[&str] = &[
    "metal_integer", "metal_common", "metal_math", "metal_texture", "metal_graphics", "metal_geometric", "metal_relational",
    "metal_atomic", "metal_compute", "metal_matrix", "metal_simdgroup", "metal_types", "metal_pack", "metal_numeric",
    "metal_quadgroup", "metal_simdgroup_matrix",
];

pub struct Options {
    pub target: Target,
    pub source: PathBuf,
    pub includes: Vec<String>,
    pub defines: Vec<String>,
    pub llvm_bin: PathBuf,
    /// Keep intermediate files next to the output (debugging).
    pub keep: Option<PathBuf>,
}

pub struct Output {
    /// The AIR module as text.
    pub ir: String,
    /// Wrapped bitcode (the .air file).
    pub air: Vec<u8>,
}

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn scratch() -> Result<Scratch, String> {
    let base = std::env::temp_dir();
    for k in 0..1000 {
        let p = base.join(format!("metal-{}-{}", std::process::id(), k));
        if std::fs::create_dir(&p).is_ok() {
            return Ok(Scratch(p));
        }
    }
    Err("can't make a temporary directory".into())
}

fn run(cmd: &mut Command) -> Result<std::process::Output, String> {
    let out = cmd.output().map_err(|e| format!("{:?}: {e}", cmd.get_program()))?;
    if !out.status.success() {
        return Err(format!("{:?} failed:\n{}", cmd.get_program(), String::from_utf8_lossy(&out.stderr)));
    }
    Ok(out)
}

fn tool(o: &Options, name: &str) -> PathBuf {
    o.llvm_bin.join(name)
}

/// LLVM 15's -O2, spelled out: named so, speculative-execution runs on every
/// target, as on Xcode's (whose GPU target has divergent branches); without
/// vector-combine, whose scalar-to-vector folds Xcode's LLVM doesn't make.
static O2: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| format!("-passes={}", include_str!("../data/o2_pipeline.txt").trim()));

pub fn compile(o: &Options) -> Result<Output, String> {
    let tmp = scratch()?;
    let inc = tmp.0.join("include");
    std::fs::create_dir_all(inc.join("simd")).map_err(|e| e.to_string())?;
    for (name, text) in HEADERS {
        std::fs::write(inc.join(name), text).map_err(|e| e.to_string())?;
    }
    for name in ALIASES {
        std::fs::write(inc.join(name), "#pragma once\n#include <metal_stdlib>\n").map_err(|e| e.to_string())?;
    }
    let (air, lang) = o.target.versions();
    let src_dir = o.source.parent().map(Path::to_path_buf).unwrap_or_default();
    let clang = tool(o, "clang");
    let preprocess = |ast_mode: bool, mangle: bool| -> Result<crate::rewrite::Rewritten, String> {
        let mut c = Command::new(&clang);
        c.args(["-E", "-x", "c++", "-std=c++17", "-target", "spir64", "-ffast-math", "-nostdinc++", "-I"]).arg(&inc).args(["-include", "metal_stdlib", "-I"]).arg(&src_dir);
        c.arg(format!("-D__METAL_VERSION__={}", lang.0 * 100 + lang.1 * 10));
        match o.target.os.as_str() {
            "macosx" => {
                c.arg("-D__METAL_MACOS__").arg("-D__MSL_TARGET_MACOS");
            }
            "ios" => {
                c.arg("-D__METAL_IOS__").arg("-D__MSL_TARGET_IOS");
            }
            "tvos" => {
                c.arg("-D__MSL_TARGET_TVOS");
            }
            _ => {
                c.arg("-D__MSL_TARGET_XROS");
            }
        }
        if o.target.simulator {
            c.arg("-D__MSL_TARGET_SIMULATOR");
        }
        if ast_mode {
            c.arg("-D__MSL_AST");
        }
        if mangle {
            c.arg("-D__MSL_MANGLE");
        }
        c.args(&o.includes).args(&o.defines).arg(&o.source);
        let text = String::from_utf8_lossy(&run(&mut c)?.stdout).to_string();
        rewrite(&text, ast_mode, air)
    };
    let base = |c: &mut Command| {
        c.args(["-x", "c++", "-std=c++17", "-target", "spir64", "-ffast-math", "-fno-builtin", "-fno-exceptions", "-Wno-unknown-attributes", "-nostdinc++"]);
    };
    // The AST pass (address spaces as attributes; its errors are address-space
    // checks codegen doesn't have, so they don't stop us).
    let ast_src = preprocess(true, false)?;
    // named as the source, which names its static initializer (_GLOBAL__sub_I_<file>)
    let cpp_name = o.source.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or("source.metal".into());
    let cpp = tmp.0.join(&cpp_name);
    std::fs::write(&cpp, &ast_src.code).map_err(|e| e.to_string())?;
    let mut c = Command::new(&clang);
    base(&mut c);
    c.args(["-fsyntax-only", "-Xclang", "-ast-dump", "-fno-color-diagnostics"]).arg(&cpp);
    let dump = c.output().map_err(|e| e.to_string())?;
    let program = ast::program(&String::from_utf8_lossy(&dump.stdout), &ast_src.instances);
    // functions' names with their parameters' address spaces (for Xcode's mangled names)
    let json_decls = |path: &Path| -> Result<Vec<(String, String, String)>, String> {
        let mut c = Command::new(&clang);
        base(&mut c);
        c.args(["-fsyntax-only", "-ferror-limit=0", "-Xclang", "-ast-dump=json"]).arg(path);
        Ok(crate::mangle::decls(&String::from_utf8_lossy(&c.output().map_err(|e| e.to_string())?.stdout)))
    };
    // Xcode's names: from the source with explicit thread and const constant too
    let mangle_src = preprocess(true, true)?;
    std::fs::write(&cpp, &mangle_src.code).map_err(|e| e.to_string())?;
    let spaced = json_decls(&cpp)?;
    // Codegen
    let code = preprocess(false, false)?;
    let mut types = Types { program: &program, layouts: Default::default() };
    std::fs::write(&cpp, format!("{}\n{}", code.code, entry_points(&types, &program))).map_err(|e| e.to_string())?;
    let flat: Vec<(String, String)> = json_decls(&cpp)?.into_iter().map(|(n, m, _)| (n, m)).collect();
    let user = crate::mangle::user_functions(&spaced, &flat);
    let renames: std::collections::HashMap<String, String> = user.iter().map(|(k, v)| (k.clone(), v.name.clone())).collect();
    let ll0 = tmp.0.join("out.0.ll");
    let mut c = Command::new(&clang);
    base(&mut c);
    // -fpreserve-vec3-type: store 3-vectors as 3 elements, as Xcode does (the 4th lane is padding)
    // value names kept: a conversion to bool (%tobool) is told from a comparison by its name
    c.args(["-O2", "-Xclang", "-disable-llvm-passes", "-Xclang", "-no-opaque-pointers", "-Xclang", "-fpreserve-vec3-type", "-Xclang", "-fconvergent-functions", "-fno-discard-value-names", "-S", "-emit-llvm", "-o"])
        .arg(&ll0)
        .args(["-Xclang", "-fdump-record-layouts"])
        .arg(&cpp);
    let out = run(&mut c)?;
    types.layouts = record_layouts(&String::from_utf8_lossy(&out.stdout));
    let named = tmp.0.join("out.0n.ll");
    run(Command::new(tool(o, "opt")).args(["-passes=instnamer", "-S", "-o"]).arg(&named).arg(&ll0))?;
    let text = std::fs::read_to_string(&named).map_err(|e| e.to_string())?;
    let text = ir::direct_struct_returns(&ir::renumber_types(&text), &types, &program.functions)?;
    let (text, constants) = ir::function_constants(&ir::direct_returns(&text));
    let text = ir::samplers(&text, &code.samplers, air);
    let entries: Vec<String> = program.functions.iter().map(|f| f.name.clone()).collect();
    // the user's functions as Xcode calls them (their stage functions are inlined into the entry points)
    let stages = crate::abi::stage_functions(&text, &entries);
    // each entry point's own layers (our noalias copy, the user's stage function), for telling inlined calls
    let mut home: std::collections::HashMap<String, String> = Default::default();
    for e in &entries {
        for s in crate::abi::stage_functions(&text, std::slice::from_ref(e)) {
            home.entry(s).or_insert_with(|| e.clone());
        }
    }
    let text = crate::abi::lower(&text, &user, &stages);
    let text = intrinsics::apply(&ir::user_scalar_conversions(&ir::noalias_params(&ir::vector_tbaa(&ir::prepare_flat(&crate::air::vector_widths(&text), &entries, &stages), &types.tbaa_struct_fields()), &entries)));
    let step = |input: &str, name: &str, passes: &[&str]| -> Result<String, String> {
        let i = tmp.0.join(format!("{name}.in.ll"));
        let out = tmp.0.join(format!("{name}.ll"));
        std::fs::write(&i, input).map_err(|e| e.to_string())?;
        run(Command::new(tool(o, "opt")).args(passes).args(["-S", "-o"]).arg(&out).arg(&i))?;
        std::fs::read_to_string(&out).map_err(|e| e.to_string())
    };
    // conversions become air.convert calls once constants are folded, before the last attribute inference
    let o2 = step(&ir::tag_user_calls(&text, &home), "o2", &[O2.as_str()])?;
    let convergent = ir::convergent_callers(&o2);
    let text = intrinsics::apply(&ir::air_conversions(&o2));
    // the stdlib wrappers are inlined by now, so the freeze lands on the caller's value as Xcode's does
    let text = ir::freeze_moved_values(&ir::resolve_as_markers(&ir::resolve_atomics(&ir::drop_scope_decls(&text))?)?);
    let text = step(&text, "infer", &["-passes=infer-address-spaces", "-assume-default-is-flat-addrspace"])?;
    // the intrinsics resolved since (atomics) get their attributes; Xcode keeps no local value names
    let text = ir::place_scalar_conversions(&ir::splat_poison(&ir::scalar_tbaa_struct(&ir::unwrap_packed(&ir::speculate_and_sink(&ir::xcode_alias_scopes(&intrinsics::apply(&ir::air_conversions(&ir::nounwind_inlined_calls(&step(&intrinsics::apply(&text), "o2b", &[O2.as_str()])?))), &entries))))));
    let text = crate::abi::strip_comdats(&crate::mangle::apply(&ir::keep_convergent(&text, &convergent), &renames));
    let order: Vec<String> = code.constants.iter().map(|c| c.name.clone()).collect();
    let source_path = std::fs::canonicalize(&o.source).unwrap_or(o.source.clone());
    let air_ir = to_air(
        &text,
        &Inputs {
            types: &types,
            program: &program,
            target: &o.target,
            source_path: &source_path.to_string_lossy(),
            source_name: &o.source.to_string_lossy(),
            constants: &constants,
            constant_order: &order,
            samplers: code.samplers.len(),
        },
    )?;
    // Xcode keeps no local value names
    let air_ir = crate::text::discard_value_names(&air_ir);
    let ll = tmp.0.join("air.ll");
    std::fs::write(&ll, &air_ir).map_err(|e| e.to_string())?;
    let bc = tmp.0.join("air.bc");
    // clang's writer doesn't keep use-list order (llvm-as does by default), so it sorts constants
    run(Command::new(tool(o, "llvm-as")).args(["-preserve-bc-uselistorder=false", "-o"]).arg(&bc).arg(&ll))?;
    if let Some(keep) = &o.keep {
        let _ = std::fs::create_dir_all(keep);
        for f in [cpp_name.as_str(), "out.0.ll", "o2.in.ll", "o2.ll", "infer.ll", "o2b.in.ll", "o2b.ll", "air.ll"] {
            let _ = std::fs::copy(tmp.0.join(f), keep.join(f));
        }
    }
    let ir = run(Command::new(tool(o, "llvm-dis")).arg("-o").arg("-").arg(&bc))?;
    let air = crate::bitcode::as_xcode(&std::fs::read(&bc).map_err(|e| e.to_string())?, crate::air::PRODUCER, crate::bitcode::Writer::Frontend)?;
    Ok(Output { ir: String::from_utf8_lossy(&ir.stdout).to_string(), air })
}

/// One function's module text -> its bitcode, as metallib writes it.
pub fn assemble(llvm_bin: &Path, ir: &str) -> Result<Vec<u8>, String> {
    let tmp = scratch()?;
    let (ll, bc) = (tmp.0.join("f.ll"), tmp.0.join("f.bc"));
    std::fs::write(&ll, ir).map_err(|e| e.to_string())?;
    // function constants this function doesn't read: metallib drops their initialisers'
    // stores, as GlobalOpt does (opt takes no AIR triple, so it borrows spir64's)
    if ir.contains("\"air.fc_initializer\"") {
        let triple = regex::Regex::new(r#"(?m)^target triple = "(.*)""#).unwrap().captures(ir).map(|c| c[1].to_string()).unwrap_or_default();
        std::fs::write(&ll, ir.replacen(&format!("target triple = \"{triple}\""), "target triple = \"spir64-unknown-unknown\"", 1)).map_err(|e| e.to_string())?;
        let opt_ll = tmp.0.join("f.opt.ll");
        run(Command::new(llvm_bin.join("opt")).args(["-passes=globalopt,globaldce", "-S", "-o"]).arg(&opt_ll).arg(&ll))?;
        let text = std::fs::read_to_string(&opt_ll).map_err(|e| e.to_string())?;
        // and the function constants whose initialiser went
        let gone: Vec<String> = regex::Regex::new(r"(?m)^(![0-9]+) = distinct !\{null, ").unwrap().captures_iter(&text).map(|c| c[1].to_string()).collect();
        let text = regex::Regex::new(r"(?m)^!air\.function_constants = !\{(.*)\}$")
            .unwrap()
            .replace(&text, |c: &regex::Captures| {
                let keep: Vec<&str> = c[1].split(", ").filter(|r| !gone.iter().any(|g| g == r)).collect();
                if keep.is_empty() { String::new() } else { format!("!air.function_constants = !{{{}}}", keep.join(", ")) }
            })
            .to_string();
        std::fs::write(&ll, text.replacen("target triple = \"spir64-unknown-unknown\"", &format!("target triple = \"{triple}\""), 1)).map_err(|e| e.to_string())?;
    }
    run(Command::new(llvm_bin.join("llvm-as")).args(["-preserve-bc-uselistorder=false", "-o"]).arg(&bc).arg(&ll))?;
    crate::bitcode::as_xcode(&std::fs::read(&bc).map_err(|e| e.to_string())?, crate::air::PRODUCER, crate::bitcode::Writer::Metallib)
}

/// Disassemble an .air file (for `metallib`).
pub fn disassemble(llvm_bin: &Path, air: &Path) -> Result<String, String> {
    let out = run(Command::new(llvm_bin.join("llvm-dis")).arg("-o").arg("-").arg(air))?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// The LLVM 15 tools: `--llvm-bin`, else $METAL_LLVM_BIN.
pub fn llvm_bin(flag: Option<String>) -> Result<PathBuf, String> {
    flag.or_else(|| std::env::var("METAL_LLVM_BIN").ok()).map(PathBuf::from).ok_or_else(|| "the LLVM 15 tools: pass --llvm-bin DIR or set METAL_LLVM_BIN".to_string())
}
