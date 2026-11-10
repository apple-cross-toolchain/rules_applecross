"""Every distinct compile error across a corpus (front end only), most frequent first."""
import collections, glob, os, re, subprocess, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import msl2air as m
root, incs = sys.argv[1], sys.argv[2:]
c = collections.Counter()
for f in sorted(glob.glob(root + '/**/*.metal', recursive=True)):
    pp = [m.LLVM + '/clang++', '-E', '-x', 'c++', '-std=c++17', '-target', 'spir64', '-ffast-math', '-nostdinc++', '-I', m.HERE + '/include',
          '-include', 'metal_stdlib', '-I', os.path.dirname(f), '-D__METAL_VERSION__=410', '-D__METAL_IOS__', '-D__MSL_TARGET_IOS'] + incs + [f]
    r = subprocess.run(pp, capture_output=True, text=True)
    if r.returncode:
        c['preprocess: ' + (re.findall(r'error: (.*)', r.stderr) or ['?'])[0]] += 1; continue
    m.SAMPLERS.clear(); m.INSTANCES.clear()
    code = m.rewrite_samplers(m.rewrite(r.stdout, False), (2, 9))
    open('/tmp/_e.cpp', 'w').write(code)
    r = subprocess.run([m.LLVM + '/clang++', '-x', 'c++', '-std=c++17', '-target', 'spir64', '-ffast-math', '-fsyntax-only', '-ferror-limit=0', '-nostdinc++', '/tmp/_e.cpp'],
                       capture_output=True, text=True)
    for e in set(re.findall(r'error: (.*)', r.stderr)):
        c[e] += 1
for k, v in c.most_common(45):
    print(v, k[:150])
