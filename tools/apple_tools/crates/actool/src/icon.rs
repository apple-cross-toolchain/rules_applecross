//! Icon Composer documents, rendered flat (default appearance, no glass) for
//! the icon renditions; see docs/clean-room/actool.md.

use crate::image::Rgba;
use std::path::Path;

/// The canvas of an iOS icon, in pixels (and points: Icon Composer's
/// documents are 1024 pt square at 1x).
pub const CANVAS: u32 = 1024;

fn fail<T>(msg: impl Into<String>) -> Result<T, String> {
    Err(msg.into())
}

/// A colour as icon.json writes it: `<space>:<r>,<g>,<b>,<a>`, or one of the
/// system's names.
fn color(spec: &str) -> Result<[f32; 4], String> {
    let (space, rest) = spec.split_once(':').ok_or_else(|| format!("unsupported colour {spec}"))?;
    let c: Vec<f32> = rest.split(',').map(|v| v.trim().parse::<f32>().map_err(|_| format!("unsupported colour {spec}"))).collect::<Result<_, _>>()?;
    match (space, c.len()) {
        // Extended sRGB beyond [0, 1] has no 8-bit sRGB rendering: clamp.
        ("srgb" | "extended-srgb", 4) => Ok([c[0], c[1], c[2], c[3]].map(|v| v.clamp(0.0, 1.0))),
        ("gray" | "extended-gray", 2) => Ok([c[0], c[0], c[0], c[1]].map(|v| v.clamp(0.0, 1.0))),
        _ => fail(format!("colour space of {spec} is not supported by this actool")),
    }
}

#[cfg(test)]
fn to_u8(v: f32) -> u8 {
    (v * 255.0).round().clamp(0.0, 255.0) as u8
}

/// A layer's or group's placement: a scale about the canvas centre, then a
/// translation in points.
fn transform(v: &serde_json::Value) -> Result<tiny_skia::Transform, String> {
    let Some(p) = v.get("position") else { return Ok(tiny_skia::Transform::identity()) };
    let scale = p.get("scale").and_then(|s| s.as_f64()).unwrap_or(1.0) as f32;
    let t = p.get("translation-in-points").and_then(|t| t.as_array());
    let (tx, ty) = match t {
        Some(t) if t.len() == 2 => (t[0].as_f64().unwrap_or(0.0) as f32, t[1].as_f64().unwrap_or(0.0) as f32),
        None => (0.0, 0.0),
        _ => return fail("a position's translation-in-points should be two numbers"),
    };
    let c = CANVAS as f32 / 2.0;
    Ok(tiny_skia::Transform::from_translate(c + tx, c + ty).pre_scale(scale, scale).pre_translate(-c, -c))
}

fn hidden(v: &serde_json::Value) -> bool {
    v.get("hidden").and_then(|h| h.as_bool()).unwrap_or(false)
}

/// Draws an SVG layer, scaled to the canvas from its own size, centred.
fn draw_svg(path: &Path, into: &mut tiny_skia::Pixmap, placement: tiny_skia::Transform, opacity: f32) -> Result<(), String> {
    let data = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let tree = usvg::Tree::from_data(&data, &usvg::Options::default()).map_err(|e| format!("{}: {e}", path.display()))?;
    let size = tree.size();
    let mut layer = tiny_skia::Pixmap::new(CANVAS, CANVAS).unwrap();
    // Icon Composer places an image at its own size in points, centred.
    let c = CANVAS as f32 / 2.0;
    let fit = tiny_skia::Transform::from_translate(c - size.width() / 2.0, c - size.height() / 2.0);
    resvg::render(&tree, placement.pre_concat(fit), &mut layer.as_mut());
    into.draw_pixmap(0, 0, layer.as_ref(), &tiny_skia::PixmapPaint { opacity, ..Default::default() }, tiny_skia::Transform::identity(), None);
    Ok(())
}

fn draw_png(path: &Path, into: &mut tiny_skia::Pixmap, placement: tiny_skia::Transform, opacity: f32) -> Result<(), String> {
    let img = crate::image::decode_png(&std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?)?;
    let mut premultiplied = Vec::with_capacity(img.data.len());
    for p in img.data.chunks_exact(4) {
        let a = p[3] as u32;
        let m = |c: u8| ((c as u32 * a + 127) / 255) as u8;
        premultiplied.extend_from_slice(&[m(p[0]), m(p[1]), m(p[2]), p[3]]);
    }
    let size = tiny_skia::IntSize::from_wh(img.width as u32, img.height as u32).ok_or("empty image")?;
    let pix = tiny_skia::Pixmap::from_vec(premultiplied, size).ok_or("bad image")?;
    let c = CANVAS as f32 / 2.0;
    let fit = tiny_skia::Transform::from_translate(c - img.width as f32 / 2.0, c - img.height as f32 / 2.0);
    into.draw_pixmap(0, 0, pix.as_ref(), &tiny_skia::PixmapPaint { opacity, quality: tiny_skia::FilterQuality::Bicubic, ..Default::default() }, placement.pre_concat(fit), None);
    Ok(())
}

/// The alpha channel blurred by three passes of a box blur in each
/// direction (close to a Gaussian).
fn blurred_alpha(src: &[u8], w: usize, h: usize, radius: usize) -> Vec<f32> {
    let mut a: Vec<f32> = src.chunks_exact(4).map(|p| p[3] as f32 / 255.0).collect();
    let mut tmp = vec![0f32; a.len()];
    let width = (2 * radius + 1) as f32;
    // (lines, line length, distance between a line's pixels, between lines)
    for (lines, len, step, stride) in [(h, w, 1, w), (w, h, w, 1)].repeat(3) {
        for l in 0..lines {
            let base = l * stride;
            let at = |a: &[f32], i: isize| if i < 0 || i as usize >= len { 0.0 } else { a[base + i as usize * step] };
            let r = radius as isize;
            let mut acc: f32 = (-r..=r).map(|i| at(&a, i)).sum();
            for i in 0..len as isize {
                tmp[base + i as usize * step] = acc / width;
                acc += at(&a, i + r + 1) - at(&a, i - r);
            }
        }
        std::mem::swap(&mut a, &mut tmp);
    }
    a
}

/// Renders the document's default appearance, opaque, at 1024 px.
pub fn render(doc: &Path) -> Result<Rgba, String> {
    let json_path = doc.join("icon.json");
    let bytes = std::fs::read(&json_path).map_err(|e| format!("{}: {e}", json_path.display()))?;
    let json: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", json_path.display()))?;
    let mut canvas = tiny_skia::Pixmap::new(CANVAS, CANVAS).unwrap();

    match json.get("fill") {
        None => canvas.fill(tiny_skia::Color::WHITE),
        Some(f) if f.as_str() == Some("automatic") || f.as_str() == Some("system-light") => canvas.fill(tiny_skia::Color::WHITE),
        Some(f) if f.as_str() == Some("system-dark") => canvas.fill(tiny_skia::Color::BLACK),
        Some(f) => {
            if let Some(s) = f.get("solid").and_then(|s| s.as_str()) {
                let c = color(s)?;
                canvas.fill(tiny_skia::Color::from_rgba(c[0], c[1], c[2], 1.0).unwrap());
            } else if let Some(g) = f.get("automatic-gradient").and_then(|s| s.as_str()) {
                let c = color(g)?;
                canvas.fill(tiny_skia::Color::from_rgba(c[0], c[1], c[2], 1.0).unwrap());
            } else {
                return fail(format!("{}: this fill is not supported by this actool", json_path.display()));
            }
        }
    }

    let groups = json.get("groups").and_then(|g| g.as_array()).cloned().unwrap_or_default();
    // Groups and layers are listed front to back.
    for group in groups.iter().rev().filter(|g| !hidden(g)) {
        let group_placement = transform(group)?;
        let group_opacity = group.get("opacity").and_then(|o| o.as_f64()).unwrap_or(1.0) as f32;
        let mut content = tiny_skia::Pixmap::new(CANVAS, CANVAS).unwrap();
        let layers = group.get("layers").and_then(|l| l.as_array()).cloned().unwrap_or_default();
        for layer in layers.iter().rev().filter(|l| !hidden(l)) {
            let Some(name) = layer.get("image-name").and_then(|n| n.as_str()) else { continue };
            if name.contains('/') || name.starts_with('.') {
                return fail(format!("{}: layer image {name} is outside the document's Assets", json_path.display()));
            }
            let placement = group_placement.pre_concat(transform(layer)?);
            let opacity = layer.get("opacity").and_then(|o| o.as_f64()).unwrap_or(1.0) as f32;
            let path = doc.join("Assets").join(name);
            match Path::new(name).extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).as_deref() {
                Some("svg") => draw_svg(&path, &mut content, placement, opacity)?,
                Some("png") => draw_png(&path, &mut content, placement, opacity)?,
                _ => return fail(format!("{}: layer image {name}: only SVG and PNG layers are supported by this actool", json_path.display())),
            }
        }

        // The group's shadow: its silhouette, blurred and offset downwards.
        if let Some(shadow) = group.get("shadow") {
            let kind = shadow.get("kind").and_then(|k| k.as_str()).unwrap_or("neutral");
            let opacity = shadow.get("opacity").and_then(|o| o.as_f64()).unwrap_or(0.5) as f32;
            if kind != "none" && opacity > 0.0 {
                let (w, h) = (CANVAS as usize, CANVAS as usize);
                let a = blurred_alpha(content.data(), w, h, 12);
                let offset = 10usize;
                let data = canvas.data_mut();
                for y in offset..h {
                    for x in 0..w {
                        let s = a[(y - offset) * w + x] * opacity;
                        let p = &mut data[(y * w + x) * 4..][..3];
                        for c in p {
                            *c = (*c as f32 * (1.0 - s)).round() as u8;
                        }
                    }
                }
            }
        }
        canvas.draw_pixmap(0, 0, content.as_ref(), &tiny_skia::PixmapPaint { opacity: group_opacity, ..Default::default() }, tiny_skia::Transform::identity(), None);
    }

    // The canvas is opaque, so its premultiplied pixels are straight ones.
    let mut data = canvas.take();
    for p in data.chunks_exact_mut(4) {
        p[3] = 255;
    }
    Ok(Rgba { width: CANVAS as usize, height: CANVAS as usize, data })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours() {
        assert_eq!(color("extended-srgb:0.07059,0.08235,0.10588,1.00000").unwrap().map(to_u8), [18, 21, 27, 255]);
        assert_eq!(color("extended-srgb:1.2,-0.1,0.5,1").unwrap().map(to_u8), [255, 0, 128, 255]);
        assert!(color("display-p3:1,0,0,1").is_err());
    }
}
