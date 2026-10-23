//! Clean-room `swift-stdlib-tool` for Linux; see
//! docs/clean-room/swift-stdlib-tool.md.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Default)]
struct Options {
    print: bool,
    copy: bool,
    verbose: u32,
    scan_executables: Vec<PathBuf>,
    scan_folders: Vec<PathBuf>,
    platform: Option<String>,
    source_libraries: Vec<PathBuf>,
    destination: Option<PathBuf>,
    unsigned_destination: Option<PathBuf>,
    resource_libraries: Vec<String>,
    resource_destination: Option<PathBuf>,
}

fn parse(args: Vec<String>) -> Result<Options, String> {
    let mut o = Options::default();
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        let mut value = |name: &str| it.next().ok_or_else(|| format!("{name} needs a value"));
        match a.as_str() {
            "--print" => o.print = true,
            "--copy" => o.copy = true,
            "--verbose" => o.verbose += 1,
            "--scan-executable" => o.scan_executables.push(value(&a)?.into()),
            "--scan-folder" => o.scan_folders.push(value(&a)?.into()),
            "--source-libraries" => o.source_libraries.push(value(&a)?.into()),
            "--destination" => o.destination = Some(value(&a)?.into()),
            "--unsigned-destination" => o.unsigned_destination = Some(value(&a)?.into()),
            "--resource-library" => o.resource_libraries.push(value(&a)?),
            "--resource-destination" => o.resource_destination = Some(value(&a)?.into()),
            // Swift libraries have carried no bitcode since Xcode 14.
            "--strip-bitcode" => {}
            "--platform" => o.platform = Some(value(&a)?),
            "--toolchain" => {
                value(&a)?;
            }
            "--sign" | "--keychain" | "--Xcodesign" => {
                let v = value(&a)?;
                return Err(format!("{a} {v}: signing is not supported by this swift-stdlib-tool; sign the bundle afterwards"));
            }
            other => return Err(format!("unknown option {other}")),
        }
    }
    if o.print == o.copy {
        return Err("exactly one of --print and --copy is required".into());
    }
    if o.source_libraries.is_empty() {
        // The documented default: <this tool>/../../lib/swift/<platform>.
        let platform = o.platform.clone().ok_or("--platform or --source-libraries is required")?;
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let usr = exe.parent().and_then(Path::parent).ok_or("this tool is not inside a toolchain's usr/bin")?;
        o.source_libraries.push(usr.join("lib/swift").join(platform));
    }
    Ok(o)
}

/// The Swift libraries a Mach-O file references, by file name. Files that
/// are not Mach-O reference none.
fn swift_references(path: &Path) -> Result<Vec<String>, String> {
    let data = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let Ok(file) = macho::Universal::parse(data) else { return Ok(Vec::new()) };
    let mut names = Vec::new();
    for image in file.images() {
        for d in &image.dylibs {
            if let Some(name) = d.strip_prefix("@rpath/") {
                if name.starts_with("libswift") && name.ends_with(".dylib") && !names.iter().any(|n| n == name) {
                    names.push(name.to_string());
                }
            }
        }
    }
    Ok(names)
}

fn files_under(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let mut entries: Vec<_> = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?.collect::<Result<_, _>>().map_err(|e| e.to_string())?;
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let p = e.path();
        let t = e.file_type().map_err(|e| e.to_string())?;
        if t.is_dir() {
            files_under(&p, out)?;
        } else if t.is_file() {
            out.push(p);
        }
    }
    Ok(())
}

/// The libraries `roots` need, with their transitive references, as paths
/// in the source directories.
fn closure(o: &Options, roots: Vec<String>) -> Result<BTreeSet<PathBuf>, String> {
    let mut found = BTreeSet::new();
    let mut seen = BTreeSet::new();
    let mut work = roots;
    while let Some(name) = work.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        let Some(path) = o.source_libraries.iter().map(|d| d.join(&name)).find(|p| p.is_file()) else {
            if o.verbose > 0 {
                println!("{name} is not in the source libraries; the OS provides it");
            }
            continue;
        };
        work.extend(swift_references(&path)?);
        found.insert(path);
    }
    Ok(found)
}

fn copy_all(libs: &BTreeSet<PathBuf>, destination: &Path, verbose: u32) -> Result<(), String> {
    std::fs::create_dir_all(destination).map_err(|e| format!("{}: {e}", destination.display()))?;
    for lib in libs {
        let to = destination.join(lib.file_name().unwrap());
        let data = std::fs::read(lib).map_err(|e| format!("{}: {e}", lib.display()))?;
        if std::fs::read(&to).is_ok_and(|existing| existing == data) {
            continue;
        }
        if verbose > 0 {
            println!("Copying {} to {}", lib.display(), to.display());
        }
        // Replace rather than write through: the destination may be a
        // read-only copy from an earlier run.
        let _ = std::fs::remove_file(&to);
        std::fs::write(&to, &data).map_err(|e| format!("{}: {e}", to.display()))?;
        let mode = std::fs::metadata(lib).map_err(|e| e.to_string())?.permissions();
        std::fs::set_permissions(&to, mode).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn run(o: &Options) -> Result<(), String> {
    let mut executables = o.scan_executables.clone();
    for f in &o.scan_folders {
        files_under(f, &mut executables)?;
    }
    let mut roots = Vec::new();
    for e in &executables {
        roots.extend(swift_references(e)?);
    }
    let libs = closure(o, roots)?;
    let resources = closure(o, o.resource_libraries.clone())?;
    if o.print {
        for l in libs.iter().chain(&resources) {
            println!("{}", l.display());
        }
        return Ok(());
    }
    for dest in [&o.destination, &o.unsigned_destination].into_iter().flatten() {
        copy_all(&libs, dest, o.verbose)?;
    }
    if let Some(dest) = &o.resource_destination {
        copy_all(&resources, dest, o.verbose)?;
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = parse(args).and_then(|o| run(&o));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
