use plist::Value;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SdkError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("plist error: {0}")]
    Plist(#[from] xcbuild_plist::PlistError),
    #[error("{0}")]
    Other(String),
}

// --- Configuration ---

/// Extra search paths for platforms and toolchains.
#[derive(Debug, Clone, Default)]
pub struct Configuration {
    pub extra_platforms_paths: Vec<String>,
    pub extra_toolchains_paths: Vec<String>,
}

impl Configuration {
    /// Default configuration file search paths.
    pub fn default_paths() -> Vec<String> {
        let mut paths = Vec::new();
        if let Ok(val) = std::env::var("XCSDK_CONFIGURATION_PATH") {
            paths.push(val);
        } else {
            if let Some(home) = home_dir() {
                paths.push(format!("{home}/.xcsdk/xcsdk_configuration.plist"));
            }
            paths.push("/var/db/xcsdk_configuration.plist".to_string());
        }
        paths
    }

    /// Load configuration from the first valid file in the paths list.
    pub fn load(paths: &[String]) -> Option<Configuration> {
        for path in paths {
            if let Ok(data) = fs::read(path) {
                if let Ok((value, _)) = xcbuild_plist::deserialize(&data) {
                    if let Value::Dictionary(dict) = value {
                        let platforms = string_array_from_dict(&dict, "ExtraPlatformsPaths");
                        let toolchains = string_array_from_dict(&dict, "ExtraToolchainsPaths");
                        return Some(Configuration {
                            extra_platforms_paths: platforms,
                            extra_toolchains_paths: toolchains,
                        });
                    }
                }
            }
        }
        None
    }
}

// --- Environment / Developer Root ---

/// Resolve a developer root: if path/Contents/Developer exists, use that.
fn resolve_developer_root(path: &str) -> String {
    let app_path = format!("{path}/Contents/Developer");
    if Path::new(&app_path).is_dir() {
        return fs::canonicalize(&app_path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(app_path);
    }
    fs::canonicalize(path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string())
}

/// Find the developer root directory.
pub fn find_developer_root() -> Option<String> {
    // 1. DEVELOPER_DIR environment variable
    if let Ok(path) = std::env::var("DEVELOPER_DIR") {
        if !path.is_empty() {
            return Some(resolve_developer_root(&path));
        }
    }

    // 2. User-specific link
    if let Some(home) = home_dir() {
        let user_link = format!("{home}/.xcsdk/xcode_select_link");
        if let Ok(target) = fs::read_link(&user_link) {
            return Some(target.to_string_lossy().to_string());
        }
    }

    // 3. Primary system link
    let primary = "/var/db/xcode_select_link";
    if let Ok(target) = fs::read_link(primary) {
        if let Ok(canonical) = fs::canonicalize(&target) {
            return Some(canonical.to_string_lossy().to_string());
        }
        return Some(target.to_string_lossy().to_string());
    }

    // 4. Secondary system link
    let secondary = "/usr/share/xcode-select/xcode_dir_path";
    if let Ok(target) = fs::read_link(secondary) {
        if let Ok(canonical) = fs::canonicalize(&target) {
            return Some(canonical.to_string_lossy().to_string());
        }
        return Some(target.to_string_lossy().to_string());
    }

    // 5. Well-known fallback paths
    let defaults = [
        "/Applications/Xcode.app/Contents/Developer",
        "/Developer",
    ];
    for path in &defaults {
        if Path::new(path).is_dir() {
            return Some(path.to_string());
        }
    }

    None
}

/// Write the developer root symlink.
pub fn write_developer_root(path: Option<&str>) -> bool {
    let link_path = "/var/db/xcode_select_link";

    // Remove existing link if present
    if Path::new(link_path).exists() || fs::symlink_metadata(link_path).is_ok() {
        if fs::remove_file(link_path).is_err() {
            return false;
        }
    }

    let path = match path {
        Some(p) => p,
        None => return true,  // Reset: just remove the link
    };

    let resolved = resolve_developer_root(path);
    if !Path::new(&resolved).is_dir() {
        eprintln!("error: invalid developer directory: '{path}'");
        return false;
    }

    // Create /var/db if needed
    let var_db = Path::new("/var/db");
    if !var_db.exists() {
        if fs::create_dir_all(var_db).is_err() {
            return false;
        }
    }

    // Create symlink
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&resolved, link_path).is_ok()
    }
    #[cfg(not(unix))]
    {
        false
    }
}

// --- Product ---

/// SDK product information (from SystemVersion.plist).
#[derive(Debug, Clone, Default)]
pub struct Product {
    pub name: Option<String>,
    pub version: Option<String>,
    pub user_visible_version: Option<String>,
    pub build_version: Option<String>,
    pub copyright: Option<String>,
}

impl Product {
    pub fn open(sdk_path: &str) -> Option<Product> {
        let plist_path = format!("{sdk_path}/System/Library/CoreServices/SystemVersion.plist");
        let data = fs::read(&plist_path).ok()?;
        let (value, _) = xcbuild_plist::deserialize(&data).ok()?;
        let dict = match &value {
            Value::Dictionary(d) => d,
            _ => return None,
        };

        Some(Product {
            name: get_string(dict, "ProductName"),
            version: get_string(dict, "ProductVersion"),
            user_visible_version: get_string(dict, "ProductUserVisibleVersion"),
            build_version: get_string(dict, "ProductBuildVersion"),
            copyright: get_string(dict, "ProductCopyright"),
        })
    }
}

// --- PlatformVersion ---

#[derive(Debug, Clone, Default)]
pub struct PlatformVersion {
    pub project_name: Option<String>,
    pub product_build_version: Option<String>,
    pub build_version: Option<String>,
    pub source_version: Option<String>,
}

impl PlatformVersion {
    pub fn open(platform_path: &str) -> Option<PlatformVersion> {
        let plist_path = format!("{platform_path}/version.plist");
        let data = fs::read(&plist_path).ok()?;
        let (value, _) = xcbuild_plist::deserialize(&data).ok()?;
        let dict = match &value {
            Value::Dictionary(d) => d,
            _ => return None,
        };

        Some(PlatformVersion {
            project_name: get_string(dict, "ProjectName"),
            product_build_version: get_string(dict, "ProductBuildVersion"),
            build_version: get_string(dict, "BuildVersion"),
            source_version: get_string(dict, "SourceVersion"),
        })
    }
}

// --- Toolchain ---

#[derive(Debug, Clone)]
pub struct Toolchain {
    pub path: String,
    pub name: String,
    pub identifier: Option<String>,
    pub display_name: Option<String>,
    pub version: Option<String>,
}

impl Toolchain {
    pub fn default_identifier() -> &'static str {
        "com.apple.dt.toolchain.XcodeDefault"
    }

    pub fn executable_paths(&self) -> Vec<String> {
        vec![
            format!("{}/usr/bin", self.path),
            format!("{}/usr/libexec", self.path),
        ]
    }

    pub fn open(path: &str) -> Option<Toolchain> {
        if path.is_empty() {
            return None;
        }

        // Try ToolchainInfo.plist first, then Info.plist
        let plist_path = {
            let tc_info = format!("{path}/ToolchainInfo.plist");
            if Path::new(&tc_info).is_file() {
                tc_info
            } else {
                let info = format!("{path}/Info.plist");
                if Path::new(&info).is_file() {
                    info
                } else {
                    return None;
                }
            }
        };

        let data = fs::read(&plist_path).ok()?;
        let (value, _) = xcbuild_plist::deserialize(&data).ok()?;
        let dict = match &value {
            Value::Dictionary(d) => d,
            _ => return None,
        };

        let real_path = fs::canonicalize(&plist_path)
            .ok()
            .and_then(|p| p.parent().map(|pp| pp.to_string_lossy().to_string()))
            .unwrap_or_else(|| path.to_string());
        let name = Path::new(&real_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        Some(Toolchain {
            path: real_path,
            name,
            identifier: get_string(dict, "Identifier")
                .or_else(|| get_string(dict, "CFBundleIdentifier")),
            display_name: get_string(dict, "DisplayName"),
            version: get_string(dict, "Version"),
        })
    }
}

// --- Platform ---

#[derive(Debug, Clone)]
pub struct Platform {
    pub path: String,
    pub name: String,
    pub identifier: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub family_identifier: Option<String>,
    pub platform_version: Option<PlatformVersion>,
    pub targets: Vec<Target>,
}

impl Platform {
    pub fn executable_paths(&self) -> Vec<String> {
        vec![
            format!("{}/Developer/usr/bin", self.path),
            format!("{}/usr/local/bin", self.path),
            format!("{}/usr/bin", self.path),
        ]
    }

    pub fn open(path: &str, toolchains: &[Toolchain]) -> Option<Platform> {
        if path.is_empty() {
            return None;
        }

        let info_path = format!("{path}/Info.plist");
        let data = fs::read(&info_path).ok()?;
        let (value, _) = xcbuild_plist::deserialize(&data).ok()?;
        let dict = match &value {
            Value::Dictionary(d) => d,
            _ => return None,
        };

        let real_path = fs::canonicalize(&info_path)
            .ok()
            .and_then(|p| p.parent().map(|pp| pp.to_string_lossy().to_string()))
            .unwrap_or_else(|| path.to_string());

        let name = get_string(dict, "Name").unwrap_or_default();
        let platform_version = PlatformVersion::open(&real_path);

        // Load SDKs
        let sdks_path = format!("{real_path}/Developer/SDKs");
        let mut targets = Vec::new();
        if let Ok(entries) = fs::read_dir(&sdks_path) {
            for entry in entries.flatten() {
                let fname = entry.file_name().to_string_lossy().to_string();
                if fname.ends_with(".sdk") {
                    let sdk_path = format!("{sdks_path}/{fname}");
                    if let Some(target) = Target::open(&sdk_path, toolchains) {
                        targets.push(target);
                    }
                }
            }
        }
        targets.sort_by(|a, b| a.canonical_name.cmp(&b.canonical_name));

        Some(Platform {
            path: real_path,
            name,
            identifier: get_string(dict, "Identifier"),
            description: get_string(dict, "Description"),
            version: get_string(dict, "Version"),
            family_identifier: get_string(dict, "FamilyIdentifier"),
            platform_version,
            targets,
        })
    }
}

// --- Target (SDK) ---

#[derive(Debug, Clone)]
pub struct Target {
    pub path: String,
    pub bundle_name: String,
    pub canonical_name: Option<String>,
    pub display_name: Option<String>,
    pub version: Option<String>,
    pub toolchain_identifiers: Vec<String>,
    pub product: Option<Product>,
}

impl Target {
    pub fn open(path: &str, toolchains: &[Toolchain]) -> Option<Target> {
        if path.is_empty() {
            return None;
        }

        // Try SDKSettings.plist first, then Info.plist
        let plist_path = {
            let settings = format!("{path}/SDKSettings.plist");
            if Path::new(&settings).is_file() {
                settings
            } else {
                let info = format!("{path}/Info.plist");
                if Path::new(&info).is_file() {
                    info
                } else {
                    return None;
                }
            }
        };

        let data = fs::read(&plist_path).ok()?;
        let (value, _) = xcbuild_plist::deserialize(&data).ok()?;
        let dict = match &value {
            Value::Dictionary(d) => d,
            _ => return None,
        };

        let real_path = fs::canonicalize(&plist_path)
            .ok()
            .and_then(|p| p.parent().map(|pp| pp.to_string_lossy().to_string()))
            .unwrap_or_else(|| path.to_string());

        let bundle_name = Path::new(&real_path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        // Resolve toolchain identifiers
        let mut toolchain_ids: Vec<String> = string_array_from_dict(dict, "Toolchains");
        // If no toolchains specified, use default
        if toolchain_ids.is_empty() {
            let default_id = Toolchain::default_identifier();
            if toolchains.iter().any(|tc| tc.identifier.as_deref() == Some(default_id)) {
                toolchain_ids.push(default_id.to_string());
            }
        }

        let product = Product::open(&real_path);

        Some(Target {
            path: real_path,
            bundle_name,
            canonical_name: get_string(dict, "CanonicalName"),
            display_name: get_string(dict, "DisplayName"),
            version: get_string(dict, "Version"),
            toolchain_identifiers: toolchain_ids,
            product,
        })
    }
}

// --- Manager ---

/// SDK Manager: discovers platforms, toolchains, and targets.
#[derive(Debug, Clone)]
pub struct Manager {
    pub path: String,
    pub toolchains: Vec<Toolchain>,
    pub platforms: Vec<Platform>,
}

impl Manager {
    /// Open and scan a developer root directory.
    pub fn open(path: &str, config: Option<&Configuration>) -> Option<Manager> {
        if path.is_empty() {
            return None;
        }

        // Discover toolchains
        let mut toolchains_paths = vec![format!("{path}/Toolchains")];
        if let Some(cfg) = config {
            toolchains_paths.extend(cfg.extra_toolchains_paths.clone());
        }

        let mut toolchains = Vec::new();
        for tc_path in &toolchains_paths {
            if let Ok(entries) = fs::read_dir(tc_path) {
                for entry in entries.flatten() {
                    let fname = entry.file_name().to_string_lossy().to_string();
                    if fname.ends_with(".xctoolchain") {
                        let full_path = format!("{tc_path}/{fname}");
                        let resolved = resolve_path(&full_path);
                        if let Some(tc) = Toolchain::open(&resolved) {
                            toolchains.push(tc);
                        }
                    }
                }
            }
        }

        // Discover platforms
        let mut platforms_paths = vec![format!("{path}/Platforms")];
        if let Some(cfg) = config {
            platforms_paths.extend(cfg.extra_platforms_paths.clone());
        }

        let mut platforms = Vec::new();
        for plat_path in &platforms_paths {
            if let Ok(entries) = fs::read_dir(plat_path) {
                for entry in entries.flatten() {
                    let fname = entry.file_name().to_string_lossy().to_string();
                    if fname.ends_with(".platform") {
                        let full_path = format!("{plat_path}/{fname}");
                        let resolved = resolve_path(&full_path);
                        if let Some(plat) = Platform::open(&resolved, &toolchains) {
                            platforms.push(plat);
                        }
                    }
                }
            }
        }
        platforms.sort_by(|a, b| a.description.cmp(&b.description));

        Some(Manager {
            path: path.to_string(),
            toolchains,
            platforms,
        })
    }

    /// Find a target (SDK) by name or path.
    pub fn find_target(&self, name: &str) -> Option<(&Platform, &Target)> {
        let resolved = resolve_path(name);
        for platform in &self.platforms {
            for target in &platform.targets {
                if target.canonical_name.as_deref() == Some(name)
                    || target.path == resolved
                {
                    return Some((platform, target));
                }
            }
            // If platform name matches, use last target
            if platform.name == name || platform.path == resolved {
                if let Some(target) = platform.targets.last() {
                    return Some((platform, target));
                }
            }
        }
        None
    }

    /// Find a toolchain by name, identifier, or path.
    pub fn find_toolchain(&self, name: &str) -> Option<&Toolchain> {
        let resolved = resolve_path(name);
        self.toolchains.iter().find(|tc| {
            tc.name == name
                || tc.identifier.as_deref() == Some(name)
                || tc.path == resolved
        })
    }

    /// Base executable paths from the developer root.
    pub fn executable_paths(&self) -> Vec<String> {
        vec![
            format!("{}/usr/bin", self.path),
            format!("{}/usr/local/bin", self.path),
            format!("{}/Tools", self.path),
        ]
    }

    /// Collect all executable search paths for tool discovery.
    pub fn all_executable_paths(
        &self,
        platform: Option<&Platform>,
        target: Option<&Target>,
        toolchains: &[&Toolchain],
    ) -> Vec<String> {
        let mut paths = Vec::new();

        // Target paths (currently empty in C++ but kept for compatibility)
        let _ = target;

        // Platform paths
        if let Some(plat) = platform {
            paths.extend(plat.executable_paths());
        }

        // Toolchain paths
        for tc in toolchains {
            paths.extend(tc.executable_paths());
        }

        // Manager paths
        paths.extend(self.executable_paths());

        paths
    }
}

// --- Utility functions ---

fn get_string(dict: &plist::Dictionary, key: &str) -> Option<String> {
    match dict.get(key) {
        Some(Value::String(s)) => Some(s.clone()),
        _ => None,
    }
}

fn string_array_from_dict(dict: &plist::Dictionary, key: &str) -> Vec<String> {
    match dict.get(key) {
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn resolve_path(path: &str) -> String {
    fs::canonicalize(path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string())
}

fn home_dir() -> Option<String> {
    std::env::var("HOME").ok()
}

/// Find an executable in the given search paths.
pub fn find_executable(name: &str, search_paths: &[String]) -> Option<PathBuf> {
    for dir in search_paths {
        let candidate = PathBuf::from(dir).join(name);
        if candidate.is_file() {
            // Check if executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(meta) = fs::metadata(&candidate) {
                    if meta.permissions().mode() & 0o111 != 0 {
                        return Some(candidate);
                    }
                }
            }
            #[cfg(not(unix))]
            {
                return Some(candidate);
            }
        }
    }
    None
}
