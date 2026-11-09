//! Image sets of PNG and JPEG images: each PNG a `dmp2` rendition of its
//! scale, each JPEG stored as it is
//! (docs/clean-room/actool.md).

use crate::car::{self, Facet, Rendition};
use crate::{dmp2, image, read_json};

/// The renditions and facets of a catalog's image sets.
#[derive(Default)]
pub struct Compiled {
    pub facets: Vec<Facet>,
    pub renditions: Vec<Rendition>,
    /// The packing groups (scale, appearance, idiom, opaque, grey) with an
    /// image small enough to pack, packed or alone.
    pub packable: Vec<(u32, u16, u16, bool, bool)>,
    /// Whether a 3x image was left out (all but iOS).
    pub dropped_3x: bool,
}

impl Compiled {
    pub fn is_empty(&self) -> bool {
        self.facets.is_empty()
    }
}

/// Compiles image sets, by name, for iOS: PNG images at 1x, 2x or 3x (or
/// none, meaning 1x), universal or for the iPhone, iPad or CarPlay (with
/// `phone_and_pad` targeted; watch, Mac and TV images left out), for any
/// appearance or one of the colour sets', with an optional template
/// rendering intent.
/// Small images are packed into an atlas per scale; anything else is
/// refused.
/// An image's scale (1 without one) and appearance.
fn slot(dir: &std::path::Path, entry: &serde_json::Value) -> Result<(u32, u16), String> {
    let scale = match entry["scale"].as_str() {
        None => 1,
        Some(s) => s.strip_suffix('x').and_then(|n| n.parse::<u32>().ok()).filter(|n| (1..=3).contains(n)).ok_or_else(|| format!("{}: scale {s} is not supported by this actool", dir.display()))?,
    };
    let appearance = crate::colorset::appearance(entry).ok_or_else(|| format!("{}: the appearances {} are not supported by this actool", dir.display(), entry["appearances"]))?;
    Ok((scale, appearance))
}

/// `mac`: for macOS, where a `mac` image takes its scale and appearance from
/// a universal one, and other idioms' are left out (eight probes).
/// `reserved`: atlases (scale, count) indexed before the catalog's, as a
/// macOS Icon Composer document's at 2x (one of the holdout's catalogs).
/// watchOS keeps only images without an appearance (two probes).
/// `relax_vector`: vector images (PDF, SVG) drawn by this actool rather
/// than refused (crate::vector).
pub fn compile(sets: &[(String, std::path::PathBuf)], phone_and_pad: bool, platform: &str, reserved: &[(u32, usize)], relax_vector: bool, identifier: impl Fn(&str) -> u16) -> Result<Compiled, String> {
    compile_with_icons(sets, phone_and_pad, platform, reserved, relax_vector, identifier, Vec::new())
}

/// [`compile`], with an app icon set's icons packed among the image sets'
/// opaque colour images (crate::iconset::compile_deferring).
#[allow(clippy::too_many_arguments)]
pub fn compile_with_icons(sets: &[(String, std::path::PathBuf)], phone_and_pad: bool, platform: &str, reserved: &[(u32, usize)], relax_vector: bool, identifier: impl Fn(&str) -> u16, icons: Vec<crate::iconset::DeferredIcon>) -> Result<Compiled, String> {
    let own = crate::colorset::own_idiom(platform);
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
        let mut preserves = false;
        if let Some(props) = contents["properties"].as_object() {
            for (k, v) in props {
                match (k.as_str(), v.as_str()) {
                    ("template-rendering-intent", Some("template")) => flags = 8,
                    ("template-rendering-intent", Some("original")) => flags = 0,
                    ("preserves-vector-representation", None) if v.is_boolean() => preserves = v.as_bool() == Some(true),
                    _ => return Err(format!("{}: the image set property {k} = {v} is not supported by this actool", dir.display())),
                }
            }
        }
        let id = identifier(name);
        let mut scales = Vec::new();
        let listed = contents["images"].as_array().map(Vec::as_slice).unwrap_or_default();
        let vector = |e: &serde_json::Value| e["filename"].as_str().is_some_and(|f| crate::vector::Kind::of(f).is_some() && dir.join(f).is_file());
        if listed.iter().any(vector) {
            if !relax_vector {
                return Err(format!("{}: vector images (PDF, SVG) are compiled only relaxed: Xcode also draws them into bitmaps with CoreGraphics, which this actool doesn't reproduce; --config=relax_actool_vector accepts its own drawing", dir.display()));
            }
            vector_set(dir, listed, platform, phone_and_pad, flags, preserves, id, &mut out, &mut images)?;
            out.facets.push(Facet { name: name.clone(), attrs: vec![(car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_IMAGE), (car::ATTR_IDENTIFIER, id)] });
            continue;
        }
        if preserves {
            return Err(format!("{}: preserves-vector-representation without vector images is not supported by this actool", dir.display()));
        }
        let present = |e: &&serde_json::Value| e["filename"].as_str().is_some_and(|f| dir.join(f).is_file());
        // Images without a scale (Any) beside images with one: Xcode reports
        // an error, which fails rules_apple's build. It reports it whatever
        // the images' idioms and appearances for iOS, macOS and tvOS (369 of
        // 369 sets of the holdouts); for watchOS and visionOS only for some,
        // not established: refused for every platform. For iOS and tvOS an
        // iPad or TV image at 3x and a vision one at 1x don't count (80
        // random sets on each, with and without appearances, where they
        // decided it; other idioms' other scales count, as before).
        let bitmaps: Vec<&serde_json::Value> = listed.iter().filter(present).collect();
        let ios_or_tv = own.is_none() || platform == "atv";
        let counts = |e: &&serde_json::Value| {
            let exempt = (e["scale"] == "3x" && (e["idiom"] == "ipad" || e["idiom"] == "tv")) || (e["scale"] == "1x" && e["idiom"] == "vision");
            e["scale"].is_string() && !(ios_or_tv && exempt)
        };
        if bitmaps.iter().any(|e| !e["scale"].is_string()) && bitmaps.iter().any(counts) {
            return Err(format!("{}: error: Image set has a child with bitmap content and the \"Any\" scale. The image set also has children with specific scales and content. Please remove the content with specific scales or change the content of the \"Any\" scale to vector content.", dir.display()));
        }
        // Left out: images at 3x but for iOS (any idiom; five probes), what
        // crate::colorset::own_leaves_out leaves out, and watchOS's and
        // visionOS's own images at 1x (one without a scale is kept; eight).
        // For iOS, iPad images at 3x and CarPlay ones at 1x, scales those
        // devices don't have (a probe of every idiom and scale against the
        // set without it, and eleven random sets).
        let left_out = |e: &serde_json::Value| {
            (own.is_some() && e["scale"] == "3x")
                || (own.is_none() && ((e["idiom"] == "ipad" && e["scale"] == "3x") || (e["idiom"] == "car" && e["scale"] == "1x")))
                || crate::colorset::own_leaves_out(platform, e)
                || (own.is_some_and(|(name, _)| ["watch", "vision"].contains(&name) && e["idiom"] == name) && e["scale"] == "1x")
        };
        // The platform's own images' slots, and whether each names its scale.
        let own_slots = match own {
            Some((name, _)) => listed.iter().filter(present).filter(|e| e["idiom"] == name && !left_out(e)).map(|e| Ok((slot(dir, e)?, e["scale"].is_string()))).collect::<Result<Vec<_>, String>>()?,
            None => Vec::new(),
        };
        // No images list is as an empty one (two probes).
        for entry in listed {
            let Some(filename) = entry["filename"].as_str() else { continue };
            // An image whose file is missing is left out, without a word
            // (one probe: rules_apple's imageset_missing_child).
            if !dir.join(filename).is_file() {
                continue;
            }
            for k in entry.as_object().unwrap().keys() {
                if !["filename", "idiom", "scale", "appearances", "resizing"].contains(&k.as_str()) {
                    return Err(format!("{}: the {k} key of an image is not supported by this actool", dir.display()));
                }
            }
            if left_out(entry) {
                out.dropped_3x |= own.is_some() && entry["scale"] == "3x";
                continue;
            }
            let here = if own.is_some() { Some(slot(dir, entry)?) } else { None };
            let idiom = match (entry["idiom"].as_str(), own) {
                (Some(n), Some((name, value))) if n == name => value,
                (Some("universal"), Some((name, _))) if own_slots.iter().any(|(s, _)| Some(*s) == here) => {
                    // One of them with a scale and one without: Xcode takes
                    // either, from run to run (one of the holdout's catalogs).
                    if own_slots.iter().any(|(s, named)| Some(*s) == here && *named != entry["scale"].is_string()) {
                        return Err(format!("{}: a {name} and a universal image for one scale and appearance, one naming the scale and one not, is not supported by this actool (Xcode takes either at random)", dir.display()));
                    }
                    continue;
                }
                (Some("universal"), _) => 0,
                (_, Some(_)) => continue,
                (Some("iphone"), None) => 1,
                (Some("ipad"), None) => 2,
                (Some("car"), None) => 4,
                // Other platforms' images, visionOS's too (seven catalogs, the
                // same without them).
                (Some("watch" | "mac" | "tv" | "vision"), None) => continue,
                _ => return Err(format!("{}: images for the idiom {} are not supported by this actool", dir.display(), entry["idiom"])),
            };
            if idiom != 0 && own.is_none() && !phone_and_pad {
                return Err(format!("{}: images for a device idiom without both --target-device iphone and ipad are not supported by this actool", dir.display()));
            }
            let lower = filename.to_ascii_lowercase();
            let jpeg = lower.ends_with(".jpg") || lower.ends_with(".jpeg");
            if !lower.ends_with(".png") && !jpeg {
                return Err(format!("{}/{filename}: image sets of images other than PNG and JPEG are not supported by this actool", dir.display()));
            }
            let scale = match entry["scale"].as_str() {
                None => 1,
                Some(s) => s.strip_suffix('x').and_then(|n| n.parse::<u32>().ok()).filter(|n| (1..=3).contains(n)).ok_or_else(|| format!("{}: scale {s} is not supported by this actool", dir.display()))?,
            };
            let appearance = crate::colorset::appearance_for(entry, platform)
                .ok_or_else(|| format!("{}: the appearances {} are not supported by this actool", dir.display(), entry["appearances"]))?;
            if platform == "watch" && appearance != 0 {
                continue;
            }
            if scales.contains(&(scale, appearance, idiom)) {
                return Err(format!("{}: two images at scale {scale}x for one appearance and idiom", dir.display()));
            }
            scales.push((scale, appearance, idiom));
            let path = dir.join(filename);
            let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            // A JPEG is stored on its own, as it is but for its comments.
            if jpeg {
                if entry.get("resizing").is_some() {
                    return Err(format!("{}: a resizable JPEG is not supported by this actool", dir.display()));
                }
                let (stored, width, height, orientation) = stored_jpeg(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
                let mut attrs = vec![(car::ATTR_SCALE, scale as u16), (car::ATTR_IDENTIFIER, id), (car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_IMAGE)];
                if appearance != 0 {
                    attrs.push((car::ATTR_APPEARANCE, appearance));
                }
                if idiom != 0 {
                    attrs.push((car::ATTR_IDIOM, idiom));
                }
                out.renditions.push(Rendition { attrs, csi: car::jpeg_csi(filename, width, height, scale, flags, orientation, &stored) });
                continue;
            }
            let (img, extended) = image::decode_png_extended(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
            if extended.is_some() && img.is_gray() {
                return Err(format!("{}: a grey image with colours outside sRGB is not supported by this actool", path.display()));
            }
            let (img, slicing) = match entry.get("resizing") {
                None => (img, None),
                Some(_) if extended.is_some() => return Err(format!("{}: a resizable image with colours outside sRGB is not supported by this actool", path.display())),
                Some(r) => {
                    let (img, slicing) = resizing(r, &img).map_err(|e| format!("{}: {e}", path.display()))?;
                    (img, Some(slicing))
                }
            };
            images.push(SetImage { scale, appearance, idiom, identifier: id, flags, filename: filename.to_string(), image: img, extended, slicing, attrs: None });
        }
        if !scales.is_empty() {
            out.facets.push(Facet { name: name.clone(), attrs: vec![(car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_IMAGE), (car::ATTR_IDENTIFIER, id)] });
        }
    }
    for icon in icons {
        images.push(SetImage { scale: icon.scale, appearance: 0, idiom: icon.idiom, identifier: icon.identifier, flags: 0, filename: icon.filename, image: icon.image, extended: None, slicing: None, attrs: Some(icon.attrs) });
    }
    // Per scale, appearance and idiom (each with atlases of its own, keyed
    // by them),
    // translucent colour, translucent grey, opaque colour and
    // opaque grey images (an atlas named for its opacity and greyness), then
    // the 16-bit renditions of colour images from wide-gamut profiles in
    // atlases of their own (gamut 1, indexed apart),
    // each largest area first (equal areas by identifier): images over 18,050 px² per scale
    // factor, or a group's only packable one, stored on their own, the rest
    // packed into an atlas, as app icons are. An atlas's dimension1 is its
    // index at its scale.
    let mut scales: Vec<(u32, u16, u16)> = images.iter().map(|i| (i.scale, i.appearance, i.idiom)).collect();
    scales.sort();
    scales.dedup();
    for (scale, appearance, idiom) in scales {
        let mut planned: Vec<(usize, bool, bool, Vec<&SetImage>)> = Vec::new();
        for (gamut, opaque, gray) in [(0, false, false), (0, false, true), (0, true, false), (0, true, true), (1, false, false), (1, true, false)] {
            let mut group: Vec<&SetImage> = images
                .iter()
                .filter(|i| {
                    (i.scale, i.appearance, i.idiom) == (scale, appearance, idiom) && is_opaque(i) == opaque && is_gray(i) == gray && (gamut == 0 || i.extended.is_some())
                })
                .collect();
            // Equal areas by identifier (12 probes of equal-area shapes).
            group.sort_by_key(|i| (std::cmp::Reverse(i.image.width * i.image.height), i.identifier));
            let cap = crate::stack::PACKING_AREA * scale as usize;
            let (standalone, packable): (Vec<&SetImage>, Vec<&SetImage>) = group.into_iter().partition(|i| i.image.width * i.image.height > cap);
            if gamut == 0 && !packable.is_empty() {
                out.packable.push((scale, appearance, idiom, opaque, gray));
            }
            let (standalone, packable) = if packable.len() < 2 { (standalone.into_iter().chain(packable).collect::<Vec<_>>(), Vec::new()) } else { (standalone, packable) };
            if !packable.is_empty() {
                if packable.windows(2).any(|w| w[0].image.width * w[0].image.height == w[1].image.width * w[1].image.height && w[0].identifier == w[1].identifier) {
                    return Err(format!("image sets with equal identifiers and areas in one {scale}x atlas are not supported by this actool"));
                }
                // Next-fit atlases: one is closed once its area reaches the
                // cap, the next taking the following index at this scale.
                // When the next image would fit in the closed atlas without
                // growing it, Xcode sometimes keeps adding a few more, in a
                // way not established: refused. Next-fit gives Xcode's atlases
                // in 296 of 300 catalogs of equal images, 1x to 3x, the four
                // others tall ones at 1x, and 158 of 200 of mixed sizes at
                // 1x. Where it would grow the atlas, Xcode closes it as
                // next-fit does. An atlas with bitmaps drawn from vector
                // images, which aren't Xcode's anyway, is packed next-fit,
                // as every atlas is with --config=relax_actool_storage.
                let mut total = 0;
                for i in packable {
                    if planned.last().is_none_or(|a| (a.0, a.1, a.2) != (gamut, opaque, gray)) || total >= cap {
                        if let Some(last) = planned.last().filter(|a| (a.0, a.1, a.2) == (gamut, opaque, gray)) {
                            let mut blocks: Vec<(u32, u32)> = last.3.iter().map(|i| (i.image.width as u32, i.image.height as u32)).collect();
                            let closed = crate::iconset::pack::pack(&blocks).1;
                            blocks.push((i.image.width as u32, i.image.height as u32));
                            // With an icon set's icons, Xcode packs next-fit, as
                            // the icon set's own atlases (two probes at 3x).
                            let icons = i.attrs.is_some() || last.3.iter().any(|j| j.attrs.is_some());
                            if crate::iconset::pack::pack(&blocks).1 == closed
                                && !drawn(i)
                                && !last.3.iter().any(|j| drawn(j))
                                && !icons
                                && !crate::dmp2::storage_relaxed()
                            {
                                return Err(format!("{scale}x images filling more than one atlas, where the next image fits in the one before, are not supported by this actool"));
                            }
                        }
                        planned.push((gamut, opaque, gray, Vec::new()));
                        total = 0;
                    }
                    total += i.image.width * i.image.height;
                    planned.last_mut().unwrap().3.push(i);
                }
            }
            if standalone.iter().any(|i| i.attrs.is_some()) {
                return Err("an app icon packed with image sets but stored on its own is not supported by this actool".into());
            }
            for i in standalone {
                // A bitmap drawn from a vector image (flag 4) isn't Xcode's:
                // the encoder may choose where Xcode's choice isn't known.
                out.renditions.push(if gamut == 0 { crate::dmp2::tolerant(drawn(i), || standalone_rendition(i))? } else { standalone16(i)? });
            }
        }
        // A scale's atlases (per gamut) take their dimension1 in the order
        // of their indices as decimal strings: the 11th of 19 is 2, the
        // 3rd is 11 (a catalog of 19, all else identical).
        for gamut in [0, 1] {
            let atlases: Vec<&(usize, bool, bool, Vec<&SetImage>)> = planned.iter().filter(|a| a.0 == gamut).collect();
            let before = if gamut == 0 && appearance == 0 && idiom == 0 { reserved.iter().filter(|r| r.0 == scale).map(|r| r.1).sum() } else { 0 };
            let mut names: Vec<String> = (0..before + atlases.len()).map(|k| k.to_string()).collect();
            names.sort();
            for (k, (_, opaque, gray, atlas)) in atlases.into_iter().enumerate() {
                let index = names.iter().position(|n| *n == (before + k).to_string()).unwrap() as u16;
                if gamut == 0 {
                    crate::dmp2::tolerant(atlas.iter().any(|i| drawn(i)), || pack(scale, (appearance, idiom), *opaque, *gray, index, atlas, &mut out.renditions))?;
                } else {
                    pack16(scale, (appearance, idiom), *opaque, index, atlas, &mut out.renditions)?;
                }
            }
        }
    }
    Ok(out)
}

/// The JPEG Xcode stores for an image set's JPEG, and its size: the file
/// without its comments, when it holds only what is established to be kept
/// as it is (JFIF, tables, frame, restart interval, scan, and the standard
/// sRGB profiles); other metadata Xcode passes through ImageIO, which writes
/// it back in its own form, and other profiles it converts.
fn stored_jpeg(j: &[u8]) -> Result<(Vec<u8>, u32, u32, u16), String> {
    const SRGB_PROFILE_LEN: usize = 3144;
    if !j.starts_with(&[0xff, 0xd8]) {
        return Err("not a JPEG".into());
    }
    let mut out = j[..2].to_vec();
    let mut size = None;
    let mut orientation = 1;
    // The metadata, which ImageIO writes back in its own order whatever the
    // file's: JFIF, Exif, the colour profile, Photoshop's, Adobe's (six
    // probes, two of the corpus's JPEGs), before the tables.
    let mut slots: [Option<Vec<u8>>; 5] = Default::default();
    let mut flushed = false;
    // Exif entries that XMP or IPTC properties map to, which must already
    // hold the same values.
    let mut merged: Merged = Vec::new();
    let mut entries: Vec<Entry> = Vec::new();
    let mut i = 2;
    loop {
        if i + 4 > j.len() || j[i] != 0xff {
            return Err("a JPEG this actool can't read".into());
        }
        let marker = j[i + 1];
        let len = u16::from_be_bytes([j[i + 2], j[i + 3]]) as usize;
        let segment = j.get(i..i + 2 + len).ok_or("a truncated JPEG")?;
        let metadata = (0xe0..=0xef).contains(&marker) || marker == 0xfe;
        if metadata && flushed {
            return Err("a JPEG with metadata after its tables is not supported by this actool".into());
        }
        if !metadata && !flushed {
            flushed = true;
            for slot in slots.iter_mut() {
                if let Some(s) = slot.take() {
                    out.extend(s);
                }
            }
        }
        let mut fill = |k: usize, s: Vec<u8>| -> Result<(), String> {
            if slots[k].replace(s).is_some() {
                return Err("a JPEG with two segments of the same metadata is not supported by this actool".into());
            }
            Ok(())
        };
        match marker {
            // The scan and what follows it, as they are.
            0xda => {
                out.extend_from_slice(&j[i..]);
                break;
            }
            // Comments and Ducky (APP12) segments are dropped, Adobe (APP14)
            // segments kept, and these, established, as they are; others
            // are refused (ten probes, and the corpus's JPEGs).
            0xfe | 0xec => {}
            0xee if segment.get(4..9) == Some(b"Adobe") => fill(4, segment.to_vec())?,
            // XMP is dropped: its properties that ImageIO maps to Exif
            // entries must agree with them.
            0xe1 if segment.get(4..33) == Some(b"http://ns.adobe.com/xap/1.0/\0") => merged.extend(xmp_to_exif(&segment[33..])?),
            // Exif with no entries is dropped; other Exif ImageIO writes
            // back as `exif` gives it, its orientation recorded.
            0xe1 if segment.get(4..10) == Some(b"Exif\0\0") => match exif(&segment[10..]) {
                Some(Exif::Empty) => {}
                Some(Exif::Tiff(tiff, o, all)) => {
                    orientation = o;
                    entries = all;
                    let mut s = vec![0xff, 0xe1];
                    s.extend_from_slice(&((tiff.len() + 8) as u16).to_be_bytes());
                    s.extend_from_slice(b"Exif\0\0");
                    s.extend(tiff);
                    fill(1, s)?;
                }
                None => return Err("a JPEG with Exif entries other than those established (which Xcode rewrites) is not supported by this actool".into()),
            },
            // Photoshop's (APP13) of only an empty IPTC record and its
            // digest is dropped (one probe, one of the corpus's); other IPTC
            // is written back as `iptc` gives it.
            0xed if segment.get(4..18) == Some(b"Photoshop 3.0\0") && is_empty_iptc(&segment[18..]) => {}
            0xed if segment.get(4..18) == Some(b"Photoshop 3.0\0") => {
                let (resources, needs) = iptc(&segment[18..])?;
                merged.extend(needs);
                let mut s = vec![0xff, 0xed];
                s.extend_from_slice(&((resources.len() + 16) as u16).to_be_bytes());
                s.extend_from_slice(b"Photoshop 3.0\0");
                s.extend(resources);
                fill(3, s)?;
            }
            0xe0 if segment.get(4..9) == Some(b"JFIF\0") => fill(0, segment.to_vec())?,
            // Apple's sRGB profile, the ICC's black-scaled sRGB and
            // Facebook's compact sRGB "c2" (two of the corpus's JPEGs) are
            // kept as they are.
            0xe2 if segment.get(4..16) == Some(b"ICC_PROFILE\0") => {
                let profile = &segment[18..];
                let srgb = profile.len() == SRGB_PROFILE_LEN && profile.windows(17).any(|w| w == b"sRGB IEC61966-2.1");
                let known = matches!((profile.len(), fnv1a(profile)), (3048, 0x131b_e18b_256c_1005) | (524, 0xef9b_4700_d18b_5f9a));
                if !(srgb || known) || segment[16..18] != [1, 1] {
                    return Err("a JPEG with a colour profile other than sRGB's is not supported by this actool".into());
                }
                fill(2, segment.to_vec())?;
            }
            0xc0 | 0xc2 => {
                let h = u16::from_be_bytes([segment[5], segment[6]]) as u32;
                let w = u16::from_be_bytes([segment[7], segment[8]]) as u32;
                size = Some((w, h));
                out.extend_from_slice(segment);
            }
            0xc4 | 0xdb | 0xdd => out.extend_from_slice(segment),
            m => return Err(format!("a JPEG with marker {m:#04x} (metadata Xcode rewrites) is not supported by this actool")),
        }
        i += 2 + len;
    }
    for (tag, value) in merged {
        if !entries.iter().any(|e| e.0 == tag && e.3 == value) {
            return Err("a JPEG whose XMP or IPTC Xcode would merge into its Exif is not supported by this actool".into());
        }
    }
    let (w, h) = size.ok_or("a JPEG without a frame")?;
    Ok((out, w, h, orientation))
}

/// Exif entries, as tag and value, that XMP or IPTC properties map to.
type Merged = Vec<(u16, Vec<u8>)>;

/// 64-bit FNV-1a, to recognise a colour profile.
fn fnv1a(b: &[u8]) -> u64 {
    b.iter().fold(0xcbf2_9ce4_8422_2325, |h, &x| (h ^ x as u64).wrapping_mul(0x0100_0000_01b3))
}

/// The Exif entries an XMP packet's properties map to, as ImageIO maps them
/// (about 20 probes): `xmp:CreateDate` to DateTimeDigitized,
/// `tiff:Orientation` to the orientation and `exif:PixelXDimension` to the
/// pixel width. These properties are dropped: the creator tool, the
/// modification and metadata dates, the document and instance ids and
/// derivation, the format, Photoshop's colour mode, profile name,
/// transmission reference, headline, city and credit, and the TIFF
/// software. Others are refused.
fn xmp_to_exif(packet: &[u8]) -> Result<Merged, String> {
    let unsupported = || "a JPEG with XMP properties this actool hasn't established (which Xcode may turn into Exif and IPTC) is not supported by this actool".to_string();
    let text = std::str::from_utf8(packet).map_err(|_| unsupported())?;
    let (Some(a), Some(b)) = (text.find("<x:xmpmeta"), text.find("</x:xmpmeta>")) else {
        return Err(unsupported());
    };
    let doc = roxmltree::Document::parse(&text[a..b + 12]).map_err(|_| unsupported())?;
    const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
    const XMP: &str = "http://ns.adobe.com/xap/1.0/";
    const MM: &str = "http://ns.adobe.com/xap/1.0/mm/";
    const DC: &str = "http://purl.org/dc/elements/1.1/";
    const PS: &str = "http://ns.adobe.com/photoshop/1.0/";
    const TIFF: &str = "http://ns.adobe.com/tiff/1.0/";
    const EXIF: &str = "http://ns.adobe.com/exif/1.0/";
    let dropped = [
        (XMP, "CreatorTool"),
        (XMP, "ModifyDate"),
        (XMP, "MetadataDate"),
        (MM, "DocumentID"),
        (MM, "InstanceID"),
        (MM, "DerivedFrom"),
        (DC, "format"),
        (PS, "ColorMode"),
        (PS, "ICCProfile"),
        (PS, "TransmissionReference"),
        (PS, "Headline"),
        (PS, "City"),
        (PS, "Credit"),
        (TIFF, "Software"),
    ];
    let mut out = Vec::new();
    for d in doc.descendants().filter(|d| d.tag_name().namespace() == Some(RDF) && d.tag_name().name() == "Description") {
        let attributes = d.attributes().filter(|a| a.namespace() != Some(RDF)).map(|a| (a.namespace().unwrap_or(""), a.name(), Some(a.value())));
        let children = d.children().filter(|c| c.is_element()).map(|c| (c.tag_name().namespace().unwrap_or(""), c.tag_name().name(), c.text()));
        for (ns, name, value) in attributes.chain(children) {
            if dropped.contains(&(ns, name)) {
                continue;
            }
            let value = value.map(str::trim).ok_or_else(unsupported)?;
            out.push(match (ns, name) {
                (XMP, "CreateDate") => (0x9004, xmp_date(value).ok_or_else(unsupported)?),
                (TIFF, "Orientation") => (0x112, value.parse::<u16>().map_err(|_| unsupported())?.to_be_bytes().to_vec()),
                (EXIF, "PixelXDimension") => (0xa002, value.parse::<u32>().map_err(|_| unsupported())?.to_be_bytes().to_vec()),
                _ => return Err(unsupported()),
            });
        }
    }
    Ok(out)
}

/// An XMP date with seconds (`2009-10-09T21:31:03`, any time zone) as an
/// Exif one, its NUL included; the zone is dropped (a probe).
fn xmp_date(v: &str) -> Option<Vec<u8>> {
    let b = v.as_bytes();
    let digits = |r: std::ops::Range<usize>| b.get(r).is_some_and(|d| d.iter().all(u8::is_ascii_digit));
    let ok = digits(0..4)
        && b.get(4) == Some(&b'-')
        && digits(5..7)
        && b.get(7) == Some(&b'-')
        && digits(8..10)
        && b.get(10) == Some(&b'T')
        && digits(11..13)
        && b.get(13) == Some(&b':')
        && digits(14..16)
        && b.get(16) == Some(&b':')
        && digits(17..19);
    let zone = &v[19.min(v.len())..];
    let zone_ok = zone.is_empty() || zone == "Z" || (zone.len() == 6 && (zone.starts_with('+') || zone.starts_with('-')) && zone.as_bytes()[3] == b':');
    if !ok || !zone_ok {
        return None;
    }
    Some(format!("{}:{}:{} {}\0", &v[0..4], &v[5..7], &v[8..10], &v[11..19]).into_bytes())
}

/// Photoshop image resources as ImageIO writes them back (about 15 probes,
/// two of the corpus's JPEGs): the IPTC record (0x404) rewritten, its
/// character set UTF-8 and record version 2 first, then the other datasets;
/// its digest (0x425), when there is one, the MD5 of the new record. Other
/// datasets are taken one, or two in an order established ({2:103, 2:40},
/// {2:62, 2:63}); a record of one dataset and no header, which ImageIO
/// keeps beside a rewritten one, and other resources are refused. Also the
/// Exif entries the record's date and time map to.
fn iptc(resources: &[u8]) -> Result<(Vec<u8>, Merged), String> {
    let unsupported = |what: &str| format!("a JPEG with {what} (which Xcode rewrites) is not supported by this actool");
    let mut i = 0;
    let mut record = None;
    let mut digest = false;
    while i < resources.len() {
        let r = resources.get(i..i + 12).ok_or_else(|| unsupported("Photoshop resources this actool can't read"))?;
        if &r[..4] != b"8BIM" || r[6..8] != [0, 0] {
            return Err(unsupported("Photoshop resources this actool can't read"));
        }
        let len = u32::from_be_bytes([r[8], r[9], r[10], r[11]]) as usize;
        let data = resources.get(i + 12..i + 12 + len).ok_or_else(|| unsupported("Photoshop resources this actool can't read"))?;
        match u16::from_be_bytes([r[4], r[5]]) {
            0x404 if record.is_none() && !digest => record = Some(data),
            0x425 if record.is_some() && !digest && len == 16 => digest = true,
            _ => return Err(unsupported("Photoshop resources other than an IPTC record and its digest")),
        }
        i += 12 + len + len % 2;
    }
    let record = record.ok_or_else(|| unsupported("Photoshop resources without an IPTC record"))?;
    let mut datasets = Vec::new();
    let mut k = 0;
    while k < record.len() {
        let d = record
            .get(k..k + 5)
            .filter(|d| d[0] == 0x1c && d[3] & 0x80 == 0)
            .ok_or_else(|| unsupported("an IPTC record this actool can't read"))?;
        let len = u16::from_be_bytes([d[3], d[4]]) as usize;
        let value = record.get(k + 5..k + 5 + len).ok_or_else(|| unsupported("an IPTC record this actool can't read"))?;
        datasets.push(((d[1], d[2]), value));
        k += 5 + len;
    }
    let header = datasets.iter().filter(|d| [(1, 90), (2, 0)].contains(&d.0)).count();
    let others: Vec<_> = datasets.iter().filter(|d| ![(1, 90), (2, 0)].contains(&d.0)).collect();
    let mut ids: Vec<(u8, u8)> = others.iter().map(|d| d.0).collect();
    ids.sort();
    let order: Vec<(u8, u8)> = match ids.as_slice() {
        [one] if header > 0 && one.0 == 2 => vec![*one],
        [(2, 40), (2, 103)] => vec![(2, 103), (2, 40)],
        [(2, 62), (2, 63)] => vec![(2, 62), (2, 63)],
        _ => return Err(unsupported("IPTC datasets whose order this actool hasn't established")),
    };
    // A value that isn't ASCII might be transcoded from another character
    // set.
    if datasets.iter().any(|d| d.0 != (1, 90) && d.0 != (2, 0) && !d.1.is_ascii()) {
        return Err(unsupported("IPTC text other than ASCII"));
    }
    let mut new = b"\x1c\x01\x5a\x00\x03\x1b\x25\x47\x1c\x02\x00\x00\x02\x00\x02".to_vec();
    for id in &order {
        let value = others.iter().find(|d| d.0 == *id).unwrap().1;
        new.extend_from_slice(&[0x1c, id.0, id.1]);
        new.extend_from_slice(&(value.len() as u16).to_be_bytes());
        new.extend_from_slice(value);
    }
    let mut out = b"8BIM\x04\x04\0\0".to_vec();
    out.extend_from_slice(&(new.len() as u32).to_be_bytes());
    out.extend_from_slice(&new);
    if new.len() % 2 == 1 {
        out.push(0);
    }
    if digest {
        out.extend_from_slice(b"8BIM\x04\x25\0\0\0\0\0\x10");
        out.extend_from_slice(&crate::md5::digest(&new));
    }
    // Its date created and time created are DateTimeDigitized (two probes).
    let mut needs = Vec::new();
    if ids.contains(&(2, 62)) {
        let date = others.iter().find(|d| d.0 == (2, 62)).unwrap().1;
        let time = others.iter().find(|d| d.0 == (2, 63)).unwrap().1;
        let ok = date.len() == 8 && date.is_ascii() && time.len() >= 6 && time.is_ascii();
        if !ok {
            return Err(unsupported("an IPTC date this actool can't read"));
        }
        let (d, t) = (std::str::from_utf8(date).unwrap(), std::str::from_utf8(time).unwrap());
        needs.push((0x9004, format!("{}:{}:{} {}:{}:{}\0", &d[..4], &d[4..6], &d[6..8], &t[..2], &t[2..4], &t[4..6]).into_bytes()));
    }
    Ok((out, needs))
}

/// An IFD entry: tag, type, count and its value's bytes.
type Entry = (u16, u16, u32, Vec<u8>);

enum Exif {
    Empty,
    /// The TIFF as ImageIO writes it back, the orientation, and its entries
    /// (not the Exif IFD's pointer).
    Tiff(Vec<u8>, u16, Vec<Entry>),
}

/// An Exif TIFF as Xcode stores it: none if it has no entries, else as
/// ImageIO writes it back (docs/clean-room/actool.md): big-endian, IFD0 at
/// 8 then its values, the Exif IFD then its values, entries by tag, values
/// of more than four bytes after their IFD in its order, padded to even.
/// Only the entries probed are taken: the orientation, resolutions and
/// their unit, YCbCr positioning (dropped), and the Exif IFD's versions,
/// components, user comment, colour space and pixel dimensions (as LONGs);
/// rationals only where ImageIO's conversion keeps them exact. Others, and
/// a second IFD, are refused.
fn exif(tiff: &[u8]) -> Option<Exif> {
    let be = match tiff.get(..4)? {
        b"MM\0\x2a" => true,
        b"II\x2a\0" => false,
        _ => return None,
    };
    let u16_at = |o: usize| -> Option<u16> {
        let b = tiff.get(o..o + 2)?;
        Some(if be { u16::from_be_bytes([b[0], b[1]]) } else { u16::from_le_bytes([b[0], b[1]]) })
    };
    let u32_at = |o: usize| -> Option<u32> {
        let b = tiff.get(o..o + 4)?;
        Some(if be { u32::from_be_bytes([b[0], b[1], b[2], b[3]]) } else { u32::from_le_bytes([b[0], b[1], b[2], b[3]]) })
    };
    // An IFD's entries, their values' bytes in the file's order.
    let read = |at: usize| -> Option<Vec<Entry>> {
        let n = u16_at(at)? as usize;
        if u32_at(at + 2 + 12 * n)? != 0 {
            return None;
        }
        (0..n)
            .map(|k| {
                let e = at + 2 + 12 * k;
                let (tag, typ, count) = (u16_at(e)?, u16_at(e + 2)?, u32_at(e + 4)?);
                let size = match typ {
                    2 => 1,
                    3 => 2,
                    4 => 4,
                    5 => 8,
                    7 => 1,
                    _ => return None,
                } * count as usize;
                let at = if size <= 4 { e + 8 } else { u32_at(e + 8)? as usize };
                Some((tag, typ, count, tiff.get(at..at + size)?.to_vec()))
            })
            .collect()
    };
    let short = |v: &[u8]| if be { u16::from_be_bytes([v[0], v[1]]) } else { u16::from_le_bytes([v[0], v[1]]) };
    let long = |v: &[u8]| if be { u32::from_be_bytes([v[0], v[1], v[2], v[3]]) } else { u32::from_le_bytes([v[0], v[1], v[2], v[3]]) };
    let ifd0 = read(u32_at(4)? as usize)?;
    if ifd0.is_empty() {
        return Some(Exif::Empty);
    }
    let mut orientation = 1;
    let (mut first, mut second) = (Vec::new(), Vec::new());
    let mut exif_ifd = None;
    for (tag, typ, count, v) in ifd0 {
        match (tag, typ, count) {
            (0x112, 3, 1) => {
                orientation = short(&v);
                if !(1..=8).contains(&orientation) {
                    return None;
                }
                first.push((tag, 3, 1, orientation.to_be_bytes().to_vec()));
            }
            (0x128, 3, 1) => first.push((tag, 3, 1, short(&v).to_be_bytes().to_vec())),
            (0x11a | 0x11b, 5, 1) => first.push((tag, 5, 1, kept_rational(long(&v[..4]), long(&v[4..]))?)),
            (0x213, 3, 1) => {}
            (0x10e | 0x10f | 0x110 | 0x131 | 0x13b | 0x8298, 2, _) => first.push(ascii(tag, &v, false)?),
            (0x132, 2, _) => first.push(ascii(tag, &v, true)?),
            (0x8769, 4, 1) => exif_ifd = Some(long(&v) as usize),
            _ => return None,
        }
    }
    if let Some(at) = exif_ifd {
        for (tag, typ, count, v) in read(at)? {
            match (tag, typ, count) {
                (0x9000 | 0xa000 | 0x9101, 7, 4) | (0x9286, 7, _) => second.push((tag, 7, count, v)),
                (0xa001, 3, 1) => second.push((tag, 3, 1, short(&v).to_be_bytes().to_vec())),
                (0x9003 | 0x9004, 2, _) => second.push(ascii(tag, &v, true)?),
                (0xa002 | 0xa003, 3, 1) => second.push((tag, 4, 1, u32::from(short(&v)).to_be_bytes().to_vec())),
                (0xa002 | 0xa003, 4, 1) => second.push((tag, 4, 1, long(&v).to_be_bytes().to_vec())),
                _ => return None,
            }
        }
        if second.is_empty() {
            return None;
        }
    }
    if first.is_empty() && second.is_empty() {
        return None;
    }
    first.sort_by_key(|e| e.0);
    second.sort_by_key(|e| e.0);
    let all: Vec<Entry> = first.iter().chain(&second).cloned().collect();
    let mut out = b"MM\0\x2a\0\0\0\x08".to_vec();
    let exif_at = 8 + ifd_len(&first, !second.is_empty());
    if !second.is_empty() {
        first.push((0x8769, 4, 1, (exif_at as u32).to_be_bytes().to_vec()));
        first.sort_by_key(|e| e.0);
    }
    write_ifd(&mut out, &first);
    if !second.is_empty() {
        write_ifd(&mut out, &second);
    }
    Some(Exif::Tiff(out, orientation, all))
}

/// An ASCII entry as ImageIO writes it back: its text to the first NUL, and
/// one NUL (`D90\0\0\0` is `D90\0`; two probes). Only NULs may follow the
/// text, which isn't empty; a date must be one (ImageIO drops `ab`, a probe).
fn ascii(tag: u16, v: &[u8], date: bool) -> Option<Entry> {
    let end = v.iter().position(|&b| b == 0)?;
    if end == 0 || v[end..].iter().any(|&b| b != 0) || !v[..end].is_ascii() {
        return None;
    }
    let text = &v[..end];
    let is_date = text.len() == 19
        && text.iter().enumerate().all(|(k, &c)| match k {
            4 | 7 | 13 | 16 => c == b':',
            10 => c == b' ',
            _ => c.is_ascii_digit(),
        });
    if date && !is_date {
        return None;
    }
    let mut value = text.to_vec();
    value.push(0);
    Some((tag, 2, value.len() as u32, value))
}

/// The length of an IFD and its values as `write_ifd` writes it, counting
/// an Exif pointer still to be added.
fn ifd_len(entries: &[Entry], pointer: bool) -> usize {
    let n = entries.len() + pointer as usize;
    2 + 12 * n + 4 + entries.iter().filter(|e| e.3.len() > 4).map(|e| e.3.len().div_ceil(2) * 2).sum::<usize>()
}

/// Appends an IFD of big-endian `entries` at the end of `out`, its values
/// of more than four bytes after it.
fn write_ifd(out: &mut Vec<u8>, entries: &[Entry]) {
    let mut data_at = out.len() + 2 + 12 * entries.len() + 4;
    let mut data = Vec::new();
    out.extend_from_slice(&(entries.len() as u16).to_be_bytes());
    for (tag, typ, count, v) in entries {
        out.extend_from_slice(&tag.to_be_bytes());
        out.extend_from_slice(&typ.to_be_bytes());
        out.extend_from_slice(&count.to_be_bytes());
        if v.len() <= 4 {
            let mut inline = v.clone();
            inline.resize(4, 0);
            out.extend_from_slice(&inline);
        } else {
            out.extend_from_slice(&(data_at as u32).to_be_bytes());
            data.extend_from_slice(v);
            if v.len() % 2 == 1 {
                data.push(0);
            }
            data_at += v.len().div_ceil(2) * 2;
        }
    }
    out.extend_from_slice(&[0; 4]);
    out.extend(data);
}

/// A rational as ImageIO writes it back, big-endian, where that is
/// established: ImageIO takes its value and finds a fraction for it again
/// (123456789/1000000 becomes 1355679/10981, a continued fraction's
/// convergent within 1e-9), so only rationals whose continued fraction's
/// earlier convergents are all far from the value (more than 1e-6) are
/// taken, which it writes in lowest terms (25400/1000 as 127/5, seven
/// probes). Zero denominators (0/1 for 72/0) and terms past a signed
/// 32-bit integer (4294967295/2 is 2147483647/1) are not.
fn kept_rational(num: u32, den: u32) -> Option<Vec<u8>> {
    if den == 0 || num > i32::MAX as u32 || den > i32::MAX as u32 {
        return None;
    }
    let (mut a, mut b) = (u64::from(num), u64::from(den));
    let g = {
        let (mut x, mut y) = (a, b);
        while y != 0 {
            (x, y) = (y, x % y);
        }
        x.max(1)
    };
    let value = num as f64 / den as f64;
    // The continued fraction's convergents before the fraction itself.
    let (mut h, mut h1, mut k, mut k1) = (1u64, 0u64, 0u64, 1u64);
    while b != 0 {
        let q = a / b;
        (a, b) = (b, a % b);
        (h, h1) = (q * h + h1, h);
        (k, k1) = (q * k + k1, k);
        if b != 0 && (h as f64 / k as f64 - value).abs() <= 1e-6 {
            return None;
        }
    }
    let (n, d) = (num as u64 / g, den as u64 / g);
    let mut out = (n as u32).to_be_bytes().to_vec();
    out.extend_from_slice(&(d as u32).to_be_bytes());
    Some(out)
}

/// Photoshop image resources of only an empty IPTC record (0x404), its
/// digest (0x425), or both.
fn is_empty_iptc(resources: &[u8]) -> bool {
    let mut i = 0;
    let mut ids = Vec::new();
    while i < resources.len() {
        let Some(r) = resources.get(i..i + 12) else { return false };
        if &r[..4] != b"8BIM" || r[6..8] != [0, 0] {
            return false;
        }
        let len = u32::from_be_bytes([r[8], r[9], r[10], r[11]]) as usize;
        let id = u16::from_be_bytes([r[4], r[5]]);
        if id == 0x404 && len != 0 {
            return false;
        }
        ids.push(id);
        i += 12 + len + len % 2;
    }
    !ids.is_empty() && ids.iter().all(|id| [0x404, 0x425].contains(id))
}

/// A resizable image's image, cropped to its caps and centre, and its
/// slicing: the slices of a three-part image left to right or top to
/// bottom, of a nine-part one by rows from the top, their y from the
/// bottom; layout 20 for three parts across, 23 down and 30 for nine, one
/// more when the centre stretches rather than tiles (16 probes). Cap insets
/// and the centre's size are in pixels.
fn resizing(r: &serde_json::Value, img: &image::Rgba) -> Result<(image::Rgba, car::Slicing), String> {
    let unsupported = || format!("the resizing {r} is not supported by this actool");
    let o = r.as_object().ok_or_else(unsupported)?;
    if o.keys().any(|k| !["mode", "center", "cap-insets"].contains(&k.as_str())) {
        return Err(unsupported());
    }
    let (across, down, base) = match r["mode"].as_str() {
        Some("3-part-horizontal") => (true, false, 20),
        Some("3-part-vertical") => (false, true, 23),
        Some("9-part") => (true, true, 30),
        _ => return Err(unsupported()),
    };
    let center = r["center"].as_object().ok_or_else(unsupported)?;
    let stretch = match center.get("mode").and_then(|m| m.as_str()) {
        Some("tile") => 0,
        Some("stretch") => 1,
        _ => return Err(unsupported()),
    };
    let insets = r["cap-insets"].as_object().ok_or_else(unsupported)?;
    let size = |v: Option<&serde_json::Value>| v.and_then(|v| v.as_u64()).map(|v| v as usize).ok_or_else(unsupported);
    let (mut wanted_center, mut wanted_insets) = (vec!["mode"], vec![]);
    let (l, cw, rr) = if across {
        wanted_center.push("width");
        wanted_insets.extend(["left", "right"]);
        (size(insets.get("left"))?, size(center.get("width"))?, size(insets.get("right"))?)
    } else {
        (0, img.width, 0)
    };
    let (t, ch, b) = if down {
        wanted_center.push("height");
        wanted_insets.extend(["top", "bottom"]);
        (size(insets.get("top"))?, size(center.get("height"))?, size(insets.get("bottom"))?)
    } else {
        (0, img.height, 0)
    };
    if center.keys().any(|k| !wanted_center.contains(&k.as_str())) || insets.keys().any(|k| !wanted_insets.contains(&k.as_str())) {
        return Err(unsupported());
    }
    if cw == 0 || ch == 0 || l + cw + rr > img.width || t + ch + b > img.height {
        return Err(format!("the resizing {r} doesn't fit its {}x{} image", img.width, img.height));
    }
    // Columns and rows kept: the caps and the centre after the first cap.
    let columns: Vec<usize> = (0..l + cw).chain(img.width - rr..img.width).collect();
    let rows: Vec<usize> = (0..t + ch).chain(img.height - b..img.height).collect();
    let mut data = Vec::with_capacity(columns.len() * rows.len() * 4);
    for &y in &rows {
        for &x in &columns {
            let p = (y * img.width + x) * 4;
            data.extend_from_slice(&img.data[p..p + 4]);
        }
    }
    let cropped = image::Rgba { width: columns.len(), height: rows.len(), data };
    let (w, h) = (cropped.width as u32, cropped.height as u32);
    let xs: Vec<(u32, u32)> = if across { vec![(0, l as u32), (l as u32, cw as u32), ((l + cw) as u32, rr as u32)] } else { vec![(0, w)] };
    let ys: Vec<(u32, u32)> = if down { vec![((b + ch) as u32, t as u32), (b as u32, ch as u32), (0, b as u32)] } else { vec![(0, h)] };
    let slices = ys.iter().flat_map(|&(y, sh)| xs.iter().map(move |&(x, sw)| [x, y, sw, sh])).collect();
    Ok((cropped, car::Slicing { layout: base + stretch, slices }))
}

struct SetImage {
    scale: u32,
    /// Its appearance's value (0 for any).
    appearance: u16,
    /// Its idiom's (0 for universal).
    idiom: u16,
    identifier: u16,
    flags: u32,
    filename: String,
    image: image::Rgba,
    /// The 16-bit rendition's pixels, for a wide-gamut profile's colours
    /// outside sRGB: premultiplied half floats, RGBA.
    extended: Option<Vec<u16>>,
    /// A resizable image's slicing.
    slicing: Option<car::Slicing>,
    /// An app icon's own key, for an icon packed with image sets' images
    /// (crate::iconset::DeferredIcon).
    attrs: Option<Vec<(u16, u16)>>,
}

/// A Messages extension's sticker icon set (`.stickersiconset`) that
/// `--app-icon` names: from iOS 14 Xcode keeps only its 1024x768 App Store
/// image, as an image of the `marketing` idiom on its own (two probes, three
/// of the corpus's). A set naming no files compiles to nothing. A catalog
/// of only a sticker icon (and data) has no appearance table.
pub fn sticker_icon(name: &str, dir: &std::path::Path, id: u16) -> Result<Option<Compiled>, String> {
    let contents = read_json(&dir.join("Contents.json"))?;
    if !crate::has_entries(&contents, "images", "filename") {
        return Ok(None);
    }
    let store = contents["images"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|e| e["idiom"] == "ios-marketing" && e["size"] == "1024x768" && e["scale"] == "1x" && e["filename"].is_string())
        .ok_or_else(|| format!("{}: a sticker icon set without its 1024x768 App Store image is not supported by this actool", dir.display()))?;
    let filename = store["filename"].as_str().expect("checked");
    let path = dir.join(filename);
    let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let (img, extended) = image::decode_png_extended(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    if extended.is_some() || (img.width, img.height) != (1024, 768) {
        return Err(format!("{}: a sticker icon with colours outside sRGB, or not 1024x768, is not supported by this actool", path.display()));
    }
    // No template rendering intent (flags 0), unlike an image set's.
    let image = SetImage { scale: 1, appearance: 0, idiom: car::IDIOM_MARKETING, identifier: id, flags: 0, filename: filename.to_string(), image: img, extended: None, slicing: None, attrs: None };
    Ok(Some(Compiled {
        facets: vec![Facet { name: name.to_string(), attrs: vec![(car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_IMAGE), (car::ATTR_IDENTIFIER, id)] }],
        renditions: vec![standalone_rendition(&image).map_err(|e| format!("{}: {e}", path.display()))?],
        packable: Vec::new(),
        dropped_3x: false,
    }))
}

/// An image set of vector images (crate::vector): per appearance and idiom
/// (universal, or the Mac's on macOS, or iPhone's and iPad's on iOS for both
/// devices), one file without a scale, or one per scale. Xcode stores a
/// file (a Vector rendition) and draws bitmaps at each of the platform's
/// scales (1x to 3x on iOS, 1x and 2x on macOS and tvOS, 2x on watchOS and
/// visionOS), compiled and packed as a PNG's, flagged 4 (and 256 when the
/// set preserves its vector representation) beside the rendering intent's
/// flags (twenty-two probes, and eighteen of these rules):
/// - the file stored is the one at the highest of the platform's scales
///   (part 181, no scale); preserved, each file at its own scale (part 42),
///   one without a scale at 1x, and the 1x file even where the platform
///   draws no 1x (watchOS, visionOS);
/// - each scale is drawn from its own file, else from the 1x file, else
///   from the highest scale's; files at scales the platform doesn't use are
///   ignored;
/// - a file at scale `f` drawn at scale `t`: a PDF at its size over `f`
///   times `t`; an SVG at its size times `t` (Xcode doesn't divide), its
///   slice its size over `f` times `t`.
///
/// A PNG at 2x or 3x beside one universal 1x vector image takes its scale's
/// place, as a bitmap of its own (three probes, on iOS, macOS and tvOS);
/// beside one without a scale Xcode reports an error; other mixes are
/// refused.
#[allow(clippy::too_many_arguments)]
fn vector_set(dir: &std::path::Path, listed: &[serde_json::Value], platform: &str, phone_and_pad: bool, flags: u32, preserves: bool, id: u16, out: &mut Compiled, images: &mut Vec<SetImage>) -> Result<(), String> {
    let scales: &[u32] = match platform {
        "ios" => &[1, 2, 3],
        "macosx" | "atv" => &[1, 2],
        "watch" | "vision" => &[2],
        _ => return Err(format!("{}: vector images for this platform are not supported by this actool", dir.display())),
    };
    // Per (appearance, idiom): its files, (scale or 0 without one, file).
    type Files = Vec<(u32, String)>;
    let mut bitmaps: Files = Vec::new();
    /// A file's kind, bytes and size in points.
    type Read = (crate::vector::Kind, Vec<u8>, (f64, f64));
    let mut groups: Vec<((u16, u16), Files)> = Vec::new();
    for entry in listed {
        let Some(filename) = entry["filename"].as_str() else { continue };
        if !dir.join(filename).is_file() {
            continue;
        }
        // A PNG at a scale beside a 1x vector image takes that scale's place;
        // checked below, once the set's files are known.
        if crate::vector::Kind::of(filename).is_none() {
            let png = filename.to_ascii_lowercase().ends_with(".png");
            let scale = entry["scale"].as_str().and_then(|s| s.strip_suffix('x')).and_then(|n| n.parse::<u32>().ok());
            let plain = entry.as_object().unwrap().keys().all(|k| ["filename", "idiom", "scale"].contains(&k.as_str()));
            if !png || scale.is_none_or(|s| !(2..=3).contains(&s)) || !plain || entry["idiom"] != "universal" || preserves {
                return Err(format!("{}: vector and bitmap images in one image set are not supported by this actool but for PNGs at 2x or 3x beside a universal 1x vector image", dir.display()));
            }
            bitmaps.push((scale.expect("checked"), filename.to_string()));
            continue;
        }
        for k in entry.as_object().unwrap().keys() {
            if !["filename", "idiom", "scale", "appearances"].contains(&k.as_str()) {
                return Err(format!("{}: the {k} key of a vector image is not supported by this actool", dir.display()));
            }
        }
        // Universal, or the Mac's own idiom on macOS (keyed as universal).
        let idiom = match (entry["idiom"].as_str(), platform) {
            (Some("universal"), _) | (Some("mac"), "macosx") => 0,
            (Some("iphone"), "ios") if phone_and_pad => 1,
            (Some("ipad"), "ios") if phone_and_pad => 2,
            _ => return Err(format!("{}: vector images for the idiom {} are not supported by this actool", dir.display(), entry["idiom"])),
        };
        let scale = match entry["scale"].as_str() {
            None => 0,
            Some(s) => s.strip_suffix('x').and_then(|n| n.parse::<u32>().ok()).filter(|n| (1..=3).contains(n)).ok_or_else(|| format!("{}: scale {s} is not supported by this actool", dir.display()))?,
        };
        let appearance = crate::colorset::appearance_for(entry, platform)
            .ok_or_else(|| format!("{}: the appearances {} are not supported by this actool", dir.display(), entry["appearances"]))?;
        let key = (appearance, idiom);
        let at = match groups.iter().position(|g| g.0 == key) {
            Some(at) => at,
            None => {
                groups.push((key, Vec::new()));
                groups.len() - 1
            }
        };
        let files = &mut groups[at].1;
        if files.iter().any(|f| f.0 == scale || (f.0 == 0) != (scale == 0)) {
            return Err(format!("{}: two vector images for one scale, or one without a scale beside others, for one appearance and idiom are not supported by this actool", dir.display()));
        }
        files.push((scale, filename.to_string()));
    }
    // Beside one without a scale, Xcode reports the error it reports for a
    // bitmap without one (a probe), which fails rules_apple's build.
    if !bitmaps.is_empty() && groups.iter().any(|g| g.1.iter().any(|f| f.0 == 0)) {
        return Err(format!("{}: error: Image set has a child with bitmap content and the \"Any\" scale. The image set also has children with specific scales and content. Please remove the content with specific scales or change the content of the \"Any\" scale to vector content.", dir.display()));
    }
    if !bitmaps.is_empty() && (groups.len() != 1 || groups[0].0 != (0, 0) || groups[0].1.iter().any(|f| f.0 != 1)) {
        return Err(format!("{}: PNGs beside vector images other than one universal 1x one are not supported by this actool", dir.display()));
    }
    for (scale, name) in &bitmaps {
        if !scales.contains(scale) {
            continue;
        }
        let path = dir.join(name);
        let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let (image, extended) = image::decode_png_extended(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        if extended.is_some() {
            return Err(format!("{}: a PNG with colours outside sRGB beside vector images is not supported by this actool", path.display()));
        }
        images.push(SetImage { scale: *scale, appearance: 0, idiom: 0, identifier: id, flags, filename: name.clone(), image, extended: None, slicing: None, attrs: None });
    }
    for ((appearance, idiom), files) in groups {
        // The platform's: a file without a scale is its 1x.
        let files: Vec<(u32, String)> = files.into_iter().map(|(s, f)| (s.max(1), f)).filter(|(s, _)| scales.contains(s) || *s == 1).collect();
        let used: Vec<&(u32, String)> = files.iter().filter(|(s, _)| scales.contains(s)).collect();
        let Some(highest) = used.iter().max_by_key(|f| f.0).copied().or_else(|| files.iter().find(|f| f.0 == 1)) else {
            continue;
        };
        let read = |name: &str| -> Result<Read, String> {
            let path = dir.join(name);
            let kind = crate::vector::Kind::of(name).expect("a vector image");
            let data = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let points = crate::vector::size(kind, &data).map_err(|e| format!("{}: {e}", path.display()))?;
            Ok((kind, data, points))
        };
        let attrs_of = |part: u16, scale: Option<u16>| {
            let mut attrs = vec![(car::ATTR_IDENTIFIER, id), (car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, part)];
            attrs.extend(scale.map(|s| (car::ATTR_SCALE, s)));
            if appearance != 0 {
                attrs.push((car::ATTR_APPEARANCE, appearance));
            }
            if idiom != 0 {
                attrs.push((car::ATTR_IDIOM, idiom));
            }
            attrs
        };
        if preserves {
            for (scale, name) in &files {
                let (kind, data, _) = read(name)?;
                out.renditions.push(Rendition { attrs: attrs_of(car::PART_VECTOR_IMAGE, Some(*scale as u16)), csi: crate::vector::csi(name, kind, &data) });
            }
        } else {
            let (kind, data, _) = read(&highest.1)?;
            out.renditions.push(Rendition { attrs: attrs_of(car::PART_IMAGE, None), csi: crate::vector::csi(&highest.1, kind, &data) });
        }
        let one = files.iter().find(|f| f.0 == 1);
        for &scale in scales {
            if bitmaps.iter().any(|b| b.0 == scale) {
                continue;
            }
            let (from_scale, name) = match used.iter().find(|f| f.0 == scale) {
                Some(f) => (f.0, &f.1),
                None => one.or(Some(highest)).map(|f| (f.0, &f.1)).expect("a file"),
            };
            let (kind, data, points) = read(name)?;
            let path = dir.join(name);
            let size = (points.0 / from_scale as f64, points.1 / from_scale as f64);
            // An SVG is drawn at its own size times the scale.
            let drawn = match kind {
                crate::vector::Kind::Pdf => size,
                crate::vector::Kind::Svg => points,
            };
            let pixels = (crate::vector::pixels(kind, drawn.0, scale as f64), crate::vector::pixels(kind, drawn.1, scale as f64));
            let image = crate::vector::render(kind, &data, points, scale as f64 * drawn.0 / points.0, pixels).map_err(|e| format!("{}: {e}", path.display()))?;
            // The slice, where it is short of the bitmap: from its top-left
            // corner (y counting from the bottom).
            let slice = (crate::vector::slice_pixels(size.0, scale as f64), crate::vector::slice_pixels(size.1, scale as f64));
            let slicing = (slice != pixels).then(|| car::Slicing { layout: car::LAYOUT_ICON_IMAGE, slices: vec![[0, pixels.1 - slice.1, slice.0, slice.1]] });
            images.push(SetImage { scale, appearance, idiom, identifier: id, flags: flags | 4 | if preserves { 256 } else { 0 }, filename: name.to_string(), image, extended: None, slicing, attrs: None });
        }
    }
    Ok(())
}

fn image_attrs(i: &SetImage) -> Vec<(u16, u16)> {
    if let Some(attrs) = &i.attrs {
        return attrs.clone();
    }
    let mut attrs = vec![(car::ATTR_SCALE, i.scale as u16), (car::ATTR_IDENTIFIER, i.identifier), (car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_IMAGE)];
    if i.appearance != 0 {
        attrs.push((car::ATTR_APPEARANCE, i.appearance));
    }
    if i.idiom != 0 {
        attrs.push((car::ATTR_IDIOM, i.idiom));
    }
    attrs
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
    Ok(Rendition { attrs: image_attrs16(i), csi: car::dmp2_pixels_csi(&i.filename, w as u32, h as u32, i.scale, i.flags, car::Pixels::Half, opaque, &data, None) })
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
        return Ok(Rendition { attrs: image_attrs(i), csi: car::dmp2_pixels_csi(&i.filename, w as u32, h as u32, i.scale, i.flags, car::Pixels::Gray, opaque, &data, i.slicing.as_ref()) });
    }
    let data = payload(w, h, &i.image.premultiplied_bgra(), &i.filename)?;
    Ok(Rendition { attrs: image_attrs(i), csi: car::dmp2_pixels_csi(&i.filename, w as u32, h as u32, i.scale, i.flags, car::Pixels::Bgra, true, &data, i.slicing.as_ref()) })
}

/// Whether the image is a bitmap drawn from a vector image (vector_set).
fn drawn(i: &SetImage) -> bool {
    i.flags & 4 != 0
}

/// One atlas of a scale's packable images, as `dmp2`, and a link to it per
/// image.
fn is_opaque(i: &SetImage) -> bool {
    i.image.data.chunks_exact(4).all(|p| p[3] == 255)
}

#[allow(clippy::too_many_arguments)]
fn pack(scale: u32, (appearance, idiom): (u16, u16), opaque: bool, gray: bool, index: u16, images: &[&SetImage], renditions: &mut Vec<Rendition>) -> Result<(), String> {
    let blocks: Vec<(u32, u32)> = images.iter().map(|i| (i.image.width as u32, i.image.height as u32)).collect();
    let (positions, (rw, rh)) = crate::iconset::pack::pack(&blocks);
    let (aw, ah) = ((rw + 1).div_ceil(2) * 2, (rh + 1).div_ceil(2) * 2);
    let stride = aw as usize * 4;
    let mut pixels = vec![0u8; stride * ah as usize];
    let mut key = vec![(car::ATTR_ELEMENT, car::ELEMENT_PACKED), (car::ATTR_PART, car::PART_PACKED), (car::ATTR_SCALE, scale as u16)];
    if index > 0 {
        key.push((car::ATTR_DIMENSION1, index));
    }
    if appearance != 0 || idiom != 0 {
        // An idiom's atlases take dimension1 0, 1, 2, ... in turn, as the
        // universal ones do (an iPhone icon set's atlas after iPhone image
        // sets' translucent one, a probe; three and four for an idiom,
        // opaque, translucent and grey, in seven random catalogs and a
        // corpus one). An appearance's second isn't established.
        if index > 0 && appearance != 0 {
            return Err(format!("more than one {scale}x atlas for an appearance is not supported by this actool"));
        }
        if appearance != 0 && idiom != 0 {
            return Err("an atlas for both an appearance and an idiom is not supported by this actool".into());
        }
    }
    if appearance != 0 {
        key.push((car::ATTR_APPEARANCE, appearance));
    }
    if idiom != 0 {
        key.push((car::ATTR_IDIOM, idiom));
    }
    for (i, (x, y)) in images.iter().zip(&positions) {
        let (w, h) = (i.image.width as u32, i.image.height as u32);
        crate::iconset::place(&mut pixels, stride, ah, &i.image.premultiplied_bgra(), *x, *y, w, h);
        let pixels = if gray { car::Pixels::Gray } else { car::Pixels::Bgra };
        renditions.push(Rendition { attrs: image_attrs(i), csi: car::link_csi_format(&i.filename, (*x, *y, w, h), scale, i.flags, pixels, &key, i.slicing.as_ref()) });
    }
    let name = format!("ZZZZPackedAsset-{scale}.{}.{}-gamut0", opaque as u8, gray as u8);
    // An atlas of an icon set's icons alone is encoded as the icon set's
    // own atlases are (a probe; beside image sets' images, as theirs, two).
    if !gray && images.iter().all(|i| i.attrs.is_some()) {
        let padded = car::stride(aw as usize);
        let mut rows = vec![0u8; padded * ah as usize];
        for (row, src) in rows.chunks_exact_mut(padded).zip(pixels.chunks_exact(stride)) {
            row[..stride].copy_from_slice(src);
        }
        renditions.push(Rendition { attrs: key, csi: car::packed_csi(&name, aw, ah, scale, &rows, car::MlecVersions::opacity(opaque)) });
        return Ok(());
    }
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
fn pack16(scale: u32, (appearance, idiom): (u16, u16), opaque: bool, index: u16, images: &[&SetImage], renditions: &mut Vec<Rendition>) -> Result<(), String> {
    let blocks: Vec<(u32, u32)> = images.iter().map(|i| (i.image.width as u32, i.image.height as u32)).collect();
    let (positions, (rw, rh)) = crate::iconset::pack::pack(&blocks);
    let (aw, ah) = ((rw + 1).div_ceil(2) * 2, (rh + 1).div_ceil(2) * 2);
    let mut halves = vec![0u16; 4 * (aw * ah) as usize];
    let mut alpha = vec![0u8; (aw * ah) as usize];
    let mut key = vec![(car::ATTR_ELEMENT, car::ELEMENT_PACKED), (car::ATTR_PART, car::PART_PACKED), (car::ATTR_SCALE, scale as u16), (car::ATTR_GAMUT, 1)];
    if index > 0 {
        key.push((car::ATTR_DIMENSION1, index));
    }
    if appearance != 0 || idiom != 0 {
        // An idiom's second atlas takes dimension1 1 (an iPhone icon set's
        // atlas after iPhone image sets' translucent one, a probe).
        if index > 0 && (appearance != 0 || index > 1) {
            return Err(format!("more than one {scale}x atlas for an appearance, or more than two for an idiom, is not supported by this actool"));
        }
        if appearance != 0 && idiom != 0 {
            return Err("an atlas for both an appearance and an idiom is not supported by this actool".into());
        }
    }
    if appearance != 0 {
        key.push((car::ATTR_APPEARANCE, appearance));
    }
    if idiom != 0 {
        key.push((car::ATTR_IDIOM, idiom));
    }
    for (i, (x, y)) in images.iter().zip(&positions) {
        let (w, h) = (i.image.width, i.image.height);
        let source = i.extended.as_ref().expect("a 16-bit rendition");
        let source_alpha: Vec<u8> = i.image.data.chunks_exact(4).map(|p| p[3]).collect();
        crate::iconset::place_n(&mut halves, aw as usize, 4, ah, source, *x, *y, w as u32, h as u32);
        crate::iconset::place_n(&mut alpha, aw as usize, 1, ah, &source_alpha, *x, *y, w as u32, h as u32);
        renditions.push(Rendition {
            attrs: image_attrs16(i),
            csi: car::link_csi_format(&i.filename, (*x, *y, w as u32, h as u32), scale, i.flags, car::Pixels::Half, &key, None),
        });
    }
    let name = format!("ZZZZPackedAsset-{scale}.{}.0-gamut1", opaque as u8);
    let data = dmp2::encode16(aw as usize, ah as usize, &halves, &alpha, opaque).map_err(|e| format!("{name}: {e}"))?;
    renditions.push(Rendition { attrs: key, csi: car::packed_dmp2_csi(&name, aw, ah, scale, car::Pixels::Half, opaque, &data) });
    Ok(())
}
