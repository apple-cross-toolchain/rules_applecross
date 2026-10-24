//! Image sets of PNG images: each image a `dmp2` rendition of its scale
//! (docs/clean-room/actool.md).

use crate::car::{self, Facet, Rendition};
use crate::{dmp2, image, read_json};

/// The renditions and facets of a catalog's image sets.
#[derive(Default)]
pub struct Compiled {
    pub facets: Vec<Facet>,
    pub renditions: Vec<Rendition>,
}

impl Compiled {
    pub fn is_empty(&self) -> bool {
        self.facets.is_empty()
    }
}

/// Compiles image sets, by name, for iOS: universal PNG images at 1x, 2x or
/// 3x (or none, meaning 1x) with an optional template rendering intent.
/// Anything else is refused, as are images small enough that Xcode would
/// pack several into an atlas.
pub fn compile(sets: &[(String, std::path::PathBuf)], identifier: impl Fn(&str) -> u16) -> Result<Compiled, String> {
    let mut out = Compiled::default();
    let mut packable_per_scale = std::collections::BTreeMap::<u32, usize>::new();
    for (name, dir) in sets {
        let contents = read_json(&dir.join("Contents.json"))?;
        for key in contents.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()).unwrap_or_default() {
            if !["images", "info", "properties"].contains(&key.as_str()) {
                return Err(format!("{}: the {key} key of an image set is not supported by this actool", dir.display()));
            }
        }
        // CSI flags: the rendering intent, automatic by default.
        let mut flags = 16;
        if let Some(props) = contents["properties"].as_object() {
            for (k, v) in props {
                match (k.as_str(), v.as_str()) {
                    ("template-rendering-intent", Some("template")) => flags = 8,
                    ("template-rendering-intent", Some("original")) => flags = 0,
                    _ => return Err(format!("{}: the image set property {k} = {v} is not supported by this actool", dir.display())),
                }
            }
        }
        let id = identifier(name);
        let mut scales = Vec::new();
        for entry in contents["images"].as_array().ok_or_else(|| format!("{}: an image set without an images list", dir.display()))? {
            let Some(filename) = entry["filename"].as_str() else { continue };
            for k in entry.as_object().unwrap().keys() {
                if !["filename", "idiom", "scale"].contains(&k.as_str()) {
                    return Err(format!("{}: the {k} key of an image is not supported by this actool", dir.display()));
                }
            }
            if entry["idiom"].as_str() != Some("universal") {
                return Err(format!("{}: images for idioms other than universal are not supported by this actool", dir.display()));
            }
            if !filename.to_ascii_lowercase().ends_with(".png") {
                return Err(format!("{}/{filename}: image sets of images other than PNG are not supported by this actool", dir.display()));
            }
            let scale = match entry["scale"].as_str() {
                None => 1,
                Some(s) => s.strip_suffix('x').and_then(|n| n.parse::<u32>().ok()).filter(|n| (1..=3).contains(n)).ok_or_else(|| format!("{}: scale {s} is not supported by this actool", dir.display()))?,
            };
            if scales.contains(&scale) {
                return Err(format!("{}: two images at scale {scale}x", dir.display()));
            }
            scales.push(scale);
            let path = dir.join(filename);
            let img = image::decode_png(&std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?)?;
            let (w, h) = (img.width, img.height);
            if w * h <= crate::stack::PACKING_AREA * scale as usize {
                *packable_per_scale.entry(scale).or_default() += 1;
            }
            let bgra = img.premultiplied_bgra();
            // dmp2's opaque flag: by the pixels, not the PNG's alpha channel.
            let opaque = bgra.chunks_exact(4).all(|p| p[3] == 255);
            let payload = dmp2::encode(w, h, &bgra, opaque).map_err(|e| format!("{}: {e}", path.display()))?;
            out.renditions.push(Rendition {
                attrs: vec![(car::ATTR_SCALE, scale as u16), (car::ATTR_IDENTIFIER, id), (car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_IMAGE)],
                csi: car::dmp2_image_csi_flags(filename, w as u32, h as u32, scale, flags, &payload),
            });
        }
        if !scales.is_empty() {
            out.facets.push(Facet { name: name.clone(), attrs: vec![(car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_IMAGE), (car::ATTR_IDENTIFIER, id)] });
        }
    }
    if let Some((scale, _)) = packable_per_scale.iter().find(|(_, n)| **n > 1) {
        return Err(format!("image sets with several images small enough to pack into a {scale}x atlas are not supported by this actool"));
    }
    Ok(out)
}
