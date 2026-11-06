use serde::Deserialize;
use std::fs;
use std::path::Path;

/// The type of an asset in the catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetType {
    Catalog,
    Group,
    ImageSet,
    AppIconSet,
    DataSet,
    ColorSet,
    LaunchImage,
    BrandAssets,
    SpriteAtlas,
    ComplicationSet,
    IconSet,
    StickerSequence,
    StickerPack,
    Sticker,
    CubeTextureSet,
    TextureSet,
    ARReferenceObject,
    ARResourceGroup,
    SymbolSet,
    Unknown(String),
}

impl AssetType {
    /// Determine asset type from directory extension.
    pub fn from_extension(ext: &str) -> Self {
        match ext {
            "xcassets" => AssetType::Catalog,
            "group" => AssetType::Group,
            "imageset" => AssetType::ImageSet,
            "appiconset" => AssetType::AppIconSet,
            "dataset" => AssetType::DataSet,
            "colorset" => AssetType::ColorSet,
            "launchimage" => AssetType::LaunchImage,
            "brandassets" => AssetType::BrandAssets,
            "spriteatlas" => AssetType::SpriteAtlas,
            "complicationset" => AssetType::ComplicationSet,
            "iconset" => AssetType::IconSet,
            "stickersequence" => AssetType::StickerSequence,
            "stickerpack" => AssetType::StickerPack,
            "sticker" => AssetType::Sticker,
            "cubetextureset" => AssetType::CubeTextureSet,
            "textureset" => AssetType::TextureSet,
            "arreferenceobject" => AssetType::ARReferenceObject,
            "arresourcegroup" => AssetType::ARResourceGroup,
            "symbolset" => AssetType::SymbolSet,
            other => AssetType::Unknown(other.to_string()),
        }
    }
}

/// Contents.json image entry.
#[derive(Debug, Clone, Deserialize)]
pub struct ImageEntry {
    pub filename: Option<String>,
    pub idiom: Option<String>,
    pub scale: Option<String>,
    pub size: Option<String>,
    pub role: Option<String>,
    pub subtype: Option<String>,
    pub appearances: Option<Vec<serde_json::Value>>,
}

/// Contents.json data entry.
#[derive(Debug, Clone, Deserialize)]
pub struct DataEntry {
    pub filename: Option<String>,
    pub idiom: Option<String>,
    #[serde(rename = "universal-type-identifier")]
    pub uti: Option<String>,
}

/// Contents.json color entry.
#[derive(Debug, Clone, Deserialize)]
pub struct ColorEntry {
    pub idiom: Option<String>,
    pub color: Option<serde_json::Value>,
    pub appearances: Option<Vec<serde_json::Value>>,
}

/// Contents.json info section.
#[derive(Debug, Clone, Deserialize)]
pub struct ContentsInfo {
    pub version: Option<u32>,
    pub author: Option<String>,
}

/// Contents.json properties section.
#[derive(Debug, Clone, Deserialize)]
pub struct ContentsProperties {
    #[serde(rename = "provides-namespace")]
    pub provides_namespace: Option<bool>,
    #[serde(rename = "on-demand-resource-tags")]
    pub on_demand_resource_tags: Option<Vec<String>>,
    #[serde(rename = "pre-rendered")]
    pub pre_rendered: Option<bool>,
}

/// Parsed Contents.json.
#[derive(Debug, Clone, Deserialize)]
pub struct Contents {
    pub info: Option<ContentsInfo>,
    pub images: Option<Vec<ImageEntry>>,
    pub data: Option<Vec<DataEntry>>,
    pub colors: Option<Vec<ColorEntry>>,
    pub properties: Option<ContentsProperties>,
}

/// An asset in the catalog hierarchy.
#[derive(Debug, Clone)]
pub struct Asset {
    pub name: String,
    pub path: String,
    pub asset_type: AssetType,
    pub contents: Option<Contents>,
    pub children: Vec<Asset>,
}

impl Asset {
    /// Load an asset catalog or asset directory recursively.
    pub fn load(path: &str) -> Option<Asset> {
        let p = Path::new(path);
        if !p.is_dir() {
            return None;
        }

        let ext = p
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();
        let asset_type = AssetType::from_extension(&ext);
        let name = p
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        // Try to read Contents.json
        let contents_path = p.join("Contents.json");
        let contents = if contents_path.is_file() {
            fs::read_to_string(&contents_path)
                .ok()
                .and_then(|s| serde_json::from_str::<Contents>(&s).ok())
        } else {
            None
        };

        // Load children for container types
        let children = if is_container(&asset_type) {
            load_children(p)
        } else {
            Vec::new()
        };

        Some(Asset {
            name,
            path: path.to_string(),
            asset_type,
            contents,
            children,
        })
    }
}

fn is_container(asset_type: &AssetType) -> bool {
    matches!(
        asset_type,
        AssetType::Catalog
            | AssetType::Group
            | AssetType::BrandAssets
            | AssetType::SpriteAtlas
            | AssetType::ComplicationSet
            | AssetType::StickerPack
            | AssetType::ARResourceGroup
    )
}

fn load_children(dir: &Path) -> Vec<Asset> {
    let mut children = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let p = entry.path();
            if !p.is_dir() {
                continue;
            }
            let ext = p
                .extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_default();

            // Only load directories that have a known asset extension
            let asset_type = AssetType::from_extension(&ext);
            if !matches!(asset_type, AssetType::Unknown(_)) {
                if let Some(asset) = Asset::load(&p.to_string_lossy()) {
                    children.push(asset);
                }
            }
        }
    }
    children
}

/// Recursively dump an asset catalog.
pub fn dump_asset(asset: &Asset, indent: usize) {
    let prefix = "  ".repeat(indent);
    println!("{prefix}name: {}", asset.name);
    println!("{prefix}type: {:?}", asset.asset_type);
    println!("{prefix}path: {}", asset.path);

    if let Some(contents) = &asset.contents {
        if let Some(info) = &contents.info {
            if let Some(author) = &info.author {
                println!("{prefix}author: {author}");
            }
            if let Some(version) = info.version {
                println!("{prefix}version: {version}");
            }
        }

        if let Some(props) = &contents.properties {
            if let Some(ns) = props.provides_namespace {
                println!("{prefix}provides namespace: {ns}");
            }
            if let Some(pre) = props.pre_rendered {
                println!("{prefix}pre-rendered: {pre}");
            }
            if let Some(tags) = &props.on_demand_resource_tags {
                println!("{prefix}on-demand-resource-tags: {}", tags.len());
            }
        }

        if let Some(images) = &contents.images {
            for img in images {
                println!("{prefix}  image:");
                if let Some(f) = &img.filename {
                    println!("{prefix}    file name: {f}");
                }
                if let Some(i) = &img.idiom {
                    println!("{prefix}    idiom: {i}");
                }
                if let Some(s) = &img.scale {
                    println!("{prefix}    scale: {s}");
                }
                if let Some(sz) = &img.size {
                    println!("{prefix}    size: {sz}");
                }
            }
        }

        if let Some(data) = &contents.data {
            for d in data {
                println!("{prefix}  data:");
                if let Some(f) = &d.filename {
                    println!("{prefix}    file name: {f}");
                }
                if let Some(i) = &d.idiom {
                    println!("{prefix}    idiom: {i}");
                }
            }
        }

        if let Some(colors) = &contents.colors {
            for c in colors {
                println!("{prefix}  color:");
                if let Some(i) = &c.idiom {
                    println!("{prefix}    idiom: {i}");
                }
            }
        }
    }

    for child in &asset.children {
        println!();
        dump_asset(child, indent + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_type_from_extension() {
        assert_eq!(AssetType::from_extension("xcassets"), AssetType::Catalog);
        assert_eq!(AssetType::from_extension("imageset"), AssetType::ImageSet);
        assert_eq!(
            AssetType::from_extension("appiconset"),
            AssetType::AppIconSet
        );
        assert_eq!(AssetType::from_extension("dataset"), AssetType::DataSet);
    }
}
