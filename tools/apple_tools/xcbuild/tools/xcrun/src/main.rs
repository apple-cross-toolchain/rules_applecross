use std::env;
use std::path::Path;
use std::process;
use xcbuild_sdk::*;

fn help(error: Option<&str>) -> ! {
    if let Some(e) = error {
        eprintln!("error: {e}\n");
    }

    eprintln!("Usage: xcrun [options] -- [tool] [arguments]\n");
    eprintln!("Find and execute developer tools.\n");

    eprintln!("Modes:");
    eprintln!("  -r, --run (default)");
    eprintln!("  -f, --find");
    eprintln!("  -h, --help (this message)");
    eprintln!("  --version");
    eprintln!("  --show-sdk-path");
    eprintln!("  --show-sdk-version");
    eprintln!("  --show-sdk-build-version");
    eprintln!("  --show-sdk-platform-path");
    eprintln!("  --show-sdk-platform-version");
    eprintln!("  --show-toolchain-path");
    eprintln!();

    eprintln!("Options:");
    eprintln!("  -v, --verbose");
    eprintln!("  -l, --log");
    eprintln!("  -n, --no-cache (not implemented)");
    eprintln!("  -k, --kill-cache (not implemented)");

    process::exit(if error.is_some() { 1 } else { 0 });
}

fn main() {
    let all_args: Vec<String> = env::args().collect();

    // Symlink invocation mode: if invoked under a name other than "xcrun",
    // treat argv[0] basename as the tool name and pass all args through.
    let argv0 = all_args.first().map(|s| s.as_str()).unwrap_or("");
    let basename = Path::new(argv0)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    let args: Vec<String> = all_args[1..].to_vec();

    if !basename.is_empty() && basename != "xcrun" {
        // Symlink mode: basename is the tool name, all args are tool args
        run_tool(basename.to_string(), args, None, None, false, false);
        return;
    }

    let mut show_help = false;
    let mut show_version = false;
    let mut find_mode = false;
    let mut show_sdk_path = false;
    let mut show_sdk_version = false;
    let mut show_sdk_build_version = false;
    let mut show_sdk_platform_path = false;
    let mut show_sdk_platform_version = false;
    let mut show_toolchain_path = false;
    let mut verbose = false;
    let mut log_mode = false;
    let mut sdk_arg: Option<String> = None;
    let mut toolchain_arg: Option<String> = None;
    let mut tool: Option<String> = None;
    let mut tool_args: Vec<String> = Vec::new();
    let mut separator = false;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];

        if !separator && tool.is_none() {
            match arg.as_str() {
                "-h" | "--help" | "-help" => show_help = true,
                "--version" | "-version" => show_version = true,
                "-r" | "--run" | "-run" => {} // run is default
                "-f" | "--find" | "-find" => find_mode = true,
                "--show-sdk-path" | "-show-sdk-path" => show_sdk_path = true,
                "--show-sdk-version" | "-show-sdk-version" => show_sdk_version = true,
                "--show-sdk-build-version" | "-show-sdk-build-version" => {
                    show_sdk_build_version = true
                }
                "--show-sdk-platform-path" | "-show-sdk-platform-path" => {
                    show_sdk_platform_path = true
                }
                "--show-sdk-platform-version" | "-show-sdk-platform-version" => {
                    show_sdk_platform_version = true
                }
                "--show-toolchain-path" | "-show-toolchain-path" => {
                    show_toolchain_path = true
                }
                "-l" | "--log" | "-log" => log_mode = true,
                "-v" | "--verbose" | "-verbose" => verbose = true,
                "-n" | "--no-cache" | "-no-cache" => {
                    eprintln!("warning: cache options not implemented");
                }
                "-k" | "--kill-cache" | "-kill-cache" => {
                    eprintln!("warning: cache options not implemented");
                }
                "--sdk" | "-sdk" => {
                    i += 1;
                    if i >= args.len() {
                        help(Some("missing value for --sdk"));
                    }
                    sdk_arg = Some(args[i].clone());
                }
                "--toolchain" | "-toolchain" => {
                    i += 1;
                    if i >= args.len() {
                        help(Some("missing value for --toolchain"));
                    }
                    toolchain_arg = Some(args[i].clone());
                }
                "--" => separator = true,
                _ => {
                    if arg.starts_with('-') {
                        help(Some(&format!("unknown argument {arg}")));
                    }
                    tool = Some(arg.clone());
                }
            }
        } else {
            // After tool name or separator, collect tool args
            if tool.is_none() {
                tool = Some(arg.clone());
            } else {
                tool_args.push(arg.clone());
            }
        }

        i += 1;
    }

    // Handle help/version without needing SDK
    if tool.is_none() {
        if show_help {
            help(None);
        }
        if show_version {
            println!("xcrun version 1 (xcbuild)");
            process::exit(0);
        }
    }

    // Check env for fallbacks
    let toolchain_specified = toolchain_arg.is_some();
    let toolchain_input = toolchain_arg.or_else(|| env::var("TOOLCHAINS").ok());
    let sdk_explicit = sdk_arg.is_some();
    let sdk_name = sdk_arg.or_else(|| env::var("SDKROOT").ok());
    let sdk_from_env = !sdk_explicit && sdk_name.is_some();
    if env::var("xcrun_verbose").is_ok() {
        verbose = true;
    }
    if env::var("xcrun_log").is_ok() {
        log_mode = true;
    }

    // Load SDK manager
    let developer_root = match find_developer_root() {
        Some(r) => r,
        None => {
            eprintln!("error: unable to find developer root");
            process::exit(1);
        }
    };

    let config = Configuration::load(&Configuration::default_paths());
    let manager = match Manager::open(&developer_root, config.as_ref()) {
        Some(m) => m,
        None => {
            eprintln!("error: unable to load manager from '{developer_root}'");
            process::exit(1);
        }
    };

    if verbose {
        eprintln!("verbose: using developer root '{}'", manager.path);
    }

    let show_sdk_value = show_sdk_path
        || show_sdk_version
        || show_sdk_build_version
        || show_sdk_platform_path
        || show_sdk_platform_version;

    // Resolve toolchains early (needed for --show-toolchain-path)
    let toolchain_input_ref = toolchain_input.clone();

    // Find target (SDK)
    let target_result = if !toolchain_specified {
        let name = sdk_name.as_deref().unwrap_or("macosx");
        manager.find_target(name)
    } else {
        None
    };

    if show_sdk_value && target_result.is_none() && !toolchain_specified {
        let name = sdk_name.as_deref().unwrap_or("macosx");
        eprintln!("error: unable to find sdk: '{name}'");
        process::exit(1);
    }

    if verbose {
        if let Some((_, target)) = &target_result {
            let name = target
                .canonical_name
                .as_deref()
                .unwrap_or(&target.bundle_name);
            eprintln!("verbose: using sdk '{name}': {}", target.path);
        } else {
            eprintln!("verbose: not using any SDK");
        }
    }

    // Resolve toolchains
    let mut toolchains: Vec<&Toolchain> = Vec::new();
    if let Some(tc_input) = &toolchain_input_ref {
        for token in tc_input.split_whitespace() {
            if let Some(tc) = manager.find_toolchain(token) {
                toolchains.push(tc);
            }
        }
        if toolchains.is_empty() {
            eprintln!("error: unable to find toolchains in '{tc_input}'");
            process::exit(1);
        }
    } else if let Some((_, target)) = &target_result {
        for tc_id in &target.toolchain_identifiers {
            if let Some(tc) = manager.find_toolchain(tc_id) {
                toolchains.push(tc);
            }
        }
    }

    if toolchains.is_empty() {
        if let Some(tc) = manager.find_toolchain(Toolchain::default_identifier()) {
            toolchains.push(tc);
        }
    }

    // Handle --show-toolchain-path
    if show_toolchain_path {
        if toolchains.is_empty() {
            eprintln!("error: unable to find any toolchains");
            process::exit(1);
        }
        println!("{}", toolchains[0].path);
        process::exit(0);
    }

    // Handle SDK queries
    if show_sdk_value {
        let (platform, target) = target_result.expect("target required for SDK queries");

        if show_sdk_path {
            println!("{}", target.path);
        } else if show_sdk_version {
            println!("{}", target.version.as_deref().unwrap_or(""));
        } else if show_sdk_build_version {
            let build_version = target.product.as_ref()
                .and_then(|p| p.build_version.as_deref())
                .or(target.canonical_name.as_deref());
            println!("{}", build_version.unwrap_or(""));
        } else if show_sdk_platform_path {
            println!("{}", platform.path);
        } else if show_sdk_platform_version {
            println!("{}", platform.version.as_deref().unwrap_or(""));
        }

        process::exit(0);
    }

    // Tool execution mode
    let tool_name = match &tool {
        Some(t) => t.clone(),
        None => help(Some("no tool provided")),
    };

    if toolchains.is_empty() {
        eprintln!("error: unable to find any toolchains");
        process::exit(1);
    }

    if verbose {
        eprint!("verbose: using toolchain(s):");
        for tc in &toolchains {
            if let Some(id) = &tc.identifier {
                eprint!(" '{id}'");
            }
        }
        eprintln!();
    }

    // Build executable search paths
    let platform = target_result.map(|(p, _)| p);
    let target = target_result.map(|(_, t)| t);
    let mut exec_paths = manager.all_executable_paths(platform, target, &toolchains);

    // Add system PATH
    if let Ok(sys_path) = env::var("PATH") {
        for p in sys_path.split(':') {
            exec_paths.push(p.to_string());
        }
    }

    // Check if tool is an absolute/relative path
    let executable = if tool_name.contains('/') {
        let path = std::path::PathBuf::from(&tool_name);
        if path.is_file() {
            Some(path)
        } else {
            None
        }
    } else {
        find_executable(&tool_name, &exec_paths)
    };

    // Find the tool
    let executable = match executable {
        Some(e) => e,
        None => {
            eprintln!("error: tool '{tool_name}' not found");
            process::exit(1);
        }
    };

    if verbose {
        eprintln!(
            "verbose: resolved tool '{tool_name}' to: {}",
            executable.display()
        );
    }

    if find_mode {
        println!("{}", executable.display());
        process::exit(0);
    }

    // Run mode (default)
    if let Some((_, target)) = &target_result {
        env::set_var("SDKROOT", &target.path);
        if log_mode {
            println!(
                "env SDKROOT={} {}",
                target.path,
                executable.display()
            );
        }
    }

    // Forward TOOLCHAINS to child process
    if let Some(tc_input) = &toolchain_input_ref {
        env::set_var("TOOLCHAINS", tc_input);
        if log_mode {
            println!("env TOOLCHAINS={} {}", tc_input, executable.display());
        }
    }

    // CPATH and LIBRARY_PATH env manipulation
    // When no explicit SDK is requested and SDKROOT was not in env
    if !sdk_explicit && !sdk_from_env {
        // Prepend /usr/local/include to CPATH unless -nostdinc or -nostdsysteminc in tool_args
        if !tool_args.iter().any(|a| a == "-nostdinc" || a == "-nostdsysteminc") {
            let cpath = env::var("CPATH").unwrap_or_default();
            if cpath.is_empty() {
                env::set_var("CPATH", "/usr/local/include");
            } else {
                env::set_var("CPATH", format!("/usr/local/include:{cpath}"));
            }
        }

        // Prepend /usr/local/lib to LIBRARY_PATH unless -Z in tool_args
        if !tool_args.iter().any(|a| a == "-Z") {
            let lib_path = env::var("LIBRARY_PATH").unwrap_or_default();
            if lib_path.is_empty() {
                env::set_var("LIBRARY_PATH", "/usr/local/lib");
            } else {
                env::set_var("LIBRARY_PATH", format!("/usr/local/lib:{lib_path}"));
            }
        }
    }

    if verbose {
        println!("verbose: executing tool: {}", executable.display());
    }

    // Execute the tool, replacing the current process on Unix
    #[cfg(unix)]
    exec_unix(&executable, &tool_args);

    #[cfg(not(unix))]
    {
        let status = process::Command::new(&executable)
            .args(&tool_args)
            .status()
            .expect("failed to execute tool");
        process::exit(status.code().unwrap_or(1));
    }
}

#[allow(dead_code)]
fn run_tool(
    tool_name: String,
    tool_args: Vec<String>,
    sdk_arg: Option<String>,
    toolchain_arg: Option<String>,
    verbose: bool,
    log_mode: bool,
) {
    // Load SDK manager
    let developer_root = match find_developer_root() {
        Some(r) => r,
        None => {
            eprintln!("error: unable to find developer root");
            process::exit(1);
        }
    };

    let config = Configuration::load(&Configuration::default_paths());
    let manager = match Manager::open(&developer_root, config.as_ref()) {
        Some(m) => m,
        None => {
            eprintln!("error: unable to load manager from '{developer_root}'");
            process::exit(1);
        }
    };

    let toolchain_specified = toolchain_arg.is_some();
    let toolchain_input = toolchain_arg.or_else(|| env::var("TOOLCHAINS").ok());
    let sdk_name = sdk_arg.or_else(|| env::var("SDKROOT").ok());

    let target_result = if !toolchain_specified {
        let name = sdk_name.as_deref().unwrap_or("macosx");
        manager.find_target(name)
    } else {
        None
    };

    let mut toolchains: Vec<&Toolchain> = Vec::new();
    if let Some(tc_input) = &toolchain_input {
        for token in tc_input.split_whitespace() {
            if let Some(tc) = manager.find_toolchain(token) {
                toolchains.push(tc);
            }
        }
    } else if let Some((_, target)) = &target_result {
        for tc_id in &target.toolchain_identifiers {
            if let Some(tc) = manager.find_toolchain(tc_id) {
                toolchains.push(tc);
            }
        }
    }

    if toolchains.is_empty() {
        if let Some(tc) = manager.find_toolchain(Toolchain::default_identifier()) {
            toolchains.push(tc);
        }
    }

    if toolchains.is_empty() {
        eprintln!("error: unable to find any toolchains");
        process::exit(1);
    }

    let platform = target_result.map(|(p, _)| p);
    let target = target_result.map(|(_, t)| t);
    let mut exec_paths = manager.all_executable_paths(platform, target, &toolchains);

    if let Ok(sys_path) = env::var("PATH") {
        for p in sys_path.split(':') {
            exec_paths.push(p.to_string());
        }
    }

    let executable = match find_executable(&tool_name, &exec_paths) {
        Some(e) => e,
        None => {
            eprintln!("error: tool '{tool_name}' not found");
            process::exit(1);
        }
    };

    if verbose {
        eprintln!(
            "verbose: resolved tool '{tool_name}' to: {}",
            executable.display()
        );
    }

    if let Some((_, target)) = &target_result {
        env::set_var("SDKROOT", &target.path);
        if log_mode {
            println!(
                "env SDKROOT={} {}",
                target.path,
                executable.display()
            );
        }
    }

    #[cfg(unix)]
    exec_unix(&executable, &tool_args);

    #[cfg(not(unix))]
    {
        let status = process::Command::new(&executable)
            .args(&tool_args)
            .status()
            .expect("failed to execute tool");
        process::exit(status.code().unwrap_or(1));
    }
}

#[cfg(unix)]
fn exec_unix(executable: &std::path::Path, tool_args: &[String]) -> ! {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let c_exe =
        CString::new(executable.as_os_str().as_bytes()).expect("invalid executable path");
    let mut c_args: Vec<CString> = Vec::new();
    c_args.push(c_exe.clone());
    for arg in tool_args {
        c_args.push(CString::new(arg.as_bytes()).expect("invalid argument"));
    }

    nix::unistd::execv(&c_exe, &c_args).expect("failed to exec");
    unreachable!();
}
