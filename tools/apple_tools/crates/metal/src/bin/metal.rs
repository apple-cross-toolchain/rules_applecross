//! `metal -c -target air64-apple-ios17.0 -ffast-math -o x.air x.metal`, as
//! rules_apple runs it. `-S -emit-llvm` writes the AIR module as text instead.

use metal::driver::{compile, llvm_bin, Options};
use metal::target::Target;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: metal: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let (mut target, mut out, mut source, mut llvm) = (None, None, None, None);
    let (mut includes, mut defines, mut text, mut keep) = (Vec::new(), Vec::new(), false, None);
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "-target" => target = it.next(),
            "-o" => out = it.next(),
            "--llvm-bin" => llvm = it.next(),
            "--keep" => keep = it.next().map(PathBuf::from),
            "-S" => text = true,
            "-I" | "-D" | "-U" => {
                let v = it.next().ok_or(format!("{a} needs a value"))?;
                if a == "-I" { includes.push(format!("-I{v}")) } else { defines.push(format!("{a}{v}")) }
            }
            _ if a.starts_with("-I") => includes.push(a),
            _ if a.starts_with("-D") || a.starts_with("-U") => defines.push(a),
            // -c, -ffast-math, -emit-llvm, -std=, -g...: fast math is our default
            _ if a.starts_with('-') => {}
            _ => source = Some(PathBuf::from(a)),
        }
    }
    let target = Target::parse(&target.ok_or("no -target")?)?;
    let source = source.ok_or("no input file")?;
    let out = out.ok_or("no -o")?;
    let result = compile(&Options { target, source, includes, defines, llvm_bin: llvm_bin(llvm)?, keep })?;
    if text {
        std::fs::write(&out, result.ir)
    } else {
        std::fs::write(&out, result.air)
    }
    .map_err(|e| format!("{out}: {e}"))
}
