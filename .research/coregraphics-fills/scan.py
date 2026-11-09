"""scan.py: a sub-scanline model of CoreGraphics' general fill: 255 sub-scanlines per pixel row (y = row + j/256,
j = 1..255), exact x coverage along each (nonzero winding), alpha from the summed length."""
import math
def coverage(polys, W, H, K=256/255, c0=1/32, xq=None):
    # edges: (x0, y0, x1, y1, winding)
    edges = []
    for pts in polys:
        for i in range(len(pts)):
            (x0, y0), (x1, y1) = pts[i], pts[(i + 1) % len(pts)]
            if y0 == y1: continue
            edges.append((x0, y0, x1, y1, 1 if y1 > y0 else -1))
    acc = [[0.0] * W for _ in range(H)]
    for r in range(H):
        for j in range(1, 256):
            y = r + j / 256
            xs = []
            for x0, y0, x1, y1, d in edges:
                lo, hi = min(y0, y1), max(y0, y1)
                if lo <= y < hi:
                    xv = x0 + (x1 - x0) * (y - y0) / (y1 - y0)
                    if xq: xv = xq(xv)
                    xs.append((xv, d))
            xs.sort()
            wind = 0
            for k in range(len(xs) - 1):
                wind += xs[k][1]
                if wind == 0: continue
                a, b = max(0.0, xs[k][0]), min(float(W), xs[k + 1][0])
                if b <= a: continue
                px = int(math.floor(a))
                while px < W and px < b:
                    acc[r][px] += min(b, px + 1) - max(a, px)
                    px += 1
    return [[min(255, math.floor(s * K + c0 + 1e-9)) if s > 0 else 0 for s in row] for row in acc]
