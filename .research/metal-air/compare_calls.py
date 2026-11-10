"""Per function, compare the set of air.* functions called (ours vs Xcode)."""
import re, sys
def calls(path):
    ir = open(path).read(); out = {}
    for m in re.finditer(r'^define [^@]*@([\w.]+)\(.*?\n\}', ir, re.M | re.S):
        out[m.group(1)] = sorted(set(c for c in re.findall(r'@(air\.[\w.]+)\(', m.group(0).split('\n', 1)[1])
                                     if not c.startswith('air.convert')))
    return out
a, b = calls(sys.argv[1]), calls(sys.argv[2])
bad = [k for k in b if a.get(k) != b[k]]
for k in bad[:20]:
    print(k, '\n  ours ', a.get(k), '\n  xcode', b[k])
print('functions', len(b), 'differ', len(bad))
