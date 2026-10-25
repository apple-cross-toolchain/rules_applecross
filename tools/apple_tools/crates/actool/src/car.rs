//! Assets.car (CoreUI's compiled catalog), as Xcode 27's actool writes it;
//! see docs/clean-room/actool.md.

use crate::bom::{Bom, Tree};

pub const ATTR_ELEMENT: u16 = 1;
pub const ATTR_PART: u16 = 2;
pub const ATTR_SIZE: u16 = 3;
pub const ATTR_LAYER: u16 = 11;
pub const ATTR_APPEARANCE: u16 = 7;
pub const ATTR_DIMENSION1: u16 = 8;
pub const ATTR_DIMENSION2: u16 = 9;
pub const ATTR_SCALE: u16 = 12;
pub const ATTR_LOCALIZATION: u16 = 13;
pub const ATTR_IDIOM: u16 = 15;
pub const ATTR_SUBTYPE: u16 = 16;
pub const ATTR_IDENTIFIER: u16 = 17;

pub const IDIOM_PHONE: u16 = 1;
pub const IDIOM_PAD: u16 = 2;
pub const IDIOM_WATCH: u16 = 5;
/// The App Store's (`ios-marketing`) icon.
pub const IDIOM_MARKETING: u16 = 6;

pub const ELEMENT_ICON: u16 = 85;
pub const ELEMENT_PACKED: u16 = 9;
pub const PART_PACKED: u16 = 181;
pub const PART_MULTISIZED_IMAGE: u16 = 218;
pub const PART_ICON_IMAGE: u16 = 220;

const LAYOUT_ICON_IMAGE: u16 = 12;
const LAYOUT_INTERNAL_LINK: u16 = 1003;
const LAYOUT_PACKED_IMAGE: u16 = 1004;
const LAYOUT_MULTISIZED_IMAGE: u16 = 1010;

/// The versions of the CoreUI that Xcode 27.0's actool writes with.
pub const COREUI_VERSION: u32 = 1010;
pub const STORAGE_VERSION: u32 = 17;
pub const MAIN_VERSION: &str = "@(#)PROGRAM:CoreUI  PROJECT:CoreUI-1010 [LAR]";
/// The authoring tool recorded in the catalog; its CMPhoto version differs
/// by platform.
pub fn authoring_tool(platform: &str) -> String {
    let cmphoto = if platform == "ios" { 219 } else { 218 };
    format!("@(#)PROGRAM:CoreThemeDefinition  PROJECT:CoreThemeDefinition-664 [LAR] [IIO-2851][LAR] AppleJPEG[1][32]  CMPhoto[1][{cmphoto}]] [IR-120(hw)]")
}

/// A rendition: its key attributes and its CSI value.
pub struct Rendition {
    pub attrs: Vec<(u16, u16)>,
    pub csi: Vec<u8>,
}

pub struct Facet {
    pub name: String,
    pub attrs: Vec<(u16, u16)>,
}

pub struct Catalog {
    /// Attributes of the key format, in order.
    pub key_format: Vec<u16>,
    pub facets: Vec<Facet>,
    /// Appearance names and values; macOS catalogs have no table of them.
    pub appearances: Vec<(String, u16)>,
    /// CARHEADER's schema version and key semantics: 2 and 2 for iOS and
    /// its kin, 2 and 1 for macOS, 1 and 4 for visionOS.
    pub schema_version: u32,
    pub key_semantics: u32,
    pub renditions: Vec<Rendition>,
    /// "Xcode 27.0 (27A266a)".
    pub xcode: String,
    pub deployment_target: String,
    pub platform: String,
}

fn fixed(s: &str, n: usize) -> Vec<u8> {
    let mut b = s.as_bytes().to_vec();
    b.resize(n, 0);
    b
}

fn le32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn le16(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_le_bytes());
}

impl Catalog {
    fn key(&self, r: &Rendition) -> Vec<u8> {
        let mut k = Vec::with_capacity(self.key_format.len() * 2);
        for a in &self.key_format {
            let v = r.attrs.iter().find(|(x, _)| x == a).map_or(0, |p| p.1);
            le16(&mut k, v);
        }
        k
    }

    /// Whether BITMAPKEYS' order is established for this catalog. Its
    /// blocks follow a CoreFoundation table of the identifiers, filled in
    /// the order their renditions were first allocated. That reproduces
    /// Xcode's whenever the order can't depend on how collisions resolve,
    /// and in catalogs without atlases; with atlases and identifiers that
    /// share a bucket, Xcode's order sometimes differs in a way not
    /// established (23 of 570 catalogs examined). Colours collide the same
    /// way, atlases or not (25 of 54 random colour catalogs).
    pub fn bitmap_key_order_established(&self) -> bool {
        let mut ids: Vec<u16> = Vec::new();
        let mut sensitive = false;
        for r in &self.renditions {
            sensitive |= r.attrs.iter().any(|a| a.0 == ATTR_PART && a.1 == crate::colorset::PART_COLOR);
            match r.attrs.iter().find(|a| a.0 == ATTR_IDENTIFIER).map_or(0, |a| a.1) {
                0 => sensitive = true,
                id if !ids.contains(&id) => ids.push(id),
                _ => {}
            }
        }
        if !sensitive {
            return true;
        }
        let size = cf::hash::table_size_for(ids.len()) as u64;
        let mut buckets: Vec<u64> = ids.iter().map(|&i| cf::hash::number_hash(i as i64) % size).collect();
        buckets.sort_unstable();
        buckets.windows(2).all(|w| w[0] != w[1])
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bom = Bom::new();
        let header = bom.alloc(Vec::new());
        let renditions_tree = Tree::reserve(&mut bom, 4096, false);
        let facets_tree = Tree::reserve(&mut bom, 4096, false);
        let appearances_tree = (!self.appearances.is_empty()).then(|| Tree::reserve(&mut bom, 4096, false));

        let mut appearance_entries = Vec::new();
        for (name, value) in &self.appearances {
            let k = bom.alloc(name.as_bytes().to_vec());
            let v = bom.alloc(value.to_le_bytes().to_vec());
            appearance_entries.push((k, v, name.as_bytes().to_vec()));
        }
        let mut facet_entries = Vec::new();
        let mut facets: Vec<&Facet> = self.facets.iter().collect();
        facets.sort_by(|a, b| a.name.as_bytes().cmp(b.name.as_bytes()));
        for f in facets {
            let k = bom.alloc(f.name.as_bytes().to_vec());
            let mut v = Vec::new();
            le16(&mut v, 0); // hotspot x
            le16(&mut v, 0); // hotspot y
            le16(&mut v, f.attrs.len() as u16);
            for (a, x) in &f.attrs {
                le16(&mut v, *a);
                le16(&mut v, *x);
            }
            let vi = bom.alloc(v);
            facet_entries.push((k, vi, f.name.as_bytes().to_vec()));
        }

        let mut kf = b"tmfk".to_vec();
        le32(&mut kf, 0);
        le32(&mut kf, self.key_format.len() as u32);
        for a in &self.key_format {
            le32(&mut kf, *a as u32);
        }
        let key_format = bom.alloc(kf);

        // Renditions are allocated as a balanced tree of their keys, sorted
        // by value, is walked in preorder: the middle one (the upper of two),
        // then each half; the tree lists them sorted as bytes
        // (docs/clean-room/actool.md).
        let values = |r: &Rendition| -> Vec<u16> { self.key_format.iter().map(|a| r.attrs.iter().find(|(x, _)| x == a).map_or(0, |p| p.1)).collect() };
        let mut sorted: Vec<&Rendition> = self.renditions.iter().collect();
        sorted.sort_by_key(|r| values(r));
        let mut slots: Vec<Option<(u32, u32, Vec<u8>)>> = vec![None; sorted.len()];
        let mut allocated_ids: Vec<u16> = Vec::new();
        let mut pending = vec![(0, sorted.len())];
        while let Some((lo, hi)) = pending.pop() {
            if lo >= hi {
                continue;
            }
            let mid = lo + (hi - lo) / 2;
            let id = sorted[mid].attrs.iter().find(|a| a.0 == ATTR_IDENTIFIER).map_or(0, |a| a.1);
            if !allocated_ids.contains(&id) {
                allocated_ids.push(id);
            }
            let key = self.key(sorted[mid]);
            let k = bom.alloc(key.clone());
            let v = bom.alloc(sorted[mid].csi.clone());
            slots[mid] = Some((k, v, key));
            pending.push((mid + 1, hi));
            pending.push((lo, mid));
        }
        let mut rendition_entries: Vec<(u32, u32, Vec<u8>)> = slots.into_iter().map(|s| s.expect("every rendition allocated")).collect();
        rendition_entries.sort_by(|a, b| a.2.cmp(&b.2));

        let mut meta = b"META".to_vec();
        meta.extend(fixed("", 256));
        meta.extend(fixed(&self.deployment_target, 256));
        meta.extend(fixed(&self.platform, 256));
        meta.extend(fixed(&authoring_tool(&self.platform), 256));
        let extended = bom.alloc(meta);

        let bitmap_tree = Tree::reserve(&mut bom, 1024, true);
        let mut bitmap_entries = Vec::new();
        let mut identifiers: Vec<u16> = Vec::new();
        for r in &self.renditions {
            let id = r.attrs.iter().find(|a| a.0 == ATTR_IDENTIFIER).map_or(0, |a| a.1);
            // Atlases have no identifier and no bitmap key.
            if id != 0 && !identifiers.contains(&id) {
                identifiers.push(id);
            }
        }
        // Their values are allocated in the order a CoreFoundation table of
        // the identifiers enumerates, filled in the order their renditions
        // were first allocated (an atlas's 0 among them); the tree lists
        // them in order.
        let filled: Vec<i64> = allocated_ids.iter().filter(|&&i| i != 0).map(|&i| i as i64).collect();
        let order: Vec<u16> = cf::hash::number_table_order(&filled).into_iter().map(|i| filled[i] as u16).filter(|&i| i != 0).collect();
        for id in order {
            let mut v = Vec::new();
            le32(&mut v, 1);
            le32(&mut v, 0);
            le32(&mut v, 4 + 4 * self.key_format.len() as u32);
            le32(&mut v, self.key_format.len() as u32);
            for a in &self.key_format {
                let tracked = matches!(*a, ATTR_LOCALIZATION | ATTR_SCALE | ATTR_IDIOM | ATTR_SUBTYPE | ATTR_DIMENSION2 | ATTR_DIMENSION1 | ATTR_SIZE | ATTR_LAYER);
                let mask = if tracked {
                    self.renditions
                        .iter()
                        .filter(|r| r.attrs.iter().any(|x| x.0 == ATTR_IDENTIFIER && x.1 == id))
                        .map(|r| {
                            let v = r.attrs.iter().find(|x| x.0 == *a).map_or(0, |x| x.1);
                            1u32 << if *a == ATTR_SUBTYPE { subtype_bit(v) } else { v as u32 }
                        })
                        .fold(0, |m, b| m | b)
                } else {
                    0xffff_ffff
                };
                le32(&mut v, mask);
            }
            let vi = bom.alloc(v);
            bitmap_entries.push((id as u32, vi, Vec::new()));
        }

        let mut h = b"RATC".to_vec();
        le32(&mut h, COREUI_VERSION);
        le32(&mut h, STORAGE_VERSION);
        le32(&mut h, 0); // storage timestamp
        le32(&mut h, self.renditions.len() as u32);
        h.extend(fixed(MAIN_VERSION, 128));
        h.extend(fixed(&format!("{} via AssetCatalogAgent-AssetRuntime", self.xcode), 256));
        h.extend([0u8; 16]); // uuid
        le32(&mut h, 0); // associated checksum
        le32(&mut h, self.schema_version);
        le32(&mut h, 1); // color space
        le32(&mut h, self.key_semantics);
        bom.set(header, h);

        renditions_tree.finish(&mut bom, &rendition_entries);
        facets_tree.finish(&mut bom, &facet_entries);
        if let Some(t) = &appearances_tree {
            t.finish(&mut bom, &appearance_entries);
        }
        bitmap_entries.sort_by_key(|e| e.0);
        bitmap_tree.finish(&mut bom, &bitmap_entries);

        bom.var("CARHEADER", header);
        bom.var("RENDITIONS", renditions_tree.header);
        bom.var("FACETKEYS", facets_tree.header);
        if let Some(t) = &appearances_tree {
            bom.var("APPEARANCEKEYS", t.header);
        }
        bom.var("KEYFORMAT", key_format);
        bom.var("EXTENDED_METADATA", extended);
        bom.var("BITMAPKEYS", bitmap_tree.header);
        bom.to_bytes()
    }
}

/// CSI ("ISTC") header fields.
pub struct CsiHeader<'a> {
    pub flags: u32,
    pub width: u32,
    pub height: u32,
    pub scale: u32,
    pub pixel_format: &'a [u8; 4],
    pub color_space: u32,
    pub layout: u16,
    pub name: &'a str,
}

fn tlv(out: &mut Vec<u8>, kind: u32, data: &[u8]) {
    le32(out, kind);
    le32(out, data.len() as u32);
    out.extend_from_slice(data);
}

fn csi(h: &CsiHeader, tlvs: &[u8], payload: &[u8]) -> Vec<u8> {
    let mut out = b"ISTC".to_vec();
    le32(&mut out, 1);
    le32(&mut out, h.flags);
    le32(&mut out, h.width);
    le32(&mut out, h.height);
    le32(&mut out, h.scale);
    out.extend_from_slice(h.pixel_format);
    le32(&mut out, h.color_space);
    le32(&mut out, 0); // modification time
    le16(&mut out, h.layout);
    le16(&mut out, 0);
    out.extend(fixed(h.name, 128));
    le32(&mut out, tlvs.len() as u32);
    le32(&mut out, 1); // bitmap count
    le32(&mut out, 0);
    le32(&mut out, payload.len() as u32);
    out.extend_from_slice(tlvs);
    out.extend_from_slice(payload);
    out
}

/// Rows per LZFSE chunk: the bitmap in as many chunks as it has 8 KiB (part
/// chunks counting), at most three, each a whole number of rows, the
/// remainder in a fourth.
fn rows_per_chunk(bytes_per_row: usize, height: usize) -> usize {
    let chunks = std::cmp::min(3, (bytes_per_row * height).div_ceil(8192));
    std::cmp::max(1, height / chunks)
}

/// Bitmaps smaller than this are one LZFSE stream (MLEC version 2).
const CHUNKED_FROM: usize = 16384;

/// Bytes per row of a rendition's bitmap: 32-byte aligned.
pub fn stride(width: usize) -> usize {
    (width * 4).div_ceil(32) * 32
}

/// A subtype's bit in BITMAPKEYS: an index into CoreUI's list of subtypes,
/// known for these.
pub fn subtype_bit(subtype: u16) -> u32 {
    match subtype {
        0 => 0,
        320 => 8,
        384 => 9,
        1792 => 16,
        other => panic!("subtype {other} has no known BITMAPKEYS bit"),
    }
}

/// The payload of an LZFSE-compressed ARGB rendition: an MLEC header and
/// one LZFSE stream, or for larger bitmaps one KCBC chunk per group of rows,
/// each its own LZFSE stream. `bgra` rows are `stride` bytes.
fn lzfse_bitmap(bgra: &[u8], stride: usize, height: usize, versions: MlecVersions) -> Vec<u8> {
    let mut out = b"MLEC".to_vec();
    if stride * height < CHUNKED_FROM {
        let data = lzfse::encode_stream(&bgra[..stride * height]);
        le32(&mut out, versions.single);
        le32(&mut out, 4); // LZFSE
        le32(&mut out, data.len() as u32);
        out.extend_from_slice(&data);
        return out;
    }
    let per = rows_per_chunk(stride, height);
    let version = versions.chunked;
    let mut chunks = Vec::new();
    let mut y = 0;
    while y < height {
        let rows = std::cmp::min(per, height - y);
        chunks.push((rows, lzfse::encode_stream(&bgra[y * stride..(y + rows) * stride])));
        y += rows;
    }
    le32(&mut out, version);
    le32(&mut out, 4); // LZFSE
    le32(&mut out, chunks.len() as u32);
    for (rows, data) in chunks {
        out.extend_from_slice(b"KCBC");
        le32(&mut out, 0);
        le32(&mut out, 0);
        le32(&mut out, rows as u32);
        le32(&mut out, data.len() as u32);
        out.extend_from_slice(&data);
    }
    out
}

/// Pads tightly packed BGRA rows to the rendition stride.
pub fn padded(bgra: &[u8], width: usize, height: usize) -> Vec<u8> {
    let s = stride(width);
    let mut out = vec![0u8; s * height];
    for y in 0..height {
        out[y * s..y * s + width * 4].copy_from_slice(&bgra[y * width * 4..(y + 1) * width * 4]);
    }
    out
}

/// An app icon's image at one size (part 220), at `scale`.
/// MLEC's versions for a bitmap in one stream and in chunks. An iOS icon's
/// say whether it is opaque (2 and 3) or not (0 and 1); a macOS catalog's
/// chunks are 1 (docs/clean-room/actool.md).
#[derive(Clone, Copy)]
pub struct MlecVersions {
    pub single: u32,
    pub chunked: u32,
}

impl MlecVersions {
    pub fn for_platform(platform: &str) -> MlecVersions {
        MlecVersions { single: 2, chunked: if platform == "macosx" { 1 } else { 3 } }
    }

    pub fn opacity(opaque: bool) -> MlecVersions {
        if opaque { MlecVersions { single: 2, chunked: 3 } } else { MlecVersions { single: 0, chunked: 1 } }
    }
}

pub fn icon_image_csi(name: &str, width: u32, height: u32, scale: u32, bgra: &[u8], versions: MlecVersions) -> Vec<u8> {
    let mut t = Vec::new();
    let mut slices = Vec::new();
    for v in [1u32, 0, 0, width, height] {
        le32(&mut slices, v);
    }
    tlv(&mut t, 0x3e9, &slices);
    let mut metrics = Vec::new();
    for v in [1u32, 0, 0, 0, 0, width, height] {
        le32(&mut metrics, v);
    }
    tlv(&mut t, 0x3eb, &metrics);
    let mut composition = Vec::new();
    le32(&mut composition, 0);
    composition.extend_from_slice(&1.0f32.to_le_bytes());
    tlv(&mut t, 0x3ec, &composition);
    tlv(&mut t, 0x3ee, &1u32.to_le_bytes());
    let s = stride(width as usize);
    tlv(&mut t, 0x3ef, &(s as u32).to_le_bytes());
    let payload = lzfse_bitmap(&padded(bgra, width as usize, height as usize), s, height as usize, versions);
    csi(&CsiHeader { flags: 0, width, height, scale: scale * 100, pixel_format: b"BGRA", color_space: 1, layout: LAYOUT_ICON_IMAGE, name }, &t, &payload)
}

/// A packed atlas (element 9, part 181): `bgra` is already flipped and
/// padded to the stride.
pub fn packed_csi(name: &str, width: u32, height: u32, scale: u32, bgra: &[u8], versions: MlecVersions) -> Vec<u8> {
    let mut t = Vec::new();
    let mut slices = Vec::new();
    for v in [1u32, 0, 0, 0, 0] {
        le32(&mut slices, v);
    }
    tlv(&mut t, 0x3e9, &slices);
    let mut composition = Vec::new();
    le32(&mut composition, 0);
    composition.extend_from_slice(&1.0f32.to_le_bytes());
    tlv(&mut t, 0x3ec, &composition);
    tlv(&mut t, 0x3ee, &1u32.to_le_bytes());
    let s = stride(width as usize);
    tlv(&mut t, 0x3ef, &(s as u32).to_le_bytes());
    let payload = lzfse_bitmap(bgra, s, height as usize, versions);
    csi(&CsiHeader { flags: 0, width, height, scale: scale * 100, pixel_format: b"BGRA", color_space: 1, layout: LAYOUT_PACKED_IMAGE, name }, &t, &payload)
}

/// An image set's atlas (layout 1004), its `dmp2` payload given: BGRA, or
/// for greyscale images GA8 (rows a byte a pixel when `opaque`).
pub fn packed_dmp2_csi(name: &str, width: u32, height: u32, scale: u32, gray: bool, opaque: bool, payload: &[u8]) -> Vec<u8> {
    let mut t = Vec::new();
    let mut slices = Vec::new();
    for v in [1u32, 0, 0, 0, 0] {
        le32(&mut slices, v);
    }
    tlv(&mut t, 0x3e9, &slices);
    composition(&mut t);
    tlv(&mut t, 0x3ee, &1u32.to_le_bytes());
    let row = if gray && opaque { (width as usize).div_ceil(32) * 32 } else { stride(width as usize) };
    tlv(&mut t, 0x3ef, &(row as u32).to_le_bytes());
    let (pixel_format, color_space) = if gray { (b" 8AG", 2) } else { (b"BGRA", 1) };
    csi(&CsiHeader { flags: 0, width, height, scale: scale * 100, pixel_format, color_space, layout: LAYOUT_PACKED_IMAGE, name }, &t, payload)
}

/// An image stored in an atlas (layout 1003): its rectangle there and the
/// atlas's key (attribute, value) in attribute order.
pub fn link_csi(name: &str, rect: (u32, u32, u32, u32), scale: u32, atlas: &[(u16, u16)]) -> Vec<u8> {
    link_csi_flags(name, rect, scale, 0, atlas)
}

/// A link with CSI flags given: an image set's rendering intent.
pub fn link_csi_flags(name: &str, rect: (u32, u32, u32, u32), scale: u32, flags: u32, atlas: &[(u16, u16)]) -> Vec<u8> {
    link_csi_format(name, rect, scale, flags, false, atlas)
}

/// The same for a greyscale image (GA8, colour space 2) when `gray`.
pub fn link_csi_format(name: &str, rect: (u32, u32, u32, u32), scale: u32, flags: u32, gray: bool, atlas: &[(u16, u16)]) -> Vec<u8> {
    let (x, y, w, h) = rect;
    let mut t = Vec::new();
    let mut slices = Vec::new();
    for v in [1u32, 0, 0, w, h] {
        le32(&mut slices, v);
    }
    tlv(&mut t, 0x3e9, &slices);
    let mut metrics = Vec::new();
    for v in [1u32, 0, 0, 0, 0, w, h] {
        le32(&mut metrics, v);
    }
    tlv(&mut t, 0x3eb, &metrics);
    let mut link = b"KLNI".to_vec();
    for v in [0u32, x, y, w, h] {
        le32(&mut link, v);
    }
    le16(&mut link, 12);
    let mut key: Vec<(u16, u16)> = atlas.iter().copied().filter(|a| a.1 != 0).collect();
    key.sort();
    le32(&mut link, 4 * key.len() as u32 + 4);
    for (a, v) in key {
        le16(&mut link, a);
        le16(&mut link, v);
    }
    le32(&mut link, 0);
    tlv(&mut t, 0x3f2, &link);
    let mut composition = Vec::new();
    le32(&mut composition, 0);
    composition.extend_from_slice(&1.0f32.to_le_bytes());
    tlv(&mut t, 0x3ec, &composition);
    tlv(&mut t, 0x3ee, &1u32.to_le_bytes());
    let (pixel_format, color_space) = if gray { (b" 8AG", 2) } else { (b"BGRA", 1) };
    csi(&CsiHeader { flags, width: w, height: h, scale: scale * 100, pixel_format, color_space, layout: LAYOUT_INTERNAL_LINK, name }, &t, &[])
}

/// An app icon's index of its sizes (part 218).
pub fn multisized_csi(name: &str, sizes: &[(u32, u32, u32)]) -> Vec<u8> {
    let mut t = Vec::new();
    let mut composition = Vec::new();
    le32(&mut composition, 0);
    composition.extend_from_slice(&0.0f32.to_le_bytes());
    tlv(&mut t, 0x3ec, &composition);
    tlv(&mut t, 0x3ee, &1u32.to_le_bytes());
    let mut payload = b"SISM".to_vec();
    le32(&mut payload, 1);
    le32(&mut payload, sizes.len() as u32);
    for (w, h, index) in sizes {
        le32(&mut payload, *w);
        le32(&mut payload, *h);
        le32(&mut payload, *index);
    }
    csi(&CsiHeader { flags: 0, width: 0, height: 0, scale: 0, pixel_format: &[0; 4], color_space: 0, layout: LAYOUT_MULTISIZED_IMAGE, name }, &t, &payload)
}

pub const ELEMENT_TEXTURE: u16 = 41;
pub const PART_STACK: u16 = 181;
pub const PART_FLATTENED_IMAGE: u16 = 208;
pub const PART_RADIOSITY_IMAGE: u16 = 209;
pub const IDIOM_TV: u16 = 3;
pub const IDIOM_VISION: u16 = 8;

const LAYOUT_IMAGE: u16 = 0;
pub const LAYOUT_LAYER_STACK: u16 = 1002;
pub const LAYOUT_SOLID_LAYER_STACK: u16 = 1018;
const LAYOUT_TEXTURE: u16 = 1007;
const LAYOUT_TEXTURE_IMAGE: u16 = 1008;

fn composition(t: &mut Vec<u8>) {
    let mut c = Vec::new();
    le32(&mut c, 0);
    c.extend_from_slice(&1.0f32.to_le_bytes());
    tlv(t, 0x3ec, &c);
}

/// An image stored as `dmp2` (layout 12): an image set's or an image stack
/// layer's. `payload` is from dmp2::encode.
pub fn dmp2_image_csi(name: &str, width: u32, height: u32, scale: u32, payload: &[u8]) -> Vec<u8> {
    dmp2_image_csi_flags(name, width, height, scale, 16, payload)
}

/// A `dmp2` image with CSI flags given: an image set's rendering intent
/// (16 automatic, 8 template, 0 original).
pub fn dmp2_image_csi_flags(name: &str, width: u32, height: u32, scale: u32, flags: u32, payload: &[u8]) -> Vec<u8> {
    let mut t = Vec::new();
    let mut slices = Vec::new();
    for v in [1u32, 0, 0, width, height] {
        le32(&mut slices, v);
    }
    tlv(&mut t, 0x3e9, &slices);
    let mut metrics = Vec::new();
    for v in [1u32, 0, 0, 0, 0, width, height] {
        le32(&mut metrics, v);
    }
    tlv(&mut t, 0x3eb, &metrics);
    composition(&mut t);
    tlv(&mut t, 0x3ee, &1u32.to_le_bytes());
    // Bytes per row, rounded up to 32.
    tlv(&mut t, 0x3ef, &(stride(width as usize) as u32).to_le_bytes());
    csi(&CsiHeader { flags, width, height, scale: scale * 100, pixel_format: b"BGRA", color_space: 1, layout: LAYOUT_ICON_IMAGE, name }, &t, payload)
}

/// A greyscale `dmp2` image (GA8, colour space 2), flags as for
/// [`dmp2_image_csi_flags`]: bytes per row are a byte a pixel when opaque
/// and four otherwise, rounded up to 32.
pub fn dmp2_gray_image_csi(name: &str, width: u32, height: u32, scale: u32, flags: u32, opaque: bool, payload: &[u8]) -> Vec<u8> {
    let mut t = Vec::new();
    let mut slices = Vec::new();
    for v in [1u32, 0, 0, width, height] {
        le32(&mut slices, v);
    }
    tlv(&mut t, 0x3e9, &slices);
    let mut metrics = Vec::new();
    for v in [1u32, 0, 0, 0, 0, width, height] {
        le32(&mut metrics, v);
    }
    tlv(&mut t, 0x3eb, &metrics);
    composition(&mut t);
    tlv(&mut t, 0x3ee, &1u32.to_le_bytes());
    let row = if opaque { (width as usize).div_ceil(32) * 32 } else { stride(width as usize) };
    tlv(&mut t, 0x3ef, &(row as u32).to_le_bytes());
    csi(&CsiHeader { flags, width, height, scale: scale * 100, pixel_format: b" 8AG", color_space: 2, layout: LAYOUT_ICON_IMAGE, name }, &t, payload)
}

/// An image set's images (part 181, as image stacks').
pub const PART_IMAGE: u16 = 181;

/// An image stack's flattened or radiosity image (layout 0), its payload
/// given.
pub fn stack_image_csi(name: &str, width: u32, height: u32, scale: u32, payload: &[u8]) -> Vec<u8> {
    let mut t = Vec::new();
    let mut slices = Vec::new();
    for v in [1u32, 0, 0, 0, 0] {
        le32(&mut slices, v);
    }
    tlv(&mut t, 0x3e9, &slices);
    composition(&mut t);
    tlv(&mut t, 0x3ee, &1u32.to_le_bytes());
    tlv(&mut t, 0x3ef, &(width * 4).to_le_bytes());
    csi(&CsiHeader { flags: 0, width, height, scale: scale * 100, pixel_format: b"BGRA", color_space: 1, layout: LAYOUT_IMAGE, name }, &t, payload)
}

/// An LZFSE-compressed bitmap payload of rows `stride` bytes apart.
pub fn bitmap_payload(data: &[u8], stride: usize, height: usize, chunked_version: u32) -> Vec<u8> {
    lzfse_bitmap(data, stride, height, MlecVersions { single: 2, chunked: chunked_version })
}

/// A layer of an image stack as its stack rendition lists it.
pub struct StackLayer {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub identifier: u16,
}

/// An image stack (layout 1002 for tvOS, 1018 for visionOS): its layers,
/// back to front, each its position and size in points and its image's
/// rendition key.
pub fn stack_csi(width: u32, height: u32, layout: u16, layers: &[StackLayer]) -> Vec<u8> {
    let n = layers.len() as u32;
    let mut t = Vec::new();
    let mut list = Vec::new();
    le32(&mut list, n);
    le32(&mut list, 0);
    for l in layers {
        for v in [0, l.x, l.y, l.width, l.height, 0] {
            le32(&mut list, v);
        }
        list.extend_from_slice(&1.0f32.to_le_bytes()); // opacity
        le32(&mut list, 16);
        for (a, v) in [(ATTR_ELEMENT, ELEMENT_ICON), (ATTR_PART, PART_STACK), (ATTR_IDENTIFIER, l.identifier)] {
            le16(&mut list, a);
            le16(&mut list, v);
        }
        le32(&mut list, 0);
    }
    tlv(&mut t, 0x3f4, &list);
    // Per-layer properties, at their defaults.
    for (kind, layer) in [(0x3fc, &[0u8, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0][..]), (0x3fd, &[0u8; 20][..]), (0x3fe, &[0u8; 12][..])] {
        let mut v = Vec::new();
        le32(&mut v, n);
        le32(&mut v, 0);
        for _ in layers {
            v.extend_from_slice(layer);
        }
        tlv(&mut t, kind, &v);
    }
    composition(&mut t);
    let uti = b"public.layeredimage\0";
    let mut u = Vec::new();
    le32(&mut u, uti.len() as u32);
    le32(&mut u, 0);
    u.extend_from_slice(uti);
    tlv(&mut t, 0x3ed, &u);
    tlv(&mut t, 0x3ee, &1u32.to_le_bytes());
    let payload = [b"DWAR".as_slice(), &[0u8; 8]].concat();
    csi(&CsiHeader { flags: 0, width, height, scale: 100, pixel_format: b"ATAD", color_space: 0, layout, name: "Contents.json" }, &t, &payload)
}

/// Metal's pixel formats of visionOS's layer textures.
pub const TEXTURE_R32_FLOAT: u32 = 55;
pub const TEXTURE_RG8_SNORM: u32 = 32;

/// A visionOS layer texture's descriptor (layout 1007): its Metal pixel
/// format and its image's rendition key, (attribute, value) in the order
/// given.
pub fn texture_csi(name: &str, width: u32, height: u32, format: u32, key: &[(u16, u16)]) -> Vec<u8> {
    let mut t = Vec::new();
    composition(&mut t);
    tlv(&mut t, 0x3ee, &1u32.to_le_bytes());
    let mut v = Vec::new();
    for x in [0u32, 0, 1] {
        le32(&mut v, x);
    }
    tlv(&mut t, 0x3f6, &v);
    let mut p = b"RTXT".to_vec();
    for x in [0, format, 1, 1] {
        le32(&mut p, x);
    }
    le16(&mut p, 1);
    le16(&mut p, 1);
    le32(&mut p, 4 * key.len() as u32 + 4);
    le32(&mut p, 0);
    for (a, x) in key {
        le16(&mut p, *a);
        le16(&mut p, *x);
    }
    le32(&mut p, 0);
    csi(&CsiHeader { flags: 0, width, height, scale: 0, pixel_format: b"BGRA", color_space: 1, layout: LAYOUT_TEXTURE, name }, &t, &p)
}

/// A visionOS layer texture's data (layout 1008), rows `row_bytes` apart.
pub fn texture_image_csi(name: &str, width: u32, height: u32, scale: u32, row_bytes: u32, data: &[u8]) -> Vec<u8> {
    let mut t = Vec::new();
    composition(&mut t);
    tlv(&mut t, 0x3ee, &1u32.to_le_bytes());
    tlv(&mut t, 0x3ef, &row_bytes.to_le_bytes());
    let payload = lzfse_bitmap(data, row_bytes as usize, height as usize, MlecVersions { single: 2, chunked: 1 });
    csi(&CsiHeader { flags: 0, width, height, scale: scale * 100, pixel_format: b"BGRA", color_space: 0, layout: LAYOUT_TEXTURE_IMAGE, name }, &t, &payload)
}

pub const LAYOUT_COLOR: u16 = 1009;

/// A colour set's colour (layout 1009, no bitmap): composition and 0x3ee
/// TLVs, then `COLR` (written `RLOC`), version 1, the colour space's code,
/// the component count and the components as doubles.
pub fn color_csi(name: &str, space: u32, components: &[f64]) -> Vec<u8> {
    let mut t = Vec::new();
    tlv(&mut t, 0x3ec, &[0u8; 8]);
    tlv(&mut t, 0x3ee, &1u32.to_le_bytes());
    let mut payload = b"RLOC".to_vec();
    le32(&mut payload, 1);
    le32(&mut payload, space);
    le32(&mut payload, components.len() as u32);
    for c in components {
        payload.extend_from_slice(&c.to_le_bytes());
    }
    csi(&CsiHeader { flags: 0, width: 0, height: 0, scale: 0, pixel_format: &[0; 4], color_space: 0, layout: LAYOUT_COLOR, name }, &t, &payload)
}
