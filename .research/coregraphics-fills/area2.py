"""area2.py: exact signed-area coverage per cell over edges whose y-extents are snapped (lower end floored to the
1/256 grid, upper end floored from just below), each edge kept on its own line; nonzero winding."""
import math
def snap_lo(y): return math.floor(y * 256 + 1e-12) / 256
def snap_hi(y): return (math.ceil(y * 256 - 1e-12) - 1) / 256
def coverage(polys, W, H, snap=True):
    cover = {}; part = {}
    def add(cx, cy, dy, frac_mid):
        cover[(cx, cy)] = cover.get((cx, cy), 0) + dy
        part[(cx, cy)] = part.get((cx, cy), 0) + dy * (1 - frac_mid)
    for pts in polys:
        for i in range(len(pts)):
            (x0, y0), (x1, y1) = pts[i], pts[(i + 1) % len(pts)]
            if y0 == y1: continue
            d = 1 if y1 > y0 else -1
            (xa, ya), (xb, yb) = ((x0, y0), (x1, y1)) if y0 < y1 else ((x1, y1), (x0, y0))
            xat = lambda y: xa + (xb - xa) * (y - ya) / (yb - ya)
            lo, hi = (snap_lo(ya), snap_hi(yb)) if snap else (ya, yb)
            if hi <= lo: continue
            # walk the segment from lo to hi, splitting at row and column boundaries
            ys = sorted(set([lo, hi] + [float(k) for k in range(math.ceil(lo), math.floor(hi) + 1) if lo < k < hi]))
            for j in range(len(ys) - 1):
                y_a, y_b = ys[j], ys[j + 1]
                xs_a, xs_b = xat(y_a), xat(y_b)
                cy = math.floor((y_a + y_b) / 2)
                # split at column boundaries
                xl, xr = min(xs_a, xs_b), max(xs_a, xs_b)
                cuts = [k for k in range(math.ceil(xl), math.floor(xr) + 1) if xl < k < xr]
                pts_x = [xs_a] + (sorted(cuts) if xs_b > xs_a else sorted(cuts, reverse=True)) + [xs_b]
                for k in range(len(pts_x) - 1):
                    p, q = pts_x[k], pts_x[k + 1]
                    if xs_b == xs_a: ya2, yb2 = y_a, y_b
                    else:
                        ya2 = y_a + (y_b - y_a) * (p - xs_a) / (xs_b - xs_a); yb2 = y_a + (y_b - y_a) * (q - xs_a) / (xs_b - xs_a)
                    dy = (yb2 - ya2) * d
                    mid = (p + q) / 2; cx = math.floor(mid)
                    add(cx, cy, dy, mid - cx)
    out = [[0] * W for _ in range(H)]
    rows = {}
    for (cx, cy) in cover: rows.setdefault(cy, []).append(cx)
    for cy, cxs in rows.items():
        if not 0 <= cy < H: continue
        acc = 0.0
        cxs = sorted(set(cxs))
        for idx, cx in enumerate(cxs):
            v = acc + part[(cx, cy)]
            if 0 <= cx < W: out[cy][cx] = v
            acc += cover[(cx, cy)]
            nxt = cxs[idx + 1] if idx + 1 < len(cxs) else W
            for x in range(max(cx + 1, 0), min(nxt, W)): out[cy][x] = acc
    return out
def alphas(cov):
    return [[min(255, math.floor(abs(v) * 256 + 1e-9)) for v in row] for row in cov]
