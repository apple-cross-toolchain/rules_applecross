"""framediff.py X.nib O.nib: views whose frames differ between Xcode's nib and ours (paired by class in object order)."""
import io, re, sys, contextlib
sys.path.insert(0, '/tmp/ak/py')
import nibdumpfull
def views(path):
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf): nibdumpfull.dump(path)
    objs = {}
    for line in buf.getvalue().splitlines():
        n, cls, rest = (line.split(' ', 2) + [''])[:3]
        objs[int(n)] = (cls, re.findall(r"(\S+?)=(b'(?:[^'\\]|\\.)*'|\S+)", rest))
    out = []
    for n, (cls, kv) in objs.items():
        d = dict(kv)
        f = d.get('NSFrame') or d.get('NSFrameSize')
        if f and f.startswith('@'):
            s = dict(objs[int(f[1:])][1]).get('NS.bytes', '')
            nums = [float(x) for x in re.findall(r'-?[\d.]+(?:e-?\d+)?', s)]
            if len(nums) == 2: nums = [0, 0] + nums
            out.append((cls, nums))
    return out
x, o = views(sys.argv[1]), views(sys.argv[2])
if [c for c, _ in x] != [c for c, _ in o]:
    print('UNPAIRED', len(x), len(o)); sys.exit()
for (c, a), (_, b) in zip(x, o):
    if a != b:
        d = [round(p - q, 3) for p, q in zip(a, b)]
        print('%s dx=%g dy=%g dw=%g dh=%g  xcode=%s ours=%s' % (c, *d, a, b))
