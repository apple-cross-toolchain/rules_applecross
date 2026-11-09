"""gray.py: a FreeType-style cell rasterizer (cover and area per cell, fixed point), with PIXEL_BITS and the
input rounding and final coverage mapping as parameters, to compare with CoreGraphics."""
import math
class Raster:
    def __init__(self, w, h, P=8, rnd='round', final='floor'):
        self.W, self.H, self.P, self.ONE = w, h, P, 1 << P
        self.rnd, self.final = rnd, final
        self.cells = {}
    def fix(self, v):
        s = v * self.ONE
        return {'round': math.floor(s + 0.5), 'floor': math.floor(s), 'trunc': int(s)}[self.rnd]
    def cell(self, ex, ey):
        return self.cells.setdefault((ex, ey), [0, 0])
    def scanline(self, ey, x1, y1, x2, y2):
        ONE = self.ONE
        ex1, ex2 = x1 >> self.P, x2 >> self.P
        fx1, fx2 = x1 - (ex1 << self.P), x2 - (ex2 << self.P)
        if y1 == y2: return
        if ex1 == ex2:
            c = self.cell(ex1, ey); d = y2 - y1; c[1] += (fx1 + fx2) * d; c[0] += d; return
        p = (ONE - fx1) * (y2 - y1); first, incr, dx = ONE, 1, x2 - x1
        if dx < 0: p = fx1 * (y2 - y1); first, incr, dx = 0, -1, -dx
        delta, mod = divmod(p, dx)
        c = self.cell(ex1, ey); c[1] += (fx1 + first) * delta; c[0] += delta
        y1 += delta; ex1 += incr
        if ex1 != ex2:
            lift, rem = divmod(ONE * (y2 - y1 + delta), dx)
            mod -= dx
            while ex1 != ex2:
                delta = lift; mod += rem
                if mod >= 0: mod -= dx; delta += 1
                c = self.cell(ex1, ey); c[1] += ONE * delta; c[0] += delta
                y1 += delta; ex1 += incr
        delta = y2 - y1
        c = self.cell(ex1, ey); c[1] += (fx2 + ONE - first) * delta; c[0] += delta
    def line(self, x1, y1, x2, y2):
        P, ONE = self.P, self.ONE
        ey1, ey2 = y1 >> P, y2 >> P
        fy1, fy2 = y1 - (ey1 << P), y2 - (ey2 << P)
        if ey1 == ey2: self.scanline(ey1, x1, fy1, x2, fy2); return
        dx, dy = x2 - x1, y2 - y1
        p = (ONE - fy1) * dx; first, incr = ONE, 1
        if dy < 0: p = fy1 * dx; first, incr, dy = 0, -1, -dy
        delta, mod = divmod(p, dy)
        x = x1 + delta
        self.scanline(ey1, x1, fy1, x, first)
        ey1 += incr
        if ey1 != ey2:
            lift, rem = divmod(ONE * dx, dy)
            mod -= dy
            while ey1 != ey2:
                delta = lift; mod += rem
                if mod >= 0: mod -= dy; delta += 1
                x2b = x + delta
                self.scanline(ey1, x, ONE - first, x2b, first)
                x = x2b; ey1 += incr
        self.scanline(ey1, x, ONE - first, x2, fy2)
    def polygon(self, pts):
        q = [(self.fix(x), self.fix(y)) for x, y in pts]
        for i in range(len(q)):
            (x1, y1), (x2, y2) = q[i], q[(i + 1) % len(q)]
            self.line(x1, y1, x2, y2)
    def alphas(self):
        P = self.P; shift = 2 * P + 1 - 8
        out = [[0] * self.W for _ in range(self.H)]
        rows = {}
        for (ex, ey), c in self.cells.items(): rows.setdefault(ey, []).append((ex, c))
        for ey, cs in rows.items():
            cs.sort(); cover = 0
            for i, (ex, (cv, ar)) in enumerate(cs):
                cover += cv
                a = cover * (self.ONE * 2) - ar
                self.put(out, ex, ey, a, shift)
                nxt = cs[i + 1][0] if i + 1 < len(cs) else self.W
                for x in range(ex + 1, nxt):
                    self.put(out, x, ey, cover * (self.ONE * 2), shift)
        return out
    def put(self, out, x, y, a, shift):
        if not (0 <= x < self.W and 0 <= y < self.H): return
        a = abs(a)
        if self.final == 'floor': v = a >> shift
        elif self.final == 'ceil-1': v = max(0, ((a - 1) >> shift)) if a else 0
        v = min(v, 255)
        out[y][x] = v
