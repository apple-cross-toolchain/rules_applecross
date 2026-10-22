use plist::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// actool output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Xml,
    Binary,
    Text,
}

/// actool options parsed from command line.
#[derive(Debug, Clone)]
pub struct Options {
    pub version: bool,
    pub print_contents: bool,
    pub compile: Option<String>,
    pub compile_output_filename: Option<String>,
    pub output_format: OutputFormat,
    pub warnings: bool,
    pub errors: bool,
    pub notices: bool,
    pub platform: Option<String>,
    pub minimum_deployment_target: Option<String>,
    pub target_devices: Vec<String>,
    pub product_type: Option<String>,
    pub app_icon: Option<String>,
    pub launch_image: Option<String>,
    pub output_partial_info_plist: Option<String>,
    pub export_dependency_info: Option<String>,
    pub compress_pngs: bool,
    pub optimization: Option<String>,
    pub accent_color: Option<String>,
    pub widget_background_color: Option<String>,
    pub include_all_app_icons: bool,
    pub alternate_app_icons: Vec<String>,
    pub skip_app_store_deployment: bool,
    pub standalone_icon_behavior: Option<String>,
    pub include_sticker_content: bool,
    pub stickers_icon_role: Option<String>,
    pub target_name: Option<String>,
    pub flattened_app_icon_path: Option<String>,
    pub filter_for_device_model: Option<String>,
    pub filter_for_device_os_version: Option<String>,
    pub inputs: Vec<String>,
}

impl Options {
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let mut opts = Options {
            version: false,
            print_contents: false,
            compile: None,
            compile_output_filename: None,
            output_format: OutputFormat::Xml,
            warnings: false,
            errors: false,
            notices: false,
            platform: None,
            minimum_deployment_target: None,
            target_devices: Vec::new(),
            product_type: None,
            app_icon: None,
            launch_image: None,
            output_partial_info_plist: None,
            export_dependency_info: None,
            compress_pngs: false,
            optimization: None,
            accent_color: None,
            widget_background_color: None,
            include_all_app_icons: false,
            alternate_app_icons: Vec::new(),
            skip_app_store_deployment: false,
            standalone_icon_behavior: None,
            include_sticker_content: false,
            stickers_icon_role: None,
            target_name: None,
            flattened_app_icon_path: None,
            filter_for_device_model: None,
            filter_for_device_os_version: None,
            inputs: Vec::new(),
        };

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--version" => opts.version = true,
                "--print-contents" => opts.print_contents = true,
                "--warnings" => opts.warnings = true,
                "--errors" => opts.errors = true,
                "--notices" => opts.notices = true,
                "--compress-pngs" => opts.compress_pngs = true,
                "--compile" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("missing value for --compile".into());
                    }
                    opts.compile = Some(args[i].clone());
                }
                "--compile-output-filename" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("missing value for --compile-output-filename".into());
                    }
                    opts.compile_output_filename = Some(args[i].clone());
                }
                "--output-format" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("missing value for --output-format".into());
                    }
                    opts.output_format = match args[i].as_str() {
                        "xml1" => OutputFormat::Xml,
                        "binary1" => OutputFormat::Binary,
                        "human-readable-text" => OutputFormat::Text,
                        other => return Err(format!("unknown output format: {other}")),
                    };
                }
                "--platform" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("missing value for --platform".into());
                    }
                    opts.platform = Some(args[i].clone());
                }
                "--minimum-deployment-target" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("missing value for --minimum-deployment-target".into());
                    }
                    opts.minimum_deployment_target = Some(args[i].clone());
                }
                "--target-device" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("missing value for --target-device".into());
                    }
                    opts.target_devices.push(args[i].clone());
                }
                "--product-type" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("missing value for --product-type".into());
                    }
                    opts.product_type = Some(args[i].clone());
                }
                "--app-icon" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("missing value for --app-icon".into());
                    }
                    opts.app_icon = Some(args[i].clone());
                }
                "--launch-image" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("missing value for --launch-image".into());
                    }
                    opts.launch_image = Some(args[i].clone());
                }
                "--output-partial-info-plist" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("missing value for --output-partial-info-plist".into());
                    }
                    opts.output_partial_info_plist = Some(args[i].clone());
                }
                "--export-dependency-info" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("missing value for --export-dependency-info".into());
                    }
                    opts.export_dependency_info = Some(args[i].clone());
                }
                "--optimization" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("missing value for --optimization".into());
                    }
                    opts.optimization = Some(args[i].clone());
                }
                "--accent-color" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("missing value for --accent-color".into());
                    }
                    opts.accent_color = Some(args[i].clone());
                }
                "--widget-background-color" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("missing value for --widget-background-color".into());
                    }
                    opts.widget_background_color = Some(args[i].clone());
                }
                "--include-all-app-icons" => opts.include_all_app_icons = true,
                "--alternate-app-icon" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("missing value for --alternate-app-icon".into());
                    }
                    opts.alternate_app_icons.push(args[i].clone());
                }
                "--skip-app-store-deployment" => opts.skip_app_store_deployment = true,
                "--standalone-icon-behavior" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("missing value for --standalone-icon-behavior".into());
                    }
                    opts.standalone_icon_behavior = Some(args[i].clone());
                }
                "--include-sticker-content" => opts.include_sticker_content = true,
                "--stickers-icon-role" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("missing value for --stickers-icon-role".into());
                    }
                    opts.stickers_icon_role = Some(args[i].clone());
                }
                "--target-name" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("missing value for --target-name".into());
                    }
                    opts.target_name = Some(args[i].clone());
                }
                "--flattened-app-icon-path" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("missing value for --flattened-app-icon-path".into());
                    }
                    opts.flattened_app_icon_path = Some(args[i].clone());
                }
                "--filter-for-device-model" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("missing value for --filter-for-device-model".into());
                    }
                    opts.filter_for_device_model = Some(args[i].clone());
                }
                "--filter-for-device-os-version" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("missing value for --filter-for-device-os-version".into());
                    }
                    opts.filter_for_device_os_version = Some(args[i].clone());
                }
                // Skip other unsupported options that take values
                "--sticker-pack-identifier-prefix"
                | "--sticker-pack-strings-file"
                | "--leaderboard-identifier-prefix"
                | "--leaderboard-set-identifier-prefix"
                | "--asset-pack-output-specifications" => {
                    i += 1; // skip value
                }
                "--enable-on-demand-resources" | "--enable-incremental-distill" => {
                    i += 1; // skip value
                }
                _ => {
                    if args[i].starts_with('-') {
                        return Err(format!("unknown argument {}", args[i]));
                    }
                    opts.inputs.push(args[i].clone());
                }
            }
            i += 1;
        }

        Ok(opts)
    }
}

/// Accumulated result from actool operations.
#[derive(Debug, Clone)]
pub struct ActoolResult {
    pub errors: Vec<Message>,
    pub warnings: Vec<Message>,
    pub notices: Vec<Message>,
    pub output_files: Vec<String>,
    pub version: Option<plist::Dictionary>,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub description: String,
    pub failure_reason: Option<String>,
}

impl ActoolResult {
    pub fn new() -> Self {
        ActoolResult {
            errors: Vec::new(),
            warnings: Vec::new(),
            notices: Vec::new(),
            output_files: Vec::new(),
            version: None,
        }
    }

    pub fn success(&self) -> bool {
        self.errors.is_empty()
    }

    /// Serialize the result as a plist dictionary.
    pub fn to_plist(&self) -> Value {
        let mut dict = plist::Dictionary::new();

        if !self.errors.is_empty() {
            dict.insert(
                "com.apple.actool.errors".to_string(),
                messages_to_plist(&self.errors),
            );
        }
        if !self.warnings.is_empty() {
            dict.insert(
                "com.apple.actool.warnings".to_string(),
                messages_to_plist(&self.warnings),
            );
        }
        if !self.notices.is_empty() {
            dict.insert(
                "com.apple.actool.notices".to_string(),
                messages_to_plist(&self.notices),
            );
        }

        if !self.output_files.is_empty() {
            let mut compilation = plist::Dictionary::new();
            compilation.insert(
                "output-files".to_string(),
                Value::Array(
                    self.output_files
                        .iter()
                        .map(|f| Value::String(f.clone()))
                        .collect(),
                ),
            );
            dict.insert(
                "com.apple.actool.compilation-results".to_string(),
                Value::Dictionary(compilation),
            );
        }

        if let Some(version) = &self.version {
            dict.insert(
                "com.apple.actool.version".to_string(),
                Value::Dictionary(version.clone()),
            );
        }

        Value::Dictionary(dict)
    }

    /// Format the result as text.
    pub fn to_text(&self) -> String {
        let mut out = String::new();

        if !self.notices.is_empty() {
            out.push_str("/* com.apple.actool.notices */\n");
            for msg in &self.notices {
                out.push_str(&format!("notice: {}\n", msg.description));
            }
        }
        if !self.warnings.is_empty() {
            out.push_str("/* com.apple.actool.warnings */\n");
            for msg in &self.warnings {
                out.push_str(&format!("warning: {}\n", msg.description));
            }
        }
        if !self.errors.is_empty() {
            out.push_str("/* com.apple.actool.errors */\n");
            for msg in &self.errors {
                out.push_str(&format!("error: {}\n", msg.description));
                if let Some(reason) = &msg.failure_reason {
                    out.push_str(&format!("    Failure Reason: {reason}\n"));
                }
            }
        }
        if !self.output_files.is_empty() {
            out.push_str("/* com.apple.actool.compilation-results */\n");
            for f in &self.output_files {
                out.push_str(&format!("  {f}\n"));
            }
        }

        out
    }
}

impl Default for ActoolResult {
    fn default() -> Self {
        Self::new()
    }
}

fn messages_to_plist(messages: &[Message]) -> Value {
    Value::Array(
        messages
            .iter()
            .map(|m| {
                let mut d = plist::Dictionary::new();
                d.insert(
                    "description".to_string(),
                    Value::String(m.description.clone()),
                );
                if let Some(reason) = &m.failure_reason {
                    d.insert(
                        "failure-reason".to_string(),
                        Value::String(reason.clone()),
                    );
                }
                Value::Dictionary(d)
            })
            .collect(),
    )
}

/// Run the actool driver with the given options.
pub fn run(args: &[String]) -> i32 {
    let opts = match Options::parse(args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("error: {e}");
            return 1;
        }
    };

    let mut result = ActoolResult::new();

    if opts.version {
        result.notices.push(Message {
            description: "actool version 1 (xcbuild)".to_string(),
            failure_reason: None,
        });
        let mut version_dict = plist::Dictionary::new();
        version_dict.insert(
            "bundle-version".to_string(),
            Value::String("1".to_string()),
        );
        version_dict.insert(
            "short-bundle-version".to_string(),
            Value::String("1.0".to_string()),
        );
        result.version = Some(version_dict);
    } else if opts.print_contents {
        // Print asset catalog contents
        for input in &opts.inputs {
            if let Some(asset) = xcbuild_xcassets::Asset::load(input) {
                xcbuild_xcassets::dump_asset(&asset, 0);
            } else {
                result.errors.push(Message {
                    description: format!("unable to load asset catalog '{input}'"),
                    failure_reason: None,
                });
            }
        }
    } else if let Some(compile_path) = &opts.compile {
        // Compile action
        run_compile(&opts, compile_path, &mut result);
    } else {
        result.errors.push(Message {
            description: "no action specified".to_string(),
            failure_reason: Some(
                "use --compile, --print-contents, or --version".to_string(),
            ),
        });
    }

    // Format and output the result
    let output = match opts.output_format {
        OutputFormat::Xml => {
            let plist_value = result.to_plist();
            match xcbuild_plist::serialize(&plist_value, xcbuild_plist::PlistFormat::Xml) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("error: failed to serialize output: {e}");
                    return 1;
                }
            }
        }
        OutputFormat::Binary => {
            let plist_value = result.to_plist();
            match xcbuild_plist::serialize(&plist_value, xcbuild_plist::PlistFormat::Binary) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("error: failed to serialize output: {e}");
                    return 1;
                }
            }
        }
        OutputFormat::Text => result.to_text().into_bytes(),
    };

    use std::io::Write;
    let _ = std::io::stdout().write_all(&output);

    if result.success() {
        0
    } else {
        1
    }
}

fn warn_unsupported_options(opts: &Options) {
    if opts.include_sticker_content {
        eprintln!("warning: --include-sticker-content is not yet implemented");
    }
    if opts.stickers_icon_role.is_some() {
        eprintln!("warning: --stickers-icon-role is not yet implemented");
    }
    if opts.standalone_icon_behavior.is_some() {
        eprintln!("warning: --standalone-icon-behavior is not yet implemented");
    }
    if opts.widget_background_color.is_some() {
        eprintln!("warning: --widget-background-color is not yet implemented");
    }
    if opts.filter_for_device_model.is_some() {
        eprintln!("warning: --filter-for-device-model is not yet implemented");
    }
    if opts.filter_for_device_os_version.is_some() {
        eprintln!("warning: --filter-for-device-os-version is not yet implemented");
    }
}

fn run_compile(opts: &Options, compile_path: &str, result: &mut ActoolResult) {
    warn_unsupported_options(opts);

    let output_dir = Path::new(compile_path);
    if let Err(e) = fs::create_dir_all(output_dir) {
        result.errors.push(Message {
            description: format!("failed to create output directory: {e}"),
            failure_reason: None,
        });
        return;
    }

    let output_filename = opts
        .compile_output_filename
        .as_deref()
        .unwrap_or("Assets.car");

    if opts.inputs.is_empty() {
        result.errors.push(Message {
            description: "no input asset catalogs provided".to_string(),
            failure_reason: None,
        });
        return;
    }

    // Collect all assets from input catalogs
    let mut all_images: HashMap<String, Vec<String>> = HashMap::new();

    for input in &opts.inputs {
        let asset = match xcbuild_xcassets::Asset::load(input) {
            Some(a) => a,
            None => {
                result.errors.push(Message {
                    description: format!("unable to load asset catalog '{input}'"),
                    failure_reason: None,
                });
                return;
            }
        };

        // Collect image references from the catalog
        collect_asset_files(&asset, input, &mut all_images);
    }

    // For now, copy image files to the output directory
    // A full implementation would create a .car archive
    for (name, files) in &all_images {
        for file_path in files {
            let src = Path::new(file_path);
            if src.exists() {
                let dst_name = src.file_name().unwrap_or_default();
                let dst = output_dir.join(dst_name);
                let _ = fs::remove_file(&dst);
                if let Err(e) = fs::copy(src, &dst) {
                    result.warnings.push(Message {
                        description: format!("failed to copy {}: {}", name, e),
                        failure_reason: None,
                    });
                } else {
                    result.output_files.push(dst.to_string_lossy().to_string());
                }
            }
        }
    }

    // Write partial Info.plist if requested
    if let Some(plist_path) = &opts.output_partial_info_plist {
        let mut info_dict = plist::Dictionary::new();

        // Add app icon name if specified
        if let Some(icon) = &opts.app_icon {
            info_dict.insert(
                "CFBundleIconName".to_string(),
                Value::String(icon.clone()),
            );
        }

        // Add launch image name if specified
        if let Some(launch) = &opts.launch_image {
            info_dict.insert(
                "UILaunchImageName".to_string(),
                Value::String(launch.clone()),
            );
        }

        // Add accent color name if specified
        if let Some(accent) = &opts.accent_color {
            info_dict.insert(
                "NSAccentColorName".to_string(),
                Value::String(accent.clone()),
            );
        }

        let info_value = Value::Dictionary(info_dict);
        if let Ok(data) = xcbuild_plist::serialize(&info_value, xcbuild_plist::PlistFormat::Xml) {
            if let Some(parent) = Path::new(plist_path).parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Err(e) = fs::write(plist_path, &data) {
                result.warnings.push(Message {
                    description: format!("failed to write partial Info.plist: {e}"),
                    failure_reason: None,
                });
            } else {
                result.output_files.push(plist_path.clone());
            }
        }
    }

    // Record output car path
    let car_path = output_dir.join(output_filename);
    result
        .output_files
        .push(car_path.to_string_lossy().to_string());
}

fn collect_asset_files(
    asset: &xcbuild_xcassets::Asset,
    catalog_path: &str,
    images: &mut HashMap<String, Vec<String>>,
) {
    if let Some(contents) = &asset.contents {
        if let Some(imgs) = &contents.images {
            for img in imgs {
                if let Some(filename) = &img.filename {
                    let full_path = format!("{}/{}", asset.path, filename);
                    images
                        .entry(asset.name.clone())
                        .or_default()
                        .push(full_path);
                }
            }
        }
        if let Some(data) = &contents.data {
            for d in data {
                if let Some(filename) = &d.filename {
                    let full_path = format!("{}/{}", asset.path, filename);
                    images
                        .entry(asset.name.clone())
                        .or_default()
                        .push(full_path);
                }
            }
        }
    }

    for child in &asset.children {
        collect_asset_files(child, catalog_path, images);
    }
}
