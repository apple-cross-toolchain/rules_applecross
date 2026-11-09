"""eval.py: for each layout-refused document, the prototype's frames against Xcode's (from /tmp/ak/fd.txt pairs)."""
import subprocess, collections, re
refused = [l.split()[0] for l in open('/tmp/ak/layout_refused.txt')]
files = dict(l.rstrip('\n').split('\t') for l in open('/tmp/maccorp/files.tsv'))
files = {k[:8]: v for k, v in files.items()}
relaxed = dict(l.split()[:2] for l in open('/tmp/ak/r1.out'))
fd = collections.defaultdict(list)
for l in open('/tmp/ak/fd.txt'):
    h = l.split()[0]
    x = [float(v) for v in re.search(r'xcode=\[([^\]]*)\]', l).group(1).split(',')]
    o = [float(v) for v in re.search(r'ours=\[([^\]]*)\]', l).group(1).split(',')]
    fd[h].append((o, x))
tally = collections.Counter(); blocked = collections.Counter(); detail = []
for h in refused:
    if relaxed.get(h) not in ('SAME', 'SHAPE'):
        tally['relaxed refused'] += 1; continue
    out = subprocess.run(['/tmp/lv/bin/python', '/tmp/lay/solve.py', files[h]], capture_output=True, text=True).stdout
    if out.startswith('BLOCKED') or 'Traceback' in out:
        reason = out.strip().splitlines()[-1][:70]; blocked[reason] += 1; tally['blocked'] += 1; continue
    expected_changes = {tuple(o): x for o, x in fd[h]}
    wrong = 0; covered = set()
    for line in out.splitlines():
        vid, rest = line.split(' ', 1); a, b = rest.split('] [')
        doc = tuple(float(v) for v in a.strip('[ ').split(',')); new = [round(float(v), 4) for v in b.strip('] \n').split(',')]
        exp = expected_changes.get(doc, list(doc))
        if doc in expected_changes: covered.add(doc)
        if [round(v, 4) for v in exp] != new: wrong += 1
    missed = len(set(expected_changes) - covered)   # changed views the prototype doesn't lay out
    if wrong == 0 and missed == 0: tally['exact'] += 1
    else: tally['wrong'] += 1; detail.append((h, wrong, missed))
print(dict(tally)); print(blocked.most_common()); print(sorted(detail, key=lambda d: d[1] + d[2]))
