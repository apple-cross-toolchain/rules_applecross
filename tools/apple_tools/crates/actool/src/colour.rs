//! Colour-managed PNGs converted to sRGB as Xcode's actool converts them
//! (through CoreGraphics; docs/clean-room/actool.md#colour-managed-pngs),
//! for the profiles whose conversion is established. Each channel's curve
//! gives a 12-bit linear value, a matrix in 14 fractional bits mixes them,
//! the sum is floored to 13 bits and rounded to 12, and a table encodes it
//! as sRGB. The curves' and matrices' values and the table's exceptions are
//! measured from Xcode's output, not derived; that section says how, should
//! anyone want to replace them with a derivation.

/// Generic RGB's curve (`curv` gamma 461/256, taken as 1.8) in 12 bits:
/// `max(round(x^1.8 * 4096), v)`, the 1/16 toe (measured, all 256).
const GAMMA_18_TOE: [u16; 256] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 14, 17, 19, 22, 25,
    28, 31, 35, 38, 42, 46, 50, 54, 58, 63, 67, 72, 77, 82, 87, 92,
    98, 103, 109, 115, 121, 127, 133, 139, 146, 153, 159, 166, 173, 180, 188, 195,
    203, 210, 218, 226, 234, 242, 251, 259, 268, 276, 285, 294, 303, 312, 321, 331,
    340, 350, 360, 369, 379, 390, 400, 410, 421, 431, 442, 453, 464, 475, 486, 497,
    508, 520, 531, 543, 555, 567, 579, 591, 603, 616, 628, 641, 654, 667, 680, 693,
    706, 719, 732, 746, 760, 773, 787, 801, 815, 829, 844, 858, 872, 887, 902, 917,
    931, 947, 962, 977, 992, 1008, 1023, 1039, 1055, 1071, 1087, 1103, 1119, 1135, 1151, 1168,
    1185, 1201, 1218, 1235, 1252, 1269, 1286, 1304, 1321, 1339, 1356, 1374, 1392, 1410, 1428, 1446,
    1464, 1483, 1501, 1520, 1538, 1557, 1576, 1595, 1614, 1633, 1652, 1672, 1691, 1711, 1730, 1750,
    1770, 1790, 1810, 1830, 1851, 1871, 1891, 1912, 1933, 1953, 1974, 1995, 2016, 2037, 2059, 2080,
    2101, 2123, 2145, 2166, 2188, 2210, 2232, 2254, 2276, 2299, 2321, 2344, 2366, 2389, 2412, 2435,
    2458, 2481, 2504, 2527, 2551, 2574, 2598, 2621, 2645, 2669, 2693, 2717, 2741, 2765, 2790, 2814,
    2839, 2863, 2888, 2913, 2938, 2963, 2988, 3013, 3038, 3063, 3089, 3114, 3140, 3166, 3192, 3218,
    3244, 3270, 3296, 3322, 3349, 3375, 3402, 3428, 3455, 3482, 3509, 3536, 3563, 3590, 3618, 3645,
    3673, 3700, 3728, 3756, 3783, 3811, 3839, 3868, 3896, 3924, 3953, 3981, 4010, 4038, 4067, 4096,
];

/// BT.709's parametric curve, replaced by gamma 2.4, in 12 bits:
/// `round(x^2.4 * 4096)` (measured, all 256).
const GAMMA_24: [u16; 256] = [
    0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 3, 3, 4, 5,
    5, 6, 7, 8, 9, 10, 11, 13, 14, 16, 17, 19, 20, 22, 24, 26,
    28, 30, 33, 35, 37, 40, 42, 45, 48, 51, 54, 57, 60, 64, 67, 71,
    74, 78, 82, 86, 90, 94, 99, 103, 108, 112, 117, 122, 127, 132, 138, 143,
    148, 154, 160, 166, 172, 178, 184, 190, 197, 204, 210, 217, 224, 231, 239, 246,
    254, 261, 269, 277, 285, 293, 302, 310, 319, 327, 336, 345, 355, 364, 373, 383,
    393, 403, 413, 423, 433, 444, 454, 465, 476, 487, 498, 510, 521, 533, 545, 556,
    569, 581, 593, 606, 619, 631, 644, 658, 671, 684, 698, 712, 726, 740, 754, 769,
    783, 798, 813, 828, 843, 859, 874, 890, 906, 922, 938, 955, 971, 988, 1005, 1022,
    1039, 1057, 1074, 1092, 1110, 1128, 1146, 1165, 1183, 1202, 1221, 1240, 1259, 1279, 1298, 1318,
    1338, 1358, 1379, 1399, 1420, 1441, 1462, 1483, 1505, 1526, 1548, 1570, 1592, 1614, 1637, 1659,
    1682, 1705, 1729, 1752, 1775, 1799, 1823, 1847, 1872, 1896, 1921, 1946, 1971, 1996, 2021, 2047,
    2073, 2099, 2125, 2152, 2178, 2205, 2232, 2259, 2286, 2314, 2342, 2369, 2398, 2426, 2454, 2483,
    2512, 2541, 2570, 2600, 2629, 2659, 2689, 2720, 2750, 2781, 2812, 2843, 2874, 2905, 2937, 2969,
    3001, 3033, 3066, 3098, 3131, 3164, 3197, 3231, 3265, 3299, 3333, 3367, 3401, 3436, 3471, 3506,
    3541, 3577, 3613, 3649, 3685, 3721, 3758, 3794, 3831, 3868, 3906, 3943, 3981, 4019, 4058, 4096,
];

/// Where the sRGB encoding of `k / 4096` (k = 0..4096) is not
/// `round(enc(k / 4096) * 255)`: each within 0.03 of a half (measured).
const ENCODE_EXCEPTIONS: [(u16, u8); 65] = [
    (85, 39), (101, 44), (188, 61), (292, 76), (308, 78), (316, 79), (333, 80), (350, 82),
    (495, 97), (657, 112), (683, 113), (722, 116), (847, 126), (892, 128), (907, 129), (922, 130),
    (953, 132), (1134, 143), (1222, 148), (1333, 155), (1352, 156), (1371, 157), (1410, 159), (1430, 159),
    (1450, 160), (1470, 161), (1531, 164), (1657, 171), (1790, 176), (1835, 178), (1858, 179), (1999, 186),
    (2072, 189), (2121, 191), (2147, 191), (2172, 192), (2197, 193), (2274, 196), (2300, 197), (2379, 201),
    (2514, 206), (2570, 207), (2598, 208), (2626, 209), (2654, 210), (2740, 213), (2798, 216), (2916, 220),
    (2946, 221), (3007, 223), (3038, 223), (3069, 224), (3100, 225), (3131, 226), (3419, 236), (3452, 237),
    (3485, 238), (3518, 239), (3553, 239), (3620, 241), (3723, 244), (3969, 252), (4005, 253), (4041, 254),
    (4077, 255),
];

/// A profile whose conversion is established: its colorants and white
/// point (the XYZ tags' s15Fixed16 values), its one curve tag for all three
/// channels, the curve in 12 bits and the matrix (s15Fixed16; units of
/// 2^-16, though always multiples of 4).
pub struct Known {
    pub name: &'static str,
    colorants: [[i32; 3]; 3],
    white: [i32; 3],
    trc: &'static [u8],
    curve: &'static [u16; 256],
    matrix: [[i64; 3]; 3],
}

/// The profiles converted, each verified for all 16,777,216 colours.
const KNOWN: [Known; 2] = [
    Known {
        name: "Generic RGB",
        colorants: [[0x744d, 0x3dee, 0x03d0], [0x5a75, 0xac73, 0x1734], [0x281a, 0x159f, 0xb836]],
        white: [0xf352, 0x10000, 0x116cf],
        trc: &[0x63, 0x75, 0x72, 0x76, 0, 0, 0, 0, 0, 0, 0, 1, 0x01, 0xcd],
        curve: &GAMMA_18_TOE,
        matrix: [[67192, -1740, 84], [1272, 62128, 2136], [-116, -96, 65744]],
    },
    Known {
        name: "ITU-709",
        colorants: [[0x6fa2, 0x38f5, 0x0390], [0x6299, 0xb785, 0x18da], [0x24a0, 0x0f84, 0xb6cf]],
        white: [0xf6d5, 0x10000, 0xd32c],
        trc: &[
            0x70, 0x61, 0x72, 0x61, 0, 0, 0, 0, 0, 3, 0, 0, 0, 2, 0x38, 0xe4, 0, 0, 0xe8, 0xf0, 0, 0, 0x17, 0x10, 0, 0, 0x38, 0xe4,
            0, 0, 0x14, 0xbc,
        ],
        curve: &GAMMA_24,
        matrix: [[65536, 0, 0], [0, 65536, 0], [0, 0, 65536]],
    },
];

/// The sRGB encoding of 12-bit linear values, 0..=4096.
fn encode_table() -> Vec<u8> {
    let mut t: Vec<u8> = (0..=4096u32)
        .map(|k| {
            let l = k as f64 / 4096.0;
            let e = if l <= 0.0031308 { 12.92 * l } else { 1.055 * l.powf(1.0 / 2.4) - 0.055 };
            (e * 255.0).round() as u8
        })
        .collect();
    for (k, v) in ENCODE_EXCEPTIONS {
        t[k as usize] = v;
    }
    t
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
        for p in rgba.chunks_exact_mut(4) {
            let q = [0, 1, 2].map(|c| self.curve[p[c] as usize] as i64);
            for (c, row) in self.matrix.iter().enumerate() {
                let acc = row[0] * q[0] + row[1] * q[1] + row[2] * q[2];
                let index = (acc >> 15).clamp(0, 8191);
                p[c] = table[((index + 1) >> 1) as usize];
            }
        }
    }
}

/// FNV-1a of the table as measured.
#[cfg(test)]
const ENCODE_TABLE_FNV1A: u64 = 0xe056bd75ccb0f653;

#[cfg(test)]
mod tests {
    use super::*;

    /// The table as measured on macOS: a platform whose `powf` rounds a
    /// value differently would fail here rather than in a build.
    #[test]
    fn encode_table_is_as_measured() {
        let hash = encode_table().iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| (h ^ b as u64).wrapping_mul(0x0100_0000_01b3));
        assert_eq!(hash, ENCODE_TABLE_FNV1A);
    }
}
