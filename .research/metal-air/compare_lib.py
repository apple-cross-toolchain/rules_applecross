"""Compare two metallibs' function lists and public metadata (not the modules)."""
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import rebuild
def summary(path):
    d, funcs = rebuild.split(open(path, 'rb').read())
    return d['hdr'], [(f['name'], f['type'], f['air'], f['lang'], f['pub'], f['priv']) for f in funcs]
a, b = summary(sys.argv[1]), summary(sys.argv[2])
if a == b: print('OK'); sys.exit(0)
print('ours ', a[0]); print('xcode', b[0])
for x, y in zip(a[1], b[1]):
    if x != y: print(' ours ', x[:4], x[4].hex()); print(' xcode', y[:4], y[4].hex())
if len(a[1]) != len(b[1]): print('function count', len(a[1]), len(b[1]))
sys.exit(1)
