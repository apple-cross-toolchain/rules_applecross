//! Path filling as CoreGraphics does it for a vector image's bitmap, found
//! black-box (docs/clean-room/actool.md, "Vector images"): an 8-bit coverage
//! mask from a fixed-point scan converter.
//!
//! - Coordinates go through float32, then round half up to 1/4096 of a pixel.
//!   The path is clipped to the canvas first.
//! - Each edge is stepped from its lower end by two DDAs whose slopes are
//!   truncated to 1/4096: x at each integer y (rows), and y at each integer x
//!   (columns). The first crossing is the vertex plus the floor of the
//!   fraction times the slope. The edge ends where the row stepper reaches its
//!   top y, not at the vertex, and gets a point at every vertex y of the path
//!   inside its span, stepped from the row crossing below.
//! - A column crossing belongs to the row whose two neighbouring points
//!   bracket it in x, wherever its own y drifted.
//! - Each row is swept in bands cut at every vertex y and every crossing of
//!   two edges; in each band the winding rule picks the edges that bound
//!   filled spans, and only those add their area (Green's theorem) to the
//!   pixels. Alpha is floor(256 · area), at most 255.
//! - Cubics are halved at t = 1/2 until 3/4 of the largest second difference
//!   of their control points, per coordinate, is at most 1/20 of a pixel.

pub static RAW: std::sync::Mutex<Vec<f64>> = std::sync::Mutex::new(Vec::new());

/// 1/4096 of a pixel.
const UNIT: i64 = 4096;
fn knob(name: &str) -> String { std::env::var(name).unwrap_or_else(|_| "floor".into()) }
/// floor, or outward/inward by side (side > 0: the interior is to the right of this edge, i.e. it opens spans)
fn rdiv(a: i64, b: i64, mode: &str, side: i32) -> i64 {
    let f = floor_div(a, b);
    let c = -floor_div(-a, b);
    match mode { "trunc" => a / b, "out" => if side > 0 { f } else { c }, "in" => if side > 0 { c } else { f }, "ceil" => c, "round" => floor_div(2 * a + b, 2 * b), _ => f }
}

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
fn tol() -> f64 { std::env::var("CG_TOL").ok().and_then(|v| v.parse().ok()).unwrap_or(0.05 / 0.75) }
fn measure(q: &[Point; 4]) -> f64 {
    let d = |a: Point, b: Point, c: Point| (a.x - 2.0 * b.x + c.x).abs().max((a.y - 2.0 * b.y + c.y).abs());
    match std::env::var("CG_MEAS").as_deref() {
        Ok("euclid") => { let e = |a: Point, b: Point, c: Point| ((a.x - 2.0 * b.x + c.x).powi(2) + (a.y - 2.0 * b.y + c.y).powi(2)).sqrt(); e(q[0], q[1], q[2]).max(e(q[1], q[2], q[3])) }
        Ok("sum") => { let e = |a: Point, b: Point, c: Point| (a.x - 2.0 * b.x + c.x).abs() + (a.y - 2.0 * b.y + c.y).abs(); e(q[0], q[1], q[2]).max(e(q[1], q[2], q[3])) }
        _ => d(q[0], q[1], q[2]).max(d(q[1], q[2], q[3])),
    }
}
fn flatten(q: [Point; 4], depth: u32, out: &mut Vec<Point>) {
    if std::env::var("CG_UNIFORM").is_ok() && depth == 0 {
        // uniform: the smallest power of two whose pieces are all flat
        let mut pieces = vec![q];
        let mut n = 0;
        while pieces.iter().any(|p| measure(p) > tol()) && n < 12 {
            pieces = pieces.iter().flat_map(|p| { let (a, b) = halves(*p); [a, b] }).collect();
            n += 1;
        }
        for p in pieces.iter().skip(1) { out.push(p[0]); }
        return;
    }
    if measure(&q) <= tol() || depth > 12 {
        return;
    }
    let f32m = std::env::var("CG_F32FLAT").is_ok();
    let mid = |a: Point, b: Point| if f32m { Point { x: ((a.x as f32 + b.x as f32) * 0.5) as f64, y: ((a.y as f32 + b.y as f32) * 0.5) as f64 } } else { Point { x: (a.x + b.x) / 2.0, y: (a.y + b.y) / 2.0 } };
    let q = if f32m { q.map(|p| Point { x: p.x as f32 as f64, y: p.y as f32 as f64 }) } else { q };
    let (a, b, c) = (mid(q[0], q[1]), mid(q[1], q[2]), mid(q[2], q[3]));
    let (l, r) = (mid(a, b), mid(b, c));
    let f = mid(l, r);
    flatten([q[0], a, l, f], depth + 1, out);
    out.push(f);
    flatten([f, r, c, q[3]], depth + 1, out);
}

fn halves(q: [Point; 4]) -> ([Point; 4], [Point; 4]) {
    let mid = |a: Point, b: Point| Point { x: (a.x + b.x) / 2.0, y: (a.y + b.y) / 2.0 };
    let (a, b, c) = (mid(q[0], q[1]), mid(q[1], q[2]), mid(q[2], q[3]));
    let (l, r) = (mid(a, b), mid(b, c));
    let f = mid(l, r);
    ([q[0], a, l, f], [f, r, c, q[3]])
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
fn sdiv(a: i64, b: i64, mode: &str) -> i64 { match mode { "floor" => floor_div(a, b), "ceil" => -floor_div(-a, b), _ => a / b } }
fn slope_mode(row: bool, side: i32) -> String {
    let k = match (row, side > 0) { (true, true) => "CG_SLOPE_O", (true, false) => "CG_SLOPE_C", (false, true) => "CG_CSLOPE_O", (false, false) => "CG_CSLOPE_C" };
    std::env::var(k).unwrap_or_else(|_| "trunc".into())
}
fn stepper(pa: i64, pb: i64, qa: i64, qb: i64, mode: &str, side: i32) -> Vec<(i64, i64)> { stepper2(pa, pb, qa, qb, mode, side, true) }
fn stepper2(pa: i64, pb: i64, qa: i64, qb: i64, mode: &str, side: i32, row: bool) -> Vec<(i64, i64)> {
    if pa == qa {
        return Vec::new();
    }
    let up = qa > pa;
    let m = sdiv((qb - pb) * UNIT, (qa - pa).abs(), &slope_mode(row, side));
    let k0 = if up { floor_div(pa, UNIT) + 1 } else { -floor_div(-pa, UNIT) - 1 };
    let frac = (k0 * UNIT - pa).abs();
    let b0 = pb + rdiv(frac * m, UNIT, mode, side);
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
fn pieces(bottom: Fixed, top: Fixed, breaks: &[i64], side: i32, start_x: Option<i64>, end_out: &mut i64, exact_end: bool, join_x: Option<i64>) -> Vec<(Fixed, Fixed, i64)> {
    let ((mut vbx, by), (mut tx, ty)) = (bottom, top);
    if std::env::var("CG_ZSLOPE").is_ok() && tx != vbx && ((tx - vbx) * UNIT / (ty - by)) == 0 {
        // a slope that truncates to zero: CG keeps the edge vertical at its outer end
        let xv = if side > 0 { vbx.min(tx) } else { vbx.max(tx) };
        vbx = xv; tx = xv;
    }
    let xch = std::env::var("CG_XCHAIN").is_ok() && start_x.is_some();
    let bx = if xch { floor_div(start_x.unwrap(), UNIT) } else { start_x.unwrap_or(vbx) };
    let sx = if std::env::var("CG_CHAINSLOPE").is_ok() || xch { vbx } else { bx };
    let exchain = std::env::var("CG_XCHAIN").is_ok() && start_x.is_some();
    let bnum: i64 = if exchain { start_x.unwrap() } else { 0 };
    let mr_true = (tx - vbx) * UNIT / (ty - by);
    let rows = if exchain {
        let k0 = floor_div(by, UNIT) + 1;
        let mut v = Vec::new();
        let mut k = k0;
        while k * UNIT < ty { v.push((k, floor_div(bnum + (k * UNIT - by) * mr_true, UNIT))); k += 1; }
        v
    } else { stepper(by, bx, ty, tx + (bx - sx), &knob("CG_START"), side) };
    let cols: std::collections::HashMap<i64, i64> = stepper2(bx, by, tx + (bx - sx), ty, &knob("CG_CSTART"), side, false).into_iter().collect();
    let mr = if exchain { mr_true } else { sdiv((tx - sx) * UNIT, ty - by, &slope_mode(true, side)) };
    let exact_acc = std::env::var("CG_ACC").is_ok();
    // x at yv stepped from the stepper's exact state (its first step's fraction kept)
    let xat = |yv: i64| -> i64 {
        if exchain { return floor_div(bnum + (yv - by) * mr, UNIT); }
        match rows.iter().rposition(|&(k, _)| k * UNIT <= yv) {
            Some(i) => { let k = rows[i].0; let frac0 = rows[0].0 * UNIT - by; floor_div(bx * UNIT + frac0 * mr + i as i64 * mr * UNIT + (yv - k * UNIT) * mr, UNIT) }
            None => floor_div(bx * UNIT + (yv - by) * mr, UNIT),
        }
    };
    let (ky, kx) = rows.last().map(|&(k, x)| (k * UNIT, x)).unwrap_or((by, bx));
    let ysnap = |v: i64, m: &str| -> i64 { let g: i64 = std::env::var("CG_YS_G").ok().and_then(|v| v.parse().ok()).unwrap_or(16); match m { "floor" => floor_div(v, g) * g, "ceil" => -floor_div(-v, g) * g, "floorm" => floor_div(v - 1, g) * g, _ => v } };
    let tmode = std::env::var("CG_YS_TOP").unwrap_or_default();
    let ty_e = ysnap(ty, &tmode);
    let end = if exact_end { (tx, ty) } else if exact_acc { (xat(ty_e), ty_e) } else { (kx + rdiv((ty - ky) * mr, UNIT, &knob("CG_END"), side), ty) };
    *end_out = if std::env::var("CG_XCHAIN").is_ok() { if exchain { bnum + (ty - by) * mr } else { bx * UNIT + (ty - by) * mr } } else { end.0 };
    let mut ends: Vec<Fixed> = Vec::with_capacity(rows.len() + 2);
    let bmode = std::env::var("CG_YS_BOT").unwrap_or_default();
    let by_s = ysnap(by, &bmode);
    ends.push(if let Some(j) = join_x { (j, by) } else if by_s != by { (floor_div(bx * UNIT + (by_s - by) * mr, UNIT), by_s) } else { (bx, by) });
    ends.extend(rows.iter().map(|&(k, x)| (x, k * UNIT)));
    ends.push(end);
    let bandside = std::env::var("CG_BANDSIDE").ok();
    let skip_bands = match bandside.as_deref() { Some("open") => side < 0, Some("close") => side > 0, Some("none") => true, _ => false };
    for &yv in breaks {
        if by < yv && yv < ty && !skip_bands {
            let (ky, kx) = rows.iter().rev().find(|&&(k, _)| k * UNIT <= yv).map(|&(k, x)| (k * UNIT, x)).unwrap_or((by, bx));
            let stepped = if exact_acc { xat(yv) } else if std::env::var("CG_BANDFROM").as_deref() == Ok("line") {
                vbx + rdiv((tx - vbx) * (yv - by), ty - by, &knob("CG_BAND"), side)
            } else if std::env::var("CG_BANDFROM").as_deref() == Ok("above") {
                // from the row crossing above (or the end)
                let (ay, ax) = rows.iter().find(|&&(k, _)| k * UNIT >= yv).map(|&(k, x)| (k * UNIT, x)).unwrap_or((ty, end.0));
                ax + rdiv((yv - ay) * mr, UNIT, &knob("CG_BAND"), side)
            } else if std::env::var("CG_BANDFROM").as_deref() == Ok("nearest") {
                let below = (ky, kx);
                let above = rows.iter().find(|&&(k, _)| k * UNIT >= yv).map(|&(k, x)| (k * UNIT, x)).unwrap_or((ty, end.0));
                let (ry, rx) = if yv - below.0 <= above.0 - yv { below } else { above };
                rx + rdiv((yv - ry) * mr, UNIT, &knob("CG_BAND"), side)
            } else { kx + rdiv((yv - ky) * mr, UNIT, &knob("CG_BAND"), side) };
            let xv = if std::env::var("CG_BANDX").as_deref() == Ok("poly") {
                // on the polyline through the row crossings and column crossings
                let mut pts: Vec<(i64, i64)> = vec![(bx, by)];
                pts.extend(rows.iter().map(|&(k, x)| (x, k * UNIT)));
                pts.extend(cols.iter().map(|(&k, &y)| (k * UNIT, y)));
                pts.push(end);
                pts.sort_by_key(|p| p.1);
                let i = pts.iter().position(|p| p.1 >= yv).unwrap_or(pts.len() - 1).max(1);
                let (a, b) = (pts[i - 1], pts[i]);
                if b.1 == a.1 { a.0 } else { a.0 + rdiv((b.0 - a.0) * (yv - a.1), b.1 - a.1, &knob("CG_BAND"), side) }
            } else { stepped };
            let off: i64 = if side < 0 { std::env::var("CG_BANDOFF").ok().and_then(|v| v.parse().ok()).unwrap_or(0) } else { std::env::var("CG_BANDOFF_O").ok().and_then(|v| v.parse().ok()).unwrap_or(0) };
            ends.push((xv + off, yv));
        }
    }
    ends.sort_by_key(|p| p.1);
    let rs = std::env::var("CG_RESTART").ok();
    if rs.as_deref() == Some("all") || (rs.as_deref() == Some("open") && side > 0) || (rs.as_deref() == Some("close") && side < 0) {
        let (mut anum, mut ay) = (ends[0].0 * UNIT, ends[0].1);
        for i in 1..ends.len() {
            let yv = ends[i].1;
            let x = floor_div(anum + (yv - ay) * mr, UNIT);
            ends[i].0 = x;
            let is_band = breaks.binary_search(&yv).is_ok() && yv != ty;
            if is_band { anum = x * UNIT; ay = yv; }
        }
    }
    let mut out = Vec::new();
    for w in ends.windows(2) {
        let (a, b) = (w[0], w[1]);
        let row = floor_div(a.1, UNIT);
        let (lo, hi) = (a.0.min(b.0), a.0.max(b.0));
        let mb = breaks.iter().any(|&b| b > row * UNIT && b < (row + 1) * UNIT);
        let nocol = match std::env::var("CG_NOCOL").as_deref() { Ok("mb") => mb, Ok("all") => true, Ok("mbopen") => mb && side > 0, _ => false };
        let mut ks: Vec<i64> = if nocol { Vec::new() } else { ((floor_div(lo, UNIT) + 1)..=(-floor_div(-hi, UNIT) - 1)).filter(|k| cols.contains_key(k)).collect() };
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
    if std::env::var("CG_NORECT").is_ok() || polys.len() != 1 || polys[0].len() != 4 { return None; }
    let p = &polys[0];
    let xs: Vec<f64> = p.iter().map(|q| q.x).collect();
    let ys: Vec<f64> = p.iter().map(|q| q.y).collect();
    let fx = |v: f64| ((v as f32) as f64 * 4096.0 + 0.5).floor() as i64;
    let axis = if std::env::var("CG_RECTQ").is_ok() {
        (0..4).all(|i| { let (a, b) = (p[i], p[(i + 1) % 4]); let (dx, dy) = ((fx(b.x) - fx(a.x)).abs(), (fx(b.y) - fx(a.y)).abs()); dx * 4096 < dy || dy * 4096 < dx || dx == 0 || dy == 0 })
    } else { (0..4).all(|i| { let (a, b) = (p[i], p[(i + 1) % 4]); a.x == b.x || a.y == b.y }) };
    let (x0, x1) = (xs.iter().cloned().fold(f64::MAX, f64::min), xs.iter().cloned().fold(f64::MIN, f64::max));
    let (y0, y1) = (ys.iter().cloned().fold(f64::MAX, f64::min), ys.iter().cloned().fold(f64::MIN, f64::max));
    if !axis || x0 == x1 || y0 == y1 { return None; }
    let qm = |v: f64, m: &str| { let t = (v as f32) as f64 * 8192.0; match m { "floor" => t.floor() as i64, "ceil" => t.ceil() as i64, _ => (t + 0.5).floor() as i64 } };
    let (ms, me) = (std::env::var("CG_RS").unwrap_or("floor".into()), std::env::var("CG_RE").unwrap_or("floor".into()));
    let cov = |lo: f64, hi: f64, n: usize| -> Vec<i64> {
        let (l, r) = (qm(lo.max(0.0), &ms), qm(hi.min(n as f64), &me));
        let sl = (l + 1).div_euclid(32) * 32;
        (0..n as i64).map(|i| { let (a, b) = (sl.max(i * 8192), r.min((i + 1) * 8192)); if b <= a { 0 } else { ((b - a - 1) >> 5).clamp(0, 255) } }).collect()
    };
    let (cx, cy) = (cov(x0, x1, w), cov(y0, y1, h));
    Some(cy.iter().map(|&ay| cx.iter().map(|&ax| (ax * ay / 255) as u8).collect()).collect())
}

pub fn coverage(polys: &[Vec<Point>], w: usize, h: usize, rule: Rule) -> Vec<Vec<u8>> {
    if let Some(r) = rect_fast(polys, w, h) { return r; }
    let clipped: Vec<Vec<Point>> = polys.iter().map(|p| clip(p, w as f64, h as f64)).filter(|p| !p.is_empty()).collect();
    let raws: Vec<Vec<f64>> = clipped.iter().map(|p| p.iter().map(|q| (q.x as f32) as f64 * UNIT as f64).collect()).collect();
    let polys: Vec<Vec<Fixed>> = clipped.iter().map(|p| p.iter().map(|q| (fixed(q.x), fixed(q.y))).collect()).collect();
    let mut breaks: Vec<i64> = polys.iter().flatten().map(|p| p.1).collect();
    if let Ok(m) = std::env::var("CG_BANDY") {
        let g: i64 = std::env::var("CG_BANDY_G").ok().and_then(|v| v.parse().ok()).unwrap_or(16);
        for b in breaks.iter_mut() { *b = match m.as_str() { "floor" => floor_div(*b, g) * g, "ceil" => -floor_div(-*b, g) * g, _ => floor_div(*b + g / 2, g) * g }; }
    }
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
    let mut specs: Vec<(Fixed, Fixed, i32, i32)> = Vec::new();
    for (pi, p) in polys.iter().enumerate() {
        let area2: i128 = (0..p.len()).map(|i| { let (a, b) = (p[(i + p.len() - 1) % p.len()], p[i]); a.0 as i128 * b.1 as i128 - b.0 as i128 * a.1 as i128 }).sum();
        let orient: i32 = if area2 > 0 { 1 } else { -1 };
        for i in 0..p.len() {
            let (a, b) = (p[i], p[(i + 1) % p.len()]);
            if a.1 == b.1 {
                continue;
            }
            let (mut a, mut b) = (a, b);
            let side0 = -(if b.1 > a.1 { 1 } else { -1 }) * orient;
            if let Ok(mode) = std::env::var("CG_VXO") {
                if side0 > 0 {
                    let rq = |r: f64| match mode.as_str() { "floor" => r.floor() as i64, "ceil" => r.ceil() as i64, "m1" => (r + 0.5).floor() as i64 - 1, "p1" => (r + 0.5).floor() as i64 + 1, _ => (r + 0.5).floor() as i64 };
                    a.0 = rq(raws[pi][i]); b.0 = rq(raws[pi][(i + 1) % p.len()]);
                }
            }
            let (bottom, top, dir) = if b.1 > a.1 { (a, b, 1) } else { (b, a, -1) };
            let side = -dir * orient;
            specs.push((bottom, top, dir, side));
        }
    }
    let chain = std::env::var("CG_CHAIN").ok();
    let open_bottoms: std::collections::HashSet<Fixed> = specs.iter().filter(|s| s.3 > 0).map(|s| s.0).collect();
    let open_tops: std::collections::HashSet<Fixed> = specs.iter().filter(|s| s.3 > 0).map(|s| s.1).collect();
    let chainset: std::collections::HashSet<(i64, i64)> = open_bottoms.intersection(&open_tops).copied().collect();
    let close_bottoms: std::collections::HashSet<Fixed> = specs.iter().filter(|s| s.3 < 0).map(|s| s.0).collect();
    let close_tops: std::collections::HashSet<Fixed> = specs.iter().filter(|s| s.3 < 0).map(|s| s.1).collect();
    let cchainset: std::collections::HashSet<(i64, i64)> = close_bottoms.intersection(&close_tops).copied().collect();
    let mut order: Vec<usize> = (0..specs.len()).collect();
    order.sort_by_key(|&i| specs[i].0 .1);
    let mut ends_at: std::collections::HashMap<Fixed, i64> = std::collections::HashMap::new();
    let mut prev_end: std::collections::HashMap<Fixed, i64> = std::collections::HashMap::new();
    let mut built: Vec<Option<Edge>> = (0..specs.len()).map(|_| None).collect();
    for &i in &order {
        let (bottom, top, dir, side) = specs[i];
        let chained = match chain.as_deref() { Some("open") => side > 0, Some("close") => side < 0, Some("all") => true, _ => false };
        let mut start = if chained { ends_at.get(&bottom).copied() } else { None };
        if side > 0 && chainset.contains(&bottom) {
            if let Some(o) = std::env::var("CG_STARTOFF").ok().and_then(|v| v.parse::<i64>().ok()) { start = Some(bottom.0 + o); }
        }
        let mut endx = 0;
        let ec = std::env::var("CG_ENDCHAIN").ok();
        let exact_end = match ec.as_deref() { Some("open") => side > 0 && chainset.contains(&top), Some("close") => side < 0 && cchainset.contains(&top), Some("all") => (side > 0 && chainset.contains(&top)) || (side < 0 && cchainset.contains(&top)), _ => false };
        let endoff = if side > 0 && chainset.contains(&top) { std::env::var("CG_ENDOFF").ok().and_then(|v| v.parse::<i64>().ok()).unwrap_or(0) } else { 0 };
        let jm = std::env::var("CG_JOIN").ok();
        let joinable = match jm.as_deref() { Some("open") => side > 0 && chainset.contains(&bottom), Some("close") => side < 0 && cchainset.contains(&bottom), Some("all") => (side > 0 && chainset.contains(&bottom)) || (side < 0 && cchainset.contains(&bottom)), _ => false };
        let join_x = if joinable { prev_end.get(&bottom).copied() } else { None };
        let pcs = pieces(bottom, top, &breaks, side, start, &mut endx, exact_end, join_x)
            .into_iter()
            .map(|(a, b, r)| { let b = if b.1 == top.1 { (b.0 + endoff, b.1) } else { b }; let a = if a.1 == top.1 { (a.0 + endoff, a.1) } else { a }; ((a.0 as f64 / u, a.1 as f64 / u), (b.0 as f64 / u, b.1 as f64 / u), r) })
            .collect::<Vec<_>>();
        let pcs: Vec<((f64, f64), (f64, f64), i64)> = if std::env::var("CG_VIN").is_ok() && !pcs.is_empty() && pcs.iter().all(|p| p.0 .0 == pcs[0].0 .0 && p.1 .0 == pcs[0].0 .0) {
            let e = if side > 0 { 1e-7 } else { -1e-7 };
            pcs.into_iter().map(|(a, b, r)| ((a.0 + e, a.1), (b.0 + e, b.1), r)).collect()
        } else { pcs };
        if chained { ends_at.insert(top, endx); }
        let last_end_x = endx;
        prev_end.insert(top, last_end_x);
        built[i] = Some(Edge { bottom, top, dir, pieces: pcs });
    }
    edges.extend(built.into_iter().flatten());
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
                let mut winding = 0;
                let bandq = std::env::var("CG_BQ").ok();
                let multiband = vb.len() > 2 || sub.len() > 2;
                let snap: Vec<f64> = if bandq.is_some() && (multiband || std::env::var("CG_BQ_ALL").is_ok()) { row.clone() } else { Vec::new() };
                let oq = std::env::var("CG_OQ").ok().filter(|_| multiband || std::env::var("CG_OQ_ALL").is_ok());
                let mut obuf: Vec<f64> = vec![0.0; if oq.is_some() { w } else { 0 }];
                let shmode = std::env::var("CG_OQ_SHALLOW").ok();
                let mut sbuf: Vec<f64> = vec![0.0; if oq.is_some() && shmode.is_some() { w } else { 0 }];
                let span_mode = std::env::var("CG_OQ_SPAN").ok();
                let oqc = std::env::var("CG_OQC").ok();
                let mut cbuf: Vec<f64> = vec![0.0; if oq.is_some() && (span_mode.is_some() || oqc.is_some()) { w } else { 0 }];
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
                    let per_edge = std::env::var("CG_PQ_EDGE").is_ok();
                    let mut parts: Vec<f64> = vec![0.0; if per_edge { w } else { 0 }];
                    let mut touched: Vec<usize> = Vec::new();
                    let (mut bxmin, mut bxmax) = (f64::MAX, f64::MIN);
                    for &(a_, b_, r) in &e.pieces {
                        if r != y as i64 { continue; }
                        let m = (a_.1 + b_.1) / 2.0;
                        if !((v0 <= m && m < v1) || (bi == vb.len() - 2 && m >= v1) || (bi == 0 && m < v0)) { continue; }
                        bxmin = bxmin.min(a_.0.min(b_.0)); bxmax = bxmax.max(a_.0.max(b_.0));
                    }
                    let band_single = bxmin.floor() == (bxmax - 1e-12).floor();
                    for &(a, b, r) in &e.pieces {
                        if r != y as i64 {
                            continue;
                        }
                        let m = (a.1 + b.1) / 2.0;
                        let first = bi == 0;
                        let last = bi == vb.len() - 2;
                        if !((v0 <= m && m < v1) || (last && m >= v1) || (first && m < v0)) {
                            continue;
                        }
                        let Some((a, b)) = clip_y(a, b, lo, hi) else { continue };
                        let (a, b) = match std::env::var("CG_XQO").ok().and_then(|v| v.parse::<f64>().ok()) {
                            Some(g) if role > 0.0 && (vb.len() > 2 || std::env::var("CG_XQO_ALL").is_ok()) => { let f = |v: f64| (v * 4096.0 / g).floor() * g / 4096.0; ((f(a.0), a.1), (f(b.0), b.1)) }
                            _ => (a, b),
                        };
                        let d = b.1 - a.1;
                        let x0 = (a.0.min(b.0).floor().max(0.0)) as usize;
                        let x1 = (a.0.max(b.0).floor() as i64).min(w as i64 - 1);
                        if x1 >= 0 {
                            for x in x0..=(x1 as usize) {
                                let xf = x as f64;
                                let part = integral(a, b, xf + 1.0) - integral(a, b, xf);
                                if per_edge { if !touched.contains(&x) { touched.push(x); } parts[x] += part; row[x] += role * d; continue; }
                                let vpiece = a.1 == e.bottom.1 as f64 / u || b.1 == e.top.1 as f64 / u;
                                let vmode = std::env::var("CG_PQ_V").ok();
                                let applies = match vmode.as_deref() { Some("bottom") => a.1 == e.bottom.1 as f64 / u, Some("top") => b.1 == e.top.1 as f64 / u, Some("short") => floor_div(e.bottom.1, UNIT) == floor_div(e.top.1 - 1, UNIT), Some("shortchain") => floor_div(e.bottom.1, UNIT) == floor_div(e.top.1 - 1, UNIT) && (chainset.contains(&e.bottom) || chainset.contains(&e.top)), Some("chain") => (a.1 == e.bottom.1 as f64 / u && chainset.contains(&e.bottom)) || (b.1 == e.top.1 as f64 / u && chainset.contains(&e.top)), Some("chainrow") => chainset.iter().any(|v| (v.1 as f64 / u).floor() as i64 == y as i64), Some("multiband") => vb.len() > 2, Some("multiband_or_v") => vb.len() > 2 || vpiece, Some(_) => vpiece, None => true };
                                let part = match std::env::var("CG_PQ").ok().filter(|_| applies && (role > 0.0 || std::env::var("CG_PQ_BOTH").is_ok())) { Some(v) => { let (u, m) = v.split_once(':').unwrap(); let u: f64 = u.parse().unwrap(); match m { "ceil" => (part * u).ceil() / u, "floor" => (part * u).floor() / u, "round" => (part * u).round() / u, "trunc" => (part * u).trunc() / u, _ => part } } None => part };
                                let part = if role < 0.0 && applies { match std::env::var("CG_PQC").ok() { Some(v) => { let (uu, m) = v.split_once(':').unwrap(); let uu: f64 = uu.parse().unwrap(); match m { "ceil" => (part * uu).ceil() / uu, "floor" => (part * uu).floor() / uu, "round" => (part * uu).round() / uu, _ => part } } None => part } } else { part };
                                let padd: f64 = if applies && role > 0.0 { std::env::var("CG_PADD").ok().and_then(|v| v.parse().ok()).unwrap_or(0.0) / 4096.0 } else { 0.0 };
                                let single = band_single;
                                let oq_ok = oq.is_some() && role > 0.0 && (std::env::var("CG_OQ_STEEP").is_err() || single);
                                if oq_ok { obuf[x] += d - part; } else if role > 0.0 && !sbuf.is_empty() && !single && (multiband || std::env::var("CG_OQ_ALL").is_ok()) { sbuf[x] += d - part; } else if role < 0.0 && !cbuf.is_empty() { cbuf[x] += role * (d - part); } else { row[x] += role * (d - part) + padd; }
                                if let Ok(dbg) = std::env::var("CG_DEBUG") { let (dx, dy) = dbg.split_once(',').unwrap(); if dx.parse::<usize>().unwrap() == x && dy.parse::<usize>().unwrap() == y { eprintln!("band [{:.5},{:.5}] edge {} role {} piece ({:.5},{:.5})->({:.5},{:.5}) contrib {:.6}", s0, s1, i, role, a.0, a.1, b.0, b.1, (role * (d - part)) * 256.0); } }
                            }
                        }
                        if let Ok(dbg) = std::env::var("CG_DEBUG") { let (dx, dy) = dbg.split_once(',').unwrap(); let dx: i64 = dx.parse().unwrap(); if dy.parse::<usize>().unwrap() == y && dx > x1 { eprintln!("band [{:.5},{:.5}] edge {} role {} full piece ({:.5},{:.5})->({:.5},{:.5}) contrib {:.6}", s0, s1, i, role, a.0, a.1, b.0, b.1, role * d * 256.0); } }
                        for v in row.iter_mut().skip((x1 + 1).max(0) as usize) {
                            *v += role * d;
                        }
                    }
                    for &x in &touched {
                        let part = parts[x];
                        let part = match std::env::var("CG_PQ").ok().filter(|_| role > 0.0 || std::env::var("CG_PQ_BOTH").is_ok()) { Some(v) => { let (u, m) = v.split_once(':').unwrap(); let u: f64 = u.parse().unwrap(); match m { "ceil" => (part * u).ceil() / u, "floor" => (part * u).floor() / u, "round" => (part * u).round() / u, _ => part } } None => part };
                        row[x] -= role * part;
                    }
                }
                if let Some(sm) = shmode.as_deref() {
                    for x in 0..w { if !sbuf.is_empty() && sbuf[x] != 0.0 { let v = sbuf[x] * 4096.0; row[x] += match sm { "ceil" => (v - 1e-7).ceil(), "floor" => (v + 1e-7).floor(), "round" => v.round(), _ => v } / 4096.0; } }
                }
                if let Some(m) = oq.as_deref() {
                    let qf = |v: f64, m: &str| match m { "ceil" => (v - 1e-7).ceil(), "floor" => (v + 1e-7).floor(), "round" => v.round(), _ => v };
                    for x in 0..w {
                        let o = obuf[x]; let c = if cbuf.is_empty() { 0.0 } else { cbuf[x] };
                        if o != 0.0 && c != 0.0 {
                            match span_mode.as_deref() {
                                Some("net") => row[x] += qf((o + c) * 4096.0, m) / 4096.0,
                                Some("netfloor") => row[x] += qf((o + c) * 4096.0, "floor") / 4096.0,
                                Some("exact") => row[x] += o + c,
                                Some("sep") => row[x] += qf(o * 4096.0, m) / 4096.0 + c,
                                _ => row[x] += qf(o * 4096.0, m) / 4096.0 + c,
                            }
                        } else {
                            if o != 0.0 { row[x] += qf(o * 4096.0, m) / 4096.0; }
                            if c != 0.0 {
                                match oqc.as_deref() {
                                    Some(cm) => { let v = -c * 4096.0; let qv = match cm { "ceil" => (v - 1e-7).ceil(), "floor" => (v + 1e-7).floor(), "round" => v.round(), _ => v }; row[x] -= qv / 4096.0; }
                                    None => row[x] += c,
                                }
                            }
                        }
                    }
                }
                if !snap.is_empty() {
                    let (uu, m) = bandq.as_deref().unwrap().split_once(':').unwrap();
                    let uu: f64 = uu.parse().unwrap();
                    for x in 0..w {
                        let dlt = row[x] - snap[x];
                        let q = match m { "ceil" => (dlt * uu - 1e-9).ceil() / uu, "floor" => (dlt * uu + 1e-9).floor() / uu, "round" => (dlt * uu).round() / uu, "trunc" => (dlt * uu).trunc() / uu, _ => dlt };
                        row[x] = snap[x] + q;
                    }
                }
            }
        }
    }
    if std::env::var("CG_RAW").is_ok() {
        let mut g = RAW.lock().unwrap();
        g.clear();
        for r in &acc { for v in r { g.push(v.clamp(0.0, 1.0) * 256.0); } }
    }
    let am = std::env::var("CG_ALPHA").ok();
    acc.into_iter().map(|r| r.into_iter().map(|v| {
        let c = v.clamp(0.0, 1.0);
        let a = match am.as_deref() {
            Some(m) => { let (uu, mode) = m.split_once(':').unwrap(); let uu: f64 = uu.parse().unwrap(); let q = match mode { "round" => (c * uu).round(), "ceil" => (c * uu - 1e-9).ceil(), _ => (c * uu + 1e-9).floor() }; (q * 256.0 / uu + 1e-9).floor() as i64 }
            None => if std::env::var("CG_DEC").is_ok() { ((c * 256.0 - 1e-9).ceil() as i64 - 1).max(0) } else { (c * 256.0 + 1e-9).floor() as i64 },
        };
        a.min(255) as u8 }).collect()).collect()
}
