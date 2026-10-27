//! Clean-room `codesign` for Linux: ad hoc and identity signing, and
//! `--verify`; see docs/clean-room/codesign.md.

mod cms;
mod der;
mod identity;
mod requirement;
mod resources;
mod sign;
mod timestamp;

use cf::Value;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Default)]
struct Options {
    identity: Option<String>,
    identifier: Option<String>,
    entitlements: Option<PathBuf>,
    force: bool,
    /// `--options runtime`.
    runtime: bool,
    /// `--timestamp[=URL]` (Some(Some(URL)), Apple's authority without one)
    /// or `--timestamp=none` (Some(None)); unset, codesign's default, which
    /// depends on the identity.
    timestamp: Option<Option<String>>,
    verify: bool,
    verbose: bool,
    paths: Vec<PathBuf>,
}

fn usage() -> String {
    "usage: codesign -s identity [--force] [--entitlements file] [--generate-entitlement-der] [-i identifier] path ...\n       codesign --verify path ...".into()
}

fn parse(args: Vec<String>) -> Result<Options, String> {
    let mut o = Options::default();
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        let mut value = |name: &str| it.next().ok_or_else(|| format!("{name} needs a value"));
        match a.as_str() {
            "-s" | "--sign" => o.identity = Some(value(&a)?),
            "-i" | "--identifier" => o.identifier = Some(value(&a)?),
            "--entitlements" => o.entitlements = Some(PathBuf::from(value(&a)?)),
            // DER entitlements are the default since macOS 12.
            "--generate-entitlement-der" => {}
            "-f" | "--force" => o.force = true,
            "-v" | "--verbose" => {
                if o.identity.is_none() && !o.force {
                    // `-v` alone means verify; with signing it means verbose.
                    o.verify = true;
                }
                o.verbose = true;
            }
            "--verify" => o.verify = true,
            "--timestamp=none" => o.timestamp = Some(None),
            "--timestamp" => o.timestamp = Some(Some(timestamp::APPLE.to_string())),
            "--deep" | "--strict" => {}
            "--keychain" => {
                // Identities come from the environment on Linux.
                value(&a)?;
            }
            s if s == "-o" || s == "--options" || s.starts_with("--options=") => {
                let list = match s.strip_prefix("--options=") {
                    Some(v) => v.to_string(),
                    None => value(&a)?,
                };
                for flag in list.split(',').map(str::trim).filter(|f| !f.is_empty()) {
                    match flag {
                        "runtime" => o.runtime = true,
                        f => return Err(format!("unsupported --options flag {f}: this codesign supports runtime")),
                    }
                }
            }
            s if s.starts_with("--timestamp=") => o.timestamp = Some(Some(s["--timestamp=".len()..].to_string())),
            s if s.starts_with("--preserve-metadata") => {}
            s if s.starts_with("-v") && s.chars().skip(1).all(|c| c == 'v') => o.verbose = true,
            s if s.starts_with('-') => return Err(format!("unsupported option {s}\n{}", usage())),
            _ => o.paths.push(PathBuf::from(a)),
        }
    }
    if o.identity.is_some() {
        o.verify = false;
    }
    if o.paths.is_empty() {
        return Err(usage());
    }
    Ok(o)
}

struct Bundle {
    /// The directory sealed: the bundle itself, its `Contents`, or a
    /// framework's current version.
    root: PathBuf,
    /// A versioned framework, whose executable is at its version's root.
    framework: bool,
    layout: resources::Layout,
    info_path: PathBuf,
    executable: Option<String>,
    identifier: Option<String>,
}

impl Bundle {
    /// The main executable, relative to `root`.
    fn executable_path(&self) -> Option<String> {
        let exe = self.executable.as_ref()?;
        Some(match self.layout {
            resources::Layout::Deep if !self.framework => format!("MacOS/{exe}"),
            _ => exe.clone(),
        })
    }
}

/// An iOS-style bundle keeps Info.plist at its root; a macOS one in Contents.
fn bundle(path: &Path) -> Option<Bundle> {
    if !path.is_dir() {
        return None;
    }
    // A macOS bundle keeps its Info.plist in Contents, a versioned framework
    // in its current version's Resources, sealed with the macOS rules.
    let (root, layout, framework, info_path) = if path.join("Contents/Info.plist").is_file() {
        (path.join("Contents"), resources::Layout::Deep, false, path.join("Contents/Info.plist"))
    } else if path.join("Versions/Current/Resources/Info.plist").is_file() {
        let root = path.join("Versions/Current");
        (root.clone(), resources::Layout::Deep, true, root.join("Resources/Info.plist"))
    } else {
        (path.to_path_buf(), resources::Layout::Shallow, false, path.join("Info.plist"))
    };
    let info = Value::parse(&std::fs::read(&info_path).ok()?).ok()?;
    Some(Bundle {
        root,
        framework,
        layout,
        info_path,
        executable: info.get("CFBundleExecutable").and_then(|v| v.as_str()).map(String::from),
        identifier: info.get("CFBundleIdentifier").and_then(|v| v.as_str()).map(String::from),
    })
}

fn default_identifier(path: &Path) -> String {
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    match name.rsplit_once('.') {
        Some((stem, _)) if !stem.is_empty() => stem.to_string(),
        _ => name,
    }
}

/// The identifier codesign gives a Mach-O file with none of its own: the
/// file name without its extension, made unique, when signing ad hoc and it
/// has no dot, by appending "-", then "UUID" and the image's LC_UUID, in hex.
/// An identity signature keeps the bare name.
fn file_identifier(path: &Path, image: &macho::Image, ad_hoc: bool) -> Result<String, String> {
    let stem = default_identifier(path);
    if stem.contains('.') || !ad_hoc {
        return Ok(stem);
    }
    let uuid = image.uuid.ok_or_else(|| format!("{}: has no LC_UUID to derive an identifier from; pass -i", path.display()))?;
    let hex: String = b"UUID".iter().chain(&uuid).map(|b| format!("{b:02x}")).collect();
    Ok(format!("{stem}-{hex}"))
}

fn sign_path(path: &Path, o: &Options, signer: Option<&identity::Identity>) -> Result<(), String> {
    // codesign timestamps Developer ID signatures unless told not to, and
    // others only when asked; ad hoc signatures never.
    let timestamp_url = match (signer, &o.timestamp) {
        (None, _) => None,
        (Some(_), Some(choice)) => choice.clone(),
        (Some(id), None) => (id.kind() == Ok(identity::Kind::DeveloperId)).then(|| timestamp::APPLE.to_string()),
    };
    let entitlements = match &o.entitlements {
        Some(p) => Some(std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()))?),
        None => None,
    };
    let entitlements_value = match &entitlements {
        Some(e) => Some(Value::parse(e).map_err(|e| format!("entitlements: {e}"))?),
        None => None,
    };
    let der = match &entitlements_value {
        Some(v) => Some(der::encode(v)?),
        None => None,
    };

    let (exe_path, identifier, info, code_resources) = if let Some(b) = bundle(path) {
        let exe = b.executable_path().ok_or_else(|| format!("{}: bundle has no CFBundleExecutable", path.display()))?;
        let entries = resources::collect(&b.root, Some(&exe), b.layout).map_err(|e| e.to_string())?;
        let seal = cf::xml::to_bytes(&resources::build(&entries, b.layout).map_err(|e| format!("{}: {e}", path.display()))?);
        let dir = b.root.join("_CodeSignature");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        std::fs::write(dir.join("CodeResources"), &seal).map_err(|e| e.to_string())?;
        let info = std::fs::read(&b.info_path).map_err(|e| e.to_string())?;
        let identifier = o.identifier.clone().or(b.identifier).unwrap_or_else(|| default_identifier(path));
        (b.root.join(exe), Some(identifier), Some(info), Some(seal))
    } else {
        (path.to_path_buf(), None, None, None)
    };

    let data = std::fs::read(&exe_path).map_err(|e| format!("{}: {e}", exe_path.display()))?;
    let mut universal = macho::Universal::parse(data).map_err(|e| format!("{}: {e}", exe_path.display()))?;
    // A lone Mach-O file binds the Info.plist embedded in it, if any, and is
    // named by it.
    let images = universal.images();
    let preferred = sign::preferred_slice(&images).ok_or_else(|| format!("{}: universal files without an arm64 slice are not supported by this codesign", exe_path.display()))?;
    let (identifier, info) = match identifier {
        Some(i) => (i, info),
        None => {
            // Named by the slice the signing Mac would run.
            let first = images[preferred];
            let embedded = first.section("__TEXT", "__info_plist").map(<[u8]>::to_vec);
            let embedded_id = embedded.as_ref().and_then(|e| Value::parse(e).ok()).and_then(|v| v.get("CFBundleIdentifier").and_then(|i| i.as_str()).map(String::from));
            let identifier = match o.identifier.clone().or(embedded_id) {
                Some(i) => i,
                None => file_identifier(path, first, signer.is_none())?,
            };
            (identifier, embedded)
        }
    };
    let team = signer.and_then(|s| s.team());
    for image in universal.images_mut() {
        if image.code_signature.is_some() && !o.force {
            let cd = sign::existing_code_directory(image);
            // A linker's ad-hoc signature may always be replaced.
            let linker_signed = cd.map_or(true, |cd| cd.len() > 16 && u32::from_be_bytes(cd[12..16].try_into().unwrap()) & 0x20000 != 0);
            if !linker_signed {
                return Err(format!("{}: is already signed", path.display()));
            }
        }
        let inputs = sign::Inputs {
            identifier: &identifier,
            info_plist: info.as_deref(),
            code_resources: code_resources.as_deref(),
            entitlements: entitlements.as_deref(),
            der_entitlements: der.as_deref(),
            entitlement_flags: sign::exec_seg_flags(entitlements_value.as_ref()),
            runtime: o.runtime,
            identity: signer,
            signing_time: cms::signing_time(),
            team: team.as_deref(),
            timestamp_url: timestamp_url.as_deref(),
        };
        sign::sign(image, &inputs).map_err(|e| format!("{}: {e}", exe_path.display()))?;
    }
    // codesign lays a universal file out again with its slices aligned to
    // at least 16 KB.
    if let macho::Universal::Fat(slices) = &mut universal {
        for (align, _) in slices.iter_mut() {
            *align = (*align).max(14);
        }
    }
    let out = universal.into_bytes();
    replace(&exe_path, &out).map_err(|e| format!("{}: {e}", exe_path.display()))?;
    if o.verbose {
        eprintln!("{}: signed", path.display());
    }
    Ok(())
}

/// Writes `data` over `path` as a new file renamed into place, keeping its
/// permissions: Bazel's outputs are read-only, and codesign signs them.
fn replace(path: &Path, data: &[u8]) -> std::io::Result<()> {
    let permissions = std::fs::metadata(path)?.permissions();
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let temp = path.with_file_name(format!(".{name}.codesign"));
    std::fs::write(&temp, data)?;
    std::fs::set_permissions(&temp, permissions)?;
    std::fs::rename(&temp, path)
}

fn verify_path(path: &Path) -> Result<(), String> {
    let exe = match bundle(path) {
        Some(b) => b.root.join(b.executable_path().ok_or("bundle has no executable")?),
        None => path.to_path_buf(),
    };
    let data = std::fs::read(&exe).map_err(|e| format!("{}: {e}", exe.display()))?;
    let mut universal = macho::Universal::parse(data).map_err(|e| e.to_string())?;
    for image in universal.images_mut() {
        if sign::existing_code_directory(image).is_none() {
            return Err(format!("{}: code object is not signed at all", path.display()));
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let o = match parse(std::env::args().skip(1).collect()) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };
    // An identity other than "-" must be the one the environment provides.
    let signer = match o.identity.as_deref() {
        Some(name) if name != "-" && !o.verify => match identity::Identity::from_environment() {
            Ok(Some(id)) if id.matches(name) && id.kind().is_err() => {
                eprintln!("{name}: {}", id.kind().unwrap_err());
                return ExitCode::FAILURE;
            }
            Ok(Some(id)) if id.matches(name) => Some(id),
            Ok(Some(_)) => {
                eprintln!("{name}: no identity found");
                return ExitCode::FAILURE;
            }
            Ok(None) => {
                eprintln!(
                    "{name}: no identity found: this codesign reads its identity from {} (the private key, PEM) and {} (its certificates, PEM)",
                    identity::KEY_ENV,
                    identity::CERTIFICATES_ENV
                );
                return ExitCode::FAILURE;
            }
            Err(e) => {
                eprintln!("{name}: {e}");
                return ExitCode::FAILURE;
            }
        },
        _ => None,
    };
    let mut status = ExitCode::SUCCESS;
    for p in &o.paths {
        let r = if o.verify { verify_path(p) } else if o.identity.is_some() { sign_path(p, &o, signer.as_ref()) } else { Err(usage()) };
        if let Err(e) = r {
            eprintln!("{e}");
            status = ExitCode::FAILURE;
        }
    }
    status
}
