//! Symbol sets: custom SF Symbols templates, as Xcode compiles them
//! (docs/clean-room/actool.md "Symbol sets"). Xcode stores each of the
//! template's glyphs (a weight at a size) again as a small SVG of its own,
//! with its metrics, and draws the Regular-M one with CoreGraphics into a
//! bitmap per size and scale; this actool
//! writes the SVG and metrics as Xcode does and draws the bitmaps with
//! resvg, which isn't CoreGraphics: compiled only relaxed, as vector images
//! are (`--config=relax_actool_vector`).

use crate::car::{self, Facet, Rendition};
use crate::imageset::SymbolBitmap;
use std::path::PathBuf;

/// The glyph a template's paths make: its SVG as Xcode stores it, its
/// size in points at the medium size and its depth below the baseline.
struct Glyph {
    /// Its weight and size attributes (1 to 9, Ultralight to Black; 1 to
    /// 3, small to large).
    weight: u16,
    size: u16,
    svg: String,
    width: f64,
    height: f64,
    /// The template's lowest point, in its units (positive below the
    /// baseline).
    max_y: f64,
    /// Its size's distance from baseline to capline, in template units.
    cap_height: f64,
}

/// A platform's glyph sizes: (scales, small, medium and large point sizes).
fn sizes(platform: &str) -> Option<(&'static [u32], [u32; 3])> {
    match platform {
        "ios" => Some((&[1, 2, 3], [15, 17, 20])),
        "macosx" => Some((&[1, 2], [11, 13, 17])),
        _ => None,
    }
}

/// The deployment attribute of a symbol's renditions, and its bitmaps'
/// weight (regular) and size (medium), on every platform and deployment
/// target probed (iOS 13 to 18, macOS 14).
const DEPLOYMENT: u16 = 5;
const WEIGHT_REGULAR: u16 = 4;
const SIZE_MEDIUM: u16 = 2;

/// The template's weights, by the glyph weight attribute's value less one.
const WEIGHTS: [&str; 9] = ["Ultralight", "Thin", "Light", "Regular", "Medium", "Semibold", "Bold", "Heavy", "Black"];

/// The template's sizes (the glyph size attribute's value less one) and the
/// distance between each one's Baseline and Capline guides in the SF Symbols
/// app's templates; others aren't established.
const SIZES: [(&str, f64); 3] = [("S", 70.459), ("M", 70.46), ("L", 70.46)];
const PART_GLYPH: u16 = 59;
const LAYOUT_GLYPH: u16 = 1017;

/// Symbol sets, each its facet, its glyph's rendition and its bitmaps.
pub fn compile(sets: &[(String, PathBuf)], platform: &str, relax_vector: bool, identifier: impl Fn(&str) -> u16) -> Result<(Vec<Facet>, Vec<Rendition>, Vec<SymbolBitmap>), String> {
    let mut facets = Vec::new();
    let mut renditions = Vec::new();
    let mut bitmaps = Vec::new();
    for (name, dir) in sets {
        let Some((scales, points)) = sizes(platform) else {
            return Err(format!("{}: symbol sets for this platform are not supported by this actool", dir.display()));
        };
        if !relax_vector {
            return Err(format!("{}: a symbol set is compiled only relaxed: Xcode draws its glyph into bitmaps with CoreGraphics, which this actool doesn't reproduce; --config=relax_actool_vector accepts its own drawing", dir.display()));
        }
        let contents = crate::read_json(&dir.join("Contents.json"))?;
        for key in contents.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()).unwrap_or_default() {
            if !["info", "symbols"].contains(&key.as_str()) {
                return Err(format!("{}: the {key} key of a symbol set is not supported by this actool", dir.display()));
            }
        }
        let listed: Vec<&serde_json::Value> = contents["symbols"].as_array().into_iter().flatten().filter(|e| !e["filename"].is_null()).collect();
        let [entry] = listed.as_slice() else {
            return Err(format!("{}: a symbol set of more than one symbol is not supported by this actool", dir.display()));
        };
        if entry.as_object().is_some_and(|o| o.keys().any(|k| !["filename", "idiom"].contains(&k.as_str()))) || entry["idiom"] != "universal" {
            return Err(format!("{}: a symbol other than a universal one is not supported by this actool", dir.display()));
        }
        let filename = entry["filename"].as_str().expect("checked");
        let path = dir.join(filename);
        let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let text = std::str::from_utf8(&bytes).map_err(|_| format!("{}: not UTF-8", path.display()))?;
        let medium = points[1] as f64;
        let glyphs = glyphs(text, medium / 100.0).map_err(|e| format!("{}: {e}", path.display()))?;
        let id = identifier(name);
        facets.push(Facet { name: name.clone(), attrs: vec![(car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_IMAGE), (car::ATTR_IDENTIFIER, id), (car::ATTR_DEPLOYMENT, DEPLOYMENT)] });
        let glyph_attrs = |part: u16, weight: u16, size: u16| vec![(car::ATTR_GLYPH_WEIGHT, weight), (car::ATTR_GLYPH_SIZE, size), (car::ATTR_DEPLOYMENT, DEPLOYMENT), (car::ATTR_IDENTIFIER, id), (car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, part)];
        // Each glyph is stored as it is, none interpolated; only Regular-M's
        // lists the platform's point sizes, and only it is drawn (probes of
        // three to five glyphs of six weights and all three sizes).
        for glyph in &glyphs {
            let mut attrs = glyph_attrs(PART_GLYPH, glyph.weight, glyph.size);
            attrs.push((car::ATTR_SCALE, 1));
            let regular = glyph.weight == WEIGHT_REGULAR && glyph.size == SIZE_MEDIUM;
            renditions.push(Rendition { attrs, csi: glyph_csi(filename, glyph, medium, regular.then_some(points)) });
        }
        let glyph = glyphs.iter().find(|g| g.weight == WEIGHT_REGULAR && g.size == SIZE_MEDIUM).expect("checked");
        for &scale in scales {
            for (size, &pt) in points.iter().enumerate() {
                let f = pt as f64 / medium;
                // Its view box's side times (point size × scale) / medium,
                // rounded up, in doubles and in that order (45 of 45 sizes
                // of three glyphs, where an exact product rounds either way).
                let pixels = |v: f64| (v * ((pt * scale) as f64 / medium)).ceil() as u32;
                let (w, h) = (pixels(glyph.width), pixels(glyph.height));
                let image = crate::vector::render(crate::vector::Kind::Svg, glyph.svg.as_bytes(), (glyph.width * f, glyph.height * f), scale as f64, (w, h)).map_err(|e| format!("{}: {e}", path.display()))?;
                let mut attrs = glyph_attrs(car::PART_IMAGE, WEIGHT_REGULAR, SIZE_MEDIUM);
                attrs.push((car::ATTR_SCALE, scale as u16));
                if size > 0 {
                    attrs.push((car::ATTR_DIMENSION2, size as u16));
                }
                bitmaps.push(SymbolBitmap { scale, identifier: id, attrs, deployment: DEPLOYMENT, filename: filename.to_string(), image });
            }
        }
    }
    Ok((facets, renditions, bitmaps))
}

/// The glyph rendition (layout 1017, part 59): flags 4, scale 100, `SVG `,
/// the composition, 0x3ee, its metrics (0x3fa) and 0x3fb, then the SVG as
/// a vector image's is stored (LZFSE-compressed when smaller). `points`:
/// the platform's point sizes, which Regular-M's metrics list.
fn glyph_csi(name: &str, glyph: &Glyph, medium: f64, points: Option<[u32; 3]>) -> Vec<u8> {
    let s = medium / 100.0;
    let mut t = Vec::new();
    let tlv = |t: &mut Vec<u8>, kind: u32, data: &[u8]| {
        t.extend_from_slice(&kind.to_le_bytes());
        t.extend_from_slice(&(data.len() as u32).to_le_bytes());
        t.extend_from_slice(data);
    };
    let mut composition = 0u32.to_le_bytes().to_vec();
    composition.extend_from_slice(&1.0f32.to_le_bytes());
    tlv(&mut t, 0x3ec, &composition);
    tlv(&mut t, 0x3ee, &1u32.to_le_bytes());
    // Three sizes, the medium point size; the glyph's depth below the
    // baseline, its size's cap height (its guides'), 0, half a unit and
    // 35/512 of the point size, twice; then the number of point sizes
    // listed and each one's (Regular-M's alone; probes of iOS's and
    // macOS's).
    let mut m = Vec::new();
    for v in [3u32, medium as u32] {
        m.extend_from_slice(&v.to_le_bytes());
    }
    for v in [glyph.max_y * s, glyph.cap_height * s, 0.0, 0.5 * s, medium * 35.0 / 512.0, 0.5 * s, medium * 35.0 / 512.0] {
        m.extend_from_slice(&(v as f32).to_le_bytes());
    }
    let points = points.unwrap_or_default();
    let listed = points.iter().filter(|&&p| p > 0).count() as u32;
    m.extend_from_slice(&listed.to_le_bytes());
    for (k, p) in points.iter().enumerate().take(listed as usize) {
        m.extend_from_slice(&(k as u32).to_le_bytes());
        m.extend_from_slice(&p.to_le_bytes());
    }
    tlv(&mut t, 0x3fa, &m);
    let mut flags = Vec::new();
    for v in [1u32, 0, 0] {
        flags.extend_from_slice(&v.to_le_bytes());
    }
    tlv(&mut t, 0x3fb, &flags);
    let data = glyph.svg.as_bytes();
    let compressed = Some(lzfse::encode(data)).filter(|c| c.len() < data.len());
    let (flag, stored) = match &compressed {
        Some(c) => (1u32, c.as_slice()),
        None => (0, data),
    };
    let mut payload = b"DWAR".to_vec();
    payload.extend_from_slice(&flag.to_le_bytes());
    payload.extend_from_slice(&(stored.len() as u32).to_le_bytes());
    payload.extend_from_slice(stored);
    crate::icon::data_csi(name, LAYOUT_GLYPH, 4, 0, 100, b" GVS", &t, &payload)
}

/// The glyphs of a template, a group of paths for each weight and size it
/// has, Regular-M among them (Xcode requires it), at `scale` (points per
/// template unit, the medium size's for every size).
fn glyphs(text: &str, scale: f64) -> Result<Vec<Glyph>, String> {
    let doc = roxmltree::Document::parse(text).map_err(|e| format!("not an SVG: {e}"))?;
    let group = |id: &str| doc.descendants().find(|n| n.has_tag_name("g") && n.attribute("id") == Some(id));
    let guides = group("Guides").ok_or("an SF Symbols template without guides is not supported by this actool")?;
    let guide = |id: &str| -> Option<f64> { guides.children().find(|n| n.attribute("id") == Some(id)).and_then(|n| n.attribute("y1")).and_then(|v| v.parse().ok()) };
    let mut cap_heights = [0.0; 3];
    for (i, (size, standard)) in SIZES.iter().enumerate() {
        match (guide(&format!("Baseline-{size}")), guide(&format!("Capline-{size}"))) {
            (Some(b), Some(c)) if ((b - c) - standard).abs() < 1e-6 => cap_heights[i] = b - c,
            _ => return Err(format!("an SF Symbols template whose {size} guides aren't the SF Symbols app's is not supported by this actool")),
        }
    }
    let symbols = group("Symbols").ok_or("an SF Symbols template without symbols is not supported by this actool")?;
    let mut glyphs = Vec::new();
    for node in symbols.children().filter(|n| n.is_element()) {
        let id = node.attribute("id").unwrap_or("");
        let parsed = id.split_once('-').and_then(|(w, s)| Some((WEIGHTS.iter().position(|&x| x == w)? as u16 + 1, SIZES.iter().position(|&(x, _)| x == s)? as u16 + 1)));
        let Some((weight, size)) = parsed.filter(|_| node.has_tag_name("g")) else {
            return Err(format!("an SF Symbols template's <{} id=\"{id}\"> other than a glyph of a weight and size is not supported by this actool", node.tag_name().name()));
        };
        if glyphs.iter().any(|g: &Glyph| (g.weight, g.size) == (weight, size)) {
            return Err(format!("an SF Symbols template with two {id} glyphs is not supported by this actool"));
        }
        glyphs.push(glyph(node, weight, size, cap_heights[size as usize - 1], scale)?);
    }
    if !glyphs.iter().any(|g| g.weight == WEIGHT_REGULAR && g.size == SIZE_MEDIUM) {
        return Err("an SF Symbols template without a Regular-M glyph, which Xcode requires, is not supported by this actool".into());
    }
    Ok(glyphs)
}

/// A glyph's group of paths at `scale`: its paths rewritten (each
/// command's letter, then its numbers as %g writes them, a space between
/// numbers but before a minus), its view box their bounds (SVG's, arcs'
/// radii scaled up to reach), the group moved to its corner (probes of
/// paths, margins, offsets and commands).
fn glyph(group: roxmltree::Node, weight: u16, size: u16, cap_height: f64, scale: f64) -> Result<Glyph, String> {
    // Its position on the template's canvas doesn't count (a probe).
    for a in group.attributes() {
        let translation = a.name() == "transform" && a.value().strip_prefix("matrix(1 0 0 1 ").is_some_and(|r| r.ends_with(')'));
        if a.name() != "id" && !translation {
            return Err(format!("the glyph group's {} attribute is not supported by this actool", a.name()));
        }
    }
    let mut paths = Vec::new();
    let mut bounds = Bounds::default();
    for p in group.children().filter(|n| n.is_element()) {
        if !p.has_tag_name("path") || p.attributes().any(|a| a.name() != "d") {
            return Err(format!("a glyph's <{}> other than a path with its data alone is not supported by this actool", p.tag_name().name()));
        }
        let d = p.attribute("d").unwrap_or("");
        paths.push(rewrite(d, &mut bounds)?);
    }
    let (Some((min_x, max_x)), Some((min_y, max_y))) = (bounds.x, bounds.y) else {
        return Err("an empty glyph is not supported by this actool".into());
    };
    let (width, height) = ((max_x - min_x) * scale, (max_y - min_y) * scale);
    let mut svg = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    svg += &format!(
        "<svg version=\"1.1\" viewBox=\"0 0 {} {}\"><g id=\"s\" transform=\"matrix({} 0 0 {} {} {})\">",
        g(width),
        g(height),
        g(scale),
        g(scale),
        g(-min_x * scale),
        g(-min_y * scale)
    );
    for p in &paths {
        svg += &format!("<path d=\"{p}\"/>");
    }
    svg += "</g></svg>\n";
    Ok(Glyph { weight, size, svg, width, height, max_y, cap_height })
}

/// A number as C's `%g` writes it (six significant digits, no trailing
/// zeros), a negative zero as 0.
fn g(v: f64) -> String {
    if v == 0.0 {
        return "0".into();
    }
    let e = format!("{v:.5e}");
    let (mantissa, exp) = e.split_once('e').expect("an exponent");
    let exp: i32 = exp.parse().expect("an integer exponent");
    let trim = |s: String| if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s };
    if !(-4..6).contains(&exp) {
        format!("{}e{}{:02}", trim(mantissa.to_string()), if exp < 0 { '-' } else { '+' }, exp.abs())
    } else {
        trim(format!("{v:.*}", (5 - exp).max(0) as usize))
    }
}

#[derive(Default)]
struct Bounds {
    x: Option<(f64, f64)>,
    y: Option<(f64, f64)>,
}

impl Bounds {
    fn add(&mut self, x: f64, y: f64) {
        let grow = |b: &mut Option<(f64, f64)>, v: f64| *b = Some(b.map_or((v, v), |(lo, hi)| (lo.min(v), hi.max(v))));
        grow(&mut self.x, x);
        grow(&mut self.y, y);
    }
}

/// A path's data rewritten, its points added to `bounds`: a curve's control
/// points with its ends, and an arc's as the cubics it is drawn with (probes
/// of each command with control points beyond the curve, and of arcs
/// starting off an axis, 90 to 200 degrees, circular and elliptical).
fn rewrite(d: &str, bounds: &mut Bounds) -> Result<String, String> {
    use svgtypes::PathSegment as S;
    let mut out = String::new();
    let (mut cx, mut cy, mut sx, mut sy) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    // The last control point, for smooth curves: (cubic, quadratic).
    let mut last_cubic: Option<(f64, f64)> = None;
    let mut last_quad: Option<(f64, f64)> = None;
    for seg in svgtypes::PathParser::from(d) {
        let seg = seg.map_err(|e| format!("its path data: {e}"))?;
        let (letter, abs, numbers): (char, bool, Vec<f64>) = match seg {
            S::MoveTo { abs, x, y } => ('m', abs, vec![x, y]),
            S::LineTo { abs, x, y } => ('l', abs, vec![x, y]),
            S::HorizontalLineTo { abs, x } => ('h', abs, vec![x]),
            S::VerticalLineTo { abs, y } => ('v', abs, vec![y]),
            S::CurveTo { abs, x1, y1, x2, y2, x, y } => ('c', abs, vec![x1, y1, x2, y2, x, y]),
            S::SmoothCurveTo { abs, x2, y2, x, y } => ('s', abs, vec![x2, y2, x, y]),
            S::Quadratic { abs, x1, y1, x, y } => ('q', abs, vec![x1, y1, x, y]),
            S::SmoothQuadratic { abs, x, y } => ('t', abs, vec![x, y]),
            S::EllipticalArc { abs, rx, ry, x_axis_rotation, large_arc, sweep, x, y } => ('a', abs, vec![rx, ry, x_axis_rotation, large_arc as u8 as f64, sweep as u8 as f64, x, y]),
            S::ClosePath { abs } => ('z', abs, Vec::new()),
        };
        out.push(if abs { letter.to_ascii_uppercase() } else { letter });
        for (k, v) in numbers.iter().enumerate() {
            let s = g(*v);
            if k > 0 && !s.starts_with('-') {
                out.push(' ');
            }
            out += &s;
        }
        // Absolute points.
        let (ox, oy) = if abs { (0.0, 0.0) } else { (cx, cy) };
        let p = |i: usize| (numbers[i] + ox, numbers[i + 1] + oy);
        match letter {
            'm' => {
                (cx, cy) = p(0);
                (sx, sy) = (cx, cy);
                bounds.add(cx, cy);
                last_cubic = None;
                last_quad = None;
            }
            'l' => {
                (cx, cy) = p(0);
                bounds.add(cx, cy);
                last_cubic = None;
                last_quad = None;
            }
            'h' => {
                cx = numbers[0] + if abs { 0.0 } else { cx };
                bounds.add(cx, cy);
                last_cubic = None;
                last_quad = None;
            }
            'v' => {
                cy = numbers[0] + if abs { 0.0 } else { cy };
                bounds.add(cx, cy);
                last_cubic = None;
                last_quad = None;
            }
            'c' | 's' => {
                let (c1, c2, end) = if letter == 'c' {
                    (p(0), p(2), p(4))
                } else {
                    (last_cubic.map_or((cx, cy), |(lx, ly)| (2.0 * cx - lx, 2.0 * cy - ly)), p(0), p(2))
                };
                for (x, y) in [(cx, cy), c1, c2, end] {
                    bounds.add(x, y);
                }
                last_cubic = Some(c2);
                last_quad = None;
                (cx, cy) = end;
            }
            'q' | 't' => {
                let (c, end) = if letter == 'q' { (p(0), p(2)) } else { (last_quad.map_or((cx, cy), |(lx, ly)| (2.0 * cx - lx, 2.0 * cy - ly)), p(0)) };
                // Its own control point, not its cubic's.
                for (x, y) in [(cx, cy), c, end] {
                    bounds.add(x, y);
                }
                last_quad = Some(c);
                last_cubic = None;
                (cx, cy) = end;
            }
            'a' => {
                let end = p(5);
                arc_bounds(bounds, (cx, cy), numbers[0], numbers[1], numbers[2], numbers[3] != 0.0, numbers[4] != 0.0, end);
                (cx, cy) = end;
                last_cubic = None;
                last_quad = None;
            }
            _ => {
                (cx, cy) = (sx, sy);
                last_cubic = None;
                last_quad = None;
            }
        }
    }
    Ok(out)
}

/// An elliptical arc's points as the cubics it is drawn with: SVG's
/// endpoint to centre conversion (radii scaled up when too small to reach),
/// then a cubic for each 90 degrees from its start and one for the rest,
/// their control points 4/3 tan(a/4) of a radius along the tangents.
#[allow(clippy::too_many_arguments)]
fn arc_bounds(b: &mut Bounds, p0: (f64, f64), rx: f64, ry: f64, rotation: f64, large: bool, sweep: bool, p1: (f64, f64)) {
    b.add(p0.0, p0.1);
    b.add(p1.0, p1.1);
    let (mut rx, mut ry) = (rx.abs(), ry.abs());
    if rx == 0.0 || ry == 0.0 || p0 == p1 {
        return;
    }
    let phi = rotation.to_radians();
    let (cos, sin) = (phi.cos(), phi.sin());
    let (dx, dy) = ((p0.0 - p1.0) / 2.0, (p0.1 - p1.1) / 2.0);
    let (x1, y1) = (cos * dx + sin * dy, -sin * dx + cos * dy);
    let lambda = x1 * x1 / (rx * rx) + y1 * y1 / (ry * ry);
    if lambda > 1.0 {
        rx *= lambda.sqrt();
        ry *= lambda.sqrt();
    }
    let num = (rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1).max(0.0);
    let den = rx * rx * y1 * y1 + ry * ry * x1 * x1;
    let mut coef = (num / den).sqrt();
    if large == sweep {
        coef = -coef;
    }
    let (cxp, cyp) = (coef * rx * y1 / ry, -coef * ry * x1 / rx);
    let (cx, cy) = (cos * cxp - sin * cyp + (p0.0 + p1.0) / 2.0, sin * cxp + cos * cyp + (p0.1 + p1.1) / 2.0);
    let angle = |ux: f64, uy: f64, vx: f64, vy: f64| (ux * vy - uy * vx).atan2(ux * vx + uy * vy);
    let theta1 = angle(1.0, 0.0, (x1 - cxp) / rx, (y1 - cyp) / ry);
    let mut delta = angle((x1 - cxp) / rx, (y1 - cyp) / ry, (-x1 - cxp) / rx, (-y1 - cyp) / ry);
    if !sweep && delta > 0.0 {
        delta -= std::f64::consts::TAU;
    } else if sweep && delta < 0.0 {
        delta += std::f64::consts::TAU;
    }
    // Sines and cosines exact at a quarter turn, as Xcode's are: where a
    // cubic meets an axis, its control point lies on the bounds, not a
    // rounding error beyond them (probes of 180 and 270 degree arcs).
    let exact = |v: f64| {
        if v.abs() < 1e-12 {
            0.0
        } else if (v.abs() - 1.0).abs() < 1e-12 {
            v.signum()
        } else {
            v
        }
    };
    let sin_cos = |t: f64| (exact(t.sin()), exact(t.cos()));
    let point = |t: f64| {
        let (st, ct) = sin_cos(t);
        (cx + rx * ct * cos - ry * st * sin, cy + rx * ct * sin + ry * st * cos)
    };
    let tangent = |t: f64| {
        let (st, ct) = sin_cos(t);
        (-rx * st * cos - ry * ct * sin, -rx * st * sin + ry * ct * cos)
    };
    let quarter = std::f64::consts::FRAC_PI_2.copysign(delta);
    let mut from = theta1;
    let mut left = delta.abs();
    while left > 0.0 {
        let step = if left > std::f64::consts::FRAC_PI_2 { quarter } else { left.copysign(delta) };
        let to = from + step;
        let k = 4.0 / 3.0 * (step / 4.0).tan();
        let (a, d) = (point(from), point(to));
        let (ta, td) = (tangent(from), tangent(to));
        for (x, y) in [a, (a.0 + k * ta.0, a.1 + k * ta.1), (d.0 - k * td.0, d.1 - k * td.1), d] {
            b.add(x, y);
        }
        left -= step.abs();
        from = to;
    }
}
