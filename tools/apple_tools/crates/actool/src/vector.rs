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
pub fn pixels(kind: Kind, points: f64, scale: f64) -> u32 {
    let v = points * scale;
    match kind {
        Kind::Pdf => (v + 0.5).floor() as u32,
        Kind::Svg => (v - 1e-9).ceil() as u32,
    }
}

/// The pixels of the image a bitmap's slice says it holds: a side of
/// `points` at `scale` rounded to the nearest, halves up, for both kinds.
/// An SVG's bitmap is rounded up, so its slice can be a pixel short of it
/// (ten probes).
pub fn slice_pixels(points: f64, scale: f64) -> u32 {
    (points * scale + 0.5).floor() as u32
}

/// The image drawn at `scale` (its size in points, `points`, times it)
/// into a bitmap of `pixels`, transparent where it paints nothing.
pub fn render(kind: Kind, data: &[u8], points: (f64, f64), scale: f64, pixels: (u32, u32)) -> Result<Rgba, String> {
    let (width, height) = pixels;
    let premultiplied: Vec<u8> = match kind {
        Kind::Svg => {
            let tree = usvg::Tree::from_data(data, &usvg::Options::default()).map_err(|e| e.to_string())?;
            let fit = (scale * points.0 / tree.size().width() as f64, scale * points.1 / tree.size().height() as f64);
            // Opaque fills, drawn as CoreGraphics draws a PDF of them (Xcode
            // draws the two the same: probes of a PDF and an SVG of one
            // drawing at 1x, 2x and 3x).
            // Read in double precision where this actool can (crate::svgfills),
            // else from usvg's float32 paths.
            if let Some(fills) = crate::svgfills::fills(data, fit, tree.size().height() as f64).or_else(|| svg_fills(tree.root(), fit, tree.size().height() as f64)) {
                return Ok(straight(draw_fills(&fills, 1.0, width as usize, height as usize), width, height));
            }
            let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or("an empty image")?;
            let s = scale as f32;
            let fit = tiny_skia::Transform::from_scale(s * points.0 as f32 / tree.size().width(), s * points.1 as f32 / tree.size().height());
            resvg::render(&tree, fit, &mut pixmap.as_mut());
            pixmap.take()
        }
        // A page of opaque fills, drawn as CoreGraphics draws it, from the
        // bitmap's lower left corner (its top a fraction of a pixel short of
        // the bitmap's, where the bitmap is rounded up: probes of pages of
        // 200.5 x 150.75 and 200.25 x 150.5 points, a PDF's and an SVG's the
        // same).
        Kind::Pdf if crate::pdf::fills(data).is_some() => draw_fills(&crate::pdf::fills(data).expect("fills").0, scale, width as usize, height as usize),
        Kind::Pdf => {
            use hayro::hayro_interpret::util::TransformExt;
            use hayro::vello_cpu;
            let pdf = hayro::hayro_syntax::Pdf::new(data.to_vec()).map_err(|e| format!("not a PDF Xcode could read ({e:?})"))?;
            let pages = pdf.pages();
            let page = &pages[0];
            let (w, h) = (u16::try_from(width).map_err(|_| "too wide")?, u16::try_from(height).map_err(|_| "too tall")?);
            let mut ctx = vello_cpu::RenderContext::new(w, h);
            let transform = hayro::kurbo::Affine::scale(scale) * page.initial_transform(true).to_kurbo();
            let cache = hayro::RenderCache::new();
            hayro::render_into(page, &cache, &Default::default(), &Default::default(), &mut ctx, transform);
            ctx.flush();
            let mut pixmap = vello_cpu::Pixmap::new(w, h);
            let settings = vello_cpu::RasterizerSettings { target_init: vello_cpu::TargetInit::Clear(vello_cpu::color::palette::css::TRANSPARENT), ..Default::default() };
            ctx.render_with(&mut pixmap, &mut vello_cpu::Resources::default(), settings);
            pixmap.data_as_u8_slice().to_vec()
        }
    };
    Ok(straight(premultiplied, width, height))
}

/// Straight alpha, as a PNG's pixels are (premultiplying again gives every
/// premultiplied value back).
fn straight(premultiplied: Vec<u8>, width: u32, height: u32) -> Rgba {
    let mut out = Vec::with_capacity(premultiplied.len());
    for p in premultiplied.chunks_exact(4) {
        let a = p[3] as u32;
        let c = |c: u8| if a == 0 { 0 } else { ((c as u32 * 255 + a / 2) / a).min(255) as u8 };
        out.extend_from_slice(&[c(p[0]), c(p[1]), c(p[2]), p[3]]);
    }
    Rgba { width: width as usize, height: height as usize, data: out }
}

/// An SVG's paths as fills in pixels, y up from its bottom edge (`height`,
/// in its own units), when all it draws is opaque
/// fills of a colour (no strokes, gradients, clips, masks, filters, group
/// opacity, images or text, and no quadratic curves); None otherwise.
fn svg_fills(group: &usvg::Group, fit: (f64, f64), height: f64) -> Option<Vec<crate::pdf::Fill>> {
    use crate::cgraster::{Element, Point};
    if group.opacity().get() != 1.0 || group.clip_path().is_some() || group.mask().is_some() || !group.filters().is_empty() || group.blend_mode() != usvg::BlendMode::Normal {
        return None;
    }
    let mut out = Vec::new();
    for node in group.children() {
        match node {
            usvg::Node::Group(g) => out.extend(svg_fills(g, fit, height)?),
            usvg::Node::Path(p) if !p.is_visible() => {}
            usvg::Node::Path(p) => {
                let fill = p.fill()?;
                let usvg::Paint::Color(c) = fill.paint() else { return None };
                if p.stroke().is_some() || fill.opacity().get() != 1.0 {
                    return None;
                }
                let t = p.abs_transform();
                let at = |q: usvg::tiny_skia_path::Point| {
                    let (x, y) = (q.x as f64, q.y as f64);
                    let (x, y) = (t.sx as f64 * x + t.kx as f64 * y + t.tx as f64, t.ky as f64 * x + t.sy as f64 * y + t.ty as f64);
                    Point { x: x * fit.0, y: (height - y) * fit.1 }
                };
                let mut path = Vec::new();
                for s in p.data().segments() {
                    use usvg::tiny_skia_path::PathSegment;
                    path.push(match s {
                        PathSegment::MoveTo(a) => Element::Move(at(a)),
                        PathSegment::LineTo(a) => Element::Line(at(a)),
                        PathSegment::CubicTo(a, b, c) => Element::Cubic(at(a), at(b), at(c)),
                        PathSegment::Close => Element::Close,
                        PathSegment::QuadTo(..) => return None,
                    });
                }
                out.push(crate::pdf::Fill { path, even_odd: fill.rule() == usvg::FillRule::EvenOdd, colour: [c.red, c.green, c.blue] });
            }
            _ => return None,
        }
    }
    Some(out)
}

/// A page's fills drawn as CoreGraphics draws them into an 8-bit
/// premultiplied sRGB bitmap at `scale`: each one's coverage
/// (crate::cgraster), each channel composited as
/// `floor((c · a + d · (255 − a)) / 255)` over what is there (384 of 384
/// channels probed, over clear, opaque and translucent pixels). Rows top
/// first.
fn draw_fills(fills: &[crate::pdf::Fill], scale: f64, width: usize, height: usize) -> Vec<u8> {
    use crate::cgraster::{coverage, polygons, Element, Point, Rule};
    let mut out = vec![0u8; width * height * 4];
    let px = |p: Point| Point { x: p.x * scale, y: p.y * scale };
    for f in fills {
        let path: Vec<Element> = f
            .path
            .iter()
            .map(|e| match *e {
                Element::Move(p) => Element::Move(px(p)),
                Element::Line(p) => Element::Line(px(p)),
                Element::Cubic(a, b, c) => Element::Cubic(px(a), px(b), px(c)),
                Element::Close => Element::Close,
            })
            .collect();
        let rule = if f.even_odd { Rule::EvenOdd } else { Rule::NonZero };
        let colour = [f.colour[0], f.colour[1], f.colour[2], 255];
        for (y, row) in coverage(&polygons(&path), width, height, rule).iter().enumerate() {
            let at = (height - 1 - y) * width * 4;
            for (x, &a) in row.iter().enumerate().filter(|(_, a)| **a != 0) {
                let a = a as u32;
                for (c, d) in colour.iter().zip(&mut out[at + x * 4..at + x * 4 + 4]) {
                    *d = ((*c as u32 * a + *d as u32 * (255 - a)) / 255) as u8;
                }
            }
        }
    }
    out
}

/// A Vector rendition (layout 9): flags 4, no size or scale, pixel format
/// `PDF ` or `SVG ` (stored backwards), the composition and 0x3ee TLVs, and
/// the file as `DWAR`, a flag and its length: an SVG LZFSE-compressed (flag
/// 1) when that is smaller, a PDF uncompressed (flag 0) as CoreGraphics
/// writes it again (crate::pdf), or as it is where this actool can't.
pub fn csi(name: &str, kind: Kind, data: &[u8]) -> Vec<u8> {
    let rewritten = match kind {
        Kind::Pdf => crate::pdf::rewrite(data).inspect_err(|e| eprintln!("note: {name}: stored as it is, not as Xcode writes it again: {e}")).ok(),
        Kind::Svg => None,
    };
    let data = rewritten.as_deref().unwrap_or(data);
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
