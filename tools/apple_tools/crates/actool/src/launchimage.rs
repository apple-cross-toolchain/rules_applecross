//! Launch image sets (docs/clean-room/actool.md, "Launch images"): the one
//! `--launch-image` names is copied into the bundle as PNGs and listed in
//! the partial Info.plist's `UILaunchImages`; the rest compile to nothing.

use cf::Value;
use std::path::{Path, PathBuf};

/// What a launch image set gives the bundle.
pub struct Compiled {
    /// Each image's name in the bundle and its source, copied as it is.
    pub files: Vec<(String, PathBuf)>,
    /// An image Xcode makes instead, with its name in the bundle.
    pub made: Option<(String, Vec<u8>)>,
    /// `UILaunchImages`: one entry per image name, in the set's order.
    pub entries: Vec<Value>,
}

pub fn compile(name: &str, dir: &Path, phone: bool, pad: bool) -> Result<Compiled, String> {
    let contents = crate::read_json(&dir.join("Contents.json"))?;
    for (k, _) in contents.as_object().into_iter().flatten() {
        if !matches!(k.as_str(), "images" | "info") {
            return Err(format!("{}: the {k} key of a launch image set is not supported by this actool", dir.display()));
        }
    }
    let images = contents["images"].as_array().ok_or_else(|| format!("{}: a launch image set without an images list", dir.display()))?;
    let mut files: Vec<(String, PathBuf)> = Vec::new();
    let mut entries: Vec<(String, Value)> = Vec::new();
    for image in images {
        let field = |key: &str| image[key].as_str();
        for (k, _) in image.as_object().into_iter().flatten() {
            if !matches!(k.as_str(), "idiom" | "filename" | "minimum-system-version" | "orientation" | "scale" | "extent" | "subtype" | "size") {
                return Err(format!("{}: the {k} key of a launch image is not supported by this actool", dir.display()));
            }
        }
        // A slot without an image is Interface Builder's alone.
        let Some(filename) = field("filename") else { continue };
        if !matches!(field("extent"), None | Some("full-screen")) {
            return Err(format!("{}: launch images other than full-screen ones are not supported by this actool", dir.display()));
        }
        let version = match field("minimum-system-version") {
            Some("7.0") => "700",
            Some("8.0") => "800",
            other => return Err(format!("{}: a launch image for iOS {} is not supported by this actool", dir.display(), other.unwrap_or("before 7"))),
        };
        let orientation = match field("orientation") {
            Some("portrait") => "Portrait",
            Some("landscape") => "Landscape",
            _ => return Err(format!("{}: a launch image's orientation must be portrait or landscape for this actool", dir.display())),
        };
        let scale = match field("scale") {
            Some("1x") => "",
            Some("2x") => "@2x",
            Some("3x") => "@3x",
            _ => return Err(format!("{}: a launch image's scale must be 1x, 2x or 3x for this actool", dir.display())),
        };
        // The image's name: the set's, the system version without its dot,
        // then what tells the image apart; its size is the portrait one.
        let (suffix, size, ipad, targeted) = match (field("idiom"), field("subtype"), orientation) {
            (Some("iphone"), None, "Portrait") => (String::new(), (320, 480), false, phone),
            (Some("iphone"), Some("retina4"), "Portrait") => ("-568h".into(), (320, 568), false, phone),
            (Some("iphone"), Some("667h"), "Portrait") => ("-667h".into(), (375, 667), false, phone),
            (Some("iphone"), Some("736h"), o) => (format!("-{o}-736h"), (414, 736), false, phone),
            (Some("ipad"), None, o) => (format!("-{o}"), (768, 1024), true, pad),
            _ => return Err(format!("{}: this launch image's idiom, subtype and orientation are not supported by this actool", dir.display())),
        };
        // Images for devices the build doesn't target are left out.
        if !targeted {
            continue;
        }
        if !filename.to_ascii_lowercase().ends_with(".png") {
            return Err(format!("{}: launch images other than PNGs are not supported by this actool", dir.display()));
        }
        let base = format!("{name}-{version}{suffix}");
        let file = format!("{base}{scale}{}.png", if ipad { "~ipad" } else { "" });
        if files.iter().any(|(f, _)| *f == file) {
            return Err(format!("{}: two launch images named {file}", dir.display()));
        }
        files.push((file, dir.join(filename)));
        if !entries.iter().any(|(b, _)| *b == base) {
            let entry = Value::dict([
                ("UILaunchImageMinimumOSVersion", Value::string(field("minimum-system-version").unwrap_or_default())),
                ("UILaunchImageName", Value::string(base.clone())),
                ("UILaunchImageOrientation", Value::string(orientation)),
                ("UILaunchImageSize", Value::string(format!("{{{}, {}}}", size.0, size.1))),
            ]);
            entries.push((base, entry));
        }
    }
    // A set without a single image gives an iPhone build a black 4-inch one
    // of its own.
    let mut made = None;
    if phone && !images.iter().any(|i| i["filename"].is_string()) {
        let base = format!("{name}-700-568h");
        let black = crate::image::Rgba { width: 640, height: 1136, data: [0, 0, 0, 255].repeat(640 * 1136) };
        made = Some((format!("{base}@2x.png"), crate::image::plain_png(&black, true)));
        entries.push((base.clone(), Value::dict([
            ("UILaunchImageMinimumOSVersion", Value::string("7.0")),
            ("UILaunchImageName", Value::string(base)),
            ("UILaunchImageOrientation", Value::string("Portrait")),
            ("UILaunchImageSize", Value::string("{320, 568}")),
        ])));
    }
    Ok(Compiled { files, made, entries: entries.into_iter().map(|(_, e)| e).collect() })
}
