//! `_CodeSignature/CodeResources`, with the iOS and macOS default rule sets.

use cf::Value;
use regex::Regex;
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(Clone)]
struct Rule {
    pattern: &'static str,
    omit: bool,
    optional: bool,
    nested: bool,
    weight: f64,
    /// Whether the rule is written as `true` rather than a dictionary.
    plain: bool,
}

const fn plain(pattern: &'static str) -> Rule {
    Rule { pattern, omit: false, optional: false, nested: false, weight: 1.0, plain: true }
}

const fn rule(pattern: &'static str, omit: bool, optional: bool, weight: f64) -> Rule {
    Rule { pattern, omit, optional, nested: false, weight, plain: false }
}

const fn nested(pattern: &'static str) -> Rule {
    Rule { pattern, omit: false, optional: false, nested: true, weight: 10.0, plain: false }
}

/// Where a bundle keeps its resources: at its root (iOS and the other
/// embedded platforms) or in `Contents/Resources` (macOS).
#[derive(Clone, Copy, PartialEq)]
pub enum Layout {
    Shallow,
    Deep,
}

fn rules_v1(layout: Layout) -> Vec<Rule> {
    if layout == Layout::Deep {
        return vec![
            plain("^Resources/"),
            rule("^Resources/.*\\.lproj/", false, true, 1000.0),
            rule("^Resources/.*\\.lproj/locversion.plist$", true, false, 1100.0),
            rule("^Resources/Base\\.lproj/", false, false, 1010.0),
            plain("^version.plist$"),
        ];
    }
    vec![
        plain("^.*"),
        rule("^.*\\.lproj/", false, true, 1000.0),
        rule("^.*\\.lproj/locversion.plist$", true, false, 1100.0),
        rule("^Base\\.lproj/", false, false, 1010.0),
        plain("^version.plist$"),
    ]
}

fn rules_v2(layout: Layout) -> Vec<Rule> {
    if layout == Layout::Deep {
        return vec![
            rule(".*\\.dSYM($|/)", false, false, 11.0),
            rule("^(.*/)?\\.DS_Store$", true, false, 2000.0),
            nested("^(Frameworks|SharedFrameworks|PlugIns|Plug-ins|XPCServices|Helpers|MacOS|Library/(Automator|Spotlight|LoginItems))/"),
            plain("^.*"),
            rule("^Info\\.plist$", true, false, 20.0),
            rule("^PkgInfo$", true, false, 20.0),
            rule("^Resources/", false, false, 20.0),
            rule("^Resources/.*\\.lproj/", false, true, 1000.0),
            rule("^Resources/.*\\.lproj/locversion.plist$", true, false, 1100.0),
            rule("^Resources/Base\\.lproj/", false, false, 1010.0),
            nested("^[^/]+$"),
            rule("^embedded\\.provisionprofile$", false, false, 20.0),
            rule("^version\\.plist$", false, false, 20.0),
        ];
    }
    vec![
        rule(".*\\.dSYM($|/)", false, false, 11.0),
        rule("^(.*/)?\\.DS_Store$", true, false, 2000.0),
        plain("^.*"),
        rule("^.*\\.lproj/", false, true, 1000.0),
        rule("^.*\\.lproj/locversion.plist$", true, false, 1100.0),
        rule("^Base\\.lproj/", false, false, 1010.0),
        rule("^Info\\.plist$", true, false, 20.0),
        rule("^PkgInfo$", true, false, 20.0),
        rule("^embedded\\.provisionprofile$", false, false, 20.0),
        rule("^version\\.plist$", false, false, 20.0),
    ]
}

fn rules_value(rules: &[Rule]) -> Value {
    Value::dict(rules.iter().map(|r| {
        let v = if r.plain {
            Value::Boolean(true)
        } else {
            let mut d = Vec::new();
            if r.nested {
                d.push(("nested".to_string(), Value::Boolean(true)));
            }
            if r.omit {
                d.push(("omit".to_string(), Value::Boolean(true)));
            }
            if r.optional {
                d.push(("optional".to_string(), Value::Boolean(true)));
            }
            d.push(("weight".to_string(), Value::Real(r.weight)));
            Value::Dictionary(d)
        };
        (r.pattern, v)
    }))
}

struct Compiled {
    rule: Rule,
    regex: Regex,
}

fn compile(rules: Vec<Rule>) -> Vec<Compiled> {
    rules.into_iter().map(|rule| Compiled { regex: Regex::new(rule.pattern).unwrap(), rule }).collect()
}

/// The rule of greatest weight matching `path`; the earlier one wins a tie.
fn decide<'a>(rules: &'a [Compiled], path: &str) -> Option<&'a Rule> {
    let mut best: Option<&Rule> = None;
    for c in rules {
        if c.regex.is_match(path) && best.map_or(true, |b| c.rule.weight > b.weight) {
            best = Some(&c.rule);
        }
    }
    best
}

pub enum Entry {
    File(PathBuf),
    Symlink(String),
    /// A bundle where the rules expect nested code (a directory whose name
    /// has a dot), sealed as one entry.
    Bundle(PathBuf),
}

/// Collects the bundle's files, relative to `root`, in path order, leaving out
/// the main executable (a path relative to `root`).
pub fn collect(root: &Path, executable: Option<&str>, layout: Layout) -> std::io::Result<Vec<(String, Entry)>> {
    let mut out = Vec::new();
    walk(root, root, executable, &compile(rules_v2(layout)), &mut out)?;
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

fn walk(root: &Path, dir: &Path, executable: Option<&str>, v2: &[Compiled], out: &mut Vec<(String, Entry)>) -> std::io::Result<()> {
    let mut names: Vec<_> = std::fs::read_dir(dir)?.collect::<Result<Vec<_>, _>>()?;
    names.sort_by_key(|e| e.file_name());
    for entry in names {
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
        if (dir == root && rel == "_CodeSignature") || Some(rel.as_str()) == executable {
            continue;
        }
        let meta = std::fs::symlink_metadata(&path)?;
        if meta.file_type().is_symlink() {
            out.push((rel, Entry::Symlink(std::fs::read_link(&path)?.to_string_lossy().into_owned())));
        } else if meta.is_dir() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.contains('.') && decide(v2, &rel).is_some_and(|r| r.nested) {
                out.push((rel, Entry::Bundle(path)));
            } else {
                walk(root, &path, executable, v2, out)?;
            }
        } else {
            out.push((rel, Entry::File(path)));
        }
    }
    Ok(())
}

pub fn build(entries: &[(String, Entry)], layout: Layout) -> Result<Value, String> {
    let v1 = compile(rules_v1(layout));
    let v2 = compile(rules_v2(layout));
    let mut files = Vec::new();
    let mut files2 = Vec::new();
    for (rel, entry) in entries {
        let contents = match entry {
            Entry::File(path) => Some(std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?),
            _ => None,
        };
        if let Some(r) = decide(&v1, rel) {
            if !r.omit {
                if let (Some(data), Entry::File(_)) = (&contents, entry) {
                    let hash = Value::Data(Sha1::digest(data).to_vec());
                    files.push((rel.clone(), if r.optional { Value::dict([("hash", hash), ("optional", Value::Boolean(true))]) } else { hash }));
                }
            }
        }
        if let Some(r) = decide(&v2, rel) {
            if !r.omit {
                let mut d: Vec<(String, Value)> = match entry {
                    // Symlinks are never nested code.
                    Entry::Symlink(target) => vec![("symlink".into(), Value::string(target.clone()))],
                    Entry::File(path) | Entry::Bundle(path) if r.nested => nested_seal(path).map_err(|e| format!("{e}\nIn subcomponent: {rel}"))?,
                    Entry::File(_) => vec![("hash2".into(), Value::Data(Sha256::digest(contents.as_ref().unwrap()).to_vec()))],
                    Entry::Bundle(_) => unreachable!("only nested rules make bundle entries"),
                };
                if r.optional {
                    d.push(("optional".into(), Value::Boolean(true)));
                }
                files2.push((rel.clone(), Value::Dictionary(d)));
            }
        }
    }
    Ok(Value::dict([
        ("files", Value::Dictionary(files)),
        ("files2", Value::Dictionary(files2)),
        ("rules", rules_value(&rules_v1(layout))),
        ("rules2", rules_value(&rules_v2(layout))),
    ]))
}

/// The seal of nested code, which must already be signed: the cdhash of its
/// preferred architecture and its designated requirement: the one it was
/// signed with, or for ad hoc code, which has none, the cdhash of each
/// architecture.
fn nested_seal(path: &Path) -> Result<Vec<(String, Value)>, String> {
    let executable = if path.is_dir() { bundle_executable(path)? } else { path.to_path_buf() };
    let data = std::fs::read(&executable).map_err(|e| format!("{}: {e}", executable.display()))?;
    let universal = macho::Universal::parse(data).map_err(|_| "code object is not signed at all".to_string())?;
    let images = universal.images();
    let mut hashes = Vec::new();
    for image in &images {
        let cds = crate::sign::code_directories(image);
        let [(0, cd)] = &cds[..] else {
            return Err(if cds.is_empty() {
                "code object is not signed at all".into()
            } else {
                "nested code with more than one code directory is not supported by this codesign".into()
            });
        };
        hashes.push(crate::sign::cdhash(cd).ok_or("nested code's hash type is not supported by this codesign")?);
    }
    // The preferred architecture's first, then the others in file order.
    let first = crate::sign::preferred_slice(&images).ok_or("nested universal code without an arm64 slice is not supported by this codesign")?;
    let order: Vec<usize> = std::iter::once(first).chain((0..images.len()).filter(|&i| i != first)).collect();
    let hex = |h: &[u8]| h.iter().map(|b| format!("{b:02x}")).collect::<String>();
    // Code signed by an identity carries a designated requirement of its own;
    // ad hoc code is required by its cdhashes.
    let requirement = match crate::sign::designated_requirement_text(images[first])? {
        Some(text) => text,
        None => order.iter().map(|&i| format!("cdhash H\"{}\"", hex(&hashes[i]))).collect::<Vec<_>>().join(" or "),
    };
    Ok(vec![("cdhash".into(), Value::Data(hashes[first].clone())), ("requirement".into(), Value::string(requirement))])
}

/// A nested bundle's main executable: a macOS bundle's in Contents/MacOS,
/// a framework's beside Resources/Info.plist in its current version, or
/// beside Info.plist at its root.
fn bundle_executable(dir: &Path) -> Result<PathBuf, String> {
    let candidates = [
        (dir.join("Contents/Info.plist"), dir.join("Contents/MacOS")),
        (dir.join("Versions/Current/Resources/Info.plist"), dir.join("Versions/Current")),
        (dir.join("Resources/Info.plist"), dir.to_path_buf()),
        (dir.join("Info.plist"), dir.to_path_buf()),
    ];
    for (info, base) in candidates {
        if let Ok(bytes) = std::fs::read(&info) {
            let plist = Value::parse(&bytes).map_err(|e| format!("{}: {e}", info.display()))?;
            if let Some(exe) = plist.get("CFBundleExecutable").and_then(|v| v.as_str()) {
                return Ok(base.join(exe));
            }
            break;
        }
    }
    Err("code object is not signed at all".into())
}
