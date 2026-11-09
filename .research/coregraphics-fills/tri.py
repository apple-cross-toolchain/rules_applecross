import random, subprocess, sys, collections
sys.path.insert(0, '/tmp/cg'); from model import *
random.seed(int(sys.argv[1]) if len(sys.argv) > 1 else 3)
N = 8; Q = float(sys.argv[2]) if len(sys.argv) > 2 else 0   # vertex quantization (0 = none)
tris = [[(random.uniform(0, N), random.uniform(0, N)) for _ in range(3)] for _ in range(40)]
inp = ''.join('m %r %r\nl %r %r\nl %r %r\nh\nf\nn\n' % (t[0] + t[1] + t[2]) for t in tris)
out = subprocess.run(['./cov', str(N), str(N)], input=inp, capture_output=True, text=True).stdout.split('--\n')
q = (lambda v: round(v * Q) / Q) if Q else (lambda v: v)
cnt = collections.Counter(); bad = []
for t, img in zip(tris, out):
    rows = [list(map(int, r.split())) for r in img.strip().split('\n') if r]
    tq = [(q(x), q(y)) for x, y in t]
    for y in range(N):
        for x in range(N):
            obs = rows[N - 1 - y][x]; a = area(clip(tq, x, y, x + 1, y + 1))
            if a == 0 and obs == 0: continue
            d = alpha(a) - obs; cnt[d] += 1
            if d and len(bad) < 6: bad.append((round(a * 256, 3), alpha(a), obs))
print(sorted(cnt.items()), bad)
