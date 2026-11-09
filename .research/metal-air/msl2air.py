"""MSL -> AIR -> metallib with clang 15 (M1 prototype).

usage: msl2air.py -target air64-apple-ios27.2 [-I dir]... -o out.metallib in.metal

1. Rewrite the MSL attribute syntax clang can't keep: [[x]] becomes an
   annotate attribute, vector constructors call metal::__vec, half literals
   become casts. Stage keywords are macros in our metal_stdlib.
2. Read each stage function's signature from clang's AST dump.
3. Append an extern "C" entry point per stage function with AIR's flat
   signature, calling the user's function (inlined away at -O2).
4. Compile to LLVM IR for spir64 with typed pointers; keep record layouts.
5. Turn the IR into AIR: triple, datalayout, attributes, metadata.
6. llvm-as and the metallib writer.
"""
import os, re, shutil, subprocess, sys, tempfile, struct
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import mtlb

LLVM = os.environ.get('LLVM15', '/tmp/llvm15/bin')
DATALAYOUT = ('e-p:64:64:64-i1:8:8-i8:8:8-i16:16:16-i32:32:32-i64:64:64-f32:32:32-f64:64:64'
              '-v16:16:16-v24:32:32-v32:32:32-v48:64:64-v64:64:64-v96:128:128-v128:128:128'
              '-v192:256:256-v256:256:256-v512:512:512-v1024:1024:1024-n8:16:32')

# ---------------------------------------------------------------- targets
# (os, min version) -> AIR (major, minor), language (major, minor); FORMAT.md
def air_versions(os_name, ver):
    major = ver[0]
    if os_name == 'macosx':
        ladder = {10: None, 11: 3, 12: 4, 13: 5, 14: 6, 15: 7, 26: 8, 27: 9}
        if major == 10:
            air_minor = {14: 1, 15: 2}.get(ver[1], 1)
        else:
            air_minor = ladder.get(major, 9 if major > 27 else 7)
    else:
        if os_name in ('xros',):
            ladder = {1: 6, 2: 7, 26: 8, 27: 9}
        else:  # ios, tvos (tvOS follows iOS numbering)
            ladder = {11: 0, 12: 1, 13: 2, 14: 3, 15: 4, 16: 5, 17: 6, 18: 7, 26: 8, 27: 9}
        air_minor = ladder.get(major, 9 if major > 27 else 0)
    lang = {0: (2, 0), 1: (2, 1), 2: (2, 2), 3: (2, 3), 4: (2, 4), 5: (3, 0), 6: (3, 1),
            7: (3, 2), 8: (4, 0), 9: (4, 1)}[air_minor]
    return (2, air_minor), lang

def parse_target(t):
    m = re.match(r'air64-apple-(ios|macosx|tvos|xros)(\d+)(?:\.(\d+))?(?:\.(\d+))?(-simulator)?$', t)
    if not m:
        raise SystemExit(f'unsupported target {t}')
    os_name = m.group(1)
    ver = (int(m.group(2)), int(m.group(3) or 0), int(m.group(4) or 0))
    return os_name, ver, bool(m.group(5))

# ---------------------------------------------------------------- 1. rewrite
VEC = r'(?:bool|char|uchar|short|ushort|int|uint|long|ulong|half|float)[234]'
def rewrite(src):
    # [[a, b(c)]] -> __attribute__((annotate("msl:a"), annotate("msl:b(c)")))
    def attr(m):
        body = m.group(1)
        parts, depth, cur = [], 0, ''
        for ch in body:
            if ch == ',' and depth == 0:
                parts.append(cur.strip()); cur = ''; continue
            depth += ch in '(<'
            depth -= ch in ')>'
            cur += ch
        parts.append(cur.strip())
        return ' '.join('__MSL_ATTR("msl:%s")' % p.replace('"', '\\"') for p in parts if p)
    src = re.sub(r'\[\[\s*((?:[^\[\]]|\[[^\[\]]*\])*?)\s*\]\]', attr, src)
    src = rewrite_function_constants(src)
    src = re.sub(r'\b(' + VEC + r')\s*\(', r'metal::__vec<\1>(', src)
    src = re.sub(r'(?<![\w.])(\d+\.\d*|\.\d+|\d+)(?:[eE][-+]?\d+)?[hH]\b',
                 lambda m: '((half)' + m.group(0)[:-1] + ')', src)
    return src

FC = re.compile(r'\bconstant\s+([\w:<>, ]+?)\s+(\w+)\s*__MSL_ATTR\("msl:function_constant\((\d+)\)"\)\s*;')
def rewrite_function_constants(src):
    """constant T x [[function_constant(i)]]; becomes an internal global copied at
    static-init time from an external initializer the IR pass turns into AIR's."""
    idx = {m.group(2): m.group(3) for m in FC.finditer(src)}
    FC_ORDER[:] = list(idx)
    src = FC.sub(lambda m: 'extern "C" __MSL_CONSTANT %s __msl_fcinit_%s_%s; static __MSL_CONSTANT %s %s = __msl_fcinit_%s_%s;'
                 % (m.group(1), m.group(3), m.group(2), m.group(1), m.group(2), m.group(3), m.group(2)), src)
    FC_CHECKED.update(re.findall(r'\bis_function_constant_defined\s*\(\s*(\w+)\s*\)', src))
    src = re.sub(r'\bis_function_constant_defined\s*\(\s*(\w+)\s*\)',
                 lambda m: 'metal::__air_is_function_constant_defined(&__msl_fcinit_%s_%s)' % (idx[m.group(1)], m.group(1)), src)
    return src

ITANIUM = {'bool': 'b', 'char': 'c', 'uchar': 'h', 'short': 's', 'ushort': 't', 'int': 'i', 'uint': 'j',
           'long': 'l', 'ulong': 'm', 'half': 'Dh', 'float': 'f'}
def itanium(t):
    m = re.match(r'(\w+?)([234])$', t)
    if m and m.group(1) in ITANIUM:
        return 'Dv%s_%s' % (m.group(2), ITANIUM[m.group(1)])
    return ITANIUM[t]

def function_constant_ir(ir):
    """Give function constants AIR's shape: initializer globals, static init, metadata."""
    entries = []
    for m in list(re.finditer(r'^@__msl_fcinit_(\d+)_(\w+) = external (?:dso_local )?addrspace\(2\) global (.*?), align (\d+)$', ir, re.M)):
        idx, name, lty, align = m.group(1), m.group(2), m.group(3), m.group(4)
        entries.append((idx, name, lty))
        ir = ir.replace(m.group(0), '@__msl_fcinit_%s_%s = linkonce_odr hidden addrspace(2) externally_initialized constant %s undef, section "air.fc_initializer", align %s' % (idx, name, lty, align))
    if entries:
        ir = re.sub(r'^(define internal (?:spir_func )?void @_GLOBAL__sub_I_[^{]*?)(\s*\{)$', lambda m: m.group(1) + ' section "air.static_init"' + m.group(2), ir, flags=re.M)
        for idx, name, lty in entries:
            ir = re.sub(r'^(@_ZL%d%s = internal (?:unnamed_addr )?addrspace\(2\) global (?:<[^>]*>|\[[^\]]*\]|\S+)) .*?(, align \d+)$' % (len(name), name),
                        r'\1 undef\2', ir, flags=re.M)
    return ir, entries
FC_TYPES = {}
FC_ORDER = []

# Constexpr samplers: the state word(s) Xcode writes (FORMAT.md, "Constexpr samplers").
SAMPLER_ENUMS = {
    'address': {'clamp_to_zero': 0, 'clamp_to_edge': 1, 'repeat': 2, 'mirrored_repeat': 3, 'clamp_to_border': 0},
    'filter': {'nearest': 0, 'linear': 1}, 'mip_filter': {'none': 0, 'nearest': 1, 'linear': 2},
    'coord': {'normalized': 0, 'pixel': 1},
    'compare_func': {'none': 0, 'less': 1, 'less_equal': 2, 'greater': 3, 'greater_equal': 4, 'equal': 5,
                     'not_equal': 6, 'always': 7, 'never': 8},
    'border_color': {'transparent_black': 0, 'opaque_black': 1, 'opaque_white': 2}}

def half_bits(x):
    return struct.unpack('<H', struct.pack('<e', x))[0]

def sampler_state(args):
    st = dict(s=1, t=1, r=1, mag=0, min=0, mip=0, coord=0, cmp=8, aniso=1, lodmin=0.0, lodmax=65504.0, border=0)
    for a in split_args(args):
        a = a.replace('metal::', '').strip()
        m = re.match(r'(\w+)::(\w+)$', a)
        if m:
            kind, val = m.groups()
            if kind == 'address':
                st['s'] = st['t'] = st['r'] = SAMPLER_ENUMS['address'][val]
            elif kind in ('s_address', 't_address', 'r_address'):
                st[kind[0]] = SAMPLER_ENUMS['address'][val]
            elif kind == 'filter':
                st['mag'] = st['min'] = SAMPLER_ENUMS['filter'][val]
            elif kind in ('mag_filter', 'min_filter'):
                st[kind[:3]] = SAMPLER_ENUMS['filter'][val]
            elif kind == 'mip_filter':
                st['mip'] = SAMPLER_ENUMS['mip_filter'][val]
            elif kind == 'coord':
                st['coord'] = SAMPLER_ENUMS['coord'][val]
            elif kind == 'compare_func':
                st['cmp'] = SAMPLER_ENUMS['compare_func'][val]
            elif kind == 'border_color':
                st['border'] = SAMPLER_ENUMS['border_color'][val]
            else:
                raise SystemExit('unsupported sampler argument ' + a)
            continue
        m = re.match(r'max_anisotropy\((\d+)\)$', a)
        if m:
            st['aniso'] = int(m.group(1)); continue
        m = re.match(r'lod_clamp\(([^,]+),([^)]+)\)$', a)
        if m:
            st['lodmin'], st['lodmax'] = (float(x.strip().rstrip('fF')) for x in m.groups()); continue
        raise SystemExit('unsupported sampler argument ' + a)
    return (st['s'] | st['t'] << 3 | st['r'] << 6 | st['mag'] << 9 | st['min'] << 11 | st['mip'] << 13
            | st['coord'] << 15 | st['cmp'] << 16 | (st['aniso'] - 1) << 20 | half_bits(st['lodmin']) << 24
            | half_bits(st['lodmax']) << 40 | st['border'] << 56)

SAMPLERS = []
def rewrite_samplers(src, air):
    """constexpr sampler s(args); refers to an address-space-2 state the IR pass defines."""
    decls = []
    def one(m):
        k = len(SAMPLERS)
        word = sampler_state(m.group(2))
        SAMPLERS.append(word)
        decls.append('extern "C" __attribute__((address_space(2))) unsigned long __msl_sampler_%d%s;' % (k, '[2]' if air >= (2, 7) else ''))
        return 'const metal::sampler %s = {(__MSL_CONSTANT _sampler_t *)&__msl_sampler_%d};' % (m.group(1), k)
    src = re.sub(r'\bconstexpr\s+sampler\s+(\w+)\s*\(((?:[^()]|\([^()]*\))*)\)\s*;', one, src)
    return '\n'.join(decls) + ('\n' if decls else '') + src

def sampler_ir(ir, air):
    for k, word in enumerate(SAMPLERS):
        if air >= (2, 7):
            init = '[2 x i64] [i64 %d, i64 0]' % word
        else:
            init = 'i64 %d' % ((word | 1 << 63) - (1 << 64))
        ir = re.sub(r'^@__msl_sampler_%d = external (?:dso_local )?addrspace\(2\) global .*$' % k,
                    '@__msl_sampler_%d = internal addrspace(2) constant %s, align 8' % (k, init), ir, flags=re.M)
    return ir

# ---------------------------------------------------------------- 2. AST
NODE = re.compile(r'^(?P<pre>[| `-]*)(?P<kind>[A-Za-z]+)(?P<rest>.*)$')
def ast_tree(text):
    root = {'kind': 'root', 'rest': '', 'kids': []}
    stack = [(-1, root)]
    for line in text.splitlines():
        m = NODE.match(line)
        if not m:
            continue
        depth = len(m.group('pre'))
        node = {'kind': m.group('kind'), 'rest': m.group('rest'), 'kids': []}
        while stack[-1][0] >= depth:
            stack.pop()
        stack[-1][1]['kids'].append(node)
        stack.append((depth, node))
    return root

def decl_name_type(rest):
    # ' 0x.. <range> col:N used name 'type':'desugared''
    m = re.search(r"(?:\s(?:col|line):\d+(?::\d+)?)\s(?:(?:used|referenced|invalid|implicit|constexpr|static|cinit|callinit|listinit) )*(\w*) ?'([^']*)'(?::'([^']*)')?", rest)
    if not m:
        return None, None, None
    return m.group(1), m.group(2), m.group(3) or m.group(2)

def annotations(node):
    return [re.search(r'"(.*)"$', k['rest']).group(1) for k in node['kids'] if k['kind'] == 'AnnotateAttr']

TYPEDEFS = {}
RECORDS = {}

def stage_functions(tree):
    out = []
    def walk(n, ns):
        for k in n['kids']:
            if k['kind'] in ('NamespaceDecl', 'TranslationUnitDecl'):
                nm = re.search(r' (\w+)$', k['rest']) if k['kind'] == 'NamespaceDecl' else None
                walk(k, ns + nm.group(1) + '::' if nm else ns)
            elif k['kind'] == 'VarDecl' and '__msl_fcinit_' in k['rest']:
                vn, vt, vd = decl_name_type(k['rest'])
                if vn:
                    FC_TYPES[re.match(r'__msl_fcinit_\d+_(\w+)', vn).group(1)] = vt
            elif k['kind'] == 'CXXRecordDecl' and ' definition' in k['rest']:
                rm = re.search(r' (?:struct|class) (\w+) definition', k['rest'])
                if rm:
                    fields = []
                    for f in k['kids']:
                        if f['kind'] == 'FieldDecl':
                            fn, ft, fd = decl_name_type(f['rest'])
                            fields.append(dict(name=fn, type=ft, desugared=fd, attrs=[a[4:] for a in annotations(f) if a.startswith('msl:')]))
                    RECORDS[ns + rm.group(1)] = fields
                    RECORDS.setdefault(rm.group(1), fields)
            elif k['kind'] in ('TypedefDecl', 'TypeAliasDecl'):
                tn, tt, td = decl_name_type(k['rest'])
                if tn:
                    TYPEDEFS[ns + tn] = td
                    TYPEDEFS.setdefault(tn, td)
            elif k['kind'] == 'LinkageSpecDecl':
                walk(k, ns)
            elif k['kind'] == 'FunctionDecl':
                ann = annotations(k)
                stage = [a.split(':', 1)[1] for a in ann if a.startswith('msl.stage:')]
                if not stage:
                    continue
                name, ftype, _ = decl_name_type(k['rest'])
                params = []
                for p in k['kids']:
                    if p['kind'] == 'ParmVarDecl':
                        pn, pt, pd = decl_name_type(p['rest'])
                        params.append(dict(name=pn, type=pt, desugared=pd,
                                           attrs=[a[4:] for a in annotations(p) if a.startswith('msl:')]))
                fattrs = [a[4:] for a in ann if a.startswith('msl:')]
                out.append(dict(name=name, stage=stage[0], type=ftype, params=params, attrs=fattrs))
    walk(tree, '')
    return out

# ---------------------------------------------------------------- record layouts
def record_layouts(text):
    """'*** Dumping AST Record Layout' blocks -> {name: dict(size, align, fields)}"""
    out = {}
    for block in text.split('*** Dumping AST Record Layout')[1:]:
        lines = block.strip().splitlines()
        head = re.match(r'\s*0 \| (?:struct|class|union) (.*)$', lines[0])
        if not head:
            continue
        name = head.group(1).strip()
        fields = []
        for l in lines[1:]:
            m = re.match(r'\s*(\d+) \|   (\S.*)$', l)  # direct members only (3 spaces)
            if m:
                off, decl = int(m.group(1)), m.group(2)
                dm = re.match(r'(.*?)\s*(\w+)$', decl)
                fields.append((off, dm.group(1).replace('struct ', '').replace('class ', '').strip(), dm.group(2)))
        sm = re.search(r'\[sizeof=(\d+), dsize=\d+, align=(\d+)', block)
        out[name] = dict(size=int(sm.group(1)), align=int(sm.group(2)), fields=fields)
    return out

# ---------------------------------------------------------------- types
SCALARS = {'bool': (1, 1), 'char': (1, 1), 'uchar': (1, 1), 'short': (2, 2), 'ushort': (2, 2),
           'int': (4, 4), 'uint': (4, 4), 'long': (8, 8), 'ulong': (8, 8), 'half': (2, 2), 'float': (4, 4)}
CANON = {'unsigned int': 'uint', 'unsigned char': 'uchar', 'unsigned short': 'ushort',
         'unsigned long': 'ulong', 'signed char': 'char', '_Float16': 'half', '_Bool': 'bool'}

def strip_qual(t):
    t = re.sub(r'\b(device|constant|threadgroup|thread|const|volatile|__MSL_DEVICE|__MSL_CONSTANT|__MSL_THREADGROUP)\b', '', t)
    t = re.sub(r'__attribute__\(\(address_space\(\d\)\)\)', '', t)
    t = t.replace('metal::', '')
    return re.sub(r'\s+', ' ', t).strip()

def msl_name(t):
    t = strip_qual(t)
    m = re.match(r'(?:metal::)?matrix<(\w[\w ]*), (\d), (\d)>$', t)
    if m:
        return '%s%sx%s' % (CANON.get(m.group(1), m.group(1)), m.group(2), m.group(3))
    m = re.match(r'(?:metal::)?__packed<(\w[\w ]*), (\d)>$', t)
    if m:
        return 'packed_%s%s' % (CANON.get(m.group(1), m.group(1)), m.group(2))
    return CANON.get(t, t)

def type_size(name, layouts):
    name = msl_name(name)
    if name in SCALARS:
        return SCALARS[name]
    m = re.match(r'(\w+?)([234])$', name)
    if m and m.group(1) in SCALARS:
        s, _ = SCALARS[m.group(1)]
        n = int(m.group(2)); n = 4 if n == 3 else n
        return s * n, s * n
    m = re.match(r'packed_(\w+?)([234])$', name)
    if m and m.group(1) in SCALARS:
        e = SCALARS[m.group(1)][0]
        return e * int(m.group(2)), e
    m = re.match(r'(\w+?)([234])x([234])$', name)
    if m:
        s, _ = SCALARS[m.group(1)]
        rows = int(m.group(3)); rows = 4 if rows == 3 else rows
        return s * rows * int(m.group(2)), s * rows
    m = re.match(r'(.*)\[(\d+)\]$', name)
    if m:
        s, a = type_size(m.group(1), layouts)
        return s * int(m.group(2)), a
    if name in layouts:
        return layouts[name]['size'], layouts[name]['align']
    raise KeyError(name)

# ---------------------------------------------------------------- metadata
class MD:
    def __init__(self):
        self.nodes = []
    def add(self, body):
        self.nodes.append(body)
        return '!%d' % (len(self.nodes) - 1)
    def text(self):
        return ''.join('!%d = %s\n' % (i, b) for i, b in enumerate(self.nodes))

def s(x):
    return '!"%s"' % x

BUILTINS = {'thread_position_in_grid', 'threads_per_grid', 'thread_position_in_threadgroup',
            'threads_per_threadgroup', 'threadgroup_position_in_grid', 'threadgroups_per_grid',
            'thread_index_in_threadgroup', 'thread_index_in_simdgroup', 'thread_index_in_quadgroup',
            'simdgroup_index_in_threadgroup', 'quadgroup_index_in_threadgroup', 'threads_per_simdgroup',
            'simdgroups_per_threadgroup', 'quadgroups_per_threadgroup', 'dispatch_threads_per_threadgroup',
            'grid_origin', 'grid_size', 'vertex_id', 'instance_id', 'base_vertex', 'base_instance',
            'front_facing', 'sample_id', 'point_coord', 'sample_mask', 'primitive_id'}

def struct_info(md, tname, layouts):
    lay = layouts[tname]
    parts = []
    for off, ftype, fname in lay['fields']:
        count = 0
        base = ftype
        m = re.match(r'(.*)\[(\d+)\]$', ftype)
        if m:
            base, count = m.group(1).strip(), int(m.group(2))
        bname = msl_name(base)
        size, _ = type_size(bname, layouts)
        if bname in layouts and fname != '__s':
            parts.append('!"air.struct_type_info", ' + struct_info(md, bname, layouts))
        parts.append('i32 %d, i32 %d, i32 %d, %s, %s' % (off, size, count, s(bname), s(fname)))
    return md.add('!{' + ', '.join(parts) + '}')

def arg_descriptor(md, i, p, layouts, air, used):
    attrs = p['attrs']
    a0 = attrs[0] if attrs else ''
    kind = re.match(r'(\w+)', a0).group(1) if a0 else ''
    t = p['type']
    tail = ['!"air.arg_type_name"', None, '!"air.arg_name"', s(p['name'])]
    sname = argbuf_struct(p)
    if sname:
        slot = int(re.search(r'\((\d+)\)', a0).group(1))
        aspace = 2 if re.search(r'\bconstant\b', t) else 1
        size, align = layouts[sname]['size'], layouts[sname]['align']
        f = ['i32 %d' % i, s('air.indirect_buffer'), '!"air.buffer_size"', 'i32 %d' % size, '!"air.location_index"', 'i32 %d' % slot, 'i32 1',
             s('air.read' if aspace == 2 else 'air.read_write')]
        if air >= (2, 5):
            f += ['!"air.address_space"', 'i32 %d' % aspace]
        f += ['!"air.struct_type_info"', indirect_info(md, sname, layouts, air), '!"air.arg_type_size"', 'i32 %d' % size,
              '!"air.arg_type_align_size"', 'i32 %d' % align, '!"air.arg_type_name"', s(sname), '!"air.arg_name"', s(p['name'])]
    elif kind == 'buffer' or kind == 'threadgroup':
        slot = int(re.search(r'\((\d+)\)', a0).group(1))
        ref = t.rstrip().endswith('&')
        aspace = 3 if kind == 'threadgroup' else (2 if re.search(r'\bconstant\b', t) else 1)
        pointee = re.sub(r'\s*[*&]\s*$', '', t)
        readonly = aspace == 2 or re.search(r'\bconst\b', pointee)
        tname = msl_name(pointee)
        am = re.match(r'(?:metal::)?_atomic<(\w[\w ]*)>', strip_qual(re.sub(r'\s*[*&]\s*$', '', resolve(p['desugared']))))
        if am:
            elem = msl_name(am.group(1))
            size, align = SCALARS[elem]
            tname = 'metal::_atomic'
            layouts = dict(layouts); layouts[tname] = dict(size=size, align=align, fields=[(0, elem, '__s')])
        else:
            size, align = type_size(tname, layouts)
        f = ['i32 %d' % i, s('air.buffer')]
        if ref and aspace == 2:
            f += ['!"air.buffer_size"', 'i32 %d' % size]
        f += ['!"air.location_index"', 'i32 %d' % slot, 'i32 1', s('air.read' if readonly else 'air.read_write')]
        if air >= (2, 5):
            f += ['!"air.address_space"', 'i32 %d' % aspace]
        if tname in layouts:
            f += ['!"air.struct_type_info"', struct_info(md, tname, layouts)]
        f += ['!"air.arg_type_size"', 'i32 %d' % size, '!"air.arg_type_align_size"', 'i32 %d' % align]
        f += ['!"air.arg_type_name"', s(tname), '!"air.arg_name"', s(p['name'])]
    elif kind == 'texture':
        slot = int(re.search(r'\((\d+)\)', a0).group(1))
        tname, acc = texture_name(resolve(p['desugared']))
        f = ['i32 %d' % i, s('air.texture'), '!"air.location_index"', 'i32 %d' % slot, 'i32 1', s('air.' + acc),
             '!"air.arg_type_name"', s(tname), '!"air.arg_name"', s(p['name'])]
    elif kind == 'sampler':
        slot = int(re.search(r'\((\d+)\)', a0).group(1))
        f = ['i32 %d' % i, s('air.sampler'), '!"air.location_index"', 'i32 %d' % slot, 'i32 1',
             '!"air.arg_type_name"', s('sampler'), '!"air.arg_name"', s(p['name'])]
    elif kind == 'position':
        f = ['i32 %d' % i, s('air.position'), s('air.center'), s('air.no_perspective'), '!"air.arg_type_name"', s(msl_name(t)), '!"air.arg_name"', s(p['name'])]
    elif kind == 'color':
        n = re.search(r'\((\d+)\)', a0).group(1)
        f = ['i32 %d' % i, s('air.render_target'), 'i32 %s' % n, '!"air.arg_type_name"', s(msl_name(t)), '!"air.arg_name"', s(p['name'])]
    elif kind in BUILTINS:
        f = ['i32 %d' % i, s('air.' + kind), '!"air.arg_type_name"', s(msl_name(t)), '!"air.arg_name"', s(p['name'])]
    else:
        raise SystemExit(f'unsupported argument {p}')
    if not used and air >= (2, 5):
        f.append(s('air.arg_unused'))
    return md.add('!{' + ', '.join(f) + '}')

# ---------------------------------------------------------------- argument buffers
def pointee(t):
    return re.sub(r'\s*[*&]\s*$', '', t)

def argbuf_struct(p):
    """The struct name if p is a [[buffer(n)]] argument buffer (members with [[id(n)]])."""
    if not p['attrs'] or not p['attrs'][0].startswith('buffer('):
        return None
    name = msl_name(pointee(p['type']))
    fields = RECORDS.get(name)
    if fields and any(a.startswith('id(') for f in fields for a in f['attrs']):
        return name
    return None

def handle_of(t):
    m = re.match(r'(?:metal::)?(\w+)', strip_qual(t))
    return m.group(1) if m and m.group(1) in HANDLES else None

def copy_expr(t, src):
    """An aggregate initializer copying src (in device or constant memory) into thread memory."""
    t = resolve(t).strip()
    if handle_of(t):
        return '%s{%s.%s}' % (strip_qual(t) if not t.startswith('metal::') else t, src, '__s' if handle_of(t) == 'sampler' else '__t')
    m = re.match(r'(?:metal::)?array<(.*),\s*(\d+)>$', strip_qual(t))
    if m:
        return 'metal::array<%s, %s>{{%s}}' % (m.group(1), m.group(2), ', '.join(copy_expr(m.group(1), '%s[%d]' % (src, k)) for k in range(int(m.group(2)))))
    m = re.match(r'(.*)\[(\d+)\]$', t)
    if m:
        return '{%s}' % ', '.join(copy_expr(m.group(1), '%s[%d]' % (src, k)) for k in range(int(m.group(2))))
    name = msl_name(t)
    if name in RECORDS and not t.rstrip().endswith(('*', '&')):
        return '%s{%s}' % (name, ', '.join(copy_expr(f['desugared'], '%s.%s' % (src, f['name'])) for f in RECORDS[name]))
    return src

def indirect_info(md, sname, layouts, air):
    lay = {f[2]: f[0] for f in layouts[sname]['fields']}
    parts = []
    for k, f in enumerate(RECORDS[sname]):
        ft, fd = f['type'], resolve(f['desugared'])
        idm = [a for a in f['attrs'] if a.startswith('id(')]
        slot = int(re.search(r'\((\d+)\)', idm[0]).group(1)) if idm else k
        h = handle_of(fd)
        am = re.match(r'(?:metal::)?array<(.*),\s*(\d+)>$', strip_qual(fd))
        if h or (am and handle_of(am.group(1))):
            inner = am.group(1) if am else fd
            count = int(am.group(2)) if am else 1
            if handle_of(inner) == 'sampler':
                tname, desc = 'sampler', [s('air.sampler'), '!"air.location_index"', 'i32 %d' % slot, 'i32 %d' % count]
            else:
                tname, acc = texture_name(inner)
                desc = [s('air.texture'), '!"air.location_index"', 'i32 %d' % slot, 'i32 %d' % count, s('air.' + acc)]
            if am:
                tname = 'array<%s, %d>' % (tname, count)
            size = 8 * count
            d = md.add('!{' + ', '.join(['i32 %d' % k] + desc + ['!"air.arg_type_name"', s(tname), '!"air.arg_name"', s(f['name'])]) + '}')
        elif ft.rstrip().endswith(('*', '&')):
            pt = msl_name(pointee(ft))
            aspace = 2 if re.search(r'\bconstant\b', ft) else 1
            ps, pa = type_size(pt, layouts)
            desc = ['i32 %d' % k, s('air.buffer')]
            if ft.rstrip().endswith('&') and aspace == 2:
                desc += ['!"air.buffer_size"', 'i32 %d' % ps]
            desc += ['!"air.location_index"', 'i32 %d' % slot, 'i32 1',
                     s('air.read' if aspace == 2 or re.search(r'\bconst\b', pointee(ft)) else 'air.read_write')]
            if air >= (2, 5):
                desc += ['!"air.address_space"', 'i32 %d' % aspace]
            if pt in layouts:
                desc += ['!"air.struct_type_info"', struct_info(md, pt, layouts)]
            desc += ['!"air.arg_type_size"', 'i32 %d' % ps, '!"air.arg_type_align_size"', 'i32 %d' % pa,
                     '!"air.arg_type_name"', s(pt), '!"air.arg_name"', s(f['name'])]
            d = md.add('!{' + ', '.join(desc) + '}')
            tname, size = pt, 8
        else:
            tname = msl_name(ft)
            size = type_size(tname, layouts)[0]
            d = md.add('!{i32 %d, !"air.indirect_constant", !"air.location_index", i32 %d, i32 1, !"air.arg_type_name", %s, !"air.arg_name", %s}'
                       % (k, slot, s(tname), s(f['name'])))
        parts.append('i32 %d, i32 %d, i32 0, %s, %s, !"air.indirect_argument", %s' % (lay[f['name']], size, s(tname), s(f['name']), d))
    return md.add('!{' + ', '.join(parts) + '}')

def texture_name(t):
    m = re.match(r'(?:metal::)?(\w+)<(.*)>$', strip_qual(t))
    targs = [x.strip() for x in m.group(2).split(',')]
    acc = targs[1].split('::')[-1] if len(targs) > 1 else ('write' if m.group(1) == 'texture_buffer' else 'sample')
    return '%s<%s, %s>' % (m.group(1), msl_name(targs[0]), acc), acc

def retype_argbufs(code, funcs):
    """The user's function takes argument buffers in thread memory (see entry_points)."""
    for f in funcs:
        for p in f['params']:
            sname = argbuf_struct(p)
            if sname:
                code = re.sub(r'(?:\bconstant|\bdevice)\s+(?:const\s+)?%s\s*([*&])\s*%s\b' % (re.escape(sname), re.escape(p['name'])),
                              lambda m: 'const %s %s%s' % (sname, m.group(1), p['name']), code)
    return code

# ---------------------------------------------------------------- vertex and fragment I/O
INTERP = {'flat': ['air.flat'], 'center_perspective': ['air.center', 'air.perspective'],
          'center_no_perspective': ['air.center', 'air.no_perspective'],
          'centroid_perspective': ['air.centroid', 'air.perspective'],
          'centroid_no_perspective': ['air.centroid', 'air.no_perspective'],
          'sample_perspective': ['air.sample', 'air.perspective'], 'sample_no_perspective': ['air.sample', 'air.no_perspective']}

def interp_of(attrs):
    for a in attrs:
        if a in INTERP:
            return INTERP[a]
    return None

def generated_key(fld):
    t = msl_name(fld['type'])
    return 'generated(%d%s%s)' % (len(fld['name']), fld['name'], itanium(t))

def output_descriptors(md, f, air):
    rf = ret_fields(f)
    out = []
    if rf is None:
        t = msl_name(ret_type(f))
        if f['stage'] == 'fragment':
            out.append(md.add('!{!"air.render_target", i32 0, i32 0, !"air.arg_type_name", %s}' % s(t)))
        return md.add('!{' + ', '.join(out) + '}')
    for fld in rf:
        t, a = msl_name(fld['type']), fld['attrs']
        tail = '!"air.arg_type_name", %s, !"air.arg_name", %s' % (s(t), s(fld['name']))
        cm = [re.match(r'color\((\d+)\)', x) for x in a]
        cm = [x for x in cm if x]
        dm = [re.match(r'depth\((\w+)\)', x) for x in a]
        dm = [x for x in dm if x]
        if 'position' in a:
            out.append(md.add('!{!"air.position", %s}' % tail))
        elif 'point_size' in a:
            out.append(md.add('!{!"air.point_size", %s}' % tail))
        elif cm:
            idx = [re.match(r'index\((\d+)\)', x) for x in a]
            idx = [x for x in idx if x]
            out.append(md.add('!{!"air.render_target", i32 %s, i32 %s, %s}' % (cm[0].group(1), idx[0].group(1) if idx else '0', tail)))
        elif dm:
            out.append(md.add('!{!"air.depth", !"air.depth_qualifier", %s, %s}' % (s('air.' + dm[0].group(1)), tail)))
        else:
            q = interp_of(a) if air <= (2, 1) else None
            out.append(md.add('!{!"air.vertex_output", %s, %s%s}' % (s(generated_key(fld)), ', '.join(s(x) for x in (q or [])) + (', ' if q else ''), tail)))
    return md.add('!{' + ', '.join(out) + '}')

AIR_NOW = [(2, 9)]
def stage_in_descriptor(md, i, fld, stage, used):
    t, a = msl_name(fld['type']), fld['attrs']
    tail = ['!"air.arg_type_name"', s(t), '!"air.arg_name"', s(fld['name'])]
    if stage == 'vertex':
        am = [re.match(r'attribute\((\d+)\)', x) for x in a]
        am = [x for x in am if x]
        f = ['i32 %d' % i, s('air.vertex_input'), '!"air.location_index"', 'i32 %s' % am[0].group(1), 'i32 1']
        if AIR_NOW[0] <= (2, 1):  # 2.1 has it, 2.3 doesn't; 2.2 unprobed
            f.append(s('generated(__air_placeholder__)'))
        f += tail
    elif 'position' in a:
        f = ['i32 %d' % i, s('air.position'), s('air.center'), s('air.no_perspective')] + tail
    else:
        f = ['i32 %d' % i, s('air.fragment_input'), s(generated_key(fld))] + [s(x) for x in (interp_of(a) or INTERP['center_perspective'])] + tail
    if not used and AIR_NOW[0] >= (2, 5):
        f.append(s('air.arg_unused'))
    return md.add('!{' + ', '.join(f) + '}')

# ---------------------------------------------------------------- 3. entry points
HANDLES = {'texture1d': '__MSL_DEVICE _texture_1d_t *', 'texture2d': '__MSL_DEVICE _texture_2d_t *',
           'texture2d_array': '__MSL_DEVICE _texture_2d_array_t *', 'texture3d': '__MSL_DEVICE _texture_3d_t *',
           'texturecube': '__MSL_DEVICE _texture_cube_t *', 'depth2d': '__MSL_DEVICE _depth_2d_t *',
           'texture_buffer': '__MSL_DEVICE _texture_buffer_1d_t *', 'sampler': '__MSL_CONSTANT _sampler_t *'}

def resolve(t):
    """Expand known typedef names inside a type string (one level at a time)."""
    for _ in range(8):
        new = re.sub(r'((?:\w+::)*\w+)', lambda m: TYPEDEFS.get(m.group(1), m.group(1)) if m.group(1) in TYPEDEFS and not re.match(r'(?:metal::)?(?:u?(?:char|short|int|long)|half|float|bool)[234]?$', m.group(1)) else m.group(1), t)
        if new == t:
            return t
        t = new
    return t

def handle_kind(p):
    m = re.match(r'(?:metal::)?(\w+)', strip_qual(p['desugared']))
    return m.group(1) if m and m.group(1) in HANDLES else None

AS_WORDS = {'device': '__MSL_DEVICE', 'constant': '__MSL_CONSTANT', 'threadgroup': '__MSL_THREADGROUP'}
def explicit_as(t):
    return re.sub(r'\b(device|constant|threadgroup)\b', lambda m: AS_WORDS[m.group(1)], t)
def flat(t):
    return re.sub(r'\s+', ' ', re.sub(r'\b(device|constant|threadgroup|thread)\b', '', t)).strip()

def stage_in_param(f):
    for k, p in enumerate(f['params']):
        if 'stage_in' in p['attrs']:
            return k
    return None

def ret_type(f):
    return f['type'].split('(')[0].strip()

def ret_fields(f):
    """The return struct's fields (vertex and fragment), or None for a plain value."""
    r = msl_name(ret_type(f))
    return RECORDS.get(r) if f['stage'] != 'kernel' and r in RECORDS else None

def flat_params(f):
    """AIR's parameter list: a [[stage_in]] struct's fields replace the struct."""
    out = []
    for k, p in enumerate(f['params']):
        if 'stage_in' in p['attrs']:
            for fld in RECORDS[msl_name(p['type'])]:
                out.append(dict(fld, stage_in=True, struct=msl_name(p['type'])))
        else:
            out.append(p)
    return out

def entry_points(funcs):
    """extern "C" functions with AIR's signatures, calling the user's (flat) function."""
    out = ['namespace __air_entry {']
    for f in funcs:
        ps, args, pre = [], [], []
        for p in f['params']:
            h = handle_kind(p)
            t = p['type'].strip()
            if 'stage_in' in p['attrs']:
                sname = msl_name(t)
                fl = RECORDS[sname]
                for fld in fl:
                    ps.append('%s __si_%s' % (fld['type'], fld['name']))
                args.append('%s{%s}' % (sname, ', '.join('__si_' + fld['name'] for fld in fl)))
            elif h:
                ps.append('%s%s' % (HANDLES[h], p['name'])); args.append('%s{%s}' % (p['desugared'], p['name']))
            elif t.endswith('*'):
                ps.append('%s %s' % (explicit_as(t), p['name'])); args.append('(%s)%s' % (flat(t), p['name']))
            elif t.endswith('&'):
                ps.append('%s %s' % (explicit_as(t), p['name'])); args.append('*(%s *)&%s' % (flat(t[:-1]), p['name']))
            else:
                ps.append('%s %s' % (t, p['name'])); args.append(p['name'])
        ret = ret_type(f)
        rf = ret_fields(f)
        call = '::%s(%s)' % (f['name'], ', '.join(args))
        if rf:
            out.append('struct __attribute__((packed)) __ret_%s { %s };' % (f['name'], ' '.join('%s f%d;' % (fld['type'], k) for k, fld in enumerate(rf))))
            body = '%s __o = %s; __ret_%s __r; %s return __r;' % (ret, call, f['name'], ' '.join('__r.f%d = __o.%s;' % (k, fld['name']) for k, fld in enumerate(rf)))
            out.append('extern "C" __ret_%s %s(%s) { %s }' % (f['name'], f['name'], ', '.join(ps), body))
        else:
            out.append('extern "C" %s %s(%s) { return %s; }' % (ret, f['name'], ', '.join(ps), call))
    out.append('}')
    return '\n'.join(out) + '\n'

def direct_struct_returns(ir, funcs):
    """spir64 returns the packed output struct through sret; AIR returns it directly.
    Runs on instnamer'd IR, so dropping the sret parameter renumbers nothing."""
    for f in funcs:
        if not ret_fields(f):
            continue
        m = re.search(r'^define (?:dso_local )?(?:spir_func )?void @%s\((%%"?struct\.[^*]*__ret_%s"?)\* noalias sret\([^)]*\) align (\d+) (%%[\w.]+)(?:, )?(.*)$' % (re.escape(f['name']), re.escape(f['name'])), ir, re.M)
        if not m:
            raise SystemExit('no sret entry for ' + f['name'])
        sty, align, sret, rest = m.groups()
        lit = re.search(r'^%s = type <?(\{.*\})>?$' % re.escape(sty), ir, re.M).group(1)
        lit = '<' + lit + '>'
        start, line_end = m.start(), ir.index('\n', m.start())
        end = ir.index('\n}\n', start)
        body = ir[line_end:end]
        body = re.sub(r'^(\s*)ret void$', lambda r: '%s%%__rv.p = bitcast %s* %s to %s*\n%s%%__rv = load %s, %s* %%__rv.p, align %s\n%sret %s %%__rv'
                      % (r.group(1), sty, sret, lit, r.group(1), lit, lit, align, r.group(1), lit), body, flags=re.M)
        alloca = '\n  %s = alloca %s, align %s' % (sret, sty, align)
        lm = re.match(r'\n([\w.]+:[^\n]*)', body)
        body = ('\n' + lm.group(1) + alloca + body[lm.end():]) if lm else (alloca + body)
        ir = ir[:start] + 'define %s @%s(%s' % (lit, f['name'], rest) + body + ir[end:]
    return ir

# ---------------------------------------------------------------- 5. IR -> AIR
def to_air(ir, funcs, layouts, target, src_path, fcs=()):
    os_name, ver, sim = parse_target(target)
    air, lang = air_versions(os_name, ver)
    AIR_NOW[0] = air
    tstr = '%s%d.%d.%d%s' % (os_name, ver[0], ver[1], ver[2], '-simulator' if sim else '')
    ir = re.sub(r'target datalayout = ".*"', 'target datalayout = "%s"' % DATALAYOUT, ir)
    ir = re.sub(r'target triple = ".*"', 'target triple = "air64_v2%d-apple-%s"' % (air[1], tstr), ir)
    ir = ir.replace('dso_local ', '').replace('spir_func ', '').replace('spir_kernel ', '')
    # drop clang's own named metadata, keep the numbered nodes (TBAA etc.) renumbered after ours
    ir = re.sub(r'^!(llvm\.module\.flags|llvm\.ident|opencl\.\w+) = .*\n', '', ir, flags=re.M)
    md = MD()
    flags = [md.add('!{i32 2, !"SDK Version", [2 x i32] [i32 27, i32 2]}'),
             md.add('!{i32 1, !"wchar_size", i32 4}'),
             md.add('!{i32 7, !"frame-pointer", i32 2}')]
    if air >= (2, 3):
        for k, v in (('device_buffers', 31), ('constant_buffers', 31), ('threadgroup_buffers', 31),
                     ('textures', 128), ('read_write_textures', 8), ('samplers', 16)):
            flags.append(md.add('!{i32 7, !"air.max_%s", i32 %d}' % (k, v)))
    stages = {'kernel': [], 'vertex': [], 'fragment': []}
    for f in funcs:
        sig, body = find_define(ir, f['name'])
        params = flat_params(f)
        pnames = [a.rsplit(' ', 1)[-1] for a in split_args(sig)]
        used = [re.search(r'%s\b' % re.escape(n), body) is not None for n in pnames]
        descs = [stage_in_descriptor(md, k, p, f['stage'], used[k]) if p.get('stage_in') else arg_descriptor(md, k, p, layouts, air, used[k])
                 for k, p in enumerate(params)]
        f['used'] = used
        fty = re.sub(r'^(?:(?:dso_local|local_unnamed_addr|noundef|zeroext|signext) )*', '', re.search(r'^define ([^@]*?) ?@%s\(' % re.escape(f['name']), ir, re.M).group(1))
        argtys = ', '.join(re.sub(r'\s+(?:noalias|nocapture|noundef|readonly|writeonly|readnone|nonnull|zeroext|signext|inreg|immarg|returned|"[^"]*"|align \d+|dereferenceable(?:_or_null)?\(\d+\))', '', a).rsplit(' %', 1)[0]
                           for a in split_args(sig))
        extra = []
        for a in f['attrs']:
            m2 = re.match(r'max_total_threads_per_threadgroup\((\d+)\)', a)
            if m2:
                extra.append(md.add('!{!"air.max_work_group_size", i32 %s}' % m2.group(1)))
        node = '!{%s (%s)* @%s, %s, %s%s}' % (fty, argtys, f['name'], output_descriptors(md, f, air), md.add('!{' + ', '.join(descs) + '}'),
                                              ''.join(', ' + e for e in extra))
        stages[f['stage']].append(md.add(node))
        # buffers get "air-buffer-no-alias"
        args = split_args(sig)
        for k, p in enumerate(f['params']):
            if p['attrs'] and re.match(r'buffer\(', p['attrs'][0]) and '"air-buffer-no-alias"' not in args[k]:
                args[k] = re.sub(r' (%\d+)$', r' "air-buffer-no-alias" \1', args[k])
        ir = ir.replace('@%s(%s)' % (f['name'], sig), '@%s(%s)' % (f['name'], ', '.join(args)), 1)
    fcnodes = []
    for idx, name, lty in sorted(fcs, key=lambda e: FC_ORDER.index(e[1])):
        g = '__msl_fcinit_%s_%s' % (idx, name)
        if not re.search(r'@%s\b' % g, ir.split('\n', 1)[1].replace('@%s = ' % g, '', 1)):
            ir = re.sub(r'^@%s = .*\n' % g, '', ir, flags=re.M)
            continue
        mty = msl_name(FC_TYPES.get(name, ''))
        new = '_Z%d%s.MTL_FC_INIT_%s_%s' % (len(name), name, idx, itanium(mty))
        ir = re.sub(r'@%s\b' % g, '@' + new, ir)
        FC_GLOBALS[name] = new
        fcnodes.append(md.add('!{%s addrspace(2)* @%s, %s, %s, i32 %s}' % (lty, new, s(mty), s(name), idx)))
    snodes = []
    live = [k for k in range(len(SAMPLERS)) if re.search(r'@__msl_sampler_%d\b' % k, ir)]
    for n, k in enumerate(live):
        name = '__air_sampler_state' + ('' if n == 0 else '.%d' % n)
        ir = re.sub(r'@__msl_sampler_%d\b' % k, '@' + name, ir)
        ty = re.search(r'^@%s = internal addrspace\(2\) constant (\[2 x i64\]|i64) ' % re.escape(name), ir, re.M).group(1)
        snodes.append(md.add('!{!"air.sampler_state", %s addrspace(2)* @%s}' % (ty, name)))
    opts = [md.add('!{!"air.compile.denorms_disable"}'), md.add('!{!"air.compile.fast_math_enable"}'),
            md.add('!{!"air.compile.framebuffer_fetch_enable"}')]
    ident = md.add('!{!"Apple metal version 32023.922 (metalfe-32023.922.4)"}')
    vers = md.add('!{i32 2, i32 %d, i32 0}' % air[1])
    langv = md.add('!{!"Metal", i32 %d, i32 %d, i32 0}' % lang)
    srcn = md.add('!{%s}' % s(src_path))
    # renumber clang's nodes past ours
    base = len(md.nodes)
    ir = re.sub(r'!(\d+)\b', lambda m: '!%d' % (int(m.group(1)) + base), ir)
    named = ['!llvm.module.flags = !{%s}' % ', '.join(flags)]
    for st in ('kernel', 'vertex', 'fragment'):
        if stages[st]:
            named.append('!air.%s = !{%s}' % (st, ', '.join(stages[st])))
    named += ['!air.compile_options = !{%s}' % ', '.join(opts)]
    if fcnodes:
        named.append('!air.function_constants = !{%s}' % ', '.join(fcnodes))
    if snodes:
        named.append('!air.sampler_states = !{%s}' % ', '.join(snodes))
    named += [ '!llvm.ident = !{%s}' % ident,
              '!air.version = !{%s}' % vers, '!air.language_version = !{%s}' % langv,
              '!air.source_file_name = !{%s}' % srcn]
    return ir.rstrip() + '\n\n' + '\n'.join(named) + '\n\n' + md.text(), air, lang, (os_name, ver, sim)

# AIR functions return small structs directly; spir64 lowers them to sret.
RETURNS = {'__ret_v4f32': '{ <4 x float>, i8 }', '__ret_v4f16': '{ <4 x half>, i8 }', '__ret_f32': '{ float, i8 }'}

def direct_returns(ir):
    n = [0]
    def ty(cname):
        return RETURNS[re.search(r'(__ret_\w+)', cname).group(1)]
    def decl(m):
        return 'declare %s %s(' % (ty(m.group(2)), m.group(1))
    ir = re.sub(r'declare (?:dso_local )?(?:spir_func )?void (@"?air\.[\w.]+"?)\((%"?struct\.[\w:.]*__ret_\w+"?\*) sret\([^)]*\)(?: align \d+)?(?:, )?', decl, ir)
    def call(m):
        indent, fn, ptrty, ptr, rest = m.groups()
        n[0] += 1
        t, k = ty(ptrty), n[0]
        return ('%s%%__ret%d = call %s %s(%s\n' % (indent, k, t, fn, rest)
                + '%s%%__retp%d = bitcast %s %s to %s*\n' % (indent, k, ptrty, ptr, t)
                + '%sstore %s %%__ret%d, %s* %%__retp%d' % (indent, t, k, t, k))
    ir = re.sub(r'^(\s*)(?:tail )?call (?:spir_func )?void (@"?air\.[\w.]+"?)\((%"?struct\.[\w:.]*__ret_\w+"?\*) sret\([^)]*\)(?: align \d+)? (%[\w.]+)(?:, )?(.*)$',
                call, ir, flags=re.M)
    return ir

def prepare_flat(ir, entries):
    """Before opt: inline everything into the entry points (address spaces are
    inferred within one function), and put program-scope constants in
    address space 2."""
    ir = re.sub(r'^(define (?!.*@(?:%s)\()[^{]*\)[^{#]*)(#\d+)?\s*\{$' % '|'.join(map(re.escape, entries)),
                lambda m: m.group(1) + 'alwaysinline ' + (m.group(2) or '') + ' {', ir, flags=re.M)
    for m in list(re.finditer(r'^@([\w.$]+) = ((?:internal |private |dso_local )*)((?:unnamed_addr |local_unnamed_addr )*)(constant|global) (.*?), align (\d+)$', ir, re.M)):
        name, link, ua, kind, rest, align = m.groups()
        if name.startswith(('llvm.', '__msl_')) or 'addrspace(' in m.group(0).split('=')[1].split(kind)[0]:
            continue
        ty = global_type(rest)
        ir = ir.replace(m.group(0), '@%s = %s%saddrspace(2) %s %s, align %s' % (name, link, ua, kind, rest, align))
        ir = re.sub(r'%s\* @%s\b' % (re.escape(ty), re.escape(name)), '%s* addrspacecast (%s addrspace(2)* @%s to %s*)' % (ty, ty, name, ty), ir)
    return ir

def global_type(rest):
    depth = 0
    for i, ch in enumerate(rest):
        depth += ch in '<[{('
        depth -= ch in '>]})'
        if ch == ' ' and depth == 0:
            return rest[:i]
    return rest

def address_space_of(ir, v, depth=0):
    """Follow GEPs, bitcasts and casts back to the address space a flat pointer came from."""
    if depth > 32 or not v.startswith('%'):
        return None
    m = re.search(r'^\s*%s = (.*)$' % re.escape(v), ir, re.M)
    if not m:
        return None
    rhs = m.group(1)
    c = re.match(r'addrspacecast .*? addrspace\((\d)\)\* (%[\w.]+) to ', rhs)
    if c:
        return c.group(1)
    g = re.match(r'getelementptr (?:inbounds )?.*?, .*?\* (%[\w.]+)', rhs) or re.match(r'bitcast .*?\* (%[\w.]+) to ', rhs)
    if g:
        return address_space_of(ir, g.group(1), depth + 1)
    ph = re.match(r'(?:phi|select) .*?(%[\w.]+)', rhs)
    if ph:
        return address_space_of(ir, ph.group(1), depth + 1)
    return None

def resolve_atomics(ir):
    """__msl_atomic_<op>_<u|s>_i32(i32* p, v, order) -> air.atomic.<global|local>.<op>.<u|s>.i32 on p's address space."""
    n = [0]
    def call(m):
        indent, res, op, sg, ptr, val, order = m.groups()
        space = address_space_of(ir, ptr)
        if space not in ('1', '3'):
            raise SystemExit('atomic on a pointer of unknown address space: %s' % ptr)
        kind, scope = {'1': ('global', 2), '3': ('local', 1)}[space]
        n[0] += 1
        return ('%s%%__atomp%d = addrspacecast i32* %s to i32 addrspace(%s)*\n' % (indent, n[0], ptr, space)
                + '%s%s = call i32 @air.atomic.%s.%s.%s.i32(i32 addrspace(%s)* %%__atomp%d, i32 %s, i32 %s, i32 %d, i32 0, i1 false)'
                % (indent, res, kind, op, sg, space, n[0], val, order, scope))
    out = re.sub(r'^(\s*)(%[\w.]+) = (?:tail )?call (?:spir_func )?(?:noundef )?i32 @__msl_atomic_(\w+?)_([us])_i32\(i32\* (?:noundef )?(%[\w.]+), i32 (?:noundef )?([^,]+), i32 (?:noundef )?([^)]+)\)(?: #\d+)?$', call, ir, flags=re.M)
    decls = set(re.findall(r'@(air\.atomic\.\w+\.\w+\.[us]\.i32)\(i32 addrspace\((\d)\)', out))
    out = re.sub(r'^declare [^\n]*@__msl_atomic_\w+\([^\n]*\n', '', out, flags=re.M)
    for name, space in sorted(decls):
        out += '\ndeclare i32 @%s(i32 addrspace(%s)*, i32, i32, i32, i32, i1)\n' % (name, space)
    return out

def find_define(ir, name):
    m = re.search(r'^define [^@]*@%s\(' % re.escape(name), ir, re.M)
    i = m.end(); depth = 1; j = i
    while depth:
        depth += {'(': 1, ')': -1}.get(ir[j], 0); j += 1
    sig = ir[i:j - 1]
    end = ir.index('\n}\n', j)
    return sig, ir[ir.index('{', j):end]

def split_args(sig):
    out, depth, cur = [], 0, ''
    for ch in sig:
        if ch == ',' and depth == 0:
            out.append(cur.strip()); cur = ''; continue
        depth += ch in '([{<'
        depth -= ch in ')]}>'
        cur += ch
    if cur.strip():
        out.append(cur.strip())
    return out

# ---------------------------------------------------------------- public metadata
# MTLDataType (Metal's public enum)
def mtl_data_type(t):
    base = {'float': 3, 'half': 16, 'int': 29, 'uint': 33, 'short': 37, 'ushort': 41, 'char': 45,
            'uchar': 49, 'bool': 53, 'long': 81, 'ulong': 85}
    m = re.match(r'([a-z]+?)([234]?)$', t)
    return base[m.group(1)] + (int(m.group(2)) - 1 if m.group(2) else 0)

def tagged(tag, data):
    return tag + struct.pack('<H', len(data)) + data

def public_md(ir, fname, fcs_defined_checked, f=None):
    _, body = find_define(ir, fname)
    tags = b''
    if f is not None and f['stage'] == 'vertex':
        attrs = [(k, p) for k, p in enumerate(flat_params(f)) if p.get('stage_in')]
        if attrs:
            vatt, vaty = struct.pack('<H', len(attrs)), struct.pack('<H', len(attrs))
            for k, p in attrs:
                loc = int(re.search(r'attribute\((\d+)\)', ' '.join(p['attrs'])).group(1))
                vatt += p['name'].encode() + b'\0' + struct.pack('<H', loc | (0x8000 if f['used'][k] else 0))
                vaty += bytes([mtl_data_type(msl_name(p['type']))])
            tags += tagged(b'VATT', vatt) + tagged(b'VATY', vaty)
    used = []
    for name in FC_ORDER:
        if re.search(r'@_ZL%d%s\b' % (len(name), name), body):
            used.append(name)
    if used:
        idx = {n: int(re.search(r'MTL_FC_INIT_(\d+)_', g).group(1)) for n, g in FC_GLOBALS.items()}
        used.sort(key=lambda n: idx[n])
        data = struct.pack('<H', len(used))
        for n in used:
            data += n.encode() + b'\0' + bytes([mtl_data_type(msl_name(FC_TYPES[n]))]) + struct.pack('<H', idx[n]) + bytes([0 if n in fcs_defined_checked else 1])
        tags += tagged(b'CNST', data)
    tags += b'ENDT'
    return struct.pack('<I', len(tags)) + tags
FC_GLOBALS = {}
FC_CHECKED = set()

# ---------------------------------------------------------------- 6. metallib header
def header(os_name, ver, sim, air):
    family = 0x8001 if os_name == 'macosx' else 1
    filever = {1: 3, 3: 5, 5: 7, 6: 7, 7: 8, 8: 9, 9: 9}.get(air[1], 7)
    plat = {'macosx': 1, 'ios': 2, 'tvos': 3, 'xros': 11}[os_name]
    if sim:
        plat = {'ios': 7}.get(os_name, plat)
    if air < (2, 5):  # no platform or deployment target before file version 7
        return (family, 2, filever, 0, 0, 0)
    return (family, 2, filever, (0x80 | plat) << 8, ver[0], ver[1])

def main(argv):
    target, out, incs, src = None, None, [], None
    it = iter(argv)
    for a in it:
        if a == '-target': target = next(it)
        elif a == '-o': out = next(it)
        elif a.startswith('-I'): incs.append(a if len(a) > 2 else '-I' + next(it))
        elif a.endswith('.metal'): src = a
    with tempfile.TemporaryDirectory() as tmp:
        rew = os.path.join(tmp, os.path.basename(src) + '.cpp')
        os_name, ver, sim = parse_target(target)
        air_v, _ = air_versions(os_name, ver)
        code = rewrite_samplers(rewrite(open(src).read()), air_v)
        open(rew, 'w').write(code)
        base = [LLVM + '/clang++', '-std=c++17', '-target', 'spir64', '-ffast-math', '-fno-builtin',
                '-Wno-unknown-attributes', '-nostdinc++', '-I', HERE + '/include', '-I', os.path.dirname(os.path.abspath(src))] + incs
        ast = subprocess.run(base + ['-D__MSL_AST', '-fsyntax-only', '-Xclang', '-ast-dump', '-fno-color-diagnostics', rew],
                             capture_output=True, text=True)
        ast = ast.stdout  # errors here are address-space checks codegen doesn't have
        funcs = stage_functions(ast_tree(ast))
        open(rew, 'w').write(code + '\n' + entry_points(funcs))
        llf = os.path.join(tmp, 'out.ll')
        r = subprocess.run(base + ['-O2', '-Xclang', '-disable-llvm-passes', '-Xclang', '-no-opaque-pointers', '-S', '-emit-llvm',
                                   '-o', llf + '.0', '-Xclang', '-fdump-record-layouts', rew], capture_output=True, text=True)
        if r.returncode:
            sys.stderr.write(r.stderr); raise SystemExit(1)
        layouts = record_layouts(r.stdout)
        subprocess.run([LLVM + '/opt', '-passes=instnamer', '-S', '-o', llf + '.0n', llf + '.0'], check=True)
        ir1, fcs = function_constant_ir(direct_returns(direct_struct_returns(open(llf + '.0n').read(), funcs)))
        ir1 = sampler_ir(ir1, air_v)
        ir1 = prepare_flat(ir1, [f['name'] for f in funcs])
        open(llf + '.1', 'w').write(ir1)
        if os.environ.get('MSL2AIR_KEEP'):
            shutil.copy(llf + '.0', out + '.0.ll'); shutil.copy(llf + '.1', out + '.1.ll'); shutil.copy(rew, out + '.cpp')
        subprocess.run([LLVM + '/opt', '-O2', '-S', '-o', llf + '.2', llf + '.1'], check=True)
        ir2 = resolve_atomics(open(llf + '.2').read())
        open(llf + '.2', 'w').write(ir2)
        subprocess.run([LLVM + '/opt', '-passes=infer-address-spaces', '-assume-default-is-flat-addrspace', '-S', '-o', llf + '.3', llf + '.2'], check=True)
        subprocess.run([LLVM + '/opt', '-O2', '-S', '-o', llf + '.4', llf + '.3'], check=True)
        shutil.copy(llf + '.4', llf)
        if os.environ.get('MSL2AIR_KEEP'):
            for k in ('.2', '.3', '.4', ''):
                shutil.copy(llf + k, out + '.opt%s.ll' % k)
        air_ll, air, lang, (os_name, ver, sim) = to_air(open(llf).read(), funcs, layouts, target, os.path.abspath(src), fcs)
        open(out + '.ll', 'w').write(air_ll)
        bc = os.path.join(tmp, 'out.bc')
        subprocess.run([LLVM + '/llvm-as', '-o', bc, out + '.ll'], check=True)
        module = mtlb.wrap(open(bc, 'rb').read())
        fl = [dict(name=f['name'], type={'vertex': 0, 'fragment': 1, 'kernel': 2}[f['stage']], module=module,
                   air=air, lang=lang, pub=public_md(air_ll, f['name'], FC_CHECKED, f)) for f in funcs]
        open(out, 'wb').write(mtlb.metallib(fl, header(os_name, ver, sim, air)))

if __name__ == '__main__':
    main(sys.argv[1:])
