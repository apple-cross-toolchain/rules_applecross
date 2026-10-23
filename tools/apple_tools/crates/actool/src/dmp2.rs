//! `dmp2`, how Xcode stores images of image sets and image stack layers
//! (docs/clean-room/actool.md): whichever of a palette, a lossy predictive
//! transform or plain LZFSE is smallest.

use crate::qsort::qsort;

fn le32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn header(mode: u8, width: usize, height: usize) -> Vec<u8> {
    let mut s = b"dmp2".to_vec();
    s.extend([mode, 1, 0x0a, 4]);
    s.extend((width as u16).to_le_bytes());
    s.extend((height as u16).to_le_bytes());
    s
}

fn compressed(out: &mut Vec<u8>, data: &[u8]) {
    let z = lzfse::encode(data);
    le32(out, z.len() as u32);
    out.extend(z);
}

/// Rows of an image compressed together: as many as fit 2 MiB, at
/// `row_bytes` each rounded up to 16.
fn strip_rows(row_bytes: usize, height: usize) -> usize {
    std::cmp::min(height, (1 << 21) / (row_bytes.div_ceil(16) * 16))
}

/// A mode's stream: the header, with the rows per strip as its height, and
/// each strip's data compressed on its own.
fn striped(mode: u8, width: usize, height: usize, row_bytes: usize, mut strip: impl FnMut(usize, usize) -> Vec<u8>) -> Vec<u8> {
    let rows = strip_rows(row_bytes, height);
    let mut s = header(mode, width, rows);
    let mut y = 0;
    while y < height {
        let h = std::cmp::min(rows, height - y);
        compressed(&mut s, &strip(y, h));
        y += h;
    }
    s
}

/// Mode 4: at most 256 colours, as a palette and 8-bit indices.
fn palette(width: usize, height: usize, bgra: &[u8]) -> Option<Vec<u8>> {
    let pixels: Vec<u32> = bgra.chunks_exact(4).map(|p| u32::from_le_bytes([p[0], p[1], p[2], p[3]])).collect();
    let mut colours = pixels.clone();
    colours.sort_unstable();
    colours.dedup();
    if colours.len() > 256 {
        return None;
    }
    if strip_rows(width, height) < height {
        // How palettes go with strips is not established.
        return Some(Vec::new());
    }
    let xor = |c: &u32| c.to_le_bytes().iter().fold(0u8, |a, b| a ^ b);
    qsort(&mut colours, |a, b| xor(a).cmp(&xor(b)));
    let mut index = std::collections::HashMap::new();
    for (i, c) in colours.iter().enumerate() {
        index.insert(*c, i as u8);
    }
    let mut s = header(4, width, height);
    s.extend((colours.len() as u16).to_le_bytes());
    s.extend(4u16.to_le_bytes());
    for c in &colours {
        s.extend(c.to_le_bytes());
    }
    let indices: Vec<u8> = pixels.iter().map(|p| index[p]).collect();
    compressed(&mut s, &indices);
    Some(s)
}

/// C's truncating division by 2.
fn half(v: i32) -> i32 {
    v / 2
}

/// YCoCg as mode 2 computes it, losing a bit of G and B.
fn ycocg(b: u8, g: u8, r: u8) -> [i32; 3] {
    let (r, g, b) = (r as i32, g as i32, b as i32);
    let co = half(b - r);
    let t = r + co;
    let cg = half(g - t);
    [t + cg, co, cg]
}

/// Row predictors by code; the order ties go in.
const TIE_ORDER: [u8; 5] = [0, 2, 3, 1, 4];

fn predict(code: u8, v: &[[i32; 3]], width: usize, i: usize, x: usize, y: usize, k: usize) -> i32 {
    if code == 0 {
        return 0;
    }
    if x == 0 {
        return if y > 0 && code != 2 { v[i - width][k] } else { 0 };
    }
    let left = v[i - 1][k];
    if y == 0 {
        return left;
    }
    let up = v[i - width][k];
    match code {
        1 => {
            let ul = v[i - width - 1][0];
            if (v[i - width][0] - ul).abs() <= (v[i - 1][0] - ul).abs() { left } else { up }
        }
        2 => left,
        3 => up,
        _ => half(left + up + 1),
    }
}

/// Mode 2's data for rows of an image, and whether any row's predictor is
/// not established: costs that tie, or as good as, other than between rows
/// predicted exactly.
fn transform(width: usize, height: usize, bgra: &[u8]) -> (Vec<u8>, bool) {
    let n = width * height;
    let v: Vec<[i32; 3]> = bgra.chunks_exact(4).map(|p| ycocg(p[0], p[1], p[2])).collect();
    let mut codes = Vec::with_capacity(height);
    let mut ambiguous = false;
    let mut residuals = vec![[0i32; 3]; n];
    let row = |code: u8, y: usize| -> Vec<[i32; 3]> {
        (0..width)
            .map(|x| {
                let i = y * width + x;
                [0, 1, 2].map(|k| v[i][k] - predict(code, &v, width, i, x, y, k))
            })
            .collect()
    };
    for y in 0..height {
        let candidates: &[u8] = if y == 0 { &[0, 2] } else { &[0, 1, 2, 3, 4] };
        let rows: Vec<(u8, Vec<[i32; 3]>, f64)> = candidates
            .iter()
            .map(|&c| {
                let r = row(c, y);
                let cost = r.iter().flatten().map(|d| (1.0 + d.unsigned_abs() as f64).log2()).sum();
                (c, r, cost)
            })
            .collect();
        let best = rows.iter().map(|r| r.2).fold(f64::INFINITY, f64::min);
        let mut tied: Vec<&(u8, Vec<[i32; 3]>, f64)> = rows.iter().filter(|r| r.2 - best < 1e-3).collect();
        // Rows every candidate in the tie predicts exactly go by TIE_ORDER;
        // other ties are not established.
        ambiguous |= tied.len() > 1 && best != 0.0;
        tied.sort_by_key(|r| TIE_ORDER.iter().position(|&c| c == r.0));
        let (code, r, _) = tied[0];
        codes.push(*code);
        residuals[y * width..(y + 1) * width].copy_from_slice(r);
    }
    let mut raw = Vec::with_capacity(7 * n + height + 16);
    raw.extend(bgra.chunks_exact(4).map(|p| p[3]));
    raw.extend(&codes);
    let coded: Vec<u16> = residuals.iter().flatten().map(|&d| (2 * d.unsigned_abs() + (d < 0) as u32) as u16).collect();
    raw.extend(coded.iter().map(|c| (c >> 8) as u8));
    raw.extend(coded.iter().map(|c| *c as u8));
    raw.resize(raw.len().div_ceil(16) * 16, 0);
    (raw, ambiguous)
}

/// The `MLEC` payload of an image of premultiplied BGRA rows (tightly
/// packed), stored as Xcode stores it; `opaque` as Xcode flags it (see
/// stack.rs).
pub fn encode(width: usize, height: usize, bgra: &[u8], opaque: bool) -> Result<Vec<u8>, String> {
    if width > 0xffff || height > 0xffff {
        return Err(format!("a {width}x{height} image is too large for this actool"));
    }
    let row = 4 * width;
    let lzfse_only = striped(3, width, height, row, |y, h| bgra[y * row..(y + h) * row].to_vec());
    let mut candidates = vec![lzfse_only];
    match palette(width, height, bgra) {
        Some(p) if p.is_empty() => return Err("this image's dmp2 palette would be in strips, which is not established for this actool".into()),
        p => candidates.extend(p),
    }
    let mut ambiguous = false;
    let predictive = striped(2, width, height, 7 * width + 1, |y, h| {
        let (raw, a) = transform(width, h, &bgra[y * row..(y + h) * row]);
        ambiguous |= a;
        raw
    });
    let smallest_other = candidates.iter().map(Vec::len).min().unwrap();
    if !ambiguous {
        candidates.push(predictive);
    } else if predictive.len() <= smallest_other + smallest_other / 100 + 64 {
        // Another choice of predictors could make mode 2 the smallest.
        return Err("this image's dmp2 encoding (its row predictors) is not established for this actool".into());
    }
    let raw_len = 12 + bgra.len();
    let min = candidates.iter().map(Vec::len).min().unwrap();
    if raw_len <= min {
        return Err("this image would be stored uncompressed, which is not established for this actool".into());
    }
    if candidates.iter().filter(|c| c.len() == min).count() > 1 {
        return Err("this image's dmp2 modes tie, which is not established for this actool".into());
    }
    let stream = candidates.into_iter().find(|c| c.len() == min).unwrap();
    let mut out = b"MLEC".to_vec();
    le32(&mut out, if opaque { 2 } else { 0 });
    le32(&mut out, 11);
    le32(&mut out, 16 + stream.len() as u32);
    le32(&mut out, 1);
    le32(&mut out, 4);
    le32(&mut out, stream.len() as u32);
    le32(&mut out, 0);
    out.extend(stream);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_rounds_as_c() {
        assert_eq!(ycocg(1, 0, 0), [0, 0, 0]);
        assert_eq!(ycocg(0, 0, 1), [1, 0, 0]);
        assert_eq!(ycocg(4, 0, 0), [1, 2, -1]);
        assert_eq!(ycocg(0, 2, 0), [1, 0, 1]);
    }

    #[test]
    fn palette_order() {
        // One XOR bucket of eight colours: macOS's qsort moves the largest
        // to the front and the smallest to the back.
        let colours: Vec<[u8; 4]> = (0..8u8).map(|i| { let r = i * 32; [r ^ 0x11 ^ 0x5a, 0x11, r, 255] }).collect();
        let mut bgra = Vec::new();
        for i in 0..64 {
            bgra.extend(colours[i % 8]);
        }
        let s = palette(8, 8, &bgra).unwrap();
        let reds: Vec<u8> = (0..8).map(|i| s[16 + 4 * i + 2]).collect();
        assert_eq!(reds, [224, 32, 64, 96, 128, 160, 192, 0]);
    }
}
