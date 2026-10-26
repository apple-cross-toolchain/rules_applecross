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
pub fn compile(sets: &[(String, std::path::PathBuf)], phone_and_pad: bool, identifier: impl Fn(&str) -> u16) -> Result<Compiled, String> {
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
        // No images list is as an empty one (two probes).
        for entry in contents["images"].as_array().map(Vec::as_slice).unwrap_or_default() {
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
            let idiom = match entry["idiom"].as_str() {
                Some("universal") => 0,
                Some("iphone") => 1,
                Some("ipad") => 2,
                Some("car") => 4,
                Some("watch" | "mac" | "tv") => continue,
                _ => return Err(format!("{}: images for the idiom {} are not supported by this actool", dir.display(), entry["idiom"])),
            };
            if idiom != 0 && !phone_and_pad {
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
            let appearance = crate::colorset::appearance(entry)
                .ok_or_else(|| format!("{}: the appearances {} are not supported by this actool", dir.display(), entry["appearances"]))?;
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
            images.push(SetImage { scale, appearance, idiom, identifier: id, flags, filename: filename.to_string(), image: img, extended, slicing });
        }
        if !scales.is_empty() {
            out.facets.push(Facet { name: name.clone(), attrs: vec![(car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_IMAGE), (car::ATTR_IDENTIFIER, id)] });
        }
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
            let (standalone, packable) = if packable.len() < 2 { (standalone.into_iter().chain(packable).collect::<Vec<_>>(), Vec::new()) } else { (standalone, packable) };
            if !packable.is_empty() {
                if packable.windows(2).any(|w| w[0].image.width * w[0].image.height == w[1].image.width * w[1].image.height && w[0].identifier == w[1].identifier) {
                    return Err(format!("image sets with equal identifiers and areas in one {scale}x atlas are not supported by this actool"));
                }
                // Next-fit atlases: one is closed once its area reaches the
                // cap, the next taking the following index at this scale.
                // When the next image would fit in the closed atlas without
                // growing it, Xcode sometimes keeps adding a few more, in a
                // way not established (23 probes of equal shapes: 54 x 60
                // takes three more, 60 x 54 none): refused. Where it would
                // grow the atlas, Xcode closes it as next-fit does.
                let mut total = 0;
                for i in packable {
                    if planned.last().is_none_or(|a| (a.0, a.1, a.2) != (gamut, opaque, gray)) || total >= cap {
                        if let Some(last) = planned.last().filter(|a| (a.0, a.1, a.2) == (gamut, opaque, gray)) {
                            let mut blocks: Vec<(u32, u32)> = last.3.iter().map(|i| (i.image.width as u32, i.image.height as u32)).collect();
                            let closed = crate::iconset::pack::pack(&blocks).1;
                            blocks.push((i.image.width as u32, i.image.height as u32));
                            if crate::iconset::pack::pack(&blocks).1 == closed {
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
                    pack(scale, (appearance, idiom), *opaque, *gray, index, atlas, &mut out.renditions)?;
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
/// sRGB profile); other metadata Xcode rewrites, and other profiles it
/// converts.
fn stored_jpeg(j: &[u8]) -> Result<(Vec<u8>, u32, u32, u16), String> {
    const SRGB_PROFILE_LEN: usize = 3144;
    if !j.starts_with(&[0xff, 0xd8]) {
        return Err("not a JPEG".into());
    }
    let mut out = j[..2].to_vec();
    let mut size = None;
    let mut orientation = 1;
    let mut i = 2;
    loop {
        if i + 4 > j.len() || j[i] != 0xff {
            return Err("a JPEG this actool can't read".into());
        }
        let marker = j[i + 1];
        let len = u16::from_be_bytes([j[i + 2], j[i + 3]]) as usize;
        let segment = j.get(i..i + 2 + len).ok_or("a truncated JPEG")?;
        match marker {
            // The scan and what follows it, as they are.
            0xda => {
                out.extend_from_slice(&j[i..]);
                break;
            }
            // Xcode passes metadata through ImageIO, which rewrites Exif,
            // and turns XMP's properties into Exif and IPTC. Comments and
            // Ducky (APP12) segments are dropped, Adobe (APP14) segments
            // kept, and these, established, as they are; others are refused
            // (ten probes, and the corpus's JPEGs).
            0xfe | 0xec => {}
            0xee if segment.get(4..9) == Some(b"Adobe") => out.extend_from_slice(segment),
            // XMP without properties is dropped.
            0xe1 if segment.get(4..33) == Some(b"http://ns.adobe.com/xap/1.0/\0") => {
                if segment.windows(15).any(|w| w == b"rdf:Description") {
                    return Err("a JPEG with XMP properties (which Xcode turns into Exif and IPTC) is not supported by this actool".into());
                }
            }
            // Exif with no entries is dropped; other Exif ImageIO writes
            // back as `exif` gives it, its orientation recorded.
            0xe1 if segment.get(4..10) == Some(b"Exif\0\0") => match exif(&segment[10..]) {
                Some(Exif::Empty) => {}
                Some(Exif::Tiff(tiff, o)) => {
                    orientation = o;
                    out.extend_from_slice(&[0xff, 0xe1]);
                    out.extend_from_slice(&((tiff.len() + 8) as u16).to_be_bytes());
                    out.extend_from_slice(b"Exif\0\0");
                    out.extend(tiff);
                }
                None => return Err("a JPEG with Exif entries other than those established (which Xcode rewrites) is not supported by this actool".into()),
            },
            // Photoshop's (APP13) of only an empty IPTC record and its
            // digest is dropped (one probe, one of the corpus's).
            0xed if segment.get(4..18) == Some(b"Photoshop 3.0\0") && is_empty_iptc(&segment[18..]) => {}
            0xe0 if segment.get(4..9) == Some(b"JFIF\0") => out.extend_from_slice(segment),
            0xe2 if segment.get(4..16) == Some(b"ICC_PROFILE\0") => {
                let profile = &segment[18..];
                let srgb = profile.len() == SRGB_PROFILE_LEN && profile.windows(17).any(|w| w == b"sRGB IEC61966-2.1");
                if !srgb || segment[16..18] != [1, 1] {
                    return Err("a JPEG with a colour profile other than sRGB's is not supported by this actool".into());
                }
                out.extend_from_slice(segment);
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
    let (w, h) = size.ok_or("a JPEG without a frame")?;
    Ok((out, w, h, orientation))
}

/// An IFD entry: tag, type, count and its value's bytes.
type Entry = (u16, u16, u32, Vec<u8>);

enum Exif {
    Empty,
    /// The TIFF as ImageIO writes it back, and the orientation.
    Tiff(Vec<u8>, u16),
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
            (0x8769, 4, 1) => exif_ifd = Some(long(&v) as usize),
            _ => return None,
        }
    }
    if let Some(at) = exif_ifd {
        for (tag, typ, count, v) in read(at)? {
            match (tag, typ, count) {
                (0x9000 | 0xa000 | 0x9101, 7, 4) | (0x9286, 7, _) => second.push((tag, 7, count, v)),
                (0xa001, 3, 1) => second.push((tag, 3, 1, short(&v).to_be_bytes().to_vec())),
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
    Some(Exif::Tiff(out, orientation))
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
}

fn image_attrs(i: &SetImage) -> Vec<(u16, u16)> {
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
        if index > 0 {
            return Err(format!("more than one {scale}x atlas for an appearance or idiom is not supported by this actool"));
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
        if index > 0 {
            return Err(format!("more than one {scale}x atlas for an appearance or idiom is not supported by this actool"));
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
