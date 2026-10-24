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

pub struct Compiled {
    pub renditions: Vec<Rendition>,
    /// A macOS app's icon family.
    pub icns: Option<Vec<u8>>,
    /// Files copied into the bundle: (output name, source).
    pub bundle_files: Vec<(String, PathBuf)>,
    pub phone_files: Vec<String>,
    pub pad_files: Vec<String>,
    pub has_dimension1: bool,
}

fn idiom_value(idiom: &str) -> u16 {
    match idiom {
        "iphone" => car::IDIOM_PHONE,
        "watch" => car::IDIOM_WATCH,
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
    let top = (ah - y - h) as i64;
    for r in -1..=h as i64 {
        let sr = r.clamp(0, h as i64 - 1) as usize;
        for c in -1..=w as i64 {
            let sc = c.clamp(0, w as i64 - 1) as usize;
            let (ax, ay) = (x as i64 + c, top + r);
            let at = ay as usize * stride + ax as usize * 4;
            let from = (sr * w as usize + sc) * 4;
            atlas[at..at + 4].copy_from_slice(&img[from..from + 4]);
        }
    }
}

/// Compiles an icon set for the target devices; `mac` for a macOS catalog,
/// whose renditions have no idiom or subtype.
pub fn compile(name: &str, identifier: u16, slots: &[Slot], phone: bool, pad: bool, watch: bool, mac: bool) -> Result<Compiled, String> {
    let wanted = |s: &&Slot| (s.idiom == "iphone" && phone) || (s.idiom == "ipad" && pad) || (s.idiom == "watch" && watch) || (s.idiom == "mac" && mac);
    let version = if mac { car::chunked_version("macosx") } else { car::chunked_version("") };
    // Rendition keys: macOS catalogs have no idiom or subtype attribute.
    let group_attrs = |idiom: u16, subtype: u16| if mac { Vec::new() } else { vec![(car::ATTR_IDIOM, idiom), (car::ATTR_SUBTYPE, subtype)] };
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
        let img = image::decode_png(&std::fs::read(&s.file).map_err(|e| format!("{}: {e}", s.file.display()))?)?;
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
        let (standalone, packable): (Vec<&Item>, Vec<&Item>) = group.into_iter().partition(|i| i.image.width * i.image.height > cap);
        let (mut standalone, packable) = if packable.len() < 2 { (standalone.into_iter().chain(packable).collect::<Vec<_>>(), Vec::new()) } else { (standalone, packable) };
        let mut atlases: Vec<Vec<&Item>> = Vec::new();
        let mut total = 0;
        for i in packable {
            if atlases.is_empty() || total >= cap {
                atlases.push(Vec::new());
                total = 0;
            }
            total += i.image.width * i.image.height;
            atlases.last_mut().unwrap().push(i);
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
            let stride = car::stride(aw as usize);
            let mut pixels = vec![0u8; stride * ah as usize];
            let mut key = vec![(car::ATTR_ELEMENT, car::ELEMENT_PACKED), (car::ATTR_PART, car::PART_PACKED), (car::ATTR_SCALE, scale as u16)];
            if !mac {
                key.extend([(car::ATTR_DIMENSION1, n as u16), (car::ATTR_IDIOM, idiom), (car::ATTR_SUBTYPE, subtype)]);
            }
            for (i, (x, y)) in atlas.iter().zip(&positions) {
                let (w, h) = (i.image.width as u32, i.image.height as u32);
                place(&mut pixels, stride, ah, &i.image.premultiplied_bgra(), *x, *y, w, h);
                renditions.push(Rendition { attrs: image_attrs(i), csi: car::link_csi(&i.filename, (*x, *y, w, h), scale, &key) });
            }
            if mac && stride * (ah as usize) < 16384 {
                return Err(format!("{name}: small atlases are not supported by this actool for macOS"));
            }
            let atlas_name = format!("ZZZZPackedAsset-{scale}.{}.0-gamut0", if mac { 0 } else { 1 });
            renditions.push(Rendition { attrs: key.clone(), csi: car::packed_csi(&atlas_name, aw, ah, scale, &pixels, version) });
        }
        standalone.sort_by_key(|i| i.dimension2);
        for i in standalone {
            let (w, h) = (i.image.width as u32, i.image.height as u32);
            if mac && car::stride(w as usize) * (h as usize) < 16384 {
                return Err(format!("{}: small standalone images are not supported by this actool for macOS", i.filename));
            }
            renditions.push(Rendition { attrs: image_attrs(i), csi: car::icon_image_csi(&i.filename, w, h, scale, &i.image.premultiplied_bgra(), version) });
        }
    }

    // The bundle's icons, copied as given: 60pt@2x for iPhone, 76pt@2x for iPad.
    let mut bundle_files = Vec::new();
    let mut phone_files = Vec::new();
    let mut pad_files = Vec::new();
    let find = |idiom: &str, size: f64| slots.iter().find(|s| s.idiom == idiom && s.size == size && s.scale == 2).map(|s| s.file.clone());
    if phone {
        let f = find("iphone", 60.0).ok_or("an iPhone app icon set needs a 60pt @2x image for this actool")?;
        bundle_files.push((format!("{name}60x60@2x.png"), f));
        phone_files.push(format!("{name}60x60"));
    }
    if pad {
        let f = find("ipad", 76.0).ok_or("an iPad app icon set needs a 76pt @2x image for this actool")?;
        bundle_files.push((format!("{name}76x76@2x~ipad.png"), f));
        pad_files.push(format!("{name}60x60"));
        pad_files.push(format!("{name}76x76"));
    }
    // A macOS app's icon family: its 16 and 128 pt images at 1x and 2x, in
    // the order Xcode writes them.
    let icns = if mac {
        let image = |size: f64, scale: u32| -> Result<&image::Rgba, String> {
            items.iter().find(|i| i.scale == scale && i.dimension2 == rank(size) && sizes.contains(&size)).map(|i| &i.image).ok_or_else(|| format!("{name}: a macOS icon set needs a {size}pt @{scale}x image for this actool"))
        };
        Some(image::icns(&[
            (b"ic13", image::plain_png(image(128.0, 2)?, false)),
            (b"ic11", image::plain_png(image(16.0, 2)?, false)),
            (b"ic04", image::icns_argb(image(16.0, 1)?)),
            (b"ic07", image::plain_png(image(128.0, 1)?, false)),
        ]))
    } else {
        None
    };
    Ok(Compiled { renditions, icns, bundle_files, phone_files, pad_files, has_dimension1 })
}

pub fn slots(icon_dir: &Path, images: &serde_json::Value) -> Result<Vec<Slot>, String> {
    let mut out = Vec::new();
    for i in images.as_array().ok_or("app icon set has no images list")? {
        let Some(filename) = i["filename"].as_str() else { continue };
        let idiom = match i["idiom"].as_str() {
            Some("iphone") => "iphone",
            Some("ipad") => "ipad",
            Some("watch") => "watch",
            Some("mac") => "mac",
            other => return Err(format!("{filename}: the {} idiom is not supported by this actool in a multi-size icon set", other.unwrap_or("missing"))),
        };
        let size = i["size"].as_str().and_then(|s| s.split_once('x')).and_then(|(w, h)| (w == h).then(|| w.parse::<f64>().ok()).flatten()).ok_or_else(|| format!("{filename}: unsupported size"))?;
        let scale = i["scale"].as_str().and_then(|s| s.strip_suffix('x')).and_then(|s| s.parse().ok()).ok_or_else(|| format!("{filename}: unsupported scale"))?;
        let allowed: &[&str] = if idiom == "watch" { &["filename", "idiom", "size", "scale", "role", "subtype"] } else { &["filename", "idiom", "size", "scale"] };
        if i.as_object().is_some_and(|o| o.keys().any(|k| !allowed.contains(&k.as_str()))) {
            return Err(format!("{filename}: icon set entries with other properties are not supported by this actool"));
        }
        // Watch icons by role and size: the app launcher's is stored for all
        // sizes, as companion settings' are.
        let subtype = match (idiom, i["role"].as_str(), i["subtype"].as_str()) {
            (_, None, None) if idiom != "watch" => 0,
            ("watch", Some("companionSettings"), None) => 0,
            ("watch", Some("appLauncher"), Some("38mm")) => 0,
            ("watch", Some("notificationCenter" | "quickLook"), Some("38mm")) => 320,
            ("watch", Some("notificationCenter" | "quickLook"), Some("42mm")) => 384,
            (_, role, subtype) => return Err(format!("{filename}: role {} and subtype {} are not supported by this actool", role.unwrap_or("none"), subtype.unwrap_or("none"))),
        };
        out.push(Slot { idiom, subtype, size, scale, file: icon_dir.join(filename), filename: filename.to_string() });
    }
    Ok(out)
}
