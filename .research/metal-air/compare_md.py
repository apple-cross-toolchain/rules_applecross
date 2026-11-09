"""Compare the AIR metadata of two .ll files: named metadata, resolved."""
import re, sys
def load(path):
    s = open(path).read()
    nodes = dict(re.findall(r'^(![0-9]+) = (?:distinct )?(!\{.*\})$', s, re.M))
    named = dict(re.findall(r'^!([a-z][\w.]*) = (!\{.*\})$', s, re.M))
    def res(text, depth=0):
        return re.sub(r'(![0-9]+)\b', lambda m: res(nodes[m.group(1)], depth + 1) if m.group(1) in nodes and depth < 8 else m.group(1), text)
    out = {}
    for k, v in named.items():
        if k in ('llvm.ident', 'air.source_file_name'):
            continue
        items = re.findall(r'![0-9]+', v)
        out[k] = [res(nodes[i]) for i in items]
    return out
def per_function(md):
    out = {}
    for st in ('air.kernel', 'air.vertex', 'air.fragment'):
        for e in md.get(st, []):
            name = re.search(r'@([\w.]+)', e).group(1)
            out[name] = (st, re.sub(r'^!\{[^@]*@[\w.]+, ', '', e))
    return out
if __name__ == '__main__':
    a, b = load(sys.argv[1]), load(sys.argv[2])
    bad = 0
    for k in sorted(set(a) | set(b)):
        if k in ('air.kernel', 'air.vertex', 'air.fragment'):
            continue
        if a.get(k) != b.get(k):
            bad += 1; print('named', k, '\n  ours ', a.get(k), '\n  xcode', b.get(k))
    fa, fb = per_function(a), per_function(b)
    for k in sorted(set(fa) | set(fb)):
        if fa.get(k) != fb.get(k):
            bad += 1
            print('function', k)
            x, y = fa.get(k, ('', ''))[1], fb.get(k, ('', ''))[1]
            print('  ours ', x[:2000]); print('  xcode', y[:2000])
    print('OK' if not bad else 'DIFF %d' % bad)
    sys.exit(1 if bad else 0)
