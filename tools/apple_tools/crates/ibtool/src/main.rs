//! Clean-room `ibtool` for Linux: `--compilation-directory` and `--link`; see
//! docs/clean-room/ibtool.md.

mod fonts;
mod nib;
mod storyboard;
mod watch;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn errors_plist(message: &str) -> String {
    cf::xml::to_string(&cf::Value::dict([(
        "com.apple.ibtool.errors",
        cf::Value::Array(vec![cf::Value::dict([("description", cf::Value::string(message))])]),
    )]))
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    let mut entries: Vec<_> = std::fs::read_dir(from)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let target = to.join(e.file_name());
        if e.file_type()?.is_dir() {
            copy_dir(&e.path(), &target)?;
        } else {
            std::fs::copy(e.path(), target)?;
        }
    }
    Ok(())
}

fn run(args: Vec<String>) -> Result<(), String> {
    let mut compilation_dir: Option<PathBuf> = None;
    let mut link_dir: Option<PathBuf> = None;
    let mut inputs = Vec::new();
    let mut minimum_deployment_target: Option<String> = None;
    let mut module: Option<String> = None;
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        let mut value = |name: &str| it.next().ok_or_else(|| format!("{name} needs a value"));
        match a.as_str() {
            "--compilation-directory" => compilation_dir = Some(PathBuf::from(value(&a)?)),
            "--link" => link_dir = Some(PathBuf::from(value(&a)?)),
            "--minimum-deployment-target" => minimum_deployment_target = Some(value(&a)?),
            "--module" => module = Some(value(&a)?),
            "--output-format" | "--target-device" => {
                value(&a)?;
            }
            "--errors" | "--warnings" | "--notices" | "--auto-activate-custom-fonts" => {}
            "--version" => {
                print!("{}", cf::xml::to_string(&cf::Value::dict([("com.apple.ibtool.version", cf::Value::dict([
                    ("bundle-version", cf::Value::string("25098")), ("short-bundle-version", cf::Value::string("27.0"))]))])));
                return Ok(());
            }
            s if s.starts_with("--") => return Err(format!("Unknown argument '{s}'.")),
            _ => inputs.push(PathBuf::from(a)),
        }
    }
    if let Some(dir) = link_dir {
        for input in &inputs {
            // A WatchKit interface links to the plist alone.
            let interface = input.join("Interface.plist");
            if interface.is_file() && std::fs::read_dir(input).map_err(|e| e.to_string())?.count() == 1 {
                let compiled = std::fs::read(&interface).map_err(|e| format!("{}: {e}", interface.display()))?;
                let linked = watch::link(&compiled).map_err(|e| format!("{}: {e}", interface.display()))?;
                std::fs::write(dir.join("Interface.plist"), linked).map_err(|e| e.to_string())?;
                continue;
            }
            let name = input.file_name().ok_or("bad input path")?;
            let target = dir.join(name);
            if target.exists() {
                std::fs::remove_dir_all(&target).map_err(|e| e.to_string())?;
            }
            copy_dir(input, &target).map_err(|e| format!("{}: {e}", input.display()))?;
        }
        return Ok(());
    }
    let dir = compilation_dir.ok_or("--compilation-directory or --link is required")?;
    for input in &inputs {
        if input.extension().and_then(|e| e.to_str()) != Some("storyboard") {
            skip(format!("{}: only storyboards are supported by this ibtool", input.display()))?;
            continue;
        }
        let xml = std::fs::read_to_string(input).map_err(|e| format!("{}: {e}", input.display()))?;
        let compiled = if xml.contains("type=\"com.apple.InterfaceBuilder.WatchKit.Storyboard\"") {
            watch::compile(&xml)
        } else {
            storyboard::compile(&xml, minimum_deployment_target.as_deref(), module.as_deref()).map(|c| c.files)
        };
        let files = match compiled {
            Ok(files) => files,
            Err(e) => {
                skip(format!("{}: {e}", input.display()))?;
                continue;
            }
        };
        let stem = input.file_stem().unwrap().to_string_lossy();
        let out = dir.join(format!("{stem}.storyboardc"));
        if out.exists() {
            std::fs::remove_dir_all(&out).map_err(|e| e.to_string())?;
        }
        std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
        for (name, data) in files {
            std::fs::write(out.join(name), data).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// An input this ibtool can't compile as Xcode would: an error when strict
/// (the default), left uncompiled when relaxed
/// (RULES_APPLECROSS_RESOURCE_OUTPUT_MODE=relaxed, `--config=relaxed`).
fn skip(reason: String) -> Result<(), String> {
    if std::env::var("RULES_APPLECROSS_RESOURCE_OUTPUT_MODE").map_or(true, |v| v != "relaxed") {
        return Err(format!("{reason}; build with --config=relaxed (RULES_APPLECROSS_RESOURCE_OUTPUT_MODE=relaxed) to leave it uncompiled"));
    }
    eprintln!("note: {reason}; not compiled");
    Ok(())
}

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            print!("{}", errors_plist(&e));
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
