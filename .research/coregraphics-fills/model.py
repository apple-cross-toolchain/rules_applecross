"""model.py: a model of CoreGraphics' fill coverage, for testing against cov."""
import math
def alpha(area):
    c = area * 256
    if c <= 61 / 32: return 0
    a = math.ceil(c - 1e-12) - 1
    return 255 if a >= 254 else a
def clip(poly, x0, y0, x1, y1):
    def c(poly, inside, inter):
        o = []
        for i in range(len(poly)):
            p, q = poly[i - 1], poly[i]
            if inside(q):
                if not inside(p): o.append(inter(p, q))
                o.append(q)
            elif inside(p): o.append(inter(p, q))
        return o
    ix = lambda x: (lambda p, q: (x, p[1] + (q[1] - p[1]) * (x - p[0]) / (q[0] - p[0])))
    iy = lambda y: (lambda p, q: (p[0] + (q[0] - p[0]) * (y - p[1]) / (q[1] - p[1]), y))
    for ins, it in ((lambda p: p[0] >= x0, ix(x0)), (lambda p: p[0] <= x1, ix(x1)), (lambda p: p[1] >= y0, iy(y0)), (lambda p: p[1] <= y1, iy(y1))):
        poly = c(poly, ins, it)
        if not poly: return []
    return poly
def area(p): return abs(sum(p[i - 1][0] * p[i][1] - p[i][0] * p[i - 1][1] for i in range(len(p)))) / 2
