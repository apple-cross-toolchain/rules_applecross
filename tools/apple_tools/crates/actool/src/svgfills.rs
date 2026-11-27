//! An SVG of opaque fills as paths in pixels, read in double precision as
//! CoreSVG reads it (crate::vector draws them with crate::cgraster): usvg
//! keeps paths in float32, which CoreGraphics' 1/4096 grid shows at 2x and
//! 3x (a probe of a circle, byte-identical to Xcode's at 1x only).
//!
//! What it reads: the root's size and view box (fitted to it centred, the
//! default `xMidYMid meet`); groups; paths of lines and cubics (arcs and
//! quadratics aren't established); rectangles without rounded corners;
//! circles and ellipses as four cubics (`κ = 0.5522847498`, a probe of
//! every start and direction, all the same); polygons; transforms; and
//! `fill` (inherited, black by default, `none` drawing nothing) and
//! `fill-rule`. Anything else (strokes, styles, classes, opacity, clips,
//! masks, other elements) is None, for the other drawing.

use crate::cgraster::{Element, Point};
use crate::pdf::Fill;

const KAPPA: f64 = 0.5522847498307936;

type Matrix = [f64; 6];

fn multiply(a: &Matrix, b: &Matrix) -> Matrix {
    [
        a[0] * b[0] + a[1] * b[2],
        a[0] * b[1] + a[1] * b[3],
        a[2] * b[0] + a[3] * b[2],
        a[2] * b[1] + a[3] * b[3],
        a[4] * b[0] + a[5] * b[2] + b[4],
        a[4] * b[1] + a[5] * b[3] + b[5],
    ]
}

fn number(n: roxmltree::Node, name: &str) -> Option<f64> {
    n.attribute(name).map_or(Some(0.0), |v| v.trim().parse::<f64>().ok())
}

/// The fills of `data` in pixels, y up from the bottom of a `fit`-scaled
/// canvas of `height` points; None where it holds anything else.
pub fn fills(data: &[u8], fit: (f64, f64), height: f64) -> Option<Vec<Fill>> {
    let text = std::str::from_utf8(data).ok()?;
    let doc = roxmltree::Document::parse_with_options(text, roxmltree::ParsingOptions { allow_dtd: false, ..Default::default() }).ok()?;
    let root = doc.root_element();
    if root.tag_name().name() != "svg" || root.attribute("preserveAspectRatio").is_some() {
        return None;
    }
    let size = crate::icon::svg_size(data).ok()?;
    // The view box fitted to the size, centred.
    let mut base: Matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    if let Some(vb) = root.attribute("viewBox") {
        let n: Vec<f64> = vb.split(|c: char| c == ',' || c.is_whitespace()).filter(|s| !s.is_empty()).map(|s| s.parse::<f64>().ok()).collect::<Option<_>>()?;
        let [x, y, w, h] = n[..] else { return None };
        if w <= 0.0 || h <= 0.0 {
            return None;
        }
        let s = (size.0 / w).min(size.1 / h);
        base = [s, 0.0, 0.0, s, (size.0 - w * s) / 2.0 - x * s, (size.1 - h * s) / 2.0 - y * s];
    }
    // From the SVG's user space to pixels, y up.
    let to_pixels = |m: &Matrix, (x, y): (f64, f64)| {
        let (x, y) = (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5]);
        Point { x: x * fit.0, y: (height - y) * fit.1 }
    };
    let mut out = Vec::new();
    walk(root, &base, Some([0, 0, 0]), false, &to_pixels, &mut out)?;
    Some(out)
}

/// The element `n`'s fills (and its children's), `ctm` from its user space,
/// with the fill and rule it inherits.
fn walk(n: roxmltree::Node, ctm: &Matrix, fill: Option<[u8; 3]>, even_odd: bool, to_pixels: &dyn Fn(&Matrix, (f64, f64)) -> Point, out: &mut Vec<Fill>) -> Option<()> {
    let tag = n.tag_name().name();
    let geometry: &[&str] = match tag {
        "svg" if n.parent_element().is_none() => &["width", "height", "viewBox", "version", "x", "y"],
        "g" => &[],
        "path" => &["d"],
        "rect" => &["x", "y", "width", "height"],
        "circle" => &["cx", "cy", "r"],
        "ellipse" => &["cx", "cy", "rx", "ry"],
        "polygon" => &["points"],
        "title" | "desc" => return Some(()),
        "defs" if n.children().all(|c| !c.is_element()) => return Some(()),
        _ => return None,
    };
    for a in n.attributes() {
        let name = a.name();
        let allowed = geometry.contains(&name) || ["fill", "fill-rule", "transform", "id"].contains(&name) || (name == "stroke" && a.value().trim() == "none");
        // The root's namespace declarations come as attributes in no namespace.
        let namespace = a.namespace().is_some() || name == "xmlns";
        if !allowed && !namespace {
            return None;
        }
    }
    let ctm = match n.attribute("transform") {
        Some(t) => {
            let t: svgtypes::Transform = t.parse().ok()?;
            multiply(&[t.a, t.b, t.c, t.d, t.e, t.f], ctm)
        }
        None => *ctm,
    };
    let fill = match n.attribute("fill").map(str::trim) {
        None => fill,
        Some("none") => None,
        Some(v) => {
            let c: svgtypes::Color = v.parse().ok()?;
            if c.alpha != 255 {
                return None;
            }
            Some([c.red, c.green, c.blue])
        }
    };
    let even_odd = match n.attribute("fill-rule").map(str::trim) {
        None => even_odd,
        Some("nonzero") => false,
        Some("evenodd") => true,
        Some(_) => return None,
    };
    let p = |x: f64, y: f64| to_pixels(&ctm, (x, y));
    let mut path = Vec::new();
    match tag {
        "svg" | "g" => {
            for c in n.children().filter(|c| c.is_element()) {
                walk(c, &ctm, fill, even_odd, to_pixels, out)?;
            }
            return Some(());
        }
        "path" => {
            let (mut cur, mut start, mut last_ctrl) = ((0.0, 0.0), (0.0, 0.0), None::<(f64, f64)>);
            for seg in svgtypes::PathParser::from(n.attribute("d")?) {
                use svgtypes::PathSegment as S;
                let seg = seg.ok()?;
                let rel = |abs: bool, x: f64, y: f64, cur: (f64, f64)| if abs { (x, y) } else { (cur.0 + x, cur.1 + y) };
                match seg {
                    S::MoveTo { abs, x, y } => {
                        cur = rel(abs, x, y, cur);
                        start = cur;
                        path.push(Element::Move(p(cur.0, cur.1)));
                        last_ctrl = None;
                    }
                    S::LineTo { abs, x, y } => {
                        cur = rel(abs, x, y, cur);
                        path.push(Element::Line(p(cur.0, cur.1)));
                        last_ctrl = None;
                    }
                    S::HorizontalLineTo { abs, x } => {
                        cur = (if abs { x } else { cur.0 + x }, cur.1);
                        path.push(Element::Line(p(cur.0, cur.1)));
                        last_ctrl = None;
                    }
                    S::VerticalLineTo { abs, y } => {
                        cur = (cur.0, if abs { y } else { cur.1 + y });
                        path.push(Element::Line(p(cur.0, cur.1)));
                        last_ctrl = None;
                    }
                    S::CurveTo { abs, x1, y1, x2, y2, x, y } => {
                        let (a, b, e) = (rel(abs, x1, y1, cur), rel(abs, x2, y2, cur), rel(abs, x, y, cur));
                        path.push(Element::Cubic(p(a.0, a.1), p(b.0, b.1), p(e.0, e.1)));
                        (cur, last_ctrl) = (e, Some(b));
                    }
                    S::SmoothCurveTo { abs, x2, y2, x, y } => {
                        let a = last_ctrl.map_or(cur, |c| (2.0 * cur.0 - c.0, 2.0 * cur.1 - c.1));
                        let (b, e) = (rel(abs, x2, y2, cur), rel(abs, x, y, cur));
                        path.push(Element::Cubic(p(a.0, a.1), p(b.0, b.1), p(e.0, e.1)));
                        (cur, last_ctrl) = (e, Some(b));
                    }
                    S::ClosePath { .. } => {
                        path.push(Element::Close);
                        cur = start;
                        last_ctrl = None;
                    }
                    _ => return None,
                }
            }
        }
        "rect" => {
            let (x, y, w, h) = (number(n, "x")?, number(n, "y")?, number(n, "width")?, number(n, "height")?);
            if w <= 0.0 || h <= 0.0 {
                return Some(());
            }
            path.extend([Element::Move(p(x, y)), Element::Line(p(x + w, y)), Element::Line(p(x + w, y + h)), Element::Line(p(x, y + h)), Element::Close]);
        }
        "circle" | "ellipse" => {
            let (cx, cy) = (number(n, "cx")?, number(n, "cy")?);
            let (rx, ry) = if tag == "circle" { (number(n, "r")?, number(n, "r")?) } else { (number(n, "rx")?, number(n, "ry")?) };
            if rx <= 0.0 || ry <= 0.0 {
                return Some(());
            }
            let (kx, ky) = (rx * KAPPA, ry * KAPPA);
            path.push(Element::Move(p(cx + rx, cy)));
            path.push(Element::Cubic(p(cx + rx, cy + ky), p(cx + kx, cy + ry), p(cx, cy + ry)));
            path.push(Element::Cubic(p(cx - kx, cy + ry), p(cx - rx, cy + ky), p(cx - rx, cy)));
            path.push(Element::Cubic(p(cx - rx, cy - ky), p(cx - kx, cy - ry), p(cx, cy - ry)));
            path.push(Element::Cubic(p(cx + kx, cy - ry), p(cx + rx, cy - ky), p(cx + rx, cy)));
            path.push(Element::Close);
        }
        "polygon" => {
            let v: Vec<f64> = n.attribute("points")?.split(|c: char| c == ',' || c.is_whitespace()).filter(|s| !s.is_empty()).map(|s| s.parse::<f64>().ok()).collect::<Option<_>>()?;
            if v.len() < 6 || v.len() % 2 != 0 {
                return None;
            }
            for (i, q) in v.chunks(2).enumerate() {
                path.push(if i == 0 { Element::Move(p(q[0], q[1])) } else { Element::Line(p(q[0], q[1])) });
            }
            path.push(Element::Close);
        }
        _ => unreachable!(),
    }
    if let Some(colour) = fill {
        out.push(Fill { path, even_odd, colour });
    }
    Some(())
}
