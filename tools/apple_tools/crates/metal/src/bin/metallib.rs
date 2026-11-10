//! `metallib -o out.metallib a.air b.air ...`, as rules_apple runs it.

use metal::driver::{disassemble, llvm_bin};
use metal::library::link;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: metallib: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let (mut out, mut llvm, mut inputs) = (None, None, Vec::new());
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "-o" => out = it.next(),
            "--llvm-bin" => llvm = it.next(),
            _ if a.starts_with('-') => {}
            _ => inputs.push(PathBuf::from(a)),
        }
    }
    let out = out.ok_or("no -o")?;
    let bin = llvm_bin(llvm)?;
    let mut modules = Vec::new();
    for i in &inputs {
        let bitcode = std::fs::read(i).map_err(|e| format!("{}: {e}", i.display()))?;
        modules.push((disassemble(&bin, i)?, bitcode));
    }
    std::fs::write(&out, link(&modules)?).map_err(|e| format!("{out}: {e}"))
}
