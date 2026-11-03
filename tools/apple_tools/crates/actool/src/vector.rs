//! Vector images, PDF and SVG: Xcode stores each as it is in a Vector
//! rendition (layout 9) and also draws it into a bitmap at each of the
//! platform's scales. The bitmaps are CoreGraphics' and CoreSVG's drawing,
//! which this actool doesn't reproduce: it draws them itself (resvg, hayro),
//! so a catalog with vector images compiles only relaxed
//! (`--config=relax_actool_vector`; docs/clean-room/actool.md).

use crate::icon::{data_csi, tlv, LAYOUT_VECTOR};
use crate::image::Rgba;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Pdf,
    Svg,
}

impl Kind {
    /// The kind of a file Xcode takes as vector content, by its extension.
    pub fn of(filename: &str) -> Option<Kind> {
        let lower = filename.to_ascii_lowercase();
        if lower.ends_with(".pdf") {
            Some(Kind::Pdf)
        } else if lower.ends_with(".svg") {
            Some(Kind::Svg)
        } else {
            None
        }
    }
}

/// A vector image's size in points: an SVG's as crate::icon reads it (its
/// width and height, or its view box), a PDF's first page's (its crop box,
/// rotated).
pub fn size(kind: Kind, data: &[u8]) -> Result<(f64, f64), String> {
    let (w, h) = match kind {
        Kind::Svg => crate::icon::svg_size(data)?,
        Kind::Pdf => {
            let pdf = hayro::hayro_syntax::Pdf::new(data.to_vec()).map_err(|e| format!("not a PDF Xcode could read ({e:?})"))?;
            let pages = pdf.pages();
            if pages.len() != 1 {
                return Err(format!("a PDF of {} pages is not supported by this actool", pages.len()));
            }
            let (w, h) = pages[0].render_dimensions();
            (w as f64, h as f64)
        }
    };
    // An SVG under a point wide draws as wide as its content (one probe).
    if !(w >= 1.0 && h >= 1.0 && w <= 10_000.0 && h <= 10_000.0) {
        return Err(format!("a vector image of {w} x {h} points is not supported by this actool"));
    }
    Ok((w, h))
}

/// The pixels a side of `points` takes at `scale`: a PDF's rounded to the
/// nearest (halves up), an SVG's up (fourteen probes, 0.6 to 170.05
/// points).
pub fn pixels(kind: Kind, points: f64, scale: u32) -> u32 {
    let v = points * scale as f64;
    match kind {
        Kind::Pdf => (v + 0.5).floor() as u32,
        Kind::Svg => (v - 1e-9).ceil() as u32,
    }
}

/// The pixels of the image a bitmap's slice says it holds: a side of
/// `points` at `scale` rounded to the nearest, halves up, for both kinds.
/// An SVG's bitmap is rounded up, so its slice can be a pixel short of it
/// (ten probes).
pub fn slice_pixels(points: f64, scale: u32) -> u32 {
    (points * scale as f64 + 0.5).floor() as u32
}

/// The image drawn at `scale` (its size in points, `points`, times it)
/// into a bitmap of `pixels`, transparent where it paints nothing.
pub fn render(kind: Kind, data: &[u8], points: (f64, f64), scale: u32, pixels: (u32, u32)) -> Result<Rgba, String> {
    let (width, height) = pixels;
    let premultiplied: Vec<u8> = match kind {
        Kind::Svg => {
            let tree = usvg::Tree::from_data(data, &usvg::Options::default()).map_err(|e| e.to_string())?;
            let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or("an empty image")?;
            let s = scale as f32;
            let fit = tiny_skia::Transform::from_scale(s * points.0 as f32 / tree.size().width(), s * points.1 as f32 / tree.size().height());
            resvg::render(&tree, fit, &mut pixmap.as_mut());
            pixmap.take()
        }
        Kind::Pdf => {
            use hayro::hayro_interpret::util::TransformExt;
            use hayro::vello_cpu;
            let pdf = hayro::hayro_syntax::Pdf::new(data.to_vec()).map_err(|e| format!("not a PDF Xcode could read ({e:?})"))?;
            let pages = pdf.pages();
            let page = &pages[0];
            let (w, h) = (u16::try_from(width).map_err(|_| "too wide")?, u16::try_from(height).map_err(|_| "too tall")?);
            let mut ctx = vello_cpu::RenderContext::new(w, h);
            let transform = hayro::kurbo::Affine::scale(scale as f64) * page.initial_transform(true).to_kurbo();
            let cache = hayro::RenderCache::new();
            hayro::render_into(page, &cache, &Default::default(), &Default::default(), &mut ctx, transform);
            ctx.flush();
            let mut pixmap = vello_cpu::Pixmap::new(w, h);
            let settings = vello_cpu::RasterizerSettings { target_init: vello_cpu::TargetInit::Clear(vello_cpu::color::palette::css::TRANSPARENT), ..Default::default() };
            ctx.render_with(&mut pixmap, &mut vello_cpu::Resources::default(), settings);
            pixmap.data_as_u8_slice().to_vec()
        }
    };
    // Straight alpha, as a PNG's pixels are.
    let mut out = Vec::with_capacity(premultiplied.len());
    for p in premultiplied.chunks_exact(4) {
        let a = p[3] as u32;
        let c = |c: u8| if a == 0 { 0 } else { ((c as u32 * 255 + a / 2) / a).min(255) as u8 };
        out.extend_from_slice(&[c(p[0]), c(p[1]), c(p[2]), p[3]]);
    }
    Ok(Rgba { width: width as usize, height: height as usize, data: out })
}

/// A Vector rendition (layout 9): flags 4, no size or scale, pixel format
/// `PDF ` or `SVG ` (stored backwards), the composition and 0x3ee TLVs, and
/// the file as `DWAR`, a flag and its length: an SVG LZFSE-compressed (flag
/// 1) when that is smaller, a PDF always as it is (flag 0).
pub fn csi(name: &str, kind: Kind, data: &[u8]) -> Vec<u8> {
    let (format, compressed) = match kind {
        Kind::Pdf => (b" FDP", None),
        Kind::Svg => (b" GVS", Some(lzfse::encode(data)).filter(|c| c.len() < data.len())),
    };
    let (flag, stored) = match &compressed {
        Some(c) => (1u32, c.as_slice()),
        None => (0, data),
    };
    let mut payload = b"DWAR".to_vec();
    payload.extend_from_slice(&flag.to_le_bytes());
    payload.extend_from_slice(&(stored.len() as u32).to_le_bytes());
    payload.extend_from_slice(stored);
    let mut t = Vec::new();
    let mut composition = 0u32.to_le_bytes().to_vec();
    composition.extend_from_slice(&1.0f32.to_le_bytes());
    tlv(&mut t, 0x3ec, &composition);
    tlv(&mut t, 0x3ee, &1u32.to_le_bytes());
    data_csi(name, LAYOUT_VECTOR, 4, 0, 0, format, &t, &payload)
}
