use anyhow::{bail, Context, Result};
use plist::Value;
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use xcbuild_plist::*;

fn help(error: Option<&str>) -> ! {
    if let Some(e) = error {
        eprintln!("error: {e}\n");
    }

    eprintln!("usage: plutil -<command> [options] <files>\n");

    eprintln!("commands:");
    eprintln!("  -lint (default)");
    eprintln!("  -help (this message)");
    eprintln!("  -p");
    eprintln!("  -convert <format>");
    eprintln!("  -create <format>");
    eprintln!("  -insert <key> <value> [-append]");
    eprintln!("  -replace <key> <value>");
    eprintln!("  -remove <key>");
    eprintln!("  -extract <key> <format> [-expect <type>]");
    eprintln!("  -type <key> [-expect <type>]");

    eprintln!("\nvalues:");
    eprintln!("  -bool <YES|NO>");
    eprintln!("  -integer <number>");
    eprintln!("  -float <number>");
    eprintln!("  -string <utf8>");
    eprintln!("  -data <base64>");
    eprintln!("  -date <iso8601>");
    eprintln!("  -xml <plist>");
    eprintln!("  -json <json>");
    eprintln!("  -array");
    eprintln!("  -dictionary");

    eprintln!("\nformats:");
    eprintln!("  xml1");
    eprintln!("  binary1");
    eprintln!("  openstep1");
    eprintln!("  json");
    eprintln!("  raw");
    eprintln!("  swift");
    eprintln!("  objc");

    eprintln!("\nflags:");
    eprintln!("  -r  human readable (sorted JSON)");
    eprintln!("  -n  no trailing newline (raw format)");

    std::process::exit(if error.is_some() { 1 } else { 0 });
}

#[derive(Debug, Clone, Copy)]
enum Command {
    Lint,
    Print,
    Help,
    Create,
}

#[derive(Debug, Clone, Copy)]
enum AdjustmentType {
    Insert,
    Replace,
    Remove,
    Extract,
    Type,
}

#[derive(Debug)]
struct Adjustment {
    adj_type: AdjustmentType,
    path: String,
    value: Option<Value>,
    expect_type: Option<String>,
    append: bool,
}

fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Boolean(_) => "bool",
        Value::Integer(_) => "integer",
        Value::Real(_) => "float",
        Value::String(_) => "string",
        Value::Date(_) => "date",
        Value::Data(_) => "data",
        Value::Array(_) => "array",
        Value::Dictionary(_) => "dictionary",
        Value::Uid(_) => "uid",
        _ => "unknown",
    }
}

fn parse_value_arg(type_arg: &str, value_str: Option<&str>) -> Result<Value> {
    match type_arg {
        "-bool" => {
            let s = value_str.ok_or_else(|| anyhow::anyhow!("missing value for -bool"))?;
            let b = xcbuild_plist::parse_bool(s)
                .ok_or_else(|| anyhow::anyhow!("invalid boolean value '{s}'"))?;
            Ok(Value::Boolean(b))
        }
        "-integer" => {
            let s = value_str.ok_or_else(|| anyhow::anyhow!("missing value for -integer"))?;
            let i: i64 = s.parse().context("invalid integer argument")?;
            Ok(Value::Integer(i.into()))
        }
        "-float" => {
            let s = value_str.ok_or_else(|| anyhow::anyhow!("missing value for -float"))?;
            let f: f64 = s.parse().context("invalid float argument")?;
            Ok(Value::Real(f))
        }
        "-string" => {
            let s = value_str.ok_or_else(|| anyhow::anyhow!("missing value for -string"))?;
            Ok(Value::String(s.to_string()))
        }
        "-date" => {
            let s = value_str.ok_or_else(|| anyhow::anyhow!("missing value for -date"))?;
            let date = plist::Date::from_xml_format(s)
                .map_err(|_| anyhow::anyhow!("invalid date value '{s}'"))?;
            Ok(Value::Date(date))
        }
        "-data" => {
            let s = value_str.ok_or_else(|| anyhow::anyhow!("missing value for -data"))?;
            let decoded = xcbuild_plist::base64_decode(s)
                .map_err(|e| anyhow::anyhow!("invalid base64 data: {e}"))?;
            Ok(Value::Data(decoded))
        }
        "-xml" => {
            let s = value_str.ok_or_else(|| anyhow::anyhow!("missing value for -xml"))?;
            let data = s.as_bytes();
            let value = xcbuild_plist::deserialize_with_format(data, PlistFormat::Xml)
                .context("invalid XML value")?;
            Ok(value)
        }
        "-json" => {
            let s = value_str.ok_or_else(|| anyhow::anyhow!("missing value for -json"))?;
            let data = s.as_bytes();
            let value = xcbuild_plist::deserialize_with_format(data, PlistFormat::Json)
                .context("invalid JSON value")?;
            Ok(value)
        }
        "-array" => Ok(Value::Array(vec![])),
        "-dictionary" => Ok(Value::Dictionary(plist::Dictionary::new())),
        _ => bail!("unknown type option {type_arg}"),
    }
}

/// Returns true if the type_arg is a no-value type (-array, -dictionary)
fn is_no_value_type(type_arg: &str) -> bool {
    matches!(type_arg, "-array" | "-dictionary")
}

/// Key path segment: either a dictionary key or an array index.
#[derive(Debug, Clone)]
enum PathPart {
    Key(String),
    Index(usize),
}

/// Parse plutil's dot-separated key path with backslash escape support.
/// Escaped dots (\.) are treated as literal dots within a key.
fn parse_dot_key_path(path: &str) -> Result<Vec<PathPart>> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut escaped = false;

    for ch in path.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '.' {
            if current.is_empty() {
                bail!("invalid keypath '{path}'");
            }
            parts.push(classify_path_part(&current));
            current.clear();
            continue;
        }
        current.push(ch);
    }

    if escaped {
        bail!("invalid escape in keypath '{path}'");
    }

    if !current.is_empty() {
        parts.push(classify_path_part(&current));
    }

    Ok(parts)
}

fn classify_path_part(s: &str) -> PathPart {
    if let Ok(idx) = s.parse::<usize>() {
        PathPart::Index(idx)
    } else {
        PathPart::Key(s.to_string())
    }
}

fn read_input(path: &str) -> Result<Vec<u8>> {
    if path == "-" {
        let mut buf = Vec::new();
        io::stdin().read_to_end(&mut buf)?;
        Ok(buf)
    } else {
        fs::read(path).with_context(|| format!("unable to read {path}"))
    }
}

fn write_output(path: &str, data: &[u8]) -> Result<()> {
    if path == "-" {
        io::stdout().write_all(data)?;
        Ok(())
    } else {
        fs::write(path, data).with_context(|| format!("unable to write {path}"))
    }
}

fn output_path(output: &Option<String>, extension: &Option<String>, file: &str) -> String {
    if let Some(o) = output {
        return o.clone();
    }
    if file != "-" {
        if let Some(ext) = extension {
            let p = Path::new(file);
            let stem = p.file_stem().unwrap_or_default().to_string_lossy();
            let parent = p.parent().unwrap_or_else(|| Path::new(""));
            return parent.join(format!("{stem}.{ext}")).to_string_lossy().to_string();
        }
    }
    file.to_string()
}

fn navigate_to_value<'a>(root: &'a Value, parts: &[PathPart]) -> Result<&'a Value> {
    let mut current = root;
    for part in parts {
        current = match (part, current) {
            (PathPart::Key(key), Value::Dictionary(dict)) => dict
                .get(key.as_str())
                .ok_or_else(|| anyhow::anyhow!("invalid key path"))?,
            (PathPart::Index(idx), Value::Array(arr)) => arr
                .get(*idx)
                .ok_or_else(|| anyhow::anyhow!("invalid key path"))?,
            _ => bail!("invalid key path"),
        };
    }
    Ok(current)
}

fn navigate_to_value_mut<'a>(root: &'a mut Value, parts: &[PathPart]) -> Result<&'a mut Value> {
    let mut current = root;
    for part in parts {
        current = match part {
            PathPart::Key(key) => match current {
                Value::Dictionary(dict) => dict
                    .get_mut(key.as_str())
                    .ok_or_else(|| anyhow::anyhow!("invalid key path"))?,
                _ => bail!("invalid key path"),
            },
            PathPart::Index(idx) => match current {
                Value::Array(arr) => arr
                    .get_mut(*idx)
                    .ok_or_else(|| anyhow::anyhow!("invalid key path"))?,
                _ => bail!("invalid key path"),
            },
        };
    }
    Ok(current)
}

fn perform_adjustment(
    root: &mut Value,
    adjustment: &Adjustment,
) -> Result<Option<Value>> {
    let parts = parse_dot_key_path(&adjustment.path)?;
    if parts.is_empty() {
        // Empty path = root object
        match adjustment.adj_type {
            AdjustmentType::Type => {
                let type_name = value_type_name(root);
                if let Some(ref expect) = adjustment.expect_type {
                    if type_name != expect.as_str() {
                        bail!(
                            "expected {expect} but found {type_name} at key path {}",
                            adjustment.path
                        );
                    }
                }
                return Ok(Some(Value::String(type_name.to_string())));
            }
            AdjustmentType::Extract => {
                if let Some(ref expect) = adjustment.expect_type {
                    let type_name = value_type_name(root);
                    if type_name != expect.as_str() {
                        bail!(
                            "expected {expect} but found {type_name} at key path {}",
                            adjustment.path
                        );
                    }
                }
                return Ok(Some(root.clone()));
            }
            AdjustmentType::Replace => {
                let value = adjustment.value.as_ref().expect("replace requires value");
                *root = value.clone();
                return Ok(None);
            }
            AdjustmentType::Remove => {
                bail!("cannot remove root object");
            }
            _ => bail!("invalid key path"),
        }
    }

    // For Type and Extract, handle expect_type validation
    match adjustment.adj_type {
        AdjustmentType::Type => {
            let target = navigate_to_value(root, &parts)?;
            let type_name = value_type_name(target);
            if let Some(ref expect) = adjustment.expect_type {
                if type_name != expect.as_str() {
                    bail!(
                        "expected {expect} but found {type_name} at key path {}",
                        adjustment.path
                    );
                }
            }
            return Ok(Some(Value::String(type_name.to_string())));
        }
        _ => {}
    }

    let parent_parts = &parts[..parts.len() - 1];
    let last_part = &parts[parts.len() - 1];

    // Navigate to parent
    let parent = if parent_parts.is_empty() {
        root
    } else {
        navigate_to_value_mut(root, parent_parts)?
    };

    match adjustment.adj_type {
        AdjustmentType::Insert => {
            let value = adjustment.value.as_ref().expect("insert requires value");

            if adjustment.append {
                // Append mode: navigate to the keypath itself, it must be an array
                let target = match (last_part, &mut *parent) {
                    (PathPart::Key(key), Value::Dictionary(dict)) => dict
                        .get_mut(key.as_str())
                        .ok_or_else(|| anyhow::anyhow!("invalid key path"))?,
                    (PathPart::Index(idx), Value::Array(arr)) => arr
                        .get_mut(*idx)
                        .ok_or_else(|| anyhow::anyhow!("invalid key path"))?,
                    _ => bail!("invalid key path"),
                };
                match target {
                    Value::Array(arr) => {
                        arr.push(value.clone());
                    }
                    _ => bail!("target of -append must be an array"),
                }
            } else {
                match (last_part, parent) {
                    (PathPart::Key(key), Value::Dictionary(dict)) => {
                        if dict.contains_key(key.as_str()) {
                            bail!("key '{}' already exists", key);
                        }
                        dict.insert(key.clone(), value.clone());
                    }
                    (PathPart::Index(idx), Value::Array(arr)) => {
                        if *idx < arr.len() {
                            arr.insert(*idx, value.clone());
                        } else {
                            arr.push(value.clone());
                        }
                    }
                    _ => bail!("invalid key path"),
                }
            }
            Ok(None)
        }
        AdjustmentType::Replace => {
            let value = adjustment.value.as_ref().expect("replace requires value");
            match (last_part, parent) {
                (PathPart::Key(key), Value::Dictionary(dict)) => {
                    if !dict.contains_key(key.as_str()) {
                        bail!("key '{}' does not exist", key);
                    }
                    dict.insert(key.clone(), value.clone());
                }
                (PathPart::Index(idx), Value::Array(arr)) => {
                    if *idx >= arr.len() {
                        bail!("array index {} out of bounds", idx);
                    }
                    arr[*idx] = value.clone();
                }
                _ => bail!("invalid key path"),
            }
            Ok(None)
        }
        AdjustmentType::Remove => match (last_part, parent) {
            (PathPart::Key(key), Value::Dictionary(dict)) => {
                if dict.remove(key.as_str()).is_none() {
                    bail!("key '{}' does not exist", key);
                }
                Ok(None)
            }
            (PathPart::Index(idx), Value::Array(arr)) => {
                if *idx >= arr.len() {
                    bail!("array index {} out of bounds", idx);
                }
                arr.remove(*idx);
                Ok(None)
            }
            _ => bail!("invalid key path"),
        },
        AdjustmentType::Extract => {
            let extracted = match (last_part, &*parent) {
                (PathPart::Key(key), Value::Dictionary(dict)) => dict.get(key.as_str()).cloned(),
                (PathPart::Index(idx), Value::Array(arr)) => arr.get(*idx).cloned(),
                _ => None,
            };
            if let Some(ref value) = extracted {
                if let Some(ref expect) = adjustment.expect_type {
                    let type_name = value_type_name(value);
                    if type_name != expect.as_str() {
                        bail!(
                            "expected {expect} but found {type_name} at key path {}",
                            adjustment.path
                        );
                    }
                }
            }
            Ok(extracted)
        }
        AdjustmentType::Type => {
            unreachable!("Type handled above");
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut command: Option<Command> = None;
    let mut adjustments: Vec<Adjustment> = Vec::new();
    let mut convert_format: Option<PlistFormat> = None;
    let mut create_format: Option<PlistFormat> = None;
    let mut inputs: Vec<String> = Vec::new();
    let mut output: Option<String> = None;
    let mut extension: Option<String> = None;
    let mut silent = false;
    let mut separator = false;
    let mut human_readable = false;
    let mut no_newline = false;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];

        if separator {
            inputs.push(arg.clone());
            i += 1;
            continue;
        }

        match arg.as_str() {
            "-help" | "--help" | "-h" => {
                command = Some(Command::Help);
            }
            "-lint" => {
                command = Some(Command::Lint);
            }
            "-p" => {
                command = Some(Command::Print);
            }
            "-create" => {
                command = Some(Command::Create);
                i += 1;
                if i >= args.len() {
                    help(Some("missing value for -create"));
                }
                let fmt = PlistFormat::parse(&args[i]);
                if fmt.is_none() {
                    help(Some(&format!("unknown format {}", args[i])));
                }
                create_format = fmt;
            }
            "-convert" => {
                i += 1;
                if i >= args.len() {
                    help(Some("missing value for -convert"));
                }
                let fmt = PlistFormat::parse(&args[i]);
                if fmt.is_none() {
                    help(Some(&format!("unknown format {}", args[i])));
                }
                convert_format = fmt;
            }
            "-insert" => {
                i += 1;
                if i >= args.len() {
                    help(Some("missing arguments for -insert"));
                }
                let path = args[i].clone();
                i += 1;
                if i >= args.len() {
                    help(Some("missing arguments for -insert"));
                }
                let type_arg = &args[i];

                let value = if is_no_value_type(type_arg) {
                    match parse_value_arg(type_arg, None) {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("error: {e}");
                            std::process::exit(1);
                        }
                    }
                } else {
                    i += 1;
                    if i >= args.len() {
                        help(Some("missing arguments for -insert"));
                    }
                    let value_str = &args[i];
                    match parse_value_arg(type_arg, Some(value_str)) {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("error: {e}");
                            std::process::exit(1);
                        }
                    }
                };

                // Check for -append flag
                let mut append = false;
                if i + 1 < args.len() && args[i + 1] == "-append" {
                    append = true;
                    i += 1;
                }

                adjustments.push(Adjustment {
                    adj_type: AdjustmentType::Insert,
                    path,
                    value: Some(value),
                    expect_type: None,
                    append,
                });
            }
            "-replace" => {
                i += 1;
                if i >= args.len() {
                    help(Some("missing arguments for -replace"));
                }
                let path = args[i].clone();
                i += 1;
                if i >= args.len() {
                    help(Some("missing arguments for -replace"));
                }
                let type_arg = &args[i];

                let value = if is_no_value_type(type_arg) {
                    match parse_value_arg(type_arg, None) {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("error: {e}");
                            std::process::exit(1);
                        }
                    }
                } else {
                    i += 1;
                    if i >= args.len() {
                        help(Some("missing arguments for -replace"));
                    }
                    let value_str = &args[i];
                    match parse_value_arg(type_arg, Some(value_str)) {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("error: {e}");
                            std::process::exit(1);
                        }
                    }
                };

                adjustments.push(Adjustment {
                    adj_type: AdjustmentType::Replace,
                    path,
                    value: Some(value),
                    expect_type: None,
                    append: false,
                });
            }
            "-remove" => {
                i += 1;
                if i >= args.len() {
                    help(Some("missing argument for -remove"));
                }
                adjustments.push(Adjustment {
                    adj_type: AdjustmentType::Remove,
                    path: args[i].clone(),
                    value: None,
                    expect_type: None,
                    append: false,
                });
            }
            "-extract" => {
                i += 1;
                if i + 1 >= args.len() {
                    help(Some("missing arguments for -extract"));
                }
                let path = args[i].clone();
                i += 1;
                let fmt = PlistFormat::parse(&args[i]);
                if fmt.is_none() {
                    help(Some(&format!("unknown format {}", args[i])));
                }
                convert_format = fmt;

                // Check for -expect
                let mut expect_type = None;
                if i + 2 < args.len() && args[i + 1] == "-expect" {
                    i += 1; // skip -expect
                    i += 1; // consume type
                    expect_type = Some(args[i].clone());
                }

                adjustments.push(Adjustment {
                    adj_type: AdjustmentType::Extract,
                    path,
                    value: None,
                    expect_type,
                    append: false,
                });
            }
            "-type" => {
                i += 1;
                if i >= args.len() {
                    help(Some("missing argument for -type"));
                }
                let path = args[i].clone();

                // Check for -expect
                let mut expect_type = None;
                if i + 2 < args.len() && args[i + 1] == "-expect" {
                    i += 1; // skip -expect
                    i += 1; // consume type
                    expect_type = Some(args[i].clone());
                }

                adjustments.push(Adjustment {
                    adj_type: AdjustmentType::Type,
                    path,
                    value: None,
                    expect_type,
                    append: false,
                });
            }
            "-o" => {
                i += 1;
                if i >= args.len() {
                    help(Some("missing value for -o"));
                }
                output = Some(args[i].clone());
            }
            "-e" => {
                i += 1;
                if i >= args.len() {
                    help(Some("missing value for -e"));
                }
                extension = Some(args[i].clone());
            }
            "-s" => {
                silent = true;
            }
            "-r" => {
                human_readable = true;
            }
            "-n" => {
                no_newline = true;
            }
            "--" => {
                separator = true;
            }
            _ => {
                if arg.starts_with('-') {
                    help(Some(&format!("unknown argument {arg}")));
                }
                inputs.push(arg.clone());
            }
        }

        i += 1;
    }

    let has_type_adj = adjustments.iter().any(|a| matches!(a.adj_type, AdjustmentType::Type));
    let has_extract_raw = adjustments.iter().any(|a| {
        matches!(a.adj_type, AdjustmentType::Extract) && convert_format == Some(PlistFormat::Raw)
    });
    let modify = convert_format.is_some() || !adjustments.is_empty();

    // Handle help
    if matches!(command, Some(Command::Help)) {
        help(None);
    }

    // Validate -n flag: only valid with -extract <keypath> raw
    if no_newline && !has_extract_raw {
        help(Some("-n is only supported with -extract <keypath> raw"));
    }

    // Handle -create: create empty plist
    if matches!(command, Some(Command::Create)) {
        let fmt = create_format.unwrap_or(PlistFormat::Xml);
        let root = Value::Dictionary(plist::Dictionary::new());

        if inputs.is_empty() {
            help(Some("no output files for -create"));
        }

        for file in &inputs {
            match xcbuild_plist::serialize(&root, fmt) {
                Ok(bytes) => {
                    let out_path = output_path(&output, &extension, file);
                    if let Err(e) = write_output(&out_path, &bytes) {
                        eprintln!("error: {e}");
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            }
        }
        std::process::exit(0);
    }

    // Check conflicts
    if modify && matches!(command, Some(Command::Lint | Command::Print)) {
        help(Some("conflicting options specified"));
    }

    if inputs.is_empty() {
        help(Some("no input files"));
    }

    let mut success = true;

    for file in &inputs {
        let data = match read_input(file) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("error: {e}");
                success = false;
                continue;
            }
        };

        let (root, format) = match xcbuild_plist::deserialize(&data) {
            Ok((v, f)) => (v, f),
            Err(e) => {
                eprintln!("error: {file}: {e}");
                success = false;
                continue;
            }
        };

        if modify {
            let mut root = root;
            let mut write_value: Option<Value> = None;
            let mut type_output: Option<String> = None;

            for adj in &adjustments {
                match perform_adjustment(&mut root, adj) {
                    Ok(Some(extracted)) => {
                        if matches!(adj.adj_type, AdjustmentType::Type) {
                            // For -type, the extracted value is a string with the type name
                            if let Value::String(ref s) = extracted {
                                type_output = Some(s.clone());
                            }
                        } else {
                            write_value = Some(extracted);
                        }
                    }
                    Ok(None) => {}
                    Err(e) => {
                        eprintln!("error: {e}");
                        success = false;
                    }
                }
            }

            // Output type result
            if let Some(ref type_name) = type_output {
                println!("{type_name}");
            }

            // Only write file output if there's something to write
            // -type alone doesn't produce file output
            if !has_type_adj || write_value.is_some() || convert_format.is_some() {
                if has_type_adj && write_value.is_none() && convert_format.is_none() {
                    // -type only, no file output needed
                } else {
                    let out_value = write_value.as_ref().unwrap_or(&root);
                    let out_format = convert_format.unwrap_or(format);

                    let serialized = if human_readable && out_format == PlistFormat::Json {
                        xcbuild_plist::serialize_json_sorted(out_value)
                    } else {
                        xcbuild_plist::serialize(out_value, out_format)
                    };

                    match serialized {
                        Ok(mut bytes) => {
                            if no_newline && out_format == PlistFormat::Raw {
                                // Remove trailing newline if -n flag is set
                                if bytes.last() == Some(&b'\n') {
                                    bytes.pop();
                                }
                            }
                            let out_path = output_path(&output, &extension, file);
                            if let Err(e) = write_output(&out_path, &bytes) {
                                eprintln!("error: {e}");
                                success = false;
                            }
                        }
                        Err(e) => {
                            eprintln!("error: {e}");
                            success = false;
                        }
                    }
                }
            }
        } else if matches!(command, Some(Command::Print)) {
            match xcbuild_plist::serialize(&root, PlistFormat::Ascii) {
                Ok(bytes) => {
                    if let Err(e) = write_output("-", &bytes) {
                        eprintln!("error: {e}");
                        success = false;
                    }
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    success = false;
                }
            }
        } else {
            // Default: lint
            if !silent {
                println!("{file}: OK");
            }
        }
    }

    std::process::exit(if success { 0 } else { 1 });
}
