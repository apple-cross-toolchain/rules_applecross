//! Colour-managed PNGs converted to sRGB as Xcode's actool converts them
//! (through CoreGraphics; docs/clean-room/actool.md#colour-managed-pngs),
//! for the profiles whose conversion is established. Each channel's curve
//! gives a 12-bit linear value, a matrix derived from the profile's
//! colorants mixes them, the sum is floored to 13 bits and rounded to 12,
//! and the sRGB encoding, itself rounded to 12 bits, gives 8.

/// The sRGB profile's colorants (s15Fixed16, XYZ rows by R, G and B
/// columns), the destination of every conversion.
const SRGB_COLORANTS: [[i32; 3]; 3] = [[28578, 25241, 9376], [14581, 46981, 3972], [912, 6362, 46799]];

/// How a known profile's curve gives 12-bit linear values.
#[derive(Clone, Copy)]
enum Curve {
    /// A gamma of 1.5 or more, with its toe: `max(x^g, x / 16)`.
    GammaWithToe(f64),
    /// A pure gamma, as BT.709's parametric curve is taken (as 2.4).
    Gamma(f64),
}

impl Curve {
    /// The curve's 12-bit linear value for each 8-bit input.
    fn table(self) -> [i64; 256] {
        std::array::from_fn(|v| {
            let x = v as f64 / 255.0;
            let l = match self {
                Curve::GammaWithToe(g) => x.powf(g).max(x / 16.0),
                Curve::Gamma(g) => x.powf(g),
            };
            (l * 4096.0 + 0.5).floor() as i64
        })
    }
}

/// A profile whose conversion is established: its colorants (the XYZ
/// tags' s15Fixed16 values, R, G then B) and white point, its one curve tag
/// for all three channels, and how that curve is read.
pub struct Known {
    pub name: &'static str,
    colorants: [[i32; 3]; 3],
    white: [i32; 3],
    trc: &'static [u8],
    curve: Curve,
}

/// The profiles converted, each verified for all 16,777,216 colours.
const KNOWN: [Known; 2] = [
    Known {
        name: "Generic RGB",
        colorants: [[0x744d, 0x3dee, 0x03d0], [0x5a75, 0xac73, 0x1734], [0x281a, 0x159f, 0xb836]],
        white: [0xf352, 0x10000, 0x116cf],
        // `curv` gamma 461/256, taken as 1.8.
        trc: &[0x63, 0x75, 0x72, 0x76, 0, 0, 0, 0, 0, 0, 0, 1, 0x01, 0xcd],
        curve: Curve::GammaWithToe(1.8),
    },
    Known {
        name: "ITU-709",
        colorants: [[0x6fa2, 0x38f5, 0x0390], [0x6299, 0xb785, 0x18da], [0x24a0, 0x0f84, 0xb6cf]],
        white: [0xf6d5, 0x10000, 0xd32c],
        trc: &[
            0x70, 0x61, 0x72, 0x61, 0, 0, 0, 0, 0, 3, 0, 0, 0, 2, 0x38, 0xe4, 0, 0, 0xe8, 0xf0, 0, 0, 0x17, 0x10, 0, 0, 0x38, 0xe4,
            0, 0, 0x14, 0xbc,
        ],
        curve: Curve::Gamma(2.4),
    },
];

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

/// The known profile `profile` is, if any.
pub fn known(profile: &[u8]) -> Option<&'static Known> {
    let tags = tags(profile)?;
    let tag = |sig: &[u8; 4]| tags.iter().find(|t| &t.0 == sig).map(|t| t.1);
    let xyz = |sig: &[u8; 4]| -> Option<[i32; 3]> {
        let t = tag(sig)?;
        if t.len() < 20 || &t[..4] != b"XYZ " {
            return None;
        }
        Some([0, 1, 2].map(|k| i32::from_be_bytes(t[8 + 4 * k..12 + 4 * k].try_into().unwrap())))
    };
    KNOWN.iter().find(|k| {
        [b"rXYZ", b"gXYZ", b"bXYZ"].iter().zip(&k.colorants).all(|(sig, c)| xyz(sig) == Some(*c))
            && xyz(b"wtpt") == Some(k.white)
            && [b"rTRC", b"gTRC", b"bTRC"].iter().all(|sig| tag(sig) == Some(k.trc))
    })
}

impl Known {
    /// Converts unpremultiplied RGBA pixels to sRGB in place, alpha as is
    /// (Xcode premultiplies after converting).
    pub fn convert(&self, rgba: &mut [u8]) {
        let table = encode_table();
        let curve = self.curve.table();
        let matrix = matrix(&self.colorants);
        for p in rgba.chunks_exact_mut(4) {
            let q = [0, 1, 2].map(|c| curve[p[c] as usize]);
            for (c, row) in matrix.iter().enumerate() {
                let acc = row[0] * q[0] + row[1] * q[1] + row[2] * q[2];
                let index = (acc >> 15).clamp(0, 8191);
                p[c] = table[((index + 1) >> 1) as usize];
            }
        }
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

    /// Both curves equal the tables measured from CoreGraphics.
    #[test]
    fn curves_are_as_measured() {
        let hash = |c: Curve| fnv1a(c.table().iter().flat_map(|&v| (v as u16).to_le_bytes()));
        assert_eq!(hash(Curve::GammaWithToe(1.8)), 0x0d4f_dac5_739e_097b);
        assert_eq!(hash(Curve::Gamma(2.4)), 0x0ccb_b576_eea3_8a40);
    }

    /// The derivation gives the matrices read back from CoreGraphics' float
    /// path, for the profiles converted and for system profiles that aren't
    /// (one of them, ROMM, past 2.0 and so over 2^13).
    #[test]
    fn matrices_are_as_measured() {
        let cases: [([[i32; 3]; 3], [[i64; 3]; 3]); 5] = [
            (KNOWN[0].colorants, [[67192, -1740, 84], [1272, 62128, 2136], [-116, -96, 65744]]),
            (KNOWN[1].colorants, [[65536, 0, 0], [0, 65536, 0], [0, 0, 65536]]),
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
}
