"""Package an .ll (e.g. Xcode's -S -emit-llvm output) as a metallib with our writer,
for GPU comparisons when only the IR is at hand.  usage: ll2metallib.py -target T in.ll out.metallib"""
import os, re, subprocess, sys, tempfile
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import msl2air, mtlb
target, src, out = sys.argv[2], sys.argv[3], sys.argv[4]
os_name, ver, sim = msl2air.parse_target(target)
air, lang = msl2air.air_versions(os_name, ver)
ir = open(src).read()
with tempfile.TemporaryDirectory() as tmp:
    bc = os.path.join(tmp, 'm.bc')
    subprocess.run([msl2air.LLVM + '/llvm-as', '-o', bc, src], check=True)
    module = mtlb.wrap(open(bc, 'rb').read())
funcs = []
for stage, ty in (('kernel', 2), ('vertex', 0), ('fragment', 1)):
    m = re.search(r'^!air\.%s = !\{(.*)\}$' % stage, ir, re.M)
    for node in re.findall(r'!\d+', m.group(1)) if m else []:
        name = re.search(r'@([\w.]+)', re.search(r'^%s = (.*)$' % node, ir, re.M).group(1)).group(1)
        funcs.append(dict(name=name, type=ty, module=module, air=air, lang=lang))
open(out, 'wb').write(mtlb.metallib(funcs, msl2air.header(os_name, ver, sim, air), share_modules=True))
print(len(funcs), 'functions')
