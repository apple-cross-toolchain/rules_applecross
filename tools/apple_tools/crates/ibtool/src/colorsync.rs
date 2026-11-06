//! The colour conversions Xcode's ibtool archives, reproduced bit for bit.
//!
//! ibtool converts a calibrated RGB colour to Display P3 and on to extended
//! sRGB, a Display P3 colour to extended sRGB, and a calibrated white to the
//! gamma 2.2 grey, all in single precision. Each conversion is one or two
//! steps of: a parametric curve per channel, a 3×3 matrix, a parametric
//! curve per channel.
//!
//! Provenance (docs/clean-room/ibtool.md, Colour conversion):
//! - The steps' kinds and float parameters are ColorSync's own description
//!   of the transform (`ColorSyncTransformCopyProperty` with
//!   `kColorSyncTransformParametricConversionData`, public API).
//! - The curves are evaluated as vImage's public
//!   `vImageSymmetricPiecewiseGamma_PlanarF` evaluates them (CoreGraphics
//!   converts through vImage), which this module matches for every float
//!   from -1.25 to 1.25.
//! - vImage's power function, `exp2(g · log2 x)` in floats, was found by
//!   calling that function on constructed inputs in our own process: its
//!   exp2 for every float fraction, its log2 for every mantissa. The
//!   polynomials below reproduce them but for the listed exceptions
//!   (`vimage_pow.bin`, vImage's own values, recorded exhaustively).
//! - The matrices' evaluation order was fitted to CoreGraphics' outputs:
//!   20,000 random colours, every component exact in both steps.

use std::sync::OnceLock;

/// vImage's exp2 of a fraction in [0, 1): `1 + x·q(x)`, `q` by fused
/// Horner steps, the last product rounded before adding 1.
const EXP2: [u32; 7] = [0x3f800000, 0x3f317218, 0x3e75fe20, 0x3d634f9e, 0x3c1e2639, 0x3aa649f5, 0x395a8346];
/// vImage's log2 of a mantissa m in [1, √2) and in [√2/2, 1): `r·q(r)` with
/// r = m − 1, `q` the fused sum of its even and odd parts, each by fused
/// Horner steps in r².
const LOG2_ABOVE: [u32; 7] = [0x3fb8aa3b, 0xbf38aa06, 0x3ef62766, 0xbeb7a5bf, 0x3e8c16d0, 0xbe383a67, 0x3d8c36eb];
const LOG2_BELOW: [u32; 7] = [0x3fb8aa3b, 0xbf38a9f3, 0x3ef6559a, 0xbeb66ff0, 0x3ea80bf8, 0xbd7a8021, 0x3f126d6c];

/// vImage's values where the polynomials differ: exp2's by fraction, then
/// log2's by mantissa, above and below 1; each a count and sorted
/// (input, output) bit pairs.
const EXCEPTIONS: &[u8] = include_bytes!("vimage_pow.bin");

struct Exceptions {
    exp2: Vec<(u32, u32)>,
    log2_above: Vec<(u32, u32)>,
    log2_below: Vec<(u32, u32)>,
}

fn exceptions() -> &'static Exceptions {
    static TABLES: OnceLock<Exceptions> = OnceLock::new();
    TABLES.get_or_init(|| {
        let mut at = 0;
        let mut word = || {
            let w = u32::from_le_bytes(EXCEPTIONS[at..at + 4].try_into().expect("four bytes"));
            at += 4;
            w
        };
        let mut table = || {
            let n = word() as usize;
            (0..n).map(|_| (word(), word())).collect::<Vec<_>>()
        };
        let exp2 = table();
        let log2_above = table();
        let log2_below = table();
        Exceptions { exp2, log2_above, log2_below }
    })
}

fn listed(table: &[(u32, u32)], x: f32) -> Option<f32> {
    table.binary_search_by_key(&x.to_bits(), |e| e.0).ok().map(|i| f32::from_bits(table[i].1))
}

fn coefficients(bits: [u32; 7]) -> [f32; 7] {
    bits.map(f32::from_bits)
}

/// 2^x for x in [0, 1).
fn exp2_fraction(x: f32) -> f32 {
    if let Some(y) = listed(&exceptions().exp2, x) {
        return y;
    }
    let c = coefficients(EXP2);
    let mut q = c[6];
    for k in (1..6).rev() {
        q = q.mul_add(x, c[k]);
    }
    q * x + c[0]
}

/// 2^t, values below the smallest normal float flushed to zero as vImage
/// flushes them.
fn exp2(t: f32) -> f32 {
    let i = t.floor();
    let p = exp2_fraction(t - i);
    let i = i as i32;
    if i < -126 {
        return 0.0;
    }
    // p is in [1, 2): scale by 2^i in two exact steps.
    let half = i / 2;
    p * f32::from_bits(((127 + half) as u32) << 23) * f32::from_bits(((127 + i - half) as u32) << 23)
}

/// log2 of a mantissa in [√2/2, √2).
fn log2_mantissa(m: f32) -> f32 {
    let below = m < 1.0;
    let table = if below { &exceptions().log2_below } else { &exceptions().log2_above };
    if let Some(y) = listed(table, m) {
        return y;
    }
    let c = coefficients(if below { LOG2_BELOW } else { LOG2_ABOVE });
    let r = m - 1.0;
    let r2 = r * r;
    let even = c[6].mul_add(r2, c[4]).mul_add(r2, c[2]).mul_add(r2, c[0]);
    let odd = c[5].mul_add(r2, c[3]).mul_add(r2, c[1]);
    r * odd.mul_add(r, even)
}

/// log2 of a positive normal float: its exponent plus the log2 of its
/// mantissa in [√2/2, √2), added in floats.
fn log2(x: f32) -> f32 {
    let bits = x.to_bits();
    let mut e = ((bits >> 23) & 0xff) as i32 - 127;
    let mut m = f32::from_bits((bits & 0x007f_ffff) | 0x3f80_0000);
    if m >= f32::from_bits(0x3fb5_04f3) {
        m *= 0.5;
        e += 1;
    }
    e as f32 + log2_mantissa(m)
}

/// vImage's x^g for a positive normal x.
fn pow(x: f32, g: f32) -> f32 {
    let y = exp2(g * log2(x));
    if y.abs() < f32::MIN_POSITIVE { 0.0 } else { y }
}

/// A parametric curve: `(a·|x| + b)^g + e` from `d` up, `c·|x| + f` below
/// it, with the sign of x.
struct Curve {
    g: f32,
    a: f32,
    b: f32,
    e: f32,
    c: f32,
    f: f32,
    d: f32,
}

impl Curve {
    fn apply(&self, x: f32) -> Result<f32, String> {
        let ax = x.abs();
        let y = if ax >= self.d {
            let base = self.a.mul_add(ax, self.b);
            if !base.is_normal() {
                return Err(format!("a colour component {x} is converted in a way not established for this ibtool"));
            }
            pow(base, self.g) + self.e
        } else {
            self.c * ax + self.f
        };
        Ok(if x < 0.0 { -y } else { y })
    }
}

/// Generic RGB's (and Generic Gray's) decoding: gamma 1.8 with a linear toe.
const DECODE_GENERIC: Curve = Curve { g: 1.799_999_952_316_284_2, a: 1.0, b: 0.0, e: 0.0, c: 0.0625, f: 0.0, d: 0.031_25 };
/// Display P3's (sRGB's) decoding.
const DECODE_SRGB: Curve = Curve {
    g: 2.400_000_095_367_431_6,
    a: 0.947_867_274_284_362_8,
    b: 0.052_132_699_638_605_12,
    e: 0.0,
    c: 0.077_399_380_505_084_99,
    f: 0.0,
    d: 0.040_449_999_272_823_334,
};
/// Display P3's, extended sRGB's and gamma 2.2 grey's encoding.
const ENCODE_SRGB: Curve = Curve {
    g: 0.416_666_656_732_559_2,
    a: 1.137_119_054_794_311_5,
    b: 0.0,
    e: -0.054_999_999_701_976_776,
    c: 12.92,
    f: 0.0,
    d: 0.003_130_804_980_173_707,
};

/// Generic RGB's (and Generic Gray's) encoding: gamma 1/1.8 with a linear
/// toe, the inverse of `DECODE_GENERIC`.
const ENCODE_GENERIC: Curve = Curve { g: 0.555_555_582_046_508_8, a: 1.0, b: 0.0, e: 0.0, c: 16.0, f: 0.0, d: 0.001_953_125 };

/// sRGB to Generic RGB.
const SRGB_TO_GENERIC: [[f32; 3]; 3] = [
    [0.974_859_595_298_767_1, 0.027_293_762_192_130_09, -0.002_153_418_958_187_103_3],
    [-0.020_008_111_372_590_065, 1.054_230_332_374_572_8, -0.034_222_252_666_950_226],
    [0.001_683_750_539_086_759_1, 0.001_555_427_210_405_468_9, 0.996_760_845_184_326_2],
];

/// Generic RGB to Display P3, then Display P3 to sRGB.
const GENERIC_TO_P3: [[f32; 3]; 3] = [
    [0.846_630_096_435_546_9, 0.146_494_120_359_420_78, 0.006_875_693_332_403_898],
    [0.052_814_990_282_058_716, 0.915_651_559_829_711_9, 0.031_533_464_789_390_564],
    [0.017_304_770_648_479_462, 0.066_852_517_426_013_95, 0.915_842_711_925_506_6],
];
const P3_TO_SRGB: [[f32; 3]; 3] = [
    [1.225_002_765_655_517_6, -0.224_975_526_332_855_22, -2.723_318_539_210_595_2e-5],
    [-0.042_092_215_269_804, 1.042_070_746_421_814, 2.146_464_066_754_561e-5],
    [-0.019_624_916_836_619_377, -0.078_609_950_840_473_17, 1.098_234_891_891_479_5],
];

/// One step: decode each channel, the matrix (each row its middle product,
/// then the first and last terms fused in), encode each channel.
fn step(decode: &Curve, matrix: &[[f32; 3]; 3], encode: &Curve, rgb: [f32; 3]) -> Result<[f32; 3], String> {
    let lin = [decode.apply(rgb[0])?, decode.apply(rgb[1])?, decode.apply(rgb[2])?];
    let mut out = [0.0; 3];
    for (o, row) in out.iter_mut().zip(matrix) {
        let mixed = row[2].mul_add(lin[2], row[0].mul_add(lin[0], row[1] * lin[1]));
        *o = encode.apply(mixed)?;
    }
    Ok(out)
}

/// A calibrated (generic) RGB colour's components in extended sRGB.
pub fn calibrated_rgb(rgb: [f64; 3]) -> Result<[f64; 3], String> {
    let p3 = step(&DECODE_GENERIC, &GENERIC_TO_P3, &ENCODE_SRGB, rgb.map(|x| x as f32))?;
    Ok(step(&DECODE_SRGB, &P3_TO_SRGB, &ENCODE_SRGB, p3)?.map(f64::from))
}

/// A Display P3 colour's components in extended sRGB.
pub fn display_p3(rgb: [f64; 3]) -> Result<[f64; 3], String> {
    Ok(step(&DECODE_SRGB, &P3_TO_SRGB, &ENCODE_SRGB, rgb.map(|x| x as f32))?.map(f64::from))
}

/// A calibrated (generic) white in the gamma 2.2 grey.
pub fn calibrated_white(white: f64) -> Result<f64, String> {
    Ok(f64::from(ENCODE_SRGB.apply(DECODE_GENERIC.apply(white as f32)?)?))
}

/// An sRGB colour's components in Generic RGB, clamped to [0, 1] (in a
/// document without `colorMatched`).
pub fn srgb_to_generic(rgb: [f64; 3]) -> Result<[f64; 3], String> {
    Ok(step(&DECODE_SRGB, &SRGB_TO_GENERIC, &ENCODE_GENERIC, rgb.map(|x| x as f32))?.map(|x| f64::from(x.clamp(0.0, 1.0))))
}

/// A gamma 2.2 grey in Generic RGB (each channel the same), clamped to
/// [0, 1] (in a document without `colorMatched`).
pub fn gamma22_grey_to_generic(white: f64) -> Result<f64, String> {
    Ok(f64::from(ENCODE_GENERIC.apply(DECODE_SRGB.apply(white as f32)?)?.clamp(0.0, 1.0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interface_builder_palette() {
        // Xcode's values for the palette (probes in docs/clean-room/ibtool.md).
        assert_eq!(calibrated_white(1.0 / 3.0).unwrap(), 0.40781933069229126);
        assert_eq!(calibrated_white(0.5).unwrap(), 0.572306752204895);
        assert_eq!(calibrated_white(2.0 / 3.0).unwrap(), 0.7233663201332092);
        assert_eq!(calibrated_white(1.0).unwrap(), 1.0000001192092896);
        assert_eq!(calibrated_rgb([1.0, 0.0, 0.0]).unwrap(), [1.0110152959823608, 0.14910027384757996, -0.02276673913002014]);
        assert_eq!(calibrated_rgb([1.0, 1.0, 1.0]).unwrap(), [0.9999999403953552, 1.0000001192092896, 1.0000001192092896]);
        assert_eq!(calibrated_rgb([0.6, 0.4, 0.2]).unwrap(), [0.6679973006248474, 0.4751102924346924, 0.2586003243923187]);
        assert_eq!(display_p3([1.0, 1.0, 1.0]).unwrap(), [0.9999999403953552, 1.0000001192092896, 1.0000001192092896]);
        // Without colorMatched (probes in docs/clean-room/ibtool.md).
        assert_eq!(srgb_to_generic([1.0, 0.5, 0.25]).unwrap(), [0.9891708493232727, 0.41337311267852783, 0.19499465823173523]);
        assert_eq!(srgb_to_generic([0.0, 1.0, 0.0]).unwrap(), [0.13525280356407166, 1.0, 0.024886835366487503]);
        assert_eq!(gamma22_grey_to_generic(0.5).unwrap(), 0.42467236518859863);
    }
}

