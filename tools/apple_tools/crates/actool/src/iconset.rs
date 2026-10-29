//! App icon sets with an image per size, as Xcode packs them into atlases;
//! see docs/clean-room/actool.md#packed-images.

use crate::car::{self, Rendition};
use crate::image;
use std::path::{Path, PathBuf};

pub struct Slot {
    pub idiom: &'static str,
    /// CoreUI's subtype: a watch size, or 0.
    pub subtype: u16,
    /// Points, as written ("83.5").
    pub size: f64,
    pub scale: u32,
    pub file: PathBuf,
    pub filename: String,
    /// Its image, when it isn't read from `file`.
    pub image: Option<image::Rgba>,
}

/// A rendition to pack or store: its group, `dimension2` and pixels.
struct Item {
    scale: u32,
    idiom: u16,
    subtype: u16,
    dimension2: u16,
    filename: String,
    image: image::Rgba,
}

/// A device's entry in the partial Info.plist's icons: the bundle's flat
/// icons it lists, and whether it names the catalog's icon.
pub struct DeviceIcons {
    pub files: Vec<String>,
    pub named: bool,
}

pub struct Compiled {
    pub renditions: Vec<Rendition>,
    /// A macOS app's icon family.
    pub icns: Option<Vec<u8>>,
    /// Files copied into the bundle: (output name, source).
    pub bundle_files: Vec<(String, PathBuf)>,
    pub phone: Option<DeviceIcons>,
    pub pad: Option<DeviceIcons>,
    pub has_dimension1: bool,
}

fn idiom_value(idiom: &str) -> u16 {
    match idiom {
        "iphone" => car::IDIOM_PHONE,
        "watch" => car::IDIOM_WATCH,
        "ios-marketing" => car::IDIOM_MARKETING,
        "car" => car::IDIOM_CAR,
        "mac" => 0,
        _ => car::IDIOM_PAD,
    }
}

/// The growing binary-tree packer (Jake Gordon's), with 2 px gutters:
/// blocks of (w + 2, h + 2), placed 2 px in.
pub(crate) mod pack {
    struct Node {
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        used: bool,
        right: Option<Box<Node>>,
        down: Option<Box<Node>>,
    }

    impl Node {
        fn new(x: u32, y: u32, w: u32, h: u32) -> Node {
            Node { x, y, w, h, used: false, right: None, down: None }
        }

        fn find(&mut self, w: u32, h: u32) -> Option<&mut Node> {
            if self.used {
                if let Some(n) = self.right.as_mut().and_then(|r| r.find(w, h)) {
                    return Some(n);
                }
                return self.down.as_mut().and_then(|d| d.find(w, h));
            }
            if w <= self.w && h <= self.h {
                Some(self)
            } else {
                None
            }
        }

        fn split(&mut self, w: u32, h: u32) -> (u32, u32) {
            self.used = true;
            self.down = Some(Box::new(Node::new(self.x, self.y + h, self.w, self.h - h)));
            self.right = Some(Box::new(Node::new(self.x + w, self.y, self.w - w, h)));
            (self.x, self.y)
        }
    }

    /// Positions of `blocks` (in packing order) and the root's size.
    pub fn pack(blocks: &[(u32, u32)]) -> (Vec<(u32, u32)>, (u32, u32)) {
        let mut root: Option<Node> = None;
        let mut out = Vec::new();
        for &(bw, bh) in blocks {
            let (w, h) = (bw + 2, bh + 2);
            let r = root.get_or_insert_with(|| Node::new(0, 0, w, h));
            if let Some(n) = r.find(w, h) {
                let (x, y) = n.split(w, h);
                out.push((x + 2, y + 2));
                continue;
            }
            let (rw, rh) = (r.w, r.h);
            let can_down = w <= rw;
            let can_right = h <= rh;
            let should_right = can_right && rh >= rw + w;
            let should_down = can_down && rw >= rh + h;
            let old = root.take().unwrap();
            let mut grown = if should_right || (!should_down && can_right) {
                let mut g = Node::new(0, 0, rw + w, rh);
                g.used = true;
                g.down = Some(Box::new(old));
                g.right = Some(Box::new(Node::new(rw, 0, w, rh)));
                g
            } else {
                let mut g = Node::new(0, 0, rw, rh + h);
                g.used = true;
                g.right = Some(Box::new(old));
                g.down = Some(Box::new(Node::new(0, rh, rw, h)));
                g
            };
            let (x, y) = grown.find(w, h).expect("a grown root fits the block").split(w, h);
            out.push((x + 2, y + 2));
            root = Some(grown);
        }
        let r = root.expect("at least one block");
        (out, (r.w, r.h))
    }
}

/// Copies `img` into an atlas of `aw` x `ah` (rows bottom-up, as Core
/// Graphics draws them) at `x`, `y`, repeating its edge pixels one pixel out.
#[allow(clippy::too_many_arguments)]
pub(crate) fn place(atlas: &mut [u8], stride: usize, ah: u32, img: &[u8], x: u32, y: u32, w: u32, h: u32) {
    place_n(atlas, stride / 4, 4, ah, img, x, y, w, h);
}

/// [`place`] for pixels of `n` elements of any kind, `width` pixels a row.
#[allow(clippy::too_many_arguments)]
pub(crate) fn place_n<T: Copy>(atlas: &mut [T], width: usize, n: usize, ah: u32, img: &[T], x: u32, y: u32, w: u32, h: u32) {
    let top = (ah - y - h) as i64;
    for r in -1..=h as i64 {
        let sr = r.clamp(0, h as i64 - 1) as usize;
        for c in -1..=w as i64 {
            let sc = c.clamp(0, w as i64 - 1) as usize;
            let (ax, ay) = (x as i64 + c, top + r);
            let at = (ay as usize * width + ax as usize) * n;
            let from = (sr * w as usize + sc) * n;
            atlas[at..at + n].copy_from_slice(&img[from..from + n]);
        }
    }
}

/// Compiles an icon set for the target devices; `mac` for a macOS catalog,
/// whose renditions have no idiom or subtype.
pub fn compile(name: &str, identifier: u16, slots: &[Slot], phone: bool, pad: bool, watch: bool, mac: bool) -> Result<Compiled, String> {
    compile_with(name, identifier, slots, phone, pad, watch, mac, true)
}

/// [`compile`], with a macOS catalog's icon family left out unless `icns`.
/// Without it, the images are an Icon Composer document's flattened ones,
/// which Xcode stores in colour even when grey (two probes).
#[allow(clippy::too_many_arguments)]
pub fn compile_with(name: &str, identifier: u16, slots: &[Slot], phone: bool, pad: bool, watch: bool, mac: bool, icns: bool) -> Result<Compiled, String> {
    let wanted = |s: &&Slot| {
        (s.idiom == "iphone" && phone)
            || (s.idiom == "ipad" && pad)
            || (s.idiom == "watch" && watch)
            || (s.idiom == "mac" && mac)
            || (s.idiom == "ios-marketing" && (phone || pad))
            || (s.idiom == "car" && phone)
    };
    // CarPlay icons go with an iPhone build; without one isn't established.
    if !phone && slots.iter().any(|s| s.idiom == "car") {
        return Err(format!("{name}: CarPlay icons without --target-device iphone are not supported by this actool"));
    }
    let version = car::MlecVersions::for_platform(if mac { "macosx" } else { "" });
    // Rendition keys: macOS catalogs have no idiom or subtype attribute.
    let group_attrs = |idiom: u16, subtype: u16| if mac { Vec::new() } else { vec![(car::ATTR_IDIOM, idiom), (car::ATTR_SUBTYPE, subtype)] };
    // With no image for any target device, Xcode compiles another device's
    // (an iPad set for iPhone alone), which isn't established.
    if !slots.iter().any(|s| wanted(&s) && s.idiom != "ios-marketing") && slots.iter().any(|s| s.idiom != "ios-marketing") {
        return Err(format!("{name}: an icon set with no image for the target devices is not supported by this actool"));
    }
    let slots: Vec<&Slot> = slots.iter().filter(wanted).collect();
    let xr = slots.iter().find(|s| s.idiom == "iphone" && s.size == 60.0 && s.scale == 3).copied();
    // dimension2: a size's rank among all of them, the 90 pt iPhone XR size
    // derived from 60 pt @3x included.
    let mut sizes: Vec<f64> = slots.iter().map(|s| s.size).collect();
    if xr.is_some() {
        sizes.push(90.0);
    }
    sizes.sort_by(|a, b| a.partial_cmp(b).unwrap());
    sizes.dedup();
    let rank = |size: f64| (sizes.iter().position(|s| *s == size).unwrap() + 1) as u16;

    let mut items = Vec::new();
    let mut seen = Vec::new();
    for s in &slots {
        if seen.contains(&(s.idiom, s.subtype, s.size.to_bits(), s.scale)) {
            return Err(format!("{}: two images for one size and scale", s.filename));
        }
        seen.push((s.idiom, s.subtype, s.size.to_bits(), s.scale));
        let img = match &s.image {
            Some(img) => img.clone(),
            None => {
                let bytes = std::fs::read(&s.file).map_err(|e| format!("{}: {e}", s.file.display()))?;
                image::decode_png_srgb(&bytes).map_err(|e| format!("{}: {e}", s.file.display()))?
            }
        };
        let px = (s.size * s.scale as f64).round() as usize;
        if img.width != px || img.height != px {
            return Err(format!("{}: is {}x{} but should be {px}x{px}", s.file.display(), img.width, img.height));
        }
        items.push(Item { scale: s.scale, idiom: idiom_value(s.idiom), subtype: s.subtype, dimension2: rank(s.size), filename: s.filename.clone(), image: img });
    }
    if let Some(s) = xr {
        let img = items.iter().find(|i| i.filename == s.filename && i.scale == 3 && i.idiom == car::IDIOM_PHONE).unwrap().image.clone();
        items.push(Item { scale: 2, idiom: car::IDIOM_PHONE, subtype: 1792, dimension2: rank(90.0), filename: s.filename.clone(), image: img });
    }

    // Xcode stores grey icons as GA8: their atlases are named `S.O.1`, and
    // even those under 40 pixels are packed (one icon set, QReader's). A
    // set mixing grey and colour icons, a grey one for macOS and a grey one
    // stored on its own under 40 pixels are not established.
    let gray = icns && items.first().is_some_and(|i| i.image.is_gray());
    if icns && (items.iter().any(|i| i.image.is_gray() != gray) || (gray && mac)) {
        return Err(format!("{name}: grey icons beside colour ones, or for macOS, are not supported by this actool"));
    }
    let ga = |i: &Item| -> Vec<u8> { i.image.premultiplied_bgra().chunks_exact(4).flat_map(|p| [p[0], p[3]]).collect() };

    let mut renditions = Vec::new();
    // The index of sizes per idiom (and for the derived XR size).
    let mut groups: Vec<(u16, u16)> = items.iter().map(|i| (i.idiom, i.subtype)).collect();
    groups.sort();
    groups.dedup();
    for (idiom, subtype) in &groups {
        let mut entries: Vec<(u32, u32, u32)> = Vec::new();
        for s in &slots {
            if s.subtype == *subtype && idiom_value(s.idiom) == *idiom {
                let p = s.size as u32;
                let e = (p, p, rank(s.size) as u32);
                if !entries.contains(&e) {
                    entries.push(e);
                }
            }
        }
        if *subtype == 1792 {
            entries.push((90, 90, rank(90.0) as u32));
        }
        entries.sort();
        let mut attrs = vec![(car::ATTR_SCALE, 1)];
        attrs.extend(group_attrs(*idiom, *subtype));
        attrs.extend([(car::ATTR_IDENTIFIER, identifier), (car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_MULTISIZED_IMAGE)]);
        renditions.push(Rendition { attrs, csi: car::multisized_csi(name, &entries) });
    }

    // Per (scale, idiom, subtype): largest area first; next-fit atlases up
    // to 18,050 px² per scale; images over that, or a group's only
    // packable one, stored on their own.
    let mut has_dimension1 = false;
    let mut keys: Vec<(u32, u16, u16)> = items.iter().map(|i| (i.scale, i.idiom, i.subtype)).collect();
    keys.sort();
    keys.dedup();
    for (scale, idiom, subtype) in keys {
        let mut group: Vec<&Item> = items.iter().filter(|i| (i.scale, i.idiom, i.subtype) == (scale, idiom, subtype)).collect();
        group.sort_by_key(|i| std::cmp::Reverse(i.image.width * i.image.height));
        if group.windows(2).any(|w| w[0].image.width * w[0].image.height == w[1].image.width * w[1].image.height) {
            return Err(format!("{name}: icons of equal area in one atlas are not supported by this actool"));
        }
        let cap = 18_050 * scale as usize;
        // For iOS, translucent and opaque icons are packed apart, as image
        // sets' are: an opacity's only packable icon is stored on its own
        // (Flutter's gallery). Both needing atlases is not established.
        let (translucent, opaque): (Vec<&Item>, Vec<&Item>) = group.into_iter().partition(|i| !mac && !opaque_image(&i.image));
        let mut standalone = Vec::new();
        let mut atlases: Vec<Vec<&Item>> = Vec::new();
        for part in [translucent, opaque] {
            let (big, packable): (Vec<&Item>, Vec<&Item>) = part.into_iter().partition(|i| i.image.width * i.image.height > cap);
            standalone.extend(big);
            if packable.len() < 2 {
                standalone.extend(packable);
                continue;
            }
            if !atlases.is_empty() {
                return Err(format!("{name}: atlases of both translucent and opaque icons at one scale are not supported by this actool"));
            }
            let mut total = 0;
            for i in packable {
                if atlases.is_empty() || total >= cap {
                    atlases.push(Vec::new());
                    total = 0;
                }
                total += i.image.width * i.image.height;
                atlases.last_mut().unwrap().push(i);
            }
        }
        let image_attrs = |i: &Item| {
            let mut a = vec![(car::ATTR_SCALE, scale as u16)];
            a.extend(group_attrs(idiom, subtype));
            a.extend([(car::ATTR_DIMENSION2, i.dimension2), (car::ATTR_IDENTIFIER, identifier), (car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_ICON_IMAGE)]);
            a
        };
        for (n, atlas) in atlases.iter().enumerate() {
            has_dimension1 |= n > 0;
            if mac && n > 0 {
                return Err(format!("{name}: more than one atlas per scale is not supported by this actool for macOS"));
            }
            let blocks: Vec<(u32, u32)> = atlas.iter().map(|i| (i.image.width as u32, i.image.height as u32)).collect();
            let (positions, (rw, rh)) = pack::pack(&blocks);
            let (aw, ah) = ((rw + 1).div_ceil(2) * 2, (rh + 1).div_ceil(2) * 2);
            let stride = if gray { (aw as usize * 2).div_ceil(32) * 32 } else { car::stride(aw as usize) };
            let mut pixels = vec![0u8; stride * ah as usize];
            // An opaque grey atlas is opaque throughout, its padding too
            // (grey 0, alpha 255), as an image set's is; not the bytes
            // aligning its rows.
            if gray && atlas.iter().all(|i| opaque_image(&i.image)) {
                for row in pixels.chunks_exact_mut(stride) {
                    row[..aw as usize * 2].chunks_exact_mut(2).for_each(|p| p[1] = 255);
                }
            }
            let mut key = vec![(car::ATTR_ELEMENT, car::ELEMENT_PACKED), (car::ATTR_PART, car::PART_PACKED), (car::ATTR_SCALE, scale as u16)];
            if !mac {
                key.extend([(car::ATTR_DIMENSION1, n as u16), (car::ATTR_IDIOM, idiom), (car::ATTR_SUBTYPE, subtype)]);
            }
            for (i, (x, y)) in atlas.iter().zip(&positions) {
                let (w, h) = (i.image.width as u32, i.image.height as u32);
                if gray {
                    place_n(&mut pixels, stride / 2, 2, ah, &ga(i), *x, *y, w, h);
                    renditions.push(Rendition { attrs: image_attrs(i), csi: car::link_csi_format(&i.filename, (*x, *y, w, h), scale, 0, car::Pixels::Gray, &key, None) });
                } else {
                    place(&mut pixels, stride, ah, &i.image.premultiplied_bgra(), *x, *y, w, h);
                    renditions.push(Rendition { attrs: image_attrs(i), csi: car::link_csi(&i.filename, (*x, *y, w, h), scale, &key) });
                }
            }
            if mac && stride * (ah as usize) < 16384 {
                return Err(format!("{name}: small atlases are not supported by this actool for macOS"));
            }
            // An iOS atlas's name and MLEC version say whether its icons
            // are all opaque; one mixing opaque and translucent icons is
            // translucent (Flutter's gallery).
            let opaque = !mac && atlas.iter().all(|i| opaque_image(&i.image));
            let atlas_name = format!("ZZZZPackedAsset-{scale}.{}.{}-gamut0", opaque as u8, gray as u8);
            let atlas_version = if mac { version } else { car::MlecVersions::opacity(opaque) };
            let csi = if gray {
                car::packed_gray_csi(&atlas_name, aw, ah, scale, &pixels, stride, atlas_version)
            } else {
                car::packed_csi(&atlas_name, aw, ah, scale, &pixels, atlas_version)
            };
            renditions.push(Rendition { attrs: key.clone(), csi });
        }
        standalone.sort_by_key(|i| i.dimension2);
        for i in standalone {
            let (w, h) = (i.image.width as u32, i.image.height as u32);
            if mac && car::stride(w as usize) * (h as usize) < 16384 {
                return Err(format!("{}: small standalone images are not supported by this actool for macOS", i.filename));
            }
            // Below 40 pixels wide Xcode run-length encodes an iOS icon.
            let csi = if gray {
                if w < 40 {
                    return Err(format!("{}: a grey icon under 40 pixels stored on its own is not supported by this actool", i.filename));
                }
                car::icon_image_gray_csi(&i.filename, w, h, scale, &ga(i), car::MlecVersions::opacity(opaque_image(&i.image)))
            } else if !mac && w < 40 {
                car::icon_image_rle_csi(&i.filename, w, h, scale, &i.image.premultiplied_bgra(), opaque_image(&i.image))
            } else {
                let image_version = if mac { version } else { car::MlecVersions::opacity(opaque_image(&i.image)) };
                car::icon_image_csi(&i.filename, w, h, scale, &i.image.premultiplied_bgra(), image_version)
            };
            renditions.push(Rendition { attrs: image_attrs(i), csi });
        }
    }

    // The bundle's icons, copied as given: 60pt@2x for iPhone, 76pt@2x for
    // iPad. Each target device's entry names the icon and lists those it
    // has (iPad's, when it has its own, the iPhone one first when both are
    // targets); for iPad alone, an App Store icon also names the icon for
    // iPhone.
    let mut bundle_files = Vec::new();
    let (mut phone_icons, mut pad_icons) = (None, None);
    let find = |idiom: &str, size: f64| slots.iter().find(|s| s.idiom == idiom && s.size == size && s.scale == 2).map(|s| s.file.clone());
    let phone_file = if phone { find("iphone", 60.0) } else { None };
    if let Some(f) = &phone_file {
        bundle_files.push((format!("{name}60x60@2x.png"), f.clone()));
    }
    if phone {
        phone_icons = Some(DeviceIcons { files: phone_file.iter().map(|_| format!("{name}60x60")).collect(), named: true });
    }
    if pad {
        let mut files = Vec::new();
        if let Some(f) = find("ipad", 76.0) {
            bundle_files.push((format!("{name}76x76@2x~ipad.png"), f));
            files.extend(phone_file.iter().map(|_| format!("{name}60x60")));
            files.push(format!("{name}76x76"));
        }
        if !phone && slots.iter().any(|s| s.idiom == "ios-marketing") {
            phone_icons = Some(DeviceIcons { files: Vec::new(), named: true });
        }
        pad_icons = Some(DeviceIcons { files, named: true });
    }
    // A macOS app's icon family: its 16 and 128 pt images at 1x and 2x, in
    // the order Xcode writes them.
    let icns = if mac && icns {
        let image = |size: f64, scale: u32| -> Result<&image::Rgba, String> {
            items.iter().find(|i| i.scale == scale && i.dimension2 == rank(size) && sizes.contains(&size)).map(|i| &i.image).ok_or_else(|| format!("{name}: a macOS icon set needs a {size}pt @{scale}x image for this actool"))
        };
        Some(image::icns(&[
            (b"ic13", image::plain_png(&image(128.0, 2)?.premultiplied_round_trip(), false)),
            (b"ic11", image::plain_png(&image(16.0, 2)?.premultiplied_round_trip(), false)),
            (b"ic04", image::icns_argb(&image(16.0, 1)?.premultiplied_round_trip())),
            (b"ic07", image::plain_png(&image(128.0, 1)?.premultiplied_round_trip(), false)),
        ]))
    } else {
        None
    };
    Ok(Compiled { renditions, icns, bundle_files, phone: phone_icons, pad: pad_icons, has_dimension1 })
}

pub fn slots(icon_dir: &Path, images: &serde_json::Value) -> Result<Vec<Slot>, String> {
    let mut out = Vec::new();
    for i in images.as_array().map(Vec::as_slice).unwrap_or_default() {
        let Some(filename) = i["filename"].as_str() else { continue };
        // An image whose file is missing is left out, without a word (two
        // probes).
        if !icon_dir.join(filename).is_file() {
            continue;
        }
        // An image the set has but no slot uses (a file name, perhaps marked
        // unassigned): Xcode ignores it.
        if i.as_object().is_some_and(|o| o.keys().all(|k| k == "filename" || (k == "unassigned" && i["unassigned"] == true))) {
            continue;
        }
        // A single-size watch icon (universal, for watchOS, 1024 pt) is
        // compiled as a watch icon of that size (one probe).
        let single_watch = i["idiom"] == "universal" && i["platform"] == "watchos" && i["size"] == "1024x1024" && i.get("scale").is_none();
        let idiom = match i["idiom"].as_str() {
            _ if single_watch => "watch",
            Some("iphone") => "iphone",
            Some("ipad") => "ipad",
            Some("watch") => "watch",
            Some("mac") => "mac",
            Some("ios-marketing") => "ios-marketing",
            Some("car") => "car",
            other => return Err(format!("{filename}: the {} idiom is not supported by this actool in a multi-size icon set", other.unwrap_or("missing"))),
        };
        let size = i["size"].as_str().and_then(|s| s.split_once('x')).and_then(|(w, h)| (w == h).then(|| w.parse::<f64>().ok()).flatten()).ok_or_else(|| format!("{filename}: unsupported size"))?;
        let scale = if single_watch { 1 } else { i["scale"].as_str().and_then(|s| s.strip_suffix('x')).and_then(|s| s.parse().ok()).ok_or_else(|| format!("{filename}: unsupported scale"))? };
        // An icon generator's `expected-size` and `folder` are ignored (one
        // probe, Valdi's).
        let allowed: &[&str] = if single_watch {
            &["filename", "idiom", "size", "platform"]
        } else if idiom == "watch" {
            &["filename", "idiom", "size", "scale", "role", "subtype", "expected-size", "folder"]
        } else {
            &["filename", "idiom", "size", "scale", "expected-size", "folder"]
        };
        if i.as_object().is_some_and(|o| o.keys().any(|k| !allowed.contains(&k.as_str()))) {
            return Err(format!("{filename}: icon set entries with other properties are not supported by this actool"));
        }
        // Xcode ignores entries that aren't iOS icon slots (Flutter's icon
        // sets have iPhone 40 and 60 pt at 1x and iPad 76 pt at 3x, which
        // are left out here too); which it ignores isn't established beyond
        // those, so only the current slots, the legacy ones (iPhone 57 pt,
        // iPad 50 and 72 pt, compiled as any other) and the App Store's
        // 1024 pt are compiled, and other slots refused.
        const IGNORED: [(&str, f64, u32); 3] = [("iphone", 40.0, 1), ("iphone", 60.0, 1), ("ipad", 76.0, 3)];
        if IGNORED.contains(&(idiom, size, scale)) {
            continue;
        }
        const PHONE: [(f64, u32); 11] = [(20.0, 2), (20.0, 3), (29.0, 1), (29.0, 2), (29.0, 3), (40.0, 2), (40.0, 3), (57.0, 1), (57.0, 2), (60.0, 2), (60.0, 3)];
        const PAD: [(f64, u32); 13] = [
            (20.0, 1), (20.0, 2), (29.0, 1), (29.0, 2), (40.0, 1), (40.0, 2), (50.0, 1), (50.0, 2), (72.0, 1), (72.0, 2), (76.0, 1), (76.0, 2), (83.5, 2),
        ];
        let known = match idiom {
            "iphone" => PHONE.contains(&(size, scale)),
            "ipad" => PAD.contains(&(size, scale)),
            "ios-marketing" => (size, scale) == (1024.0, 1),
            "car" => size == 60.0 && (scale == 2 || scale == 3),
            _ => true,
        };
        if !known {
            return Err(format!("{filename}: {idiom} {size}pt @{scale}x is not an icon slot this actool supports"));
        }
        // Watch icons by role and size: the app launcher's is stored for all
        // sizes, as companion settings' are.
        let subtype = match (idiom, i["role"].as_str(), i["subtype"].as_str()) {
            (_, None, None) if idiom != "watch" => 0,
            ("watch", None, None) if single_watch => 0,
            ("watch", Some("companionSettings"), None) => 0,
            ("watch", Some("appLauncher"), Some("38mm")) => 0,
            ("watch", Some("notificationCenter" | "quickLook"), Some("38mm")) => 320,
            ("watch", Some("notificationCenter" | "quickLook"), Some("42mm")) => 384,
            (_, role, subtype) => return Err(format!("{filename}: role {} and subtype {} are not supported by this actool", role.unwrap_or("none"), subtype.unwrap_or("none"))),
        };
        out.push(Slot { idiom, subtype, size, scale, file: icon_dir.join(filename), filename: filename.to_string(), image: None });
    }
    Ok(out)
}

fn opaque_image(image: &image::Rgba) -> bool {
    image.data.chunks_exact(4).all(|p| p[3] == 255)
}
