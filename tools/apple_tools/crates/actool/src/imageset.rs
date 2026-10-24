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
/// Small images are packed into an atlas per scale; anything else is
/// refused.
pub fn compile(sets: &[(String, std::path::PathBuf)], identifier: impl Fn(&str) -> u16) -> Result<Compiled, String> {
    let mut out = Compiled::default();
    let mut images: Vec<SetImage> = Vec::new();
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
            // Xcode stores an image whose every pixel is grey (R = G = B,
            // whatever the PNG's colour type) as GA8, not reproduced here.
            if img.data.chunks_exact(4).all(|p| p[0] == p[1] && p[1] == p[2]) {
                return Err(format!("{}: greyscale images (stored by Xcode as GA8) are not supported by this actool", path.display()));
            }
            images.push(SetImage { scale, identifier: id, flags, filename: filename.to_string(), image: img });
        }
        if !scales.is_empty() {
            out.facets.push(Facet { name: name.clone(), attrs: vec![(car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_IMAGE), (car::ATTR_IDENTIFIER, id)] });
        }
    }
    // Per scale, translucent images and then opaque ones (whose atlas name
    // says 1), each largest area first (equal areas by identifier): images over 18,050 px² per scale
    // factor, or a group's only packable one, stored on their own, the rest
    // packed into an atlas, as app icons are. An atlas's dimension1 is its
    // index at its scale.
    let mut scales: Vec<u32> = images.iter().map(|i| i.scale).collect();
    scales.sort();
    scales.dedup();
    for scale in scales {
        let mut atlas_index = 0;
        for opaque in [false, true] {
            let mut group: Vec<&SetImage> = images.iter().filter(|i| i.scale == scale && is_opaque(i) == opaque).collect();
            // Equal areas by identifier (12 probes of equal-area shapes).
            group.sort_by_key(|i| (std::cmp::Reverse(i.image.width * i.image.height), i.identifier));
            let cap = crate::stack::PACKING_AREA * scale as usize;
            let (standalone, packable): (Vec<&SetImage>, Vec<&SetImage>) = group.into_iter().partition(|i| i.image.width * i.image.height > cap);
            let (standalone, packable) = if packable.len() < 2 { (standalone.into_iter().chain(packable).collect::<Vec<_>>(), Vec::new()) } else { (standalone, packable) };
            if !packable.is_empty() {
                if packable.windows(2).any(|w| w[0].image.width * w[0].image.height == w[1].image.width * w[1].image.height && w[0].identifier == w[1].identifier) {
                    return Err(format!("image sets with equal identifiers and areas in one {scale}x atlas are not supported by this actool"));
                }
                let total: usize = packable.iter().map(|i| i.image.width * i.image.height).sum();
                if total - packable.last().unwrap().image.width * packable.last().unwrap().image.height >= cap {
                    return Err(format!("image sets needing more than one {scale}x atlas of {} images are not supported by this actool", if opaque { "opaque" } else { "translucent" }));
                }
                pack(scale, opaque, atlas_index, &packable, &mut out.renditions)?;
                atlas_index += 1;
            }
            for i in standalone {
                out.renditions.push(standalone_rendition(i)?);
            }
        }
    }
    Ok(out)
}

struct SetImage {
    scale: u32,
    identifier: u16,
    flags: u32,
    filename: String,
    image: image::Rgba,
}

fn image_attrs(i: &SetImage) -> Vec<(u16, u16)> {
    vec![(car::ATTR_SCALE, i.scale as u16), (car::ATTR_IDENTIFIER, i.identifier), (car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_IMAGE)]
}

/// A `dmp2` payload, flagged opaque by the pixels (not the PNG's alpha).
fn payload(w: usize, h: usize, bgra: &[u8], what: &str) -> Result<Vec<u8>, String> {
    let opaque = bgra.chunks_exact(4).all(|p| p[3] == 255);
    dmp2::encode(w, h, bgra, opaque).map_err(|e| format!("{what}: {e}"))
}

fn standalone_rendition(i: &SetImage) -> Result<Rendition, String> {
    let (w, h) = (i.image.width, i.image.height);
    let data = payload(w, h, &i.image.premultiplied_bgra(), &i.filename)?;
    Ok(Rendition { attrs: image_attrs(i), csi: car::dmp2_image_csi_flags(&i.filename, w as u32, h as u32, i.scale, i.flags, &data) })
}

/// One atlas of a scale's packable images, as `dmp2`, and a link to it per
/// image.
fn is_opaque(i: &SetImage) -> bool {
    i.image.data.chunks_exact(4).all(|p| p[3] == 255)
}

fn pack(scale: u32, opaque: bool, index: u16, images: &[&SetImage], renditions: &mut Vec<Rendition>) -> Result<(), String> {
    let blocks: Vec<(u32, u32)> = images.iter().map(|i| (i.image.width as u32, i.image.height as u32)).collect();
    let (positions, (rw, rh)) = crate::iconset::pack::pack(&blocks);
    let (aw, ah) = ((rw + 1).div_ceil(2) * 2, (rh + 1).div_ceil(2) * 2);
    let stride = aw as usize * 4;
    let mut pixels = vec![0u8; stride * ah as usize];
    let mut key = vec![(car::ATTR_ELEMENT, car::ELEMENT_PACKED), (car::ATTR_PART, car::PART_PACKED), (car::ATTR_SCALE, scale as u16)];
    if index > 0 {
        key.push((car::ATTR_DIMENSION1, index));
    }
    for (i, (x, y)) in images.iter().zip(&positions) {
        let (w, h) = (i.image.width as u32, i.image.height as u32);
        crate::iconset::place(&mut pixels, stride, ah, &i.image.premultiplied_bgra(), *x, *y, w, h);
        renditions.push(Rendition { attrs: image_attrs(i), csi: car::link_csi_flags(&i.filename, (*x, *y, w, h), scale, i.flags, &key) });
    }
    let name = format!("ZZZZPackedAsset-{scale}.{}.0-gamut0", opaque as u8);
    // Opaque as its group is, its transparent padding notwithstanding.
    let data = dmp2::encode(aw as usize, ah as usize, &pixels, opaque).map_err(|e| format!("{name}: {e}"))?;
    renditions.push(Rendition { attrs: key, csi: car::packed_dmp2_csi(&name, aw, ah, scale, &data) });
    Ok(())
}
