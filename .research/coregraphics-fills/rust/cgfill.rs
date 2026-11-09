//! Path filling as CoreGraphics does it for a vector image's bitmap, found
//! black-box (docs/clean-room/actool.md, "Vector images"): an 8-bit coverage
//! mask from a fixed-point scan converter.
//!
//! - Coordinates go through float32, then round half up to 1/4096 of a pixel.
//!   The path is clipped to the canvas first.
//! - Each edge is stepped from its lower end by two DDAs whose slopes are
//!   truncated to 1/4096: x at each integer y (rows), and y at each integer x
//!   (columns). The first crossing is the vertex plus the floor of the
//!   fraction times the slope; the stepper keeps that fraction, so any x on
//!   the edge is the floor of the line through the lower vertex at the
//!   truncated slope. The edge ends there at its top y, not at the vertex,
//!   and gets such a point at every vertex y of the path inside its span.
//! - A column crossing belongs to the row whose two neighbouring points
//!   bracket it in x, wherever its own y drifted.
//! - Each row is swept in bands cut at every vertex y and every crossing of
//!   two edges; in each band the winding rule picks the edges that bound
//!   filled spans, and only those add their area (Green's theorem) to the
//!   pixels. Alpha is floor(256 · area), at most 255.
//! - Cubics are halved at t = 1/2 until 3/4 of the largest second difference
//!   of their control points, per coordinate, is at most 1/20 of a pixel.

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
                flatten([p0, c1, c2, p], 0, &mut current);
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
/// through its row crossings, the points at `breaks` (vertex ys, in units)
/// and its column crossings to where the row stepper ends.
fn pieces(bottom: Fixed, top: Fixed, breaks: &[i64]) -> Vec<(Fixed, Fixed, i64)> {
    let ((bx, by), (tx, ty)) = (bottom, top);
    let rows = stepper(by, bx, ty, tx);
    let cols: std::collections::HashMap<i64, i64> = stepper(bx, by, tx, ty).into_iter().collect();
    let mr = (tx - bx) * UNIT / (ty - by);
    // Any other x on the edge is stepped from the stepper's exact state: the
    // floor of the line through the lower vertex at the truncated slope.
    let xat = |yv: i64| floor_div(bx * UNIT + (yv - by) * mr, UNIT);
    let end = (xat(ty), ty);
    let mut ends: Vec<Fixed> = Vec::with_capacity(rows.len() + 2);
    ends.push((bx, by));
    ends.extend(rows.iter().map(|&(k, x)| (x, k * UNIT)));
    ends.push(end);
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

/// Clips piece a -> b to lo <= y <= hi (either bound open).
fn clip_y(a: (f64, f64), b: (f64, f64), lo: Option<f64>, hi: Option<f64>) -> Option<((f64, f64), (f64, f64))> {
    let ((x0, y0), (x1, y1)) = (a, b);
    if y0 == y1 {
        return None;
    }
    let (ya, yb) = (y0.min(y1), y0.max(y1));
    if lo.is_some_and(|l| yb <= l) || hi.is_some_and(|h| ya >= h) {
        return None;
    }
    let at = |y: f64| (x0 + (x1 - x0) * (y - y0) / (y1 - y0), y);
    let (mut a, mut b) = (a, b);
    if let Some(l) = lo {
        if y0 < l {
            a = at(l);
        }
    }
    if let Some(h) = hi {
        if y1 > h {
            b = at(h);
        }
    }
    Some((a, b))
}

/// The 8-bit coverage of `polys` filled by `rule` on a `w` x `h` canvas, rows
/// bottom first.
/// A lone axis-aligned rectangle takes CoreGraphics' rectangle path: per axis,
/// in 1/8192 of a pixel, the start edge snapped down to its 1/256 step and
/// coverage (w - 1) >> 5; alpha floor(ax * ay / 255).
fn rect_fast(polys: &[Vec<Point>], w: usize, h: usize) -> Option<Vec<Vec<u8>>> {
    if polys.len() != 1 || polys[0].len() != 4 { return None; }
    let p = &polys[0];
    let xs: Vec<f64> = p.iter().map(|q| q.x).collect();
    let ys: Vec<f64> = p.iter().map(|q| q.y).collect();
    // Axis-aligned once slopes are truncated to 1/4096: a side whose minor
    // extent times 4096 is under its major extent counts as straight.
    let fx = |v: f64| ((v as f32) as f64 * 4096.0 + 0.5).floor() as i64;
    let axis = (0..4).all(|i| {
        let (a, b) = (p[i], p[(i + 1) % 4]);
        let (dx, dy) = ((fx(b.x) - fx(a.x)).abs(), (fx(b.y) - fx(a.y)).abs());
        dx == 0 || dy == 0 || dx * 4096 < dy || dy * 4096 < dx
    });
    let (x0, x1) = (xs.iter().cloned().fold(f64::MAX, f64::min), xs.iter().cloned().fold(f64::MIN, f64::max));
    let (y0, y1) = (ys.iter().cloned().fold(f64::MAX, f64::min), ys.iter().cloned().fold(f64::MIN, f64::max));
    if !axis || x0 == x1 || y0 == y1 { return None; }
    let q = |v: f64| ((v as f32) as f64 * 8192.0).floor() as i64;
    let cov = |lo: f64, hi: f64, n: usize| -> Vec<i64> {
        let (l, r) = (q(lo.max(0.0)), q(hi.min(n as f64)));
        let sl = (l + 1).div_euclid(32) * 32;
        (0..n as i64).map(|i| { let (a, b) = (sl.max(i * 8192), r.min((i + 1) * 8192)); if b <= a { 0 } else { ((b - a - 1) >> 5).clamp(0, 255) } }).collect()
    };
    let (cx, cy) = (cov(x0, x1, w), cov(y0, y1, h));
    Some(cy.iter().map(|&ay| cx.iter().map(|&ax| (ax * ay / 255) as u8).collect()).collect())
}

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
    breaks.sort_unstable();
    breaks.dedup();
    // Edges bottom to top, each with its direction in the path.
    struct Edge {
        bottom: Fixed,
        top: Fixed,
        dir: i32,
        pieces: Vec<((f64, f64), (f64, f64), i64)>,
    }
    let u = UNIT as f64;
    let mut edges = Vec::new();
    for p in &polys {
        for i in 0..p.len() {
            let (a, b) = (p[i], p[(i + 1) % p.len()]);
            if a.1 == b.1 {
                continue;
            }
            let (bottom, top, dir) = if b.1 > a.1 { (a, b, 1) } else { (b, a, -1) };
            let pieces = pieces(bottom, top, &breaks)
                .into_iter()
                .map(|(a, b, r)| ((a.0 as f64 / u, a.1 as f64 / u), (b.0 as f64 / u, b.1 as f64 / u), r))
                .collect();
            edges.push(Edge { bottom, top, dir, pieces });
        }
    }
    // Band cuts where two edges cross.
    let to_f = |p: Fixed| (p.0 as f64 / u, p.1 as f64 / u);
    let mut cuts: Vec<f64> = Vec::new();
    for i in 0..edges.len() {
        for j in i + 1..edges.len() {
            let (a, b, c, d) = (to_f(edges[i].bottom), to_f(edges[i].top), to_f(edges[j].bottom), to_f(edges[j].top));
            let den = (b.0 - a.0) * (d.1 - c.1) - (b.1 - a.1) * (d.0 - c.0);
            if den == 0.0 {
                continue;
            }
            let t = ((c.0 - a.0) * (d.1 - c.1) - (c.1 - a.1) * (d.0 - c.0)) / den;
            let s = ((c.0 - a.0) * (b.1 - a.1) - (c.1 - a.1) * (b.0 - a.0)) / den;
            if 1e-12 < t && t < 1.0 - 1e-12 && 1e-12 < s && s < 1.0 - 1e-12 {
                cuts.push(a.1 + t * (b.1 - a.1));
            }
        }
    }
    cuts.sort_by(f64::total_cmp);
    let vbreaks: Vec<f64> = breaks.iter().map(|&b| b as f64 / u).collect();
    let mut acc = vec![vec![0.0f64; w]; h];
    for (y, row) in acc.iter_mut().enumerate() {
        let yf = y as f64;
        let mut vb = vec![yf];
        vb.extend(vbreaks.iter().copied().filter(|&b| yf < b && b < yf + 1.0));
        vb.push(yf + 1.0);
        for bi in 0..vb.len() - 1 {
            let (v0, v1) = (vb[bi], vb[bi + 1]);
            let mut sub = vec![v0];
            sub.extend(cuts.iter().copied().filter(|&c| v0 < c && c < v1 && !vbreaks.contains(&c)));
            sub.push(v1);
            for si in 0..sub.len() - 1 {
                let (s0, s1) = (sub[si], sub[si + 1]);
                let ym = (s0 + s1) / 2.0;
                let lo = if si > 0 { Some(s0) } else { None };
                let hi = if si + 2 < sub.len() { Some(s1) } else { None };
                let mut active: Vec<(f64, usize)> = edges
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| (e.bottom.1 as f64 / u) < ym && ym < (e.top.1 as f64 / u))
                    .map(|(i, e)| {
                        let (a, b) = (to_f(e.bottom), to_f(e.top));
                        (a.0 + (b.0 - a.0) * (ym - a.1) / (b.1 - a.1), i)
                    })
                    .collect();
                active.sort_by(|a, b| a.0.total_cmp(&b.0));
                // In a row a vertex splits into bands, the pixel an opening edge
                // passes through (staying in one column for the band) gets the
                // band's area to its right rounded up to 1/4096 px², unless the
                // span's closing edge is in the same pixel.
                let banded = vb.len() > 2 || sub.len() > 2;
                let mut open_part = vec![0.0f64; w];
                let mut close_part = vec![0.0f64; w];
                let mut piece_part = vec![0.0f64; w];
                let mut piece_exact = vec![0.0f64; w];
                let mut o_rect = vec![0.0f64; w];
                let mut o_tri = vec![0.0f64; w];
                let mut c_rect = vec![0.0f64; w];
                let mut c_tri = vec![0.0f64; w];
                let mut c_other = vec![false; w];
                let mut winding = 0;
                for &(_, i) in &active {
                    let e = &edges[i];
                    let before = winding;
                    winding += e.dir;
                    let role = match rule {
                        Rule::NonZero if before == 0 && winding != 0 => 1.0,
                        Rule::NonZero if before != 0 && winding == 0 => -1.0,
                        Rule::NonZero => 0.0,
                        Rule::EvenOdd if before % 2 == 0 => 1.0,
                        Rule::EvenOdd => -1.0,
                    };
                    if role == 0.0 {
                        continue;
                    }
                    let in_band = |a: (f64, f64), b: (f64, f64), r: i64| {
                        let m = (a.1 + b.1) / 2.0;
                        r == y as i64 && ((v0 <= m && m < v1) || (bi == vb.len() - 2 && m >= v1) || (bi == 0 && m < v0))
                    };
                    let (mut xmin, mut xmax) = (f64::MAX, f64::MIN);
                    for &(a, b, r) in &e.pieces {
                        if in_band(a, b, r) {
                            xmin = xmin.min(a.0.min(b.0));
                            xmax = xmax.max(a.0.max(b.0));
                        }
                    }
                    let one_column = xmin.floor() == (xmax - 1e-12).floor();
                    let band_rule = role > 0.0 && std::env::var("CG_NOBANDRULE").is_err();
                    let mut edge_right: Vec<(usize, f64)> = Vec::new();
                    let (mut top_pt, mut bot_y) = ((0.0f64, f64::MIN), f64::MAX);
                    for &(a, b, r) in &e.pieces {
                        if !in_band(a, b, r) {
                            continue;
                        }
                        let Some((a, b)) = clip_y(a, b, lo, hi) else { continue };
                        let d = b.1 - a.1;
                        let x0 = (a.0.min(b.0).floor().max(0.0)) as usize;
                        let x1 = (a.0.max(b.0).floor() as i64).min(w as i64 - 1);
                        if x1 >= 0 {
                            for x in x0..=(x1 as usize) {
                                let xf = x as f64;
                                let right = d - (integral(a, b, xf + 1.0) - integral(a, b, xf));
                                if band_rule {
                                    edge_right.push((x, right));
                                    let (lo_p, hi_p) = if a.1 <= b.1 { (a, b) } else { (b, a) };
                                    if hi_p.1 > top_pt.1 { top_pt = hi_p; }
                                    if lo_p.1 < bot_y { bot_y = lo_p.1; }
                                    continue;
                                }
                                let mode = std::env::var("CG_PIECE").unwrap_or_else(|_| "rectonly".to_string());
                                if role < 0.0 && banded {
                                    if x0 as i64 == x1 {
                                        let (lo_p, hi_p) = if a.1 <= b.1 { (a, b) } else { (b, a) };
                                        let h = hi_p.1 - lo_p.1;
                                        c_rect[x] += h * (xf + 1.0 - hi_p.0); c_tri[x] += 0.5 * h * (hi_p.0 - lo_p.0);
                                    } else { c_other[x] = true; }
                                }
                                let cmode = std::env::var("CG_CPIECE").unwrap_or_default();
                                if !cmode.is_empty() && role < 0.0 && x0 as i64 == x1 && banded {
                                    let (lo_p, hi_p) = if a.1 <= b.1 { (a, b) } else { (b, a) };
                                    let h = hi_p.1 - lo_p.1;
                                    let rect = h * (xf + 1.0 - hi_p.0);
                                    let tri = 0.5 * h * (hi_p.0 - lo_p.0);
                                    let q = |v: f64| match cmode.as_str() { "ceil" => (v * 4096.0 - 1e-7).ceil() / 4096.0, "floor" => (v * 4096.0 + 1e-7).floor() / 4096.0, "round" => (v * 4096.0).round() / 4096.0, _ => v };
                                    close_part[x] -= q(rect) + tri;
                                    continue;
                                }
                                if !mode.is_empty() && role > 0.0 && x0 as i64 == x1 {
                                    // area right of a piece inside one column: rectangle right of its upper
                                    // end plus the signed triangle; one of them rounded up to 1/4096 px²
                                    let (lo_p, hi_p) = if a.1 <= b.1 { (a, b) } else { (b, a) };
                                    let h = hi_p.1 - lo_p.1;
                                    let rect = h * (xf + 1.0 - hi_p.0);
                                    let tri = 0.5 * h * (hi_p.0 - lo_p.0);
                                    let at_vertex = lo_p.1 == e.bottom.1 as f64 / u || hi_p.1 == e.top.1 as f64 / u;
                                    let c = |v: f64| (v * 4096.0 - 1e-7).ceil() / 4096.0;
                                    let v = if at_vertex && mode != "rectonly" { rect + c(tri) } else if banded || mode == "all" { c(rect) + tri } else { right };
                                    piece_part[x] += v; piece_exact[x] += right;
                                    o_rect[x] += rect; o_tri[x] += tri;
                                    continue;
                                }
                                if banded && role > 0.0 && one_column {
                                    open_part[x] += right;
                                } else if banded && role < 0.0 {
                                    close_part[x] -= right;
                                } else {
                                    row[x] += role * right;
                                }
                            }
                        }
                        for v in row.iter_mut().skip((x1 + 1).max(0) as usize) {
                            *v += role * d;
                        }
                    }
                    if band_rule && !edge_right.is_empty() {
                        let h = top_pt.1 - bot_y;
                        let pt = (top_pt.0 - 1e-12).floor().max(0.0) as usize;
                        let wt = (pt + 1) as f64;
                        let rect = h * (wt - top_pt.0);
                        let mut per: Vec<(usize, f64)> = Vec::new();
                        for &(x, r) in &edge_right { if let Some(p) = per.iter_mut().find(|p| p.0 == x) { p.1 += r; } else { per.push((x, r)); } }
                        for (x, r) in per {
                            let v = if x == pt && banded { (rect * 4096.0 - 1e-7).ceil() / 4096.0 + (r - rect) } else { r };
                            piece_part[x] += v; piece_exact[x] += r;
                        }
                    }
                }
                for x in 0..w {
                    if piece_part[x] != 0.0 || piece_exact[x] != 0.0 {
                        let span = std::env::var("CG_PIECE_SPAN").unwrap_or_else(|_| "exact".to_string());
                        row[x] += if close_part[x] != 0.0 {
                            match span.as_str() {
                                "diff" if !c_other[x] => { let c = |v: f64| (v * 4096.0 - 1e-7).ceil() / 4096.0; c(o_rect[x] - c_rect[x]) + o_tri[x] - c_tri[x] + c_rect[x] + c_tri[x] }
                                "diffr" if !c_other[x] => { let r = |v: f64| (v * 4096.0).round() / 4096.0; r(o_rect[x] - c_rect[x]) + o_tri[x] - c_tri[x] + c_rect[x] + c_tri[x] }
                                "diffall" if !c_other[x] => { let c = |v: f64| (v * 4096.0 - 1e-7).ceil() / 4096.0; c(o_rect[x] + o_tri[x] - c_rect[x] - c_tri[x]) + c_rect[x] + c_tri[x] }
                                "exact" | "diff" | "diffr" | "diffall" => piece_exact[x],
                                _ => piece_part[x],
                            }
                        } else { piece_part[x] };
                    }
                    let (o, c) = (open_part[x], close_part[x]);
                    if o != 0.0 && c == 0.0 {
                        row[x] += (o * 4096.0 - 1e-7).ceil() / 4096.0;
                    } else {
                        row[x] += o + c;
                    }
                }
            }
        }
    }
    acc.into_iter().map(|r| r.into_iter().map(|v| ((v.clamp(0.0, 1.0) * 256.0 + 1e-9).floor() as i64).min(255) as u8).collect()).collect()
}
