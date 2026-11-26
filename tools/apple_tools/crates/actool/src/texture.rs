//! Texture sets: Metal textures in a catalog (docs/clean-room/actool.md
//! "Texture sets"), as Xcode compiles one of a single universal image. Xcode
//! stores the texture's description and each mipmap level down to 1x1, on
//! iOS also for two GPU families, the newer as ASTC; this actool writes the
//! description, keys and storage as Xcode does, but draws the smaller levels
//! with its own filter and encodes ASTC with its own encoder, where Xcode's
//! filter, colour conversion and encoder aren't reproduced: compiled only
//! relaxed (`--config=relax_actool_texture`).

use crate::car::{self, Facet, Rendition};
use std::path::PathBuf;

/// How a texture's values are read (its `interpretation`), as its
/// description records them: data 0, non-premultiplied colours 1, colours
/// (the default) 2.
#[derive(Clone, Copy, PartialEq)]
enum Interpretation {
    Data = 0,
    Straight = 1,
    Premultiplied = 2,
}

/// Metal's pixel formats for a level, RGBA8 or ASTC 4x4, linear for data
/// and sRGB for colours (MTLPixelFormat).
fn pixel_format(interpretation: Interpretation, astc: bool) -> u32 {
    match (interpretation == Interpretation::Data, astc) {
        (true, false) => 70,
        (false, false) => 71,
        (true, true) => 204,
        (false, true) => 186,
    }
}

fn le32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn le16(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn tlv(out: &mut Vec<u8>, kind: u32, data: &[u8]) {
    le32(out, kind);
    le32(out, data.len() as u32);
    out.extend_from_slice(data);
}

/// Texture sets (name, directory), each its facet and renditions. iOS
/// stores each for no GPU family and for families 1 and 2 (2 as ASTC),
/// macOS for none (probes).
pub fn compile(sets: &[(String, PathBuf)], platform: &str, relax: bool, identifier: impl Fn(&str) -> u16) -> Result<(Vec<Facet>, Vec<Rendition>), String> {
    let mut facets = Vec::new();
    let mut renditions = Vec::new();
    for (name, dir) in sets {
        let classes: &[u16] = match platform {
            "ios" => &[0, 1, 2],
            "macosx" => &[0],
            _ => return Err(format!("{}: texture sets for this platform are not supported by this actool", dir.display())),
        };
        if !relax {
            return Err(format!("{}: a texture set is compiled only relaxed: Xcode draws its smaller mipmap levels and encodes ASTC in ways this actool doesn't reproduce; --config=relax_actool_texture accepts its own", dir.display()));
        }
        let (interpretation, top_left, filename, image) = read(dir)?;
        let id = identifier(name);
        facets.push(Facet { name: name.clone(), attrs: vec![(car::ATTR_ELEMENT, car::ELEMENT_TEXTURE), (car::ATTR_IDENTIFIER, id)] });
        let levels = levels(&image, interpretation, top_left);
        for &class in classes {
            let astc = class == 2;
            let key = |level: usize, part: bool| {
                let mut k = vec![(car::ATTR_ELEMENT, car::ELEMENT_TEXTURE)];
                if part {
                    k.push((car::ATTR_PART, car::PART_IMAGE));
                }
                if level > 0 {
                    k.push((car::ATTR_DIMENSION2, level as u16));
                }
                k.push((car::ATTR_SCALE, 1));
                k.push((car::ATTR_IDENTIFIER, id));
                if class > 0 {
                    k.push((car::ATTR_GRAPHICS_CLASS, class));
                }
                k
            };
            // The description: its interpretation and opacity, then each level's key
            // (a length, a zero, the key's pairs by attribute, a zero).
            let mut t = Vec::new();
            let mut composition = 0u32.to_le_bytes().to_vec();
            composition.extend_from_slice(&1.0f32.to_le_bytes());
            tlv(&mut t, 0x3ec, &composition);
            tlv(&mut t, 0x3ee, &1u32.to_le_bytes());
            // Then whether every pixel is opaque (probes).
            let opaque = image.data.chunks_exact(4).all(|p| p[3] == 255);
            let mut kind = Vec::new();
            for v in [0, interpretation as u32, opaque as u32] {
                le32(&mut kind, v);
            }
            tlv(&mut t, 0x3f6, &kind);
            let mut list = b"RTXT".to_vec();
            for v in [0, pixel_format(interpretation, astc), 1, 1] {
                le32(&mut list, v);
            }
            le16(&mut list, 1);
            le16(&mut list, levels.len() as u16);
            for i in 0..levels.len() {
                let pairs = key(i, true);
                le32(&mut list, 4 + 4 * pairs.len() as u32);
                le32(&mut list, 0);
                for (a, v) in pairs {
                    le16(&mut list, a);
                    le16(&mut list, v);
                }
                le32(&mut list, 0);
            }
            renditions.push(Rendition { attrs: key(0, false), csi: car::texture_set_csi(name, image.width as u32, image.height as u32, &t, &list) });
            for (i, (w, h, rgba)) in levels.iter().enumerate() {
                let (data, row) = if astc { (astc::encode(rgba, *w, *h), w.div_ceil(4) * 16) } else { (rgba.clone(), w * 4) };
                let mut t = Vec::new();
                tlv(&mut t, 0x3ec, &composition);
                tlv(&mut t, 0x3ee, &1u32.to_le_bytes());
                tlv(&mut t, 0x3ef, &(row as u32).to_le_bytes());
                let payload = car::texture_bitmap(&data, row, *w, *h, astc);
                renditions.push(Rendition { attrs: key(i, true), csi: car::texture_level_csi(&filename, *w as u32, *h as u32, &t, &payload) });
            }
        }
    }
    Ok((facets, renditions))
}

/// A texture set's interpretation, origin, base image file name and image:
/// one universal mipmap set of a base level alone; other forms refused.
fn read(dir: &std::path::Path) -> Result<(Interpretation, bool, String, crate::image::Rgba), String> {
    let contents = crate::read_json(&dir.join("Contents.json"))?;
    let unsupported = |what: &str| Err(format!("{}: {what} in a texture set is not supported by this actool", dir.display()));
    for key in contents.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()).unwrap_or_default() {
        if !["info", "properties", "textures"].contains(&key.as_str()) {
            return unsupported(&format!("the {key} key"));
        }
    }
    let mut interpretation = Interpretation::Premultiplied;
    let mut top_left = false;
    for (k, v) in contents["properties"].as_object().into_iter().flatten() {
        match (k.as_str(), v.as_str()) {
            ("interpretation", Some("colors")) => interpretation = Interpretation::Premultiplied,
            ("interpretation", Some("non-premultiplied-colors")) => interpretation = Interpretation::Straight,
            ("interpretation", Some("data")) => interpretation = Interpretation::Data,
            ("origin", Some("bottom-left")) => top_left = false,
            ("origin", Some("top-left")) => top_left = true,
            _ => return unsupported(&format!("the property {k} = {v}")),
        }
    }
    let textures = contents["textures"].as_array().cloned().unwrap_or_default();
    let [texture] = textures.as_slice() else {
        return unsupported("anything but one texture");
    };
    if texture.as_object().is_some_and(|o| o.keys().any(|k| !["idiom", "filename"].contains(&k.as_str()))) || texture["idiom"] != "universal" {
        return unsupported("a texture other than a universal one");
    }
    let Some(mipmaps) = texture["filename"].as_str().filter(|f| f.ends_with(".mipmapset")) else {
        return unsupported("a texture other than a mipmap set");
    };
    let mipmaps = dir.join(mipmaps);
    let set = crate::read_json(&mipmaps.join("Contents.json"))?;
    let levels = set["levels"].as_array().cloned().unwrap_or_default();
    let [level] = levels.as_slice() else {
        return unsupported("a mipmap set of anything but its base level");
    };
    if level["mipmap-level"] != "base" || level.as_object().is_some_and(|o| o.keys().any(|k| !["filename", "mipmap-level"].contains(&k.as_str()))) {
        return unsupported("a mipmap level other than the base");
    }
    let Some(filename) = level["filename"].as_str().filter(|f| f.to_ascii_lowercase().ends_with(".png")) else {
        return unsupported("a base level other than a PNG");
    };
    let path = mipmaps.join(filename);
    let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let image = crate::image::decode_png(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok((interpretation, top_left, filename.to_string(), image))
}

/// The levels as stored, RGBA, from the base down to 1x1, each side halved
/// and rounded down: rows in the PNG's order for a bottom-left origin and
/// flipped for a top-left one, colours premultiplied for the `colors`
/// interpretation (probes). The smaller levels average the area each pixel
/// covers, which isn't Xcode's filter.
fn levels(image: &crate::image::Rgba, interpretation: Interpretation, top_left: bool) -> Vec<(usize, usize, Vec<u8>)> {
    let (w, h) = (image.width, image.height);
    let mut base = vec![0u8; w * h * 4];
    for y in 0..h {
        let from = if top_left { h - 1 - y } else { y };
        base[y * w * 4..(y + 1) * w * 4].copy_from_slice(&image.data[from * w * 4..(from + 1) * w * 4]);
    }
    if interpretation == Interpretation::Premultiplied {
        for p in base.chunks_exact_mut(4) {
            let a = p[3] as u32;
            for c in &mut p[..3] {
                *c = ((*c as u32 * a + 127) / 255) as u8;
            }
        }
    }
    let mut out = vec![(w, h, base)];
    while out.last().is_some_and(|(w, h, _)| *w > 1 || *h > 1) {
        let (sw, sh, src) = out.last().expect("a level");
        let (dw, dh) = ((sw / 2).max(1), (sh / 2).max(1));
        let mut dst = vec![0u8; dw * dh * 4];
        for y in 0..dh {
            let (y0, y1) = (y * sh / dh, ((y + 1) * sh).div_ceil(dh));
            for x in 0..dw {
                let (x0, x1) = (x * sw / dw, ((x + 1) * sw).div_ceil(dw));
                let mut sum = [0u32; 4];
                for sy in y0..y1 {
                    for sx in x0..x1 {
                        for c in 0..4 {
                            sum[c] += src[(sy * sw + sx) * 4 + c] as u32;
                        }
                    }
                }
                let n = ((y1 - y0) * (x1 - x0)) as u32;
                for c in 0..4 {
                    dst[(y * dw + x) * 4 + c] = ((sum[c] + n / 2) / n) as u8;
                }
            }
        }
        out.push((dw, dh, dst));
    }
    out
}

/// A plain ASTC 4x4 encoder: each block one partition of RGBA endpoints
/// (colour endpoint mode 12) at 8 bits, the block's per-channel minimum and
/// maximum, and 2-bit weights on a 4x4 grid, the projection of each pixel
/// onto the line between them. Valid ASTC; its choices aren't Xcode's.
mod astc {
    /// Block mode for a 4x4 grid of weights in the range 0..3, one plane:
    /// the layout table's first row (width B + 4, height A + 2) with A = 2
    /// and B = 0; R2 R1 R0 = 100 (bits 1, 0 and 4), low precision (H = 0),
    /// one plane (D = 0).
    const BLOCK_MODE: u128 = 0b000_0100_0010;
    /// Colour endpoint mode 12, LDR RGBA direct.
    const CEM: u128 = 12;
    /// The weights 0..3 unquantized to 0..64.
    const WEIGHTS: [i32; 4] = [0, 21, 43, 64];

    /// `rgba` (`w` by `h`) as ASTC 4x4 blocks, row by row from its first.
    pub fn encode(rgba: &[u8], w: usize, h: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(w.div_ceil(4) * h.div_ceil(4) * 16);
        for by in 0..h.div_ceil(4) {
            for bx in 0..w.div_ceil(4) {
                let mut texels = [[0u8; 4]; 16];
                for (i, t) in texels.iter_mut().enumerate() {
                    let (x, y) = ((bx * 4 + i % 4).min(w - 1), (by * 4 + i / 4).min(h - 1));
                    t.copy_from_slice(&rgba[(y * w + x) * 4..(y * w + x) * 4 + 4]);
                }
                out.extend_from_slice(&block(&texels).to_le_bytes());
            }
        }
        out
    }

    fn block(texels: &[[u8; 4]; 16]) -> u128 {
        // Endpoints at the texels' extremes along their principal axis
        // (power iteration on their covariance), clamped to 0..255.
        let px: Vec<[f64; 4]> = texels.iter().map(|t| [t[0] as f64, t[1] as f64, t[2] as f64, t[3] as f64]).collect();
        let mut mean = [0.0f64; 4];
        for p in &px {
            for c in 0..4 {
                mean[c] += p[c] / 16.0;
            }
        }
        let mut cov = [[0.0f64; 4]; 4];
        for p in &px {
            for i in 0..4 {
                for j in 0..4 {
                    cov[i][j] += (p[i] - mean[i]) * (p[j] - mean[j]);
                }
            }
        }
        let mut axis = [1.0f64, 1.0, 1.0, 1.0];
        for _ in 0..16 {
            let mut next = [0.0f64; 4];
            for i in 0..4 {
                for j in 0..4 {
                    next[i] += cov[i][j] * axis[j];
                }
            }
            let norm = next.iter().map(|v| v * v).sum::<f64>().sqrt();
            if norm < 1e-9 {
                break;
            }
            axis = next.map(|v| v / norm);
        }
        let project = |p: &[f64; 4]| (0..4).map(|c| (p[c] - mean[c]) * axis[c]).sum::<f64>();
        let (lo_t, hi_t) = px.iter().map(project).fold((f64::MAX, f64::MIN), |(a, b), t| (a.min(t), b.max(t)));
        let point = |t: f64| -> [u8; 4] { std::array::from_fn(|c| (mean[c] + axis[c] * t).round().clamp(0.0, 255.0) as u8) };
        let (mut lo, mut hi) = (point(lo_t), point(hi_t));
        // The second endpoint's RGB sum at or above the first's, so that no
        // blue contraction applies.
        if (hi[0] as u32 + hi[1] as u32 + hi[2] as u32) < (lo[0] as u32 + lo[1] as u32 + lo[2] as u32) {
            std::mem::swap(&mut lo, &mut hi);
        }
        let d: Vec<i32> = (0..4).map(|c| hi[c] as i32 - lo[c] as i32).collect();
        let dd: i32 = d.iter().map(|v| v * v).sum();
        let mut weights = 0u128;
        for (i, t) in texels.iter().enumerate() {
            let w = if dd == 0 {
                0
            } else {
                let dot: i32 = (0..4).map(|c| (t[c] as i32 - lo[c] as i32) * d[c]).sum();
                let s = dot * 64 / dd;
                (0..4).min_by_key(|&k| (WEIGHTS[k] - s).abs()).expect("four weights")
            };
            weights |= (w as u128) << (2 * i);
        }
        let mut bits = BLOCK_MODE | (CEM << 13);
        // Endpoint values v0..v7: (r0, r1, g0, g1, b0, b1, a0, a1).
        for c in 0..4 {
            bits |= (lo[c] as u128) << (17 + 16 * c);
            bits |= (hi[c] as u128) << (17 + 16 * c + 8);
        }
        // The weights from the block's last bit down: weight bit i at block
        // bit 127 - i.
        bits | weights.reverse_bits()
    }
}

#[cfg(test)]
mod tests {
    use super::astc;

    /// Decodes one of `astc::encode`'s blocks as the ASTC specification
    /// does for its mode (one partition, RGBA direct endpoints at 8 bits,
    /// 2-bit weights on a 4x4 grid): the texel at `i`.
    fn decode(block: u128, i: usize) -> [u8; 4] {
        assert_eq!(block & 0x7ff, 0b100_0010, "block mode");
        assert_eq!((block >> 11) & 3, 0, "one partition");
        assert_eq!((block >> 13) & 15, 12, "RGBA direct");
        let v = |k: usize| ((block >> (17 + 8 * k)) & 0xff) as i32;
        let w = [0, 21, 43, 64][((block.reverse_bits() >> (2 * i)) & 3) as usize];
        let mut out = [0u8; 4];
        for c in 0..4 {
            let (e0, e1) = (v(2 * c) * 257, v(2 * c + 1) * 257);
            out[c] = (((e0 * (64 - w) + e1 * w + 32) >> 6) >> 8) as u8;
        }
        out
    }

    #[test]
    fn astc_blocks_decode_near_their_texels() {
        // A 4x4 gradient and alpha ramp, and a flat block.
        let mut rgba = Vec::new();
        for i in 0..16u8 {
            rgba.extend_from_slice(&[i * 16, 255 - i * 16, 40, 255 - i * 8]);
        }
        let block = u128::from_le_bytes(astc::encode(&rgba, 4, 4).try_into().unwrap());
        for i in 0..16 {
            let got = decode(block, i);
            for c in 0..4 {
                assert!((got[c] as i32 - rgba[i * 4 + c] as i32).abs() <= 48, "texel {i} channel {c}: {got:?}");
            }
        }
        let flat = u128::from_le_bytes(astc::encode(&[7, 8, 9, 10].repeat(16), 4, 4).try_into().unwrap());
        assert_eq!(decode(flat, 5), [7, 8, 9, 10]);
        // A 5x3 level takes two blocks, its edges repeated.
        assert_eq!(astc::encode(&[1; 5 * 3 * 4], 5, 3).len(), 32);
    }
}
