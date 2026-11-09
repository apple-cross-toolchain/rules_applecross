//! Stroke outlines as CGPath.copy(strokingWithWidth:) builds them, for
//! widths above one pixel: offsets of lines and cubics on each side, joins,
//! caps, and closed contours. Port of strokes/stroke.py and stroke2.py.

use crate::cgfill::{Element, Point};

type P = (f64, f64);
const KAPPA: f64 = 0.5522847498307935;

#[derive(Clone, Copy, Debug)]
enum Seg {
    Line(P, P),
    Cubic([P; 4]),
}

#[derive(Clone, Copy, Debug)]
enum El {
    M(P),
    L(P),
    C(P, P, P),
    H,
}

fn dist(a: P, b: P) -> f64 {
    (a.0 - b.0).hypot(a.1 - b.1)
}
fn norm(v: P) -> P {
    let l = v.0.hypot(v.1);
    (v.0 / l, v.1 / l)
}
fn unit(a: P, b: P) -> P {
    norm((b.0 - a.0, b.1 - a.1))
}
fn bez(q: &[P; 4], t: f64) -> P {
    let u = 1.0 - t;
    let f = |i: usize| {
        let c = |p: P| if i == 0 { p.0 } else { p.1 };
        u * u * u * c(q[0]) + 3.0 * u * u * t * c(q[1]) + 3.0 * u * t * t * c(q[2]) + t * t * t * c(q[3])
    };
    (f(0), f(1))
}
fn d1(q: &[P; 4], t: f64) -> P {
    let u = 1.0 - t;
    let f = |i: usize| {
        let c = |p: P| if i == 0 { p.0 } else { p.1 };
        3.0 * u * u * (c(q[1]) - c(q[0])) + 6.0 * u * t * (c(q[2]) - c(q[1])) + 3.0 * t * t * (c(q[3]) - c(q[2]))
    };
    (f(0), f(1))
}
/// The true offset point at t: the curve point moved d along the left normal.
fn off(q: &[P; 4], t: f64, d: f64) -> P {
    let n = norm(d1(q, t));
    let p = bez(q, t);
    (p.0 - n.1 * d, p.1 + n.0 * d)
}
fn inter(p: P, u: P, q: P, v: P) -> Option<P> {
    let det = u.0 * v.1 - u.1 * v.0;
    if det.abs() < 1e-14 {
        return None;
    }
    let s = ((q.0 - p.0) * v.1 - (q.1 - p.1) * v.0) / det;
    Some((p.0 + s * u.0, p.1 + s * u.1))
}
/// The sub-curve on [a, b], by de Casteljau.
fn sub(q: &[P; 4], a: f64, b: f64) -> [P; 4] {
    let split = |q: &[P; 4], t: f64| -> ([P; 4], [P; 4]) {
        let l = |p: P, r: P| ((1.0 - t) * p.0 + t * r.0, (1.0 - t) * p.1 + t * r.1);
        let (a1, b1, c1) = (l(q[0], q[1]), l(q[1], q[2]), l(q[2], q[3]));
        let (d1, e1) = (l(a1, b1), l(b1, c1));
        let f = l(d1, e1);
        ([q[0], a1, d1, f], [f, e1, c1, q[3]])
    };
    let r = if b < 1.0 { split(q, b).0 } else { *q };
    if a > 0.0 { split(&r, a / b).1 } else { r }
}
/// Distance from p to the curve f on [0, 1]: a sampled minimum refined by
/// ternary search.
fn nearest(f: impl Fn(f64) -> P, p: P) -> f64 {
    let n = 400;
    let best = (0..=n).min_by(|&i, &j| dist(f(i as f64 / n as f64), p).total_cmp(&dist(f(j as f64 / n as f64), p))).unwrap();
    let (mut lo, mut hi) = (((best as f64 - 1.0) / n as f64).max(0.0), ((best as f64 + 1.0) / n as f64).min(1.0));
    for _ in 0..80 {
        let (a, b) = (lo + (hi - lo) / 3.0, hi - (hi - lo) / 3.0);
        if dist(f(a), p) < dist(f(b), p) {
            hi = b;
        } else {
            lo = a;
        }
    }
    dist(f((lo + hi) / 2.0), p)
}

fn seg_end(s: &Seg) -> P {
    match *s {
        Seg::Line(_, b) => b,
        Seg::Cubic(q) => q[3],
    }
}
fn seg_start(s: &Seg) -> P {
    match *s {
        Seg::Line(a, _) => a,
        Seg::Cubic(q) => q[0],
    }
}
fn rev(s: &Seg) -> Seg {
    match *s {
        Seg::Line(a, b) => Seg::Line(b, a),
        Seg::Cubic(q) => Seg::Cubic([q[3], q[2], q[1], q[0]]),
    }
}
/// Unit tangents at the start and end, falling back past coincident
/// control points.
fn seg_dirs(s: &Seg) -> (P, P) {
    match *s {
        Seg::Line(a, b) => {
            let u = unit(a, b);
            (u, u)
        }
        Seg::Cubic(q) => {
            let zero = |v: P| v.0.hypot(v.1) == 0.0;
            let mut a = d1(&q, 0.0);
            let mut b = d1(&q, 1.0);
            if zero(a) { a = (q[2].0 - q[0].0, q[2].1 - q[0].1); }
            if zero(a) { a = (q[3].0 - q[0].0, q[3].1 - q[0].1); }
            if zero(b) { b = (q[3].0 - q[1].0, q[3].1 - q[1].1); }
            if zero(b) { b = (q[3].0 - q[0].0, q[3].1 - q[0].1); }
            if zero(a) { a = (1.0, 0.0); }
            if zero(b) { b = a; }
            (norm(a), norm(b))
        }
    }
}
fn seg_len(s: &Seg) -> f64 {
    match *s {
        Seg::Line(a, b) => dist(a, b),
        Seg::Cubic(q) => (0..16).map(|i| dist(bez(&q, i as f64 / 16.0), bez(&q, (i + 1) as f64 / 16.0))).sum(),
    }
}
fn start_off(s: &Seg, d: f64) -> P {
    let (u, _) = seg_dirs(s);
    let p = seg_start(s);
    (p.0 - u.1 * d, p.1 + u.0 * d)
}

/// The join at v on the side at signed distance d: the points after the
/// incoming segment's end offset, up to the next segment's start offset.
fn join(v: P, u1: P, u2: P, d: f64, jt: i32, l1: f64, l2: f64, rev: bool) -> Vec<El> {
    let ml = 10.0;
    let n1 = (-u1.1 * d, u1.0 * d);
    let n2 = (-u2.1 * d, u2.0 * d);
    let p1 = (v.0 + n1.0, v.1 + n1.1);
    let p2 = (v.0 + n2.0, v.1 + n2.1);
    let cross = u1.0 * u2.1 - u1.1 * u2.0;
    let dot = u1.0 * u2.0 + u1.1 * u2.1;
    let mut outer = if d > 0.0 { cross < 0.0 } else { cross > 0.0 };
    if jt == 1 && dot < 0.0 && cross.abs() < 1e-3 {
        outer = rev;
    }
    let gap2 = (p1.0 - p2.0).powi(2) + (p1.1 - p2.1).powi(2);
    if gap2 < 0.01 {
        return vec![];
    }
    if !outer || (cross == 0.0 && dot > 0.0) {
        if !outer && cross != 0.0 && l1.min(l2) < d.abs() {
            return vec![El::L(v), El::L(p2)];
        }
        return vec![El::L(p2)];
    }
    if jt == 0 {
        if 2.0 / (1.0 + dot) <= ml * ml && gap2 >= 0.05 {
            let s = (n1.0 + n2.0, n1.1 + n2.1);
            let f = 2.0 / (1.0 + dot) / 2.0;
            return vec![El::L((v.0 + s.0 * f, v.1 + s.1 * f)), El::L(p2)];
        }
        return vec![El::L(p2)];
    }
    if jt == 1 && gap2 >= 0.05 {
        let th = if dot < 0.0 && cross.abs() < 1e-3 { std::f64::consts::PI } else { dot.clamp(-1.0, 1.0).acos() };
        let k = 4.0 / 3.0 * (th / 4.0).tan() * d.abs();
        return vec![El::C((p1.0 + u1.0 * k, p1.1 + u1.1 * k), (p2.0 - u2.0 * k, p2.1 - u2.1 * k), p2)];
    }
    vec![El::L(p2)]
}

/// The cap at end point v with outward direction u, from the left offset to
/// the right one.
fn cap(v: P, u: P, d: f64, ct: i32) -> Vec<El> {
    let n = (-u.1 * d, u.0 * d);
    let pl = (v.0 + n.0, v.1 + n.1);
    let pr = (v.0 - n.0, v.1 - n.1);
    match ct {
        0 => vec![El::L(pr)],
        2 => {
            let e = (u.0 * d, u.1 * d);
            vec![El::L((pl.0 + e.0, pl.1 + e.1)), El::L((pr.0 + e.0, pr.1 + e.1)), El::L(pr)]
        }
        _ => {
            let k = KAPPA * d;
            let tip = (v.0 + u.0 * d, v.1 + u.1 * d);
            vec![
                El::C((pl.0 + u.0 * k, pl.1 + u.1 * k), (tip.0 + n.0 * KAPPA, tip.1 + n.1 * KAPPA), tip),
                El::C((tip.0 - n.0 * KAPPA, tip.1 - n.1 * KAPPA), (pr.0 + u.0 * k, pr.1 + u.1 * k), pr),
            ]
        }
    }
}

/// One offset piece of a cubic: the zero-width refit through the start,
/// midpoint and end tangent lines, with handle lengths linear in d.
fn piece_one(q: &[P; 4], d: f64) -> El {
    let (u0, u3) = seg_dirs(&Seg::Cubic(*q));
    let p0 = (q[0].0 - u0.1 * d, q[0].1 + u0.0 * d);
    let p3 = (q[3].0 - u3.1 * d, q[3].1 + u3.0 * d);
    let mut t = d1(q, 0.5);
    if t.0.hypot(t.1) == 0.0 {
        t = (q[3].0 - q[0].0, q[3].1 - q[0].1);
    }
    let um = norm(t);
    let m = bez(q, 0.5);
    let ang = |a: P, b: P| (a.0 * b.1 - a.1 * b.0).atan2(a.0 * b.0 + a.1 * b.1);
    let t1 = (ang(u0, um) / 2.0).tan();
    let t2 = (ang(um, u3) / 2.0).tan();
    if t1.abs() < 1e-12 || t2.abs() < 1e-12 {
        let mid = ((q[1].0 + q[2].0) / 2.0, (q[1].1 + q[2].1) / 2.0);
        let c = (mid.0 - um.1 * d, mid.1 + um.0 * d);
        return El::C(c, c, p3);
    }
    let (Some(x1), Some(x2)) = (inter(q[0], u0, m, um), inter(q[3], u3, m, um)) else {
        return fallback(q, d);
    };
    let h1 = (q[1].0 - q[0].0, q[1].1 - q[0].1);
    let h2 = (q[2].0 - q[3].0, q[2].1 - q[3].1);
    let mut l1 = (2.0 * (x1.0 - q[0].0) - h1.0 / 2.0) * u0.0 + (2.0 * (x1.1 - q[0].1) - h1.1 / 2.0) * u0.1;
    let mut l2 = -((2.0 * (x2.0 - q[3].0) - h2.0 / 2.0) * u3.0 + (2.0 * (x2.1 - q[3].1) - h2.1 / 2.0) * u3.1);
    let g = |a: f64, b: f64| 4.0 / 3.0 * a + (b * b - a * a) / (6.0 * a * (1.0 + b * b));
    let (neg1, neg2) = (l1 < 0.0, l2 < 0.0);
    l1 -= g(t1, t2) * d;
    l2 -= g(t2, t1) * d;
    let mut c1 = (p0.0 + u0.0 * l1, p0.1 + u0.1 * l1);
    let mut c2 = (p3.0 - u3.0 * l2, p3.1 - u3.1 * l2);
    if neg1 || neg2 {
        let fb = fallback_point(q, d, p0, p3, m, um);
        if neg1 { c1 = fb; }
        if neg2 { c2 = fb; }
    }
    El::C(c1, c2, p3)
}
/// The point X with which a cubic C1 = C2 = X passes through the offset
/// midpoint.
fn fallback_point(_q: &[P; 4], d: f64, p0: P, p3: P, m: P, um: P) -> P {
    let mo = (m.0 - um.1 * d, m.1 + um.0 * d);
    ((8.0 * mo.0 - p0.0 - p3.0) / 6.0, (8.0 * mo.1 - p0.1 - p3.1) / 6.0)
}
fn fallback(q: &[P; 4], d: f64) -> El {
    let (u0, u3) = seg_dirs(&Seg::Cubic(*q));
    let p0 = (q[0].0 - u0.1 * d, q[0].1 + u0.0 * d);
    let p3 = (q[3].0 - u3.1 * d, q[3].1 + u3.0 * d);
    let mut t = d1(q, 0.5);
    if t.0.hypot(t.1) == 0.0 {
        t = (q[3].0 - q[0].0, q[3].1 - q[0].1);
    }
    let fb = fallback_point(q, d, p0, p3, bez(q, 0.5), norm(t));
    El::C(fb, fb, p3)
}
/// Whether one side's piece is close enough: the true offset at five
/// parameters within 0.1 px of it.
fn accept(q: &[P; 4], d: f64) -> bool {
    let El::C(c1, c2, c3) = piece_one(q, d) else { return false };
    let (u0, _) = seg_dirs(&Seg::Cubic(*q));
    let c = [(q[0].0 - u0.1 * d, q[0].1 + u0.0 * d), c1, c2, c3];
    [0.1, 0.3, 0.5, 0.7, 0.9].iter().all(|&t| {
        let e = nearest(|x| bez(&c, x), off(q, t, d));
        !(e > 0.1)
    })
}
/// The parameters where x'' or y'' changes sign, in order.
fn coord_inflections(q: &[P; 4]) -> Vec<f64> {
    let mut ts = Vec::new();
    for i in 0..2 {
        let c = |p: P| if i == 0 { p.0 } else { p.1 };
        let a = c(q[0]) - 2.0 * c(q[1]) + c(q[2]);
        let b = c(q[1]) - 2.0 * c(q[2]) + c(q[3]);
        if (a > 0.0 && b < 0.0) || (a < 0.0 && b > 0.0) {
            ts.push(a / (a - b));
        }
    }
    ts.retain(|&t| t > 0.0 && t < 1.0);
    ts.sort_by(f64::total_cmp);
    ts.dedup();
    ts
}
/// The offset of one segment on the side at d, without its start point.
fn seg_off(s: &Seg, d: f64) -> Vec<El> {
    let q = match *s {
        Seg::Line(a, b) => {
            let u = unit(a, b);
            return vec![El::L((b.0 - u.1 * d, b.1 + u.0 * d))];
        }
        Seg::Cubic(q) => q,
    };
    let cr = |a: P, b: P| a.0 * b.1 - a.1 * b.0;
    let l0 = (q[1].0 - q[0].0, q[1].1 - q[0].1);
    let l1 = (q[2].0 - q[1].0, q[2].1 - q[1].1);
    let l2 = (q[3].0 - q[2].0, q[3].1 - q[2].1);
    let pre = if cr(l0, l1) * cr(l1, l2) <= 0.0 { coord_inflections(&q) } else { Vec::new() };
    let mut out = Vec::new();
    fn rec(q: &[P; 4], d: f64, a: f64, b: f64, depth: u32, pre: &[f64], out: &mut Vec<El>) {
        if b - a < 1e-9 {
            return;
        }
        let sq = sub(q, a, b);
        if depth >= 8 || accept(&sq, d) {
            let middle = pre.len() == 2 && (a - pre[0]).abs() < 1e-12 && (b - pre[1]).abs() < 1e-12;
            let chord = dist(sq[0], sq[3]);
            out.push(if (middle && chord < 0.5) || chord < 0.1 { fallback(&sq, d) } else { piece_one(&sq, d) });
            return;
        }
        let m = (a + b) / 2.0;
        rec(q, d, a, m, depth + 1, pre, out);
        rec(q, d, m, b, depth + 1, pre, out);
    }
    let mut cuts = vec![0.0];
    cuts.extend(&pre);
    cuts.push(1.0);
    for w in cuts.windows(2) {
        rec(&q, d, w[0], w[1], 0, &pre, &mut out);
    }
    out
}

fn side(segs: &[Seg], d: f64, jt: i32, rev_: bool) -> (P, Vec<El>) {
    let mut out = Vec::new();
    for (i, s) in segs.iter().enumerate() {
        out.extend(seg_off(s, d));
        if i + 1 < segs.len() {
            let (_, u1) = seg_dirs(s);
            let (u2, _) = seg_dirs(&segs[i + 1]);
            out.extend(join(seg_end(s), u1, u2, d, jt, seg_len(s), seg_len(&segs[i + 1]), rev_));
        }
    }
    (start_off(&segs[0], d), out)
}

fn stroke_sub(segs: &[Seg], w: f64, jt: i32, ct: i32, closed: bool) -> Vec<El> {
    let d = w / 2.0;
    let mut res = Vec::new();
    let r: Vec<Seg> = segs.iter().rev().map(rev).collect();
    if !closed {
        let (s, o) = side(segs, d, jt, false);
        res.push(El::M(s));
        res.extend(o);
        let (_, ue) = seg_dirs(segs.last().unwrap());
        res.extend(cap(seg_end(segs.last().unwrap()), ue, d, ct));
        let (_, o2) = side(&r, d, jt, true);
        res.extend(o2);
        let (_, ue2) = seg_dirs(r.last().unwrap());
        res.extend(cap(seg_end(r.last().unwrap()), ue2, d, ct));
        res.push(El::H);
        return res;
    }
    let (s, o) = side(segs, d, jt, false);
    res.push(El::M(s));
    res.extend(o);
    let (_, u1) = seg_dirs(segs.last().unwrap());
    let (u2, _) = seg_dirs(&segs[0]);
    res.extend(join(seg_start(&segs[0]), u1, u2, d, jt, seg_len(segs.last().unwrap()), seg_len(&segs[0]), false));
    res.push(El::H);
    let (s2, o2) = side(&r, d, jt, true);
    let (uf, _) = seg_dirs(&segs[0]);
    let n = (uf.1 * d, -uf.0 * d);
    let (_, ur) = seg_dirs(r.last().unwrap());
    let (ur2, _) = seg_dirs(&r[0]);
    let p0 = seg_start(&segs[0]);
    let j0 = join(p0, ur, ur2, d, jt, seg_len(&segs[0]), seg_len(segs.last().unwrap()), true);
    if j0.is_empty() {
        res.push(El::M(s2));
    } else {
        res.push(El::M((p0.0 + n.0, p0.1 + n.1)));
        res.extend(j0);
    }
    res.extend(o2);
    res.push(El::H);
    res
}

/// The outline of `path` stroked at `width` (> 1 px) with CGLineJoin
/// `join` and CGLineCap `cap` raw values, miter limit 10.
pub fn stroke(path: &[Element], width: f64, join: i32, cap: i32) -> Vec<Element> {
    let pt = |p: Point| (p.x, p.y);
    let mut subs: Vec<(Vec<Seg>, bool)> = Vec::new();
    let mut segs: Vec<Seg> = Vec::new();
    let (mut cur, mut start) = ((0.0, 0.0), (0.0, 0.0));
    for e in path {
        match *e {
            Element::Move(p) => {
                if !segs.is_empty() {
                    subs.push((std::mem::take(&mut segs), false));
                }
                cur = pt(p);
                start = cur;
            }
            Element::Line(p) => {
                let p = pt(p);
                if p != cur {
                    segs.push(Seg::Line(cur, p));
                }
                cur = p;
            }
            Element::Cubic(c1, c2, p) => {
                let p = pt(p);
                segs.push(Seg::Cubic([cur, pt(c1), pt(c2), p]));
                cur = p;
            }
            Element::Close => {
                if cur != start {
                    segs.push(Seg::Line(cur, start));
                }
                if !segs.is_empty() {
                    subs.push((std::mem::take(&mut segs), true));
                }
                cur = start;
            }
        }
    }
    if !segs.is_empty() {
        subs.push((segs, false));
    }
    let mut out = Vec::new();
    let p = |v: P| Point { x: v.0, y: v.1 };
    for (segs, closed) in subs {
        for e in stroke_sub(&segs, width, join, cap, closed) {
            out.push(match e {
                El::M(a) => Element::Move(p(a)),
                El::L(a) => Element::Line(p(a)),
                El::C(a, b, c) => Element::Cubic(p(a), p(b), p(c)),
                El::H => Element::Close,
            });
        }
    }
    out
}
