use anyhow::{Context, Result};
use plist::Value;
use rustyline::DefaultEditor;
use std::env;
use std::fs;
use std::path::Path;
use xcbuild_plist::*;

fn help(error: Option<&str>) -> ! {
    if let Some(e) = error {
        eprintln!("Error: {e}\n");
    }

    eprintln!("Usage: PlistBuddy [options] <file.plist>\n");
    eprintln!("Options:");
    eprintln!("  -c \"<command>\" command to execute, otherwise run in interactive mode");
    eprintln!("  -x output will be in xml plist format");
    eprintln!("  -l do not follow symlinks");
    eprintln!("  -h print help including commands");

    std::process::exit(if error.is_some() { 1 } else { 0 });
}

fn command_help() {
    eprintln!("\nCommands help:");
    eprintln!("  Help - Print this information");
    eprintln!("  Exit - Exits this program");
    eprintln!("  Save - Save the changed plist file");
    eprintln!("  Revert - Revert to the saved plist file");
    eprintln!("  Clear <Type> - Clears all data, and sets root to an empty Type");
    eprintln!("  Print [<KeyPath>] - Print value at KeyPath or root");
    eprintln!("  Set <KeyPath> <Value> - Set value at KeyPath to Value");
    eprintln!("  Add <KeyPath> <Type> <Value> - Set value at KeyPath to Value");
    eprintln!("  Copy <SrcKeyPath> <DstKeyPath> - Copy SrcKeyPath to DstKeyPath ");
    eprintln!("  Delete <KeyPath> - Removes entry at KeyPath");
    eprintln!("  Merge <File> [<KeyPath>] - Merges data from <File> to KeyPath or root");
    eprintln!("  Import <KeyPath> <File> - Import <File> as data at <KeyPath>");
    eprintln!();
    eprintln!("<KeyPath>");
    eprintln!("  := \"\"                             => root object");
    eprintln!("  := <KeyPath>[:<Dictionary Key>]   => indexes into dictionary");
    eprintln!("  := <KeyPath>[:<Array Index>]      => indexes into Array");
    eprintln!("\n<Type> := (string|dictionary|array|bool|real|integer|date|data)\n");
}

/// Navigate to a value, leaving the last key unconsumed.
fn navigate_to_parent<'a>(
    root: &'a mut Value,
    keys: &[String],
) -> Option<&'a mut Value> {
    if keys.is_empty() {
        return Some(root);
    }
    let parent_keys = &keys[..keys.len() - 1];
    get_at_key_path_mut(root, parent_keys)
}

fn set_value_at_key(
    root: &mut Value,
    keys: &[String],
    value: Value,
    overwrite: bool,
) -> bool {
    if keys.is_empty() {
        eprintln!("Invalid key path (target object not found)");
        return false;
    }

    let last_key = &keys[keys.len() - 1];
    let parent = match navigate_to_parent(root, keys) {
        Some(p) => p,
        None => {
            eprintln!("Invalid key path (target object not found)");
            return false;
        }
    };

    match parent {
        Value::Dictionary(dict) => {
            if !overwrite && dict.get(last_key.as_str()).is_some() {
                eprintln!("Cannot overwrite key path");
                return false;
            }
            dict.insert(last_key.clone(), value);
            true
        }
        Value::Array(arr) => {
            if last_key.is_empty() {
                // Append to array
                arr.push(value);
                true
            } else if let Ok(idx) = last_key.parse::<usize>() {
                if overwrite {
                    if idx < arr.len() {
                        arr[idx] = value;
                    } else {
                        eprintln!("Invalid array index");
                        return false;
                    }
                } else {
                    if idx < arr.len() {
                        arr.insert(idx, value);
                    } else {
                        arr.push(value);
                    }
                }
                true
            } else {
                eprintln!("Invalid array index");
                false
            }
        }
        _ => {
            eprintln!("Invalid key path (setting value on non-collection object)");
            false
        }
    }
}

fn delete_at_key(root: &mut Value, keys: &[String]) -> bool {
    if keys.is_empty() {
        eprintln!("Invalid key path (target object not found)");
        return false;
    }

    let last_key = &keys[keys.len() - 1];
    let parent = match navigate_to_parent(root, keys) {
        Some(p) => p,
        None => {
            eprintln!("Invalid key path (target object not found)");
            return false;
        }
    };

    match parent {
        Value::Dictionary(dict) => {
            if dict.remove(last_key.as_str()).is_none() {
                eprintln!("Invalid key path (no object at key path)");
                return false;
            }
            true
        }
        Value::Array(arr) => {
            if let Ok(idx) = last_key.parse::<usize>() {
                if idx < arr.len() {
                    arr.remove(idx);
                    true
                } else {
                    eprintln!("Invalid array index");
                    false
                }
            } else {
                eprintln!("Invalid array index");
                false
            }
        }
        _ => {
            eprintln!("Invalid key path (setting value on non-collection object)");
            false
        }
    }
}

fn print_value(root: &Value, keys: &[String], use_xml: bool) -> bool {
    let target = if keys.is_empty() {
        root
    } else {
        match get_at_key_path(root, keys) {
            Some(v) => v,
            None => {
                eprintln!("Invalid key path (no object at key path)");
                return false;
            }
        }
    };

    let format = if use_xml {
        PlistFormat::Xml
    } else {
        PlistFormat::Ascii
    };

    match xcbuild_plist::serialize(target, format) {
        Ok(bytes) => {
            let _ = std::io::Write::write_all(&mut std::io::stdout(), &bytes);
            true
        }
        Err(e) => {
            eprintln!("Error: {e}");
            false
        }
    }
}

fn save_plist(root: &Value, format: PlistFormat, path: &str) -> bool {
    match xcbuild_plist::serialize(root, format) {
        Ok(bytes) => match fs::write(path, bytes) {
            Ok(()) => true,
            Err(e) => {
                eprintln!("Could not write to output: {e}");
                false
            }
        },
        Err(e) => {
            eprintln!("Error: {e}");
            false
        }
    }
}

fn load_plist(path: &str) -> Result<(Value, PlistFormat)> {
    let data = fs::read(path).with_context(|| format!("unable to read {path}"))?;
    xcbuild_plist::deserialize(&data).with_context(|| format!("unable to parse {path}"))
}

/// Tokenize a command line, respecting quoted strings, backslash escaping,
/// and both single and double quotes.
fn tokenize(input: &str) -> Result<Vec<String>, String> {
    let input = input.trim();
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut in_double_quote = false;
    let mut in_single_quote = false;

    while let Some(ch) = chars.next() {
        if ch == '\\' && !in_single_quote {
            // Backslash escaping: next character is literal
            match chars.next() {
                Some(escaped) => current.push(escaped),
                None => return Err("Trailing backslash".to_string()),
            }
        } else if ch == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
        } else if ch == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
        } else if ch.is_whitespace() && !in_double_quote && !in_single_quote {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
        } else {
            current.push(ch);
        }
    }

    if in_double_quote {
        return Err("Unterminated double quote".to_string());
    }
    if in_single_quote {
        return Err("Unterminated single quote".to_string());
    }

    if !current.is_empty() {
        tokens.push(current);
    }
    Ok(tokens)
}

fn process_command(
    path: &str,
    use_xml: bool,
    save_format: PlistFormat,
    root: &mut Value,
    input: &str,
    mutated: &mut bool,
    keep_reading: &mut bool,
    last_saved: &mut Value,
) -> bool {
    let tokens = match tokenize(input) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Tokenize error: {e}");
            return false;
        }
    };
    if tokens.is_empty() {
        return true;
    }

    let command = &tokens[0];
    match command.to_ascii_lowercase().as_str() {
        "print" => {
            let keys = if tokens.len() > 1 {
                parse_key_path(&tokens[1])
            } else {
                vec![]
            };
            print_value(root, &keys, use_xml)
        }
        "save" => {
            let result = save_plist(root, save_format, path);
            if result {
                *last_saved = root.clone();
            }
            result
        }
        "revert" => {
            *root = last_saved.clone();
            *mutated = true;
            true
        }
        "exit" => {
            *keep_reading = false;
            true
        }
        "set" => {
            if tokens.len() < 3 {
                eprintln!("Set requires <KeyPath> <Value>");
                return false;
            }
            let keys = parse_key_path(&tokens[1]);
            // Require that the key path already exists
            if get_at_key_path(root, &keys).is_none() {
                eprintln!("Invalid key path (no object at key path)");
                return false;
            }
            let value_str = tokens[2..].join(" ");
            let value = Value::String(value_str);
            let result = set_value_at_key(root, &keys, value, true);
            if result {
                *mutated = true;
            }
            result
        }
        "add" => {
            if tokens.len() < 3 {
                eprintln!("Add command requires KeyPath and Type");
                return false;
            }
            let keys = parse_key_path(&tokens[1]);
            let obj_type = match ObjectType::parse(&tokens[2]) {
                Some(t) => t,
                None => {
                    eprintln!("Unsupported type: {}", tokens[2]);
                    return false;
                }
            };
            let value_str = if tokens.len() > 3 {
                tokens[3..].join(" ")
            } else {
                String::new()
            };
            let value = match create_value(obj_type, &value_str) {
                Some(v) => v,
                None => {
                    eprintln!("Invalid value");
                    return false;
                }
            };
            let result = set_value_at_key(root, &keys, value, false);
            if result {
                *mutated = true;
            }
            result
        }
        "clear" => {
            let type_name = if tokens.len() > 1 {
                tokens[1].as_str()
            } else {
                "dictionary"
            };
            let obj_type = match ObjectType::parse(type_name) {
                Some(t) => t,
                None => {
                    eprintln!("Unsupported type");
                    return false;
                }
            };
            if let Some(new_root) = create_value(obj_type, "") {
                *root = new_root;
                *mutated = true;
                true
            } else {
                eprintln!("Unsupported type");
                false
            }
        }
        "delete" => {
            if tokens.len() < 2 {
                eprintln!("Delete command requires KeyPath");
                return false;
            }
            let keys = parse_key_path(&tokens[1]);
            let result = delete_at_key(root, &keys);
            if result {
                *mutated = true;
            }
            result
        }
        "copy" => {
            if tokens.len() < 3 {
                eprintln!("Copy command requires SrcKeyPath and DstKeyPath");
                return false;
            }
            let src_keys = parse_key_path(&tokens[1]);
            let dst_keys = parse_key_path(&tokens[2]);

            // Get source value
            let source_value = match get_at_key_path(root, &src_keys) {
                Some(v) => v.clone(),
                None => {
                    eprintln!("Invalid key path (source object not found)");
                    return false;
                }
            };

            let result = set_value_at_key(root, &dst_keys, source_value, true);
            if result {
                *mutated = true;
            }
            result
        }
        "merge" => {
            if tokens.len() < 2 {
                eprintln!("Merge command requires KeyPath");
                return false;
            }
            let merge_file = &tokens[1];
            let keys = if tokens.len() > 2 {
                parse_key_path(&tokens[2])
            } else {
                vec![]
            };

            let merge_value = match load_plist(merge_file) {
                Ok((v, _)) => v,
                Err(e) => {
                    eprintln!("Unable to read merge source file: {e}");
                    return false;
                }
            };

            let target = if keys.is_empty() {
                root
            } else {
                match get_at_key_path_mut(root, &keys) {
                    Some(v) => v,
                    None => {
                        eprintln!("Invalid key path (no object at key path)");
                        return false;
                    }
                }
            };

            match (target, merge_value) {
                (Value::Dictionary(target_dict), Value::Dictionary(merge_dict)) => {
                    for (k, v) in merge_dict.into_iter() {
                        if target_dict.get(&k).is_some() {
                            eprintln!("Skipping duplicate key: {k}");
                        } else {
                            target_dict.insert(k, v);
                        }
                    }
                    *mutated = true;
                    true
                }
                (Value::Array(target_arr), Value::Array(merge_arr)) => {
                    for v in merge_arr {
                        target_arr.push(v);
                    }
                    *mutated = true;
                    true
                }
                (Value::Dictionary(_), Value::Array(_)) => {
                    eprintln!("Cannot merge array into dictionary");
                    false
                }
                (Value::Array(_), Value::Dictionary(_)) => {
                    eprintln!("Cannot merge dictionary into array");
                    false
                }
                (Value::Dictionary(dict), other) => {
                    dict.insert(String::new(), other);
                    *mutated = true;
                    true
                }
                (Value::Array(arr), other) => {
                    arr.push(other);
                    *mutated = true;
                    true
                }
                _ => {
                    eprintln!("Object at KeyPath is not a container");
                    false
                }
            }
        }
        "import" => {
            if tokens.len() < 3 {
                eprintln!("Import command requires KeyPath and File");
                return false;
            }
            let keys = parse_key_path(&tokens[1]);
            let source = &tokens[2];
            let data = match fs::read(source) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Could not read source file: {e}");
                    return false;
                }
            };
            let value = Value::Data(data);
            let result = set_value_at_key(root, &keys, value, true);
            if result {
                *mutated = true;
            }
            result
        }
        "help" => {
            command_help();
            true
        }
        _ => {
            eprintln!("Unrecognized command: {command}");
            false
        }
    }
}

/// Check whether any component of the given path is a symlink.
fn path_contains_symlink(path: &str) -> bool {
    let p = Path::new(path);
    let mut accumulated = std::path::PathBuf::new();
    for component in p.components() {
        accumulated.push(component);
        if let Ok(metadata) = fs::symlink_metadata(&accumulated) {
            if metadata.file_type().is_symlink() {
                return true;
            }
        }
    }
    false
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut show_help = false;
    let mut use_xml = false;
    let mut no_follow_symlinks = false;
    let mut commands: Vec<String> = Vec::new();
    let mut input = String::new();

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-h" => show_help = true,
            "-x" => use_xml = true,
            "-l" => no_follow_symlinks = true,
            "-c" => {
                i += 1;
                if i >= args.len() {
                    help(Some("missing value for -c"));
                }
                commands.push(args[i].clone());
            }
            _ => {
                if arg.starts_with('-') {
                    help(Some(&format!("unknown argument {arg}")));
                }
                input = arg.clone();
            }
        }
        i += 1;
    }

    if show_help {
        help(None);
    }

    // Load or create plist
    let mut root: Value;
    let mut save_format = PlistFormat::Xml;

    if no_follow_symlinks && !input.is_empty() {
        if path_contains_symlink(&input) {
            eprintln!("Error: path contains a symlink and -l was specified");
            std::process::exit(1);
        }
    }

    if !input.is_empty() && Path::new(&input).exists() {
        match load_plist(&input) {
            Ok((v, f)) => {
                root = v;
                save_format = f;
            }
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
    } else {
        if !input.is_empty() {
            eprintln!("File does not exist, will create {input}");
        }
        root = Value::Dictionary(plist::Dictionary::new());
    }

    let mut last_saved = root.clone();
    let mut success = true;

    if !commands.is_empty() {
        // Batch mode: process each -c command, save after each mutation
        for cmd in &commands {
            let mut mutated = false;
            let mut keep_reading = true;
            success &= process_command(
                &input,
                use_xml,
                save_format,
                &mut root,
                cmd,
                &mut mutated,
                &mut keep_reading,
                &mut last_saved,
            );
            if mutated && !input.is_empty() {
                save_plist(&root, save_format, &input);
                last_saved = root.clone();
            }
        }
    } else {
        // Interactive mode
        let mut editor = DefaultEditor::new().expect("failed to create editor");
        let mut keep_reading = true;

        while keep_reading {
            match editor.readline("Command: ") {
                Ok(line) => {
                    let _ = editor.add_history_entry(&line);
                    let mut mutated = false;
                    success &= process_command(
                        &input,
                        use_xml,
                        save_format,
                        &mut root,
                        &line,
                        &mut mutated,
                        &mut keep_reading,
                        &mut last_saved,
                    );
                }
                Err(_) => {
                    keep_reading = false;
                }
            }
        }
    }

    std::process::exit(if success { 0 } else { 1 });
}
