use std::fs;
use std::path::{Path, PathBuf};

/// A build setting: name = value.
#[derive(Debug, Clone)]
pub struct Setting {
    pub name: String,
    pub value: String,
}

impl Setting {
    /// Parse a "KEY = VALUE" line.
    pub fn parse(line: &str) -> Option<Setting> {
        let eq = line.find('=')?;
        let name = line[..eq].trim().to_string();
        let value = line[eq + 1..].trim().to_string();
        if name.is_empty() {
            return None;
        }
        Some(Setting { name, value })
    }
}

/// An entry in an xcconfig file.
#[derive(Debug, Clone)]
pub enum ConfigEntry {
    Setting(Setting),
    Include {
        path: String,
        config: Box<Config>,
    },
}

/// A parsed xcconfig file.
#[derive(Debug, Clone)]
pub struct Config {
    pub path: String,
    pub entries: Vec<ConfigEntry>,
}

impl Config {
    /// Load and parse an xcconfig file, resolving #include directives.
    pub fn load(path: &str) -> Option<Config> {
        let data = fs::read_to_string(path).ok()?;
        let directory = Path::new(path)
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_string_lossy()
            .to_string();

        let mut entries = Vec::new();
        let mut comment = false;
        let mut slash = false;
        let mut line = String::new();

        let data = if data.ends_with('\n') {
            data
        } else {
            format!("{data}\n")
        };

        for c in data.chars() {
            if c == '/' {
                if slash {
                    if !comment {
                        line.pop(); // remove first slash
                        comment = true;
                    }
                } else {
                    slash = true;
                }
            } else if slash {
                slash = false;
            }

            if c == '\r' || c == '\n' {
                comment = false;
                let trimmed = line.trim().to_string();

                if !trimmed.is_empty() {
                    if trimmed.starts_with('#') {
                        if let Some(entry) = parse_directive(&trimmed, &directory) {
                            entries.push(entry);
                        }
                    } else {
                        let setting_line = if trimmed.ends_with(';') {
                            &trimmed[..trimmed.len() - 1]
                        } else {
                            &trimmed
                        };
                        if let Some(setting) = Setting::parse(setting_line) {
                            entries.push(ConfigEntry::Setting(setting));
                        }
                    }
                }

                line.clear();
            } else if !comment {
                line.push(c);
            }
        }

        Some(Config {
            path: path.to_string(),
            entries,
        })
    }

    /// Flatten all settings (including from included configs) into a single list.
    pub fn all_settings(&self) -> Vec<Setting> {
        let mut settings = Vec::new();
        for entry in &self.entries {
            match entry {
                ConfigEntry::Setting(s) => settings.push(s.clone()),
                ConfigEntry::Include { config, .. } => {
                    settings.extend(config.all_settings());
                }
            }
        }
        settings
    }
}

fn parse_include_path(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.len() < 2 || !trimmed.starts_with('"') || !trimmed.ends_with('"') {
        return None;
    }
    let path = &trimmed[1..trimmed.len() - 1];

    let dev_prefix = "<DEVELOPER_DIR>";
    if path.starts_with(dev_prefix) {
        if let Ok(dev_dir) = std::env::var("DEVELOPER_DIR") {
            return Some(format!("{dev_dir}{}", &path[dev_prefix.len()..]));
        }
    }

    Some(path.to_string())
}

fn parse_directive(line: &str, directory: &str) -> Option<ConfigEntry> {
    let rest = &line[1..]; // skip '#'
    let rest = rest.trim();

    if rest.starts_with("include") {
        let value = &rest["include".len()..];
        if let Some(include_path) = parse_include_path(value) {
            let resolved = if Path::new(&include_path).is_absolute() {
                include_path.clone()
            } else {
                PathBuf::from(directory)
                    .join(&include_path)
                    .to_string_lossy()
                    .to_string()
            };

            if let Some(config) = Config::load(&resolved) {
                return Some(ConfigEntry::Include {
                    path: include_path,
                    config: Box::new(config),
                });
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_setting() {
        let s = Setting::parse("FOO = bar").unwrap();
        assert_eq!(s.name, "FOO");
        assert_eq!(s.value, "bar");
    }

    #[test]
    fn test_parse_setting_no_spaces() {
        let s = Setting::parse("FOO=bar").unwrap();
        assert_eq!(s.name, "FOO");
        assert_eq!(s.value, "bar");
    }
}
