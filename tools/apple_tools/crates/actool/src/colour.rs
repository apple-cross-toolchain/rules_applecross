//! Colour-managed PNGs converted to sRGB as Xcode's actool converts them
//! (through CoreGraphics; docs/clean-room/actool.md#colour-managed-pngs),
//! for the profiles whose conversion is established. Each channel's curve
//! gives a 12-bit linear value, a matrix derived from the profile's
//! colorants mixes them, the sum is floored to 13 bits and rounded to 12,
//! and the sRGB encoding, itself rounded to 12 bits, gives 8.

/// The sRGB profile's colorants (s15Fixed16, XYZ rows by R, G and B
/// columns), the destination of every conversion.
const SRGB_COLORANTS: [[i32; 3]; 3] = [[28578, 25241, 9376], [14581, 46981, 3972], [912, 6362, 46799]];

/// sRGB's parameters as a type 3 curve in s15Fixed16, as Apple's profiles
/// write them.
const SRGB_PARAMETERS: [f64; 5] = [2.399993896484375, 0.9478607177734375, 0.0521392822265625, 0.077392578125, 0.0404510498046875];

/// BT.709's curve as ITU-709's and ITU-2020's profiles write it (s15Fixed16
/// g, a, b, c, d), both taken as gamma 2.4.
const BT709_PARAMETERS: [[i32; 5]; 2] = [[0x238e4, 0xe8f0, 0x1710, 0x38e4, 0x14bc], [0x238e4, 0xe8e0, 0x1720, 0x38e4, 0x14bc]];

/// How a profile's curve gives 12-bit linear values.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Curve {
    /// No curve (`curv` with no entries, or gamma 1.0).
    Linear,
    /// A gamma other than 2.2, with its toe: `max(x^g, x / 16)`.
    GammaWithToe(f64),
    /// A pure gamma, as BT.709's parametric curve is taken (as 2.4).
    Gamma(f64),
    /// A gamma of exactly 2.2, on the input widened to 12 bits first.
    Gamma22,
    /// A parametric curve of type 3, `(a x + b)^g` from `d` on and `c x`
    /// below it, in double precision.
    Parametric([f64; 5]),
}

impl Curve {
    /// The curve's 12-bit linear value for each 8-bit input.
    fn table(self) -> [i64; 256] {
        std::array::from_fn(|v| {
            let x = v as f64 / 255.0;
            let l = match self {
                Curve::Linear => x,
                Curve::GammaWithToe(g) => x.powf(g).max(x / 16.0),
                Curve::Gamma(g) => x.powf(g),
                Curve::Gamma22 => (((257 * v + 8) >> 4) as f64 / 4096.0).powf(2.2),
                Curve::Parametric([g, a, b, c, d]) => {
                    if x >= d {
                        (a * x + b).max(0.0).powf(g)
                    } else {
                        c * x
                    }
                }
            };
            (l * 4096.0 + 0.5).floor() as i64
        })
    }

    /// The curve a `curv` or `para` tag gives, where established.
    fn parse(tag: &[u8]) -> Result<Curve, String> {
        let unsupported = |what: &str| Err(format!("this PNG's ICC profile has {what}, which is not supported by this actool"));
        let be32 = |o: usize| tag.get(o..o + 4).map(|b| i32::from_be_bytes(b.try_into().unwrap()));
        let truncated = || "this PNG's ICC profile has a truncated curve".to_string();
        match tag.get(..4) {
            Some(b"curv") => {
                let n = be32(8).ok_or_else(truncated)? as usize;
                let entries: Vec<u16> =
                    (0..n).map(|i| tag.get(12 + 2 * i..14 + 2 * i).map(|b| u16::from_be_bytes([b[0], b[1]]))).collect::<Option<_>>().ok_or_else(truncated)?;
                match entries[..] {
                    [] | [256] => Ok(Curve::Linear),
                    [461] => Ok(Curve::GammaWithToe(1.8)),
                    [563] => Ok(Curve::Gamma22),
                    [_] => unsupported("a gamma other than 1.0, 461/256 and 563/256"),
                    _ if srgb_like(&entries) => Ok(Curve::Parametric(SRGB_PARAMETERS)),
                    _ => unsupported("a curve table other than sRGB's"),
                }
            }
            Some(b"para") => {
                let count = match tag.get(8..10) {
                    Some([0, 0]) => 1,
                    Some([0, 3]) => 5,
                    _ => return unsupported("a parametric curve other than types 0 and 3"),
                };
                let raw: Vec<i32> = (0..count).map(|i| be32(12 + 4 * i)).collect::<Option<_>>().ok_or_else(truncated)?;
                let f = |v: i32| v as f64 / 65536.0;
                if count == 1 {
                    return match f(raw[0]) {
                        _ if raw[0] == 144179 => Ok(Curve::Gamma22),
                        g if (1.0..=3.0).contains(&g) => Ok(Curve::GammaWithToe(g)),
                        _ => unsupported("a gamma outside 1.0 to 3.0"),
                    };
                }
                if BT709_PARAMETERS.iter().any(|p| raw[..] == p[..]) {
                    return Ok(Curve::Gamma(2.4));
                }
                let p = [0, 1, 2, 3, 4].map(|i| f(raw[i]));
                // Parameters near BT.709's are taken as gamma 2.4 too, by a
                // rule not established.
                if (p[0] - 1.0 / 0.45).abs() < 0.1 && (p[1] - 1.0 / 1.099).abs() < 0.02 {
                    return unsupported("a curve near BT.709's");
                }
                Ok(Curve::Parametric(p))
            }
            _ => unsupported("a curve that isn't `curv` or `para`"),
        }
    }
}

/// Whether ColorSync takes a curve table for sRGB's curve. Tables whose
/// linear interpolation came within 4.78 (of 65535) of it everywhere were,
/// and 6.32 or more away weren't; between those isn't established.
fn srgb_like(table: &[u16]) -> bool {
    let n = table.len();
    (0..=4096).all(|j| {
        let x = j as f64 / 4096.0;
        let p = x * (n - 1) as f64;
        let i = (p as usize).min(n - 2);
        let t = table[i] as f64 + (table[i + 1] as f64 - table[i] as f64) * (p - i as f64);
        let srgb = if x > 0.04045 { ((x + 0.055) / 1.055).powf(2.4) } else { x / 12.92 };
        (t - srgb * 65535.0).abs() <= 4.78
    })
}

/// The matrix from linear values in `colorants`' space to linear sRGB, in
/// units of 2^-16: the source's rows scaled to sum as the sRGB colorants'
/// do (white to white), then the sRGB colorants' inverse, quantised as an
/// int16 matrix over the largest power of two up to 2^14 that holds it.
fn matrix(colorants: &[[i32; 3]; 3]) -> [[i64; 3]; 3] {
    let f = |v: i32| v as f64 / 65536.0;
    let source: [[f64; 3]; 3] = std::array::from_fn(|r| std::array::from_fn(|c| f(colorants[c][r])));
    let srgb: [[f64; 3]; 3] = std::array::from_fn(|r| std::array::from_fn(|c| f(SRGB_COLORANTS[r][c])));
    let sum = |m: &[[f64; 3]; 3], r: usize| m[r][0] + m[r][1] + m[r][2];
    let scaled: [[f64; 3]; 3] = std::array::from_fn(|r| source[r].map(|v| v * sum(&srgb, r) / sum(&source, r)));
    let inverse = invert(&srgb);
    let m: [[f64; 3]; 3] = std::array::from_fn(|r| std::array::from_fn(|c| (0..3).map(|k| inverse[r][k] * scaled[k][c]).sum()));
    let largest = m.iter().flatten().fold(0f64, |a, v| a.max(v.abs()));
    let mut shift = 14;
    while shift > 0 && largest * (1 << shift) as f64 >= 32767.5 {
        shift -= 1;
    }
    m.map(|row| row.map(|v| ((v * (1 << shift) as f64 + 0.5).floor() as i64) << (16 - shift)))
}

fn invert(m: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let [[a, b, c], [d, e, f], [g, h, i]] = *m;
    let det = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);
    [[e * i - f * h, c * h - b * i, b * f - c * e], [f * g - d * i, a * i - c * g, c * d - a * f], [d * h - e * g, b * g - a * h, a * e - b * d]]
        .map(|row| row.map(|v| v / det))
}

/// The sRGB encoding of 12-bit linear values, 0..=4096, in 8 bits by way
/// of 12: `round(round(enc(k / 4096) * 4096) * 255 / 4096)`.
fn encode_table() -> Vec<u8> {
    (0..=4096u32)
        .map(|k| {
            let l = k as f64 / 4096.0;
            let e = if l <= 0.0031308 { 12.92 * l } else { 1.055 * l.powf(1.0 / 2.4) - 0.055 };
            let e12 = (e * 4096.0 + 0.5).floor() as u32;
            ((e12 * 255 + 2048) >> 12) as u8
        })
        .collect()
}

fn tags(profile: &[u8]) -> Option<Vec<([u8; 4], &[u8])>> {
    let n = u32::from_be_bytes(profile.get(128..132)?.try_into().ok()?) as usize;
    let mut out = Vec::new();
    for i in 0..n {
        let e = profile.get(132 + 12 * i..144 + 12 * i)?;
        let off = u32::from_be_bytes(e[4..8].try_into().ok()?) as usize;
        let size = u32::from_be_bytes(e[8..12].try_into().ok()?) as usize;
        out.push((e[..4].try_into().ok()?, profile.get(off..off + size)?));
    }
    Some(out)
}

/// Whether CoreGraphics takes colorants for a wide gamut (its
/// `isWideGamutRGB`, which Xcode follows): the area of their triangle in
/// xy chromaticity, from 0.13447 on (1.2327 times sRGB's). Moving the red
/// or the green primary out, or the colorants toward Display P3's or Adobe
/// RGB's, it flipped between 0.134467 and 0.134471; the curves and white
/// point don't count. `None` near the flip.
fn wide_gamut(colorants: &[[i32; 3]; 3]) -> Option<bool> {
    let xy = |c: &[i32; 3]| {
        let sum = (c[0] + c[1] + c[2]) as f64;
        (c[0] as f64 / sum, c[1] as f64 / sum)
    };
    let [(a, b), (c, d), (e, f)] = colorants.map(|c| xy(&c));
    let area = ((c - a) * (f - b) - (e - a) * (d - b)).abs() / 2.0;
    ((area - 0.134469).abs() > 0.00001).then_some(area > 0.134469)
}

/// The linear range, in units of 2^-28, that Xcode stored in one 8-bit
/// rendition from wide-gamut profiles: values down to -0.00181 and up to
/// 1.00299 were, and -0.00189 and 1.00799 weren't (Display P3, across
/// hues). This takes -0.0017 to 1.002; outside it, Xcode adds a 16-bit
/// extended-sRGB rendition or might.
const EIGHT_BIT_RANGE: std::ops::RangeInclusive<i64> = -456_340..=268_972_327;

/// How a profile's colours convert to sRGB.
pub struct Conversion {
    curves: [[i64; 256]; 3],
    matrix: [[i64; 3]; 3],
    /// Whether colours outside sRGB give Xcode a second, 16-bit rendition.
    wide: bool,
}

/// The conversion of an RGB profile with colorants and curves, where
/// established.
pub fn conversion(profile: &[u8]) -> Result<Conversion, String> {
    let unsupported = |what: &str| Err(format!("this PNG's ICC profile {what}, which is not supported by this actool"));
    if profile.get(16..24) != Some(b"RGB XYZ ") {
        return unsupported("isn't an RGB profile with an XYZ connection space");
    }
    let Some(tags) = tags(profile) else {
        return unsupported("is malformed");
    };
    let tag = |sig: &[u8; 4]| tags.iter().find(|t| &t.0 == sig).map(|t| t.1);
    if [b"A2B0", b"A2B1", b"A2B2"].iter().any(|sig| tag(sig).is_some()) {
        return unsupported("has lookup tables");
    }
    let xyz = |sig: &[u8; 4]| -> Option<[i32; 3]> {
        let t = tag(sig)?;
        if t.len() < 20 || &t[..4] != b"XYZ " {
            return None;
        }
        Some([0, 1, 2].map(|k| i32::from_be_bytes(t[8 + 4 * k..12 + 4 * k].try_into().unwrap())))
    };
    let (Some(r), Some(g), Some(b)) = (xyz(b"rXYZ"), xyz(b"gXYZ"), xyz(b"bXYZ")) else {
        return unsupported("has no colorants");
    };
    let colorants = [r, g, b];
    let (Some(trc), Some(g), Some(b)) = (tag(b"rTRC"), tag(b"gTRC"), tag(b"bTRC")) else {
        return unsupported("has no curves");
    };
    // With a different curve per channel, ColorSync widens every channel's
    // input to 12 bits, and how it then reads an sRGB table isn't
    // established.
    if trc != g || trc != b {
        return unsupported("has a different curve per channel");
    }
    let curve = Curve::parse(trc)?.table();
    let curves = [curve; 3];
    let Some(wide) = wide_gamut(&colorants) else {
        return unsupported("has a gamut at the edge of wide, which is not established");
    };
    Ok(Conversion { curves, matrix: matrix(&colorants), wide })
}

impl Conversion {
    /// Converts unpremultiplied RGBA pixels to sRGB in place, alpha as is
    /// (Xcode premultiplies after converting). Refuses images Xcode would
    /// give a 16-bit rendition, leaving the pixels as they were.
    pub fn convert(&self, rgba: &mut [u8]) -> Result<(), String> {
        let linear = |p: &[u8], c: usize| (0..3).map(|k| self.matrix[c][k] * self.curves[k][p[k] as usize]).sum::<i64>();
        if self.wide && rgba.chunks_exact(4).any(|p| (0..3).any(|c| !EIGHT_BIT_RANGE.contains(&linear(p, c)))) {
            return Err("this PNG has colours outside sRGB, for which Xcode adds a 16-bit extended-sRGB rendition this actool doesn't write".into());
        }
        let table = encode_table();
        for p in rgba.chunks_exact_mut(4) {
            let acc = [0, 1, 2].map(|c| linear(p, c));
            for (c, acc) in acc.into_iter().enumerate() {
                let index = (acc >> 15).clamp(0, 8191);
                p[c] = table[((index + 1) >> 1) as usize];
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fnv1a(bytes: impl IntoIterator<Item = u8>) -> u64 {
        bytes.into_iter().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x0100_0000_01b3))
    }

    /// The derived encoding equals the table measured from CoreGraphics
    /// (all 4097 entries), and a platform whose `powf` rounded a value
    /// differently would fail here rather than in a build.
    #[test]
    fn encode_table_is_as_measured() {
        assert_eq!(fnv1a(encode_table()), 0xe056_bd75_ccb0_f653);
    }

    /// The curves equal the tables measured from CoreGraphics.
    #[test]
    fn curves_are_as_measured() {
        let hash = |c: Curve| fnv1a(c.table().iter().flat_map(|&v| (v as u16).to_le_bytes()));
        assert_eq!(hash(Curve::GammaWithToe(1.8)), 0x0d4f_dac5_739e_097b);
        assert_eq!(hash(Curve::Gamma(2.4)), 0x0ccb_b576_eea3_8a40);
        assert_eq!(hash(Curve::Gamma22), 0x5165_f33f_0e37_2af8);
    }

    /// The derivation gives the matrices read back from CoreGraphics' float
    /// path (ROMM's past 2.0, and so over 2^13).
    #[test]
    fn matrices_are_as_measured() {
        let cases: [([[i32; 3]; 3], [[i64; 3]; 3]); 5] = [
            ([[0x744d, 0x3dee, 0x03d0], [0x5a75, 0xac73, 0x1734], [0x281a, 0x159f, 0xb836]], [[67192, -1740, 84], [1272, 62128, 2136], [-116, -96, 65744]]),
            ([[0x6fa2, 0x38f5, 0x0390], [0x6299, 0xb785, 0x18da], [0x24a0, 0x0f84, 0xb6cf]], [[65536, 0, 0], [0, 65536, 0], [0, 0, 65536]]),
            // Display P3
            ([[33759, 15807, -69], [19135, 45367, 2745], [10296, 4363, 51385]], [[80280, -14744, 0], [-2760, 68292, 0], [-1288, -5152, 71972]]),
            // ITU-2020
            ([[44137, 18287, -127], [10857, 44259, 1965], [8195, 2989, 52222]], [[108828, -38516, -4776], [-8168, 74248, -548], [-1188, -6592, 73316]]),
            // ROMM RGB
            ([[52276, 18877, 0], [8860, 46654, 0], [2055, 6, 54080]], [[133320, -47688, -20096], [-15000, 80728, -192], [-560, -10048, 76136]]),
        ];
        for (colorants, measured) in cases {
            assert_eq!(matrix(&colorants), measured);
        }
    }

    /// The system profiles CoreGraphics calls wide and narrow.
    #[test]
    fn wide_gamuts() {
        let generic = [[0x744d, 0x3dee, 0x03d0], [0x5a75, 0xac73, 0x1734], [0x281a, 0x159f, 0xb836]];
        let p3 = [[33759, 15807, -69], [19135, 45367, 2745], [10296, 4363, 51385]];
        let adobe = [[39960, 20389, 1276], [13453, 41004, 3989], [9777, 4143, 48796]];
        let display = [[29120, 14730, 359], [24867, 47590, 5110], [9202, 3216, 48592]];
        let srgb = [[28578, 14581, 912], [25241, 46981, 6362], [9376, 3972, 46799]];
        assert_eq!([generic, display, srgb].map(|c| wide_gamut(&c)), [Some(false); 3]);
        assert_eq!([p3, adobe].map(|c| wide_gamut(&c)), [Some(true); 2]);
    }

    /// sRGB's own table (1024 entries) is taken for sRGB's curve; a table
    /// 8 away isn't.
    #[test]
    fn srgb_tables() {
        let srgb = |x: f64| if x > 0.04045 { ((x + 0.055) / 1.055).powf(2.4) } else { x / 12.92 };
        let table = |k: u16| (0..1024).map(|i| ((srgb(i as f64 / 1023.0) * 65535.0 + 0.5).floor() as u16).saturating_add(if i > 0 && i < 1023 { k } else { 0 })).collect::<Vec<_>>();
        assert!(srgb_like(&table(0)));
        assert!(srgb_like(&table(4)));
        assert!(!srgb_like(&table(8)));
    }
}
