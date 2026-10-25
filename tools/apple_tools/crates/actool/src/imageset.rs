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
        // CSI flags: the rendering intent, automatic by default; Xcode also
        // reads it from `info` (WebRTC's web_view shell icons).
        let mut flags = match contents["info"]["template-rendering-intent"].as_str() {
            None => 16,
            Some("template") => 8,
            Some("original") => 0,
            Some(v) => return Err(format!("{}: the image set info template-rendering-intent = {v} is not supported by this actool", dir.display())),
        };
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
            let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let (img, extended) = image::decode_png_extended(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
            if extended.is_some() && img.is_gray() {
                return Err(format!("{}: a grey image with colours outside sRGB is not supported by this actool", path.display()));
            }
            images.push(SetImage { scale, identifier: id, flags, filename: filename.to_string(), image: img, extended });
        }
        if !scales.is_empty() {
            out.facets.push(Facet { name: name.clone(), attrs: vec![(car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_IMAGE), (car::ATTR_IDENTIFIER, id)] });
        }
    }
    // Per scale, translucent colour, translucent grey, opaque colour and
    // opaque grey images (an atlas named for its opacity and greyness), then
    // the 16-bit renditions of colour images from wide-gamut profiles in
    // atlases of their own (gamut 1, indexed apart),
    // each largest area first (equal areas by identifier): images over 18,050 px² per scale
    // factor, or a group's only packable one, stored on their own, the rest
    // packed into an atlas, as app icons are. An atlas's dimension1 is its
    // index at its scale.
    let mut scales: Vec<u32> = images.iter().map(|i| i.scale).collect();
    scales.sort();
    scales.dedup();
    for scale in scales {
        let mut planned: Vec<(usize, bool, bool, Vec<&SetImage>)> = Vec::new();
        for (gamut, opaque, gray) in [(0, false, false), (0, false, true), (0, true, false), (0, true, true), (1, false, false), (1, true, false)] {
            let mut group: Vec<&SetImage> =
                images.iter().filter(|i| i.scale == scale && is_opaque(i) == opaque && is_gray(i) == gray && (gamut == 0 || i.extended.is_some())).collect();
            // Equal areas by identifier (12 probes of equal-area shapes).
            group.sort_by_key(|i| (std::cmp::Reverse(i.image.width * i.image.height), i.identifier));
            let cap = crate::stack::PACKING_AREA * scale as usize;
            let (standalone, packable): (Vec<&SetImage>, Vec<&SetImage>) = group.into_iter().partition(|i| i.image.width * i.image.height > cap);
            let (standalone, packable) = if packable.len() < 2 { (standalone.into_iter().chain(packable).collect::<Vec<_>>(), Vec::new()) } else { (standalone, packable) };
            if !packable.is_empty() {
                if packable.windows(2).any(|w| w[0].image.width * w[0].image.height == w[1].image.width * w[1].image.height && w[0].identifier == w[1].identifier) {
                    return Err(format!("image sets with equal identifiers and areas in one {scale}x atlas are not supported by this actool"));
                }
                // Next-fit atlases: one is closed once its area reaches the
                // cap, the next taking the following index at this scale.
                let mut total = 0;
                for i in packable {
                    if planned.last().is_none_or(|a| (a.0, a.1, a.2) != (gamut, opaque, gray)) || total >= cap {
                        planned.push((gamut, opaque, gray, Vec::new()));
                        total = 0;
                    }
                    total += i.image.width * i.image.height;
                    planned.last_mut().unwrap().3.push(i);
                }
            }
            for i in standalone {
                out.renditions.push(if gamut == 0 { standalone_rendition(i)? } else { standalone16(i)? });
            }
        }
        // A scale's atlases (per gamut) take their dimension1 in the order
        // of their indices as decimal strings: the 11th of 19 is 2, the
        // 3rd is 11 (a catalog of 19, all else identical).
        for gamut in [0, 1] {
            let atlases: Vec<&(usize, bool, bool, Vec<&SetImage>)> = planned.iter().filter(|a| a.0 == gamut).collect();
            let mut names: Vec<String> = (0..atlases.len()).map(|k| k.to_string()).collect();
            names.sort();
            for (k, (_, opaque, gray, atlas)) in atlases.into_iter().enumerate() {
                let index = names.iter().position(|n| *n == k.to_string()).unwrap() as u16;
                if gamut == 0 {
                    pack(scale, *opaque, *gray, index, atlas, &mut out.renditions)?;
                } else {
                    pack16(scale, *opaque, index, atlas, &mut out.renditions)?;
                }
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
    /// The 16-bit rendition's pixels, for a wide-gamut profile's colours
    /// outside sRGB: premultiplied half floats, RGBA.
    extended: Option<Vec<u16>>,
}

fn image_attrs(i: &SetImage) -> Vec<(u16, u16)> {
    vec![(car::ATTR_SCALE, i.scale as u16), (car::ATTR_IDENTIFIER, i.identifier), (car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_IMAGE)]
}

/// The attributes of an image's 16-bit rendition: gamut 1.
fn image_attrs16(i: &SetImage) -> Vec<(u16, u16)> {
    let mut attrs = image_attrs(i);
    attrs.push((car::ATTR_GAMUT, 1));
    attrs
}

/// An image's 16-bit rendition on its own.
fn standalone16(i: &SetImage) -> Result<Rendition, String> {
    let (w, h) = (i.image.width, i.image.height);
    let halves = i.extended.as_ref().expect("a 16-bit rendition");
    let alpha: Vec<u8> = i.image.data.chunks_exact(4).map(|p| p[3]).collect();
    let opaque = is_opaque(i);
    let data = dmp2::encode16(w, h, halves, &alpha, opaque).map_err(|e| format!("{}: {e}", i.filename))?;
    Ok(Rendition { attrs: image_attrs16(i), csi: car::dmp2_pixels_csi(&i.filename, w as u32, h as u32, i.scale, i.flags, car::Pixels::Half, opaque, &data) })
}

/// A `dmp2` payload, flagged opaque by the pixels (not the PNG's alpha).
fn payload(w: usize, h: usize, bgra: &[u8], what: &str) -> Result<Vec<u8>, String> {
    let opaque = bgra.chunks_exact(4).all(|p| p[3] == 255);
    dmp2::encode(w, h, bgra, opaque).map_err(|e| format!("{what}: {e}"))
}

fn is_gray(i: &SetImage) -> bool {
    i.image.is_gray()
}

fn standalone_rendition(i: &SetImage) -> Result<Rendition, String> {
    let (w, h) = (i.image.width, i.image.height);
    if is_gray(i) {
        let bgra = i.image.premultiplied_bgra();
        let ga: Vec<u8> = bgra.chunks_exact(4).flat_map(|p| [p[0], p[3]]).collect();
        let opaque = is_opaque(i);
        let data = dmp2::encode_gray(w, h, &ga, opaque).map_err(|e| format!("{}: {e}", i.filename))?;
        return Ok(Rendition { attrs: image_attrs(i), csi: car::dmp2_pixels_csi(&i.filename, w as u32, h as u32, i.scale, i.flags, car::Pixels::Gray, opaque, &data) });
    }
    let data = payload(w, h, &i.image.premultiplied_bgra(), &i.filename)?;
    Ok(Rendition { attrs: image_attrs(i), csi: car::dmp2_image_csi_flags(&i.filename, w as u32, h as u32, i.scale, i.flags, &data) })
}

/// One atlas of a scale's packable images, as `dmp2`, and a link to it per
/// image.
fn is_opaque(i: &SetImage) -> bool {
    i.image.data.chunks_exact(4).all(|p| p[3] == 255)
}

#[allow(clippy::too_many_arguments)]
fn pack(scale: u32, opaque: bool, gray: bool, index: u16, images: &[&SetImage], renditions: &mut Vec<Rendition>) -> Result<(), String> {
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
        let pixels = if gray { car::Pixels::Gray } else { car::Pixels::Bgra };
        renditions.push(Rendition { attrs: image_attrs(i), csi: car::link_csi_format(&i.filename, (*x, *y, w, h), scale, i.flags, pixels, &key) });
    }
    let name = format!("ZZZZPackedAsset-{scale}.{}.{}-gamut0", opaque as u8, gray as u8);
    // Opaque as its group is, its transparent padding notwithstanding.
    let data = if gray {
        // An opaque grey atlas is opaque throughout, its padding too.
        let ga: Vec<u8> = pixels.chunks_exact(4).flat_map(|p| [p[0], if opaque { 255 } else { p[3] }]).collect();
        dmp2::encode_gray(aw as usize, ah as usize, &ga, opaque)
    } else {
        dmp2::encode(aw as usize, ah as usize, &pixels, opaque)
    }
    .map_err(|e| format!("{name}: {e}"))?;
    let pixels = if gray { car::Pixels::Gray } else { car::Pixels::Bgra };
    renditions.push(Rendition { attrs: key, csi: car::packed_dmp2_csi(&name, aw, ah, scale, pixels, opaque, &data) });
    Ok(())
}

/// One atlas of a scale's packable 16-bit renditions (gamut 1), packed as
/// their 8-bit renditions are.
fn pack16(scale: u32, opaque: bool, index: u16, images: &[&SetImage], renditions: &mut Vec<Rendition>) -> Result<(), String> {
    let blocks: Vec<(u32, u32)> = images.iter().map(|i| (i.image.width as u32, i.image.height as u32)).collect();
    let (positions, (rw, rh)) = crate::iconset::pack::pack(&blocks);
    let (aw, ah) = ((rw + 1).div_ceil(2) * 2, (rh + 1).div_ceil(2) * 2);
    let mut halves = vec![0u16; 4 * (aw * ah) as usize];
    let mut alpha = vec![0u8; (aw * ah) as usize];
    let mut key = vec![(car::ATTR_ELEMENT, car::ELEMENT_PACKED), (car::ATTR_PART, car::PART_PACKED), (car::ATTR_SCALE, scale as u16), (car::ATTR_GAMUT, 1)];
    if index > 0 {
        key.push((car::ATTR_DIMENSION1, index));
    }
    for (i, (x, y)) in images.iter().zip(&positions) {
        let (w, h) = (i.image.width, i.image.height);
        let source = i.extended.as_ref().expect("a 16-bit rendition");
        let source_alpha: Vec<u8> = i.image.data.chunks_exact(4).map(|p| p[3]).collect();
        crate::iconset::place_n(&mut halves, aw as usize, 4, ah, source, *x, *y, w as u32, h as u32);
        crate::iconset::place_n(&mut alpha, aw as usize, 1, ah, &source_alpha, *x, *y, w as u32, h as u32);
        renditions.push(Rendition {
            attrs: image_attrs16(i),
            csi: car::link_csi_format(&i.filename, (*x, *y, w as u32, h as u32), scale, i.flags, car::Pixels::Half, &key),
        });
    }
    let name = format!("ZZZZPackedAsset-{scale}.{}.0-gamut1", opaque as u8);
    let data = dmp2::encode16(aw as usize, ah as usize, &halves, &alpha, opaque).map_err(|e| format!("{name}: {e}"))?;
    renditions.push(Rendition { attrs: key, csi: car::packed_dmp2_csi(&name, aw, ah, scale, car::Pixels::Half, opaque, &data) });
    Ok(())
}
