//! Path filling as CoreGraphics does it for a vector image's bitmap, found
//! black-box from its public drawing API (docs/clean-room/actool.md, "Exact
//! PDFs", the relaxed bitmaps): an 8-bit coverage mask from a fixed-point
//! scan converter.
//!
//! - Coordinates go through float32, then round half up to 1/4096 of a pixel.
//!   The path is clipped to the canvas first.
//! - Each edge is stepped from its lower end by two DDAs whose slopes are
//!   truncated to 1/4096: x at each integer y (rows), and y at each integer x
//!   (columns). The first crossing is the vertex plus the floor of the
//!   fraction times the slope; the stepper keeps that fraction, so any x on
//!   the edge is the floor of the line through the lower vertex at the
//!   truncated slope. The edge ends there at its top y, not at the vertex,
//!   and gets such a point at every vertex y of the path inside its span, and
//!   at every y where two edges that share no vertex cross (rounded up).
//! - A column crossing belongs to the row whose two neighbouring points
//!   bracket it in x, wherever its own y drifted.
//! - Each row is swept in bands cut at those ys; in each band the winding
//!   rule picks the edges that bound filled spans, and only those add their
//!   area (Green's theorem) to the pixels. In a row a vertex cuts into bands,
//!   the pixel where an opening edge tops out in the band gets the area to
//!   its right rounded up to 1/4096 px², unless a closing edge tops out in
//!   the same pixel. Alpha is floor(256 · area), at most 255.
//! - A lone axis-aligned rectangle takes a path of its own (`rect_fast`).
//! - Cubics are halved at t = 1/2 until 3/4 of the largest second difference
//!   of their control points, per coordinate, is at most 1/20 of a pixel.
//!
//! Measured against CoreGraphics' own drawing: exact for random edges,
//! clipped wedges and rectangles; ±1 in some rows holding a vertex of
//! triangles and curves (docs/clean-room/actool.md), which is why its
//! bitmaps are compiled only relaxed.

use std::collections::HashMap;

/// 1/4096 of a pixel.
const UNIT: i64 = 4096;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rule {
    NonZero,
    EvenOdd,
}

/// A path element in pixels, y up.
#[derive(Clone, Copy, Debug)]
pub enum Element {
    Move(Point),
    Line(Point),
    Cubic(Point, Point, Point),
    Close,
}

/// The subpaths of `path` as polygons, cubics flattened.
pub fn polygons(path: &[Element]) -> Vec<Vec<Point>> {
    let mut out = Vec::new();
    let mut current: Vec<Point> = Vec::new();
    let finish = |current: &mut Vec<Point>, out: &mut Vec<Vec<Point>>| {
        if current.len() > 1 && current.first() == current.last() {
            current.pop();
        }
        if current.len() > 1 {
            out.push(std::mem::take(current));
        }
        current.clear();
    };
    for e in path {
        match *e {
            Element::Move(p) => {
                finish(&mut current, &mut out);
                current.push(p);
            }
            Element::Line(p) => current.push(p),
            Element::Cubic(c1, c2, p) => {
                let p0 = *current.last().unwrap_or(&p);
                // The control points as float32; the points between go
                // straight to 1/4096.
                let f = |p: Point| Point { x: (p.x as f32) as f64, y: (p.y as f32) as f64 };
                let start = current.len();
                flatten([f(p0), f(c1), f(c2), f(p)], 0, &mut current);
                let r = |v: f64| (v * UNIT as f64 + 0.5).floor() / UNIT as f64;
                for q in &mut current[start..] {
                    *q = Point { x: r(q.x), y: r(q.y) };
                }
                current.push(p);
            }
            Element::Close => finish(&mut current, &mut out),
        }
    }
    finish(&mut current, &mut out);
    out
}

/// Pushes the points a cubic is halved at, in order, between its ends.
fn flatten(q: [Point; 4], depth: u32, out: &mut Vec<Point>) {
    let d = |a: Point, b: Point, c: Point| (a.x - 2.0 * b.x + c.x).abs().max((a.y - 2.0 * b.y + c.y).abs());
    if d(q[0], q[1], q[2]).max(d(q[1], q[2], q[3])) <= 0.05 / 0.75 || depth > 12 {
        return;
    }
    let mid = |a: Point, b: Point| Point { x: (a.x + b.x) / 2.0, y: (a.y + b.y) / 2.0 };
    let (a, b, c) = (mid(q[0], q[1]), mid(q[1], q[2]), mid(q[2], q[3]));
    let (l, r) = (mid(a, b), mid(b, c));
    let f = mid(l, r);
    flatten([q[0], a, l, f], depth + 1, out);
    out.push(f);
    flatten([f, r, c, q[3]], depth + 1, out);
}

/// A coordinate in 1/4096 of a pixel, as CoreGraphics takes it.
fn fixed(v: f64) -> i64 {
    ((v as f32) as f64 * UNIT as f64 + 0.5).floor() as i64
}

fn floor_div(a: i64, b: i64) -> i64 {
    a.div_euclid(b) - if b < 0 && a.rem_euclid(b) != 0 { 1 } else { 0 }
}

/// Rounded up to 1/4096.
fn ceil_unit(v: f64) -> f64 {
    (v * 4096.0 - 1e-7).ceil() / 4096.0
}

/// Sutherland–Hodgman against the canvas.
fn clip(poly: &[Point], w: f64, h: f64) -> Vec<Point> {
    let mut p = poly.to_vec();
    let planes: [(fn(Point, f64) -> bool, fn(Point, Point, f64) -> Point, f64); 4] = [
        (|p, v| p.x >= v, |a, b, v| Point { x: v, y: a.y + (b.y - a.y) * (v - a.x) / (b.x - a.x) }, 0.0),
        (|p, v| p.x <= v, |a, b, v| Point { x: v, y: a.y + (b.y - a.y) * (v - a.x) / (b.x - a.x) }, w),
        (|p, v| p.y >= v, |a, b, v| Point { x: a.x + (b.x - a.x) * (v - a.y) / (b.y - a.y), y: v }, 0.0),
        (|p, v| p.y <= v, |a, b, v| Point { x: a.x + (b.x - a.x) * (v - a.y) / (b.y - a.y), y: v }, h),
    ];
    for (inside, cross, v) in planes {
        let mut out = Vec::with_capacity(p.len() + 4);
        for i in 0..p.len() {
            let (a, b) = (p[(i + p.len() - 1) % p.len()], p[i]);
            if inside(b, v) {
                if !inside(a, v) {
                    out.push(cross(a, b, v));
                }
                out.push(b);
            } else if inside(a, v) {
                out.push(cross(a, b, v));
            }
        }
        p = out;
        if p.is_empty() {
            break;
        }
    }
    p
}

/// A point of an edge's polyline in units.
type Fixed = (i64, i64);

/// Values of b at each integer a strictly between the ends, stepping from
/// (pa, pb) towards qa: (a in pixels, b in units).
fn stepper(pa: i64, pb: i64, qa: i64, qb: i64) -> Vec<(i64, i64)> {
    if pa == qa {
        return Vec::new();
    }
    let up = qa > pa;
    let m = (qb - pb) * UNIT / (qa - pa).abs();
    let k0 = if up { floor_div(pa, UNIT) + 1 } else { -floor_div(-pa, UNIT) - 1 };
    let frac = (k0 * UNIT - pa).abs();
    let b0 = pb + floor_div(frac * m, UNIT);
    let mut out = Vec::new();
    let (mut k, mut b) = (k0, b0);
    while if up { k * UNIT < qa } else { k * UNIT > qa } {
        out.push((k, b));
        k += if up { 1 } else { -1 };
        b += m;
    }
    out
}

/// An edge's pieces bottom to top, each with its row: from the lower vertex
/// through its row crossings, the points at `breaks` (ys in units) and its
/// column crossings to where the row stepper ends.
fn pieces(bottom: Fixed, top: Fixed, breaks: &[i64]) -> Vec<(Fixed, Fixed, i64)> {
    let ((bx, by), (tx, ty)) = (bottom, top);
    let rows = stepper(by, bx, ty, tx);
    let cols: HashMap<i64, i64> = stepper(bx, by, tx, ty).into_iter().collect();
    let mr = (tx - bx) * UNIT / (ty - by);
    // Any other x on the edge is stepped from the stepper's exact state: the
    // floor of the line through the lower vertex at the truncated slope.
    let xat = |yv: i64| floor_div(bx * UNIT + (yv - by) * mr, UNIT);
    let mut ends: Vec<Fixed> = Vec::with_capacity(rows.len() + 2);
    ends.push((bx, by));
    ends.extend(rows.iter().map(|&(k, x)| (x, k * UNIT)));
    ends.push((xat(ty), ty));
    for &yv in breaks {
        if by < yv && yv < ty {
            ends.push((xat(yv), yv));
        }
    }
    ends.sort_by_key(|p| p.1);
    let mut out = Vec::new();
    for w in ends.windows(2) {
        let (a, b) = (w[0], w[1]);
        let row = floor_div(a.1, UNIT);
        let (lo, hi) = (a.0.min(b.0), a.0.max(b.0));
        let mut ks: Vec<i64> = ((floor_div(lo, UNIT) + 1)..=(-floor_div(-hi, UNIT) - 1)).filter(|k| cols.contains_key(k)).collect();
        if b.0 < a.0 {
            ks.reverse();
        }
        let mut prev = a;
        for k in ks {
            let p = (k * UNIT, cols[&k]);
            out.push((prev, p, row));
            prev = p;
        }
        out.push((prev, b, row));
    }
    out
}

/// The integral of min(x, X) dy along a -> b, in pixels.
fn integral(a: (f64, f64), b: (f64, f64), x: f64) -> f64 {
    let ((x0, y0), (x1, y1)) = (a, b);
    if y0 == y1 {
        return 0.0;
    }
    if x0 <= x && x1 <= x {
        return (x0 + x1) / 2.0 * (y1 - y0);
    }
    if x0 >= x && x1 >= x {
        return x * (y1 - y0);
    }
    let t = (x - x0) / (x1 - x0);
    let ym = y0 + t * (y1 - y0);
    integral(a, (x, ym), x) + integral((x, ym), b, x)
}

/// A lone axis-aligned rectangle's coverage, CoreGraphics' rectangle path:
/// per axis, in 1/8192 of a pixel, the start edge snapped down to its 1/256
/// step and coverage (w - 1) >> 5; alpha floor(ax · ay / 255).
fn rect_fast(polys: &[Vec<Point>], w: usize, h: usize) -> Option<Vec<Vec<u8>>> {
    if polys.len() != 1 || polys[0].len() != 4 {
        return None;
    }
    let p = &polys[0];
    // Axis-aligned once slopes are truncated to 1/4096: a side whose minor
    // extent times 4096 is under its major extent counts as straight.
    let axis = (0..4).all(|i| {
        let (a, b) = (p[i], p[(i + 1) % 4]);
        let (dx, dy) = ((fixed(b.x) - fixed(a.x)).abs(), (fixed(b.y) - fixed(a.y)).abs());
        dx == 0 || dy == 0 || dx * 4096 < dy || dy * 4096 < dx
    });
    let (x0, x1) = p.iter().fold((f64::MAX, f64::MIN), |(lo, hi), q| (lo.min(q.x), hi.max(q.x)));
    let (y0, y1) = p.iter().fold((f64::MAX, f64::MIN), |(lo, hi), q| (lo.min(q.y), hi.max(q.y)));
    if !axis || x0 == x1 || y0 == y1 {
        return None;
    }
    let q = |v: f64| ((v as f32) as f64 * 8192.0).floor() as i64;
    let cov = |lo: f64, hi: f64, n: usize| -> Vec<i64> {
        let (l, r) = (q(lo.max(0.0)), q(hi.min(n as f64)));
        let sl = (l + 1).div_euclid(32) * 32;
        (0..n as i64)
            .map(|i| {
                let (a, b) = (sl.max(i * 8192), r.min((i + 1) * 8192));
                if b <= a { 0 } else { ((b - a - 1) >> 5).clamp(0, 255) }
            })
            .collect()
    };
    let (cx, cy) = (cov(x0, x1, w), cov(y0, y1, h));
    Some(cy.iter().map(|&ay| cx.iter().map(|&ax| (ax * ay / 255) as u8).collect()).collect())
}

/// The ys (in units) where two edges that share no vertex cross: the lines
/// through each edge's lower vertex at its truncated slope, rounded up.
fn crossings(polys: &[Vec<Fixed>]) -> Vec<i64> {
    let mut edges: Vec<(Fixed, Fixed)> = Vec::new();
    for p in polys {
        for i in 0..p.len() {
            let (a, b) = (p[i], p[(i + 1) % p.len()]);
            if a.1 != b.1 {
                edges.push(if a.1 < b.1 { (a, b) } else { (b, a) });
            }
        }
    }
    let mut out = Vec::new();
    for i in 0..edges.len() {
        for j in i + 1..edges.len() {
            let ((p0, p1), (q0, q1)) = (edges[i], edges[j]);
            if p0 == q0 || p0 == q1 || p1 == q0 || p1 == q1 {
                continue;
            }
            let m1 = (p1.0 - p0.0) * UNIT / (p1.1 - p0.1);
            let m2 = (q1.0 - q0.0) * UNIT / (q1.1 - q0.1);
            if m1 == m2 {
                continue;
            }
            let y = ((q0.0 - p0.0) as f64 * UNIT as f64 + p0.1 as f64 * m1 as f64 - q0.1 as f64 * m2 as f64) / (m1 - m2) as f64;
            if y > p0.1.max(q0.1) as f64 && y < p1.1.min(q1.1) as f64 {
                out.push(y.ceil() as i64);
            }
        }
    }
    out
}

/// The 8-bit coverage of `polys` filled by `rule` on a `w` x `h` canvas,
/// rows bottom first.
pub fn coverage(polys: &[Vec<Point>], w: usize, h: usize, rule: Rule) -> Vec<Vec<u8>> {
    if let Some(r) = rect_fast(polys, w, h) {
        return r;
    }
    let polys: Vec<Vec<Fixed>> = polys
        .iter()
        .map(|p| clip(p, w as f64, h as f64))
        .filter(|p| !p.is_empty())
        .map(|p| p.iter().map(|q| (fixed(q.x), fixed(q.y))).collect())
        .collect();
    let mut breaks: Vec<i64> = polys.iter().flatten().map(|p| p.1).collect();
    breaks.extend(crossings(&polys));
    breaks.sort_unstable();
    breaks.dedup();
    // Edges bottom to top, each with its direction in the path.
    struct Edge {
        bottom: (f64, f64),
        top: (f64, f64),
        dir: i32,
        pieces: Vec<((f64, f64), (f64, f64), i64)>,
    }
    let u = UNIT as f64;
    let to_f = |p: Fixed| (p.0 as f64 / u, p.1 as f64 / u);
    let mut edges = Vec::new();
    for p in &polys {
        for i in 0..p.len() {
            let (a, b) = (p[i], p[(i + 1) % p.len()]);
            if a.1 == b.1 {
                continue;
            }
            let (bottom, top, dir) = if b.1 > a.1 { (a, b, 1) } else { (b, a, -1) };
            let pieces = pieces(bottom, top, &breaks).into_iter().map(|(a, b, r)| (to_f(a), to_f(b), r)).collect();
            edges.push(Edge { bottom: to_f(bottom), top: to_f(top), dir, pieces });
        }
    }
    let vbreaks: Vec<f64> = breaks.iter().map(|&b| b as f64 / u).collect();
    let mut acc = vec![vec![0.0f64; w]; h];
    for (y, row) in acc.iter_mut().enumerate() {
        let yf = y as f64;
        let mut bands = vec![yf];
        bands.extend(vbreaks.iter().copied().filter(|&b| yf < b && b < yf + 1.0));
        bands.push(yf + 1.0);
        let banded = bands.len() > 2;
        for bi in 0..bands.len() - 1 {
            let (v0, v1) = (bands[bi], bands[bi + 1]);
            let ym = (v0 + v1) / 2.0;
            let mut active: Vec<(f64, usize)> = edges
                .iter()
                .enumerate()
                .filter(|(_, e)| e.bottom.1 < ym && ym < e.top.1)
                .map(|(i, e)| (e.bottom.0 + (e.top.0 - e.bottom.0) * (ym - e.bottom.1) / (e.top.1 - e.bottom.1), i))
                .collect();
            active.sort_by(|a, b| a.0.total_cmp(&b.0));
            // Per pixel: the opening edges' area to the right, with and
            // without the top-out rounding; the closing edges'; and whether a
            // closing edge tops out there.
            let mut opening = vec![0.0f64; w];
            let mut opening_exact = vec![0.0f64; w];
            let mut closing = vec![0.0f64; w];
            let mut closing_top = vec![false; w];
            let mut winding = 0;
            for &(_, i) in &active {
                let e = &edges[i];
                let before = winding;
                winding += e.dir;
                let opens = match rule {
                    Rule::NonZero if before == 0 && winding != 0 => true,
                    Rule::NonZero if before != 0 && winding == 0 => false,
                    Rule::NonZero => continue,
                    Rule::EvenOdd => before % 2 == 0,
                };
                let in_band = |a: (f64, f64), b: (f64, f64), r: i64| {
                    let m = (a.1 + b.1) / 2.0;
                    r == y as i64 && ((v0 <= m && m < v1) || (bi == bands.len() - 2 && m >= v1) || (bi == 0 && m < v0))
                };
                let mut right_of: Vec<(usize, f64)> = Vec::new();
                let (mut top_pt, mut bottom_y) = ((0.0f64, f64::MIN), f64::MAX);
                for &(a, b, r) in &e.pieces {
                    if !in_band(a, b, r) || a.1 == b.1 {
                        continue;
                    }
                    let d = b.1 - a.1;
                    let x0 = a.0.min(b.0).floor().max(0.0) as usize;
                    let x1 = (a.0.max(b.0).floor() as i64).min(w as i64 - 1);
                    if x1 >= 0 {
                        for x in x0..=(x1 as usize) {
                            let xf = x as f64;
                            let right = d - (integral(a, b, xf + 1.0) - integral(a, b, xf));
                            if opens {
                                right_of.push((x, right));
                            } else if banded {
                                closing[x] -= right;
                            } else {
                                row[x] -= right;
                            }
                        }
                    }
                    if opens {
                        let (lo, hi) = if a.1 <= b.1 { (a, b) } else { (b, a) };
                        if hi.1 > top_pt.1 {
                            top_pt = hi;
                        }
                        bottom_y = bottom_y.min(lo.1);
                    }
                    let sign = if opens { 1.0 } else { -1.0 };
                    for v in row.iter_mut().skip((x1 + 1).max(0) as usize) {
                        *v += sign * d;
                    }
                }
                if !opens {
                    // Its top point, the first of equals.
                    let top = e.pieces.iter().filter(|&&(a, b, r)| in_band(a, b, r)).flat_map(|&(a, b, _)| [a, b]).fold(None, |t: Option<(f64, f64)>, p| if t.is_none_or(|t| p.1 > t.1) { Some(p) } else { t });
                    if let Some((tx, _)) = top {
                        closing_top[((tx - 1e-12).floor().max(0.0) as usize).min(w - 1)] = true;
                    }
                } else if !right_of.is_empty() {
                    let pt = (top_pt.0 - 1e-12).floor().max(0.0) as usize;
                    let rect = (top_pt.1 - bottom_y) * ((pt + 1) as f64 - top_pt.0);
                    let mut per: Vec<(usize, f64)> = Vec::new();
                    for (x, r) in right_of {
                        match per.iter_mut().find(|p| p.0 == x) {
                            Some(p) => p.1 += r,
                            None => per.push((x, r)),
                        }
                    }
                    for (x, r) in per {
                        opening[x] += if x == pt && banded { ceil_unit(rect) + (r - rect) } else { r };
                        opening_exact[x] += r;
                    }
                }
            }
            for x in 0..w {
                if opening[x] != 0.0 || opening_exact[x] != 0.0 {
                    row[x] += if closing[x] != 0.0 && closing_top[x] { opening_exact[x] } else { opening[x] };
                }
                row[x] += closing[x];
            }
        }
    }
    acc.into_iter().map(|r| r.into_iter().map(|v| ((v.clamp(0.0, 1.0) * 256.0 + 1e-9).floor() as i64).min(255) as u8).collect()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    #[test]
    fn pixel_aligned_rectangle_is_solid() {
        let path = [Element::Move(p(1.0, 1.0)), Element::Line(p(3.0, 1.0)), Element::Line(p(3.0, 3.0)), Element::Line(p(1.0, 3.0)), Element::Close];
        let cov = coverage(&polygons(&path), 4, 4, Rule::NonZero);
        assert_eq!(cov, vec![vec![0, 0, 0, 0], vec![0, 255, 255, 0], vec![0, 255, 255, 0], vec![0, 0, 0, 0]]);
    }

    #[test]
    fn half_pixel_triangle_covers_half() {
        // The lower-right half of one pixel.
        let path = [Element::Move(p(0.0, 0.0)), Element::Line(p(1.0, 0.0)), Element::Line(p(1.0, 1.0)), Element::Close];
        let cov = coverage(&polygons(&path), 1, 1, Rule::NonZero);
        assert_eq!(cov, vec![vec![128]]);
    }
}
