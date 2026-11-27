"""Whether two nibs are the same up to what Xcode's ibtool varies from run to
run (tools/xcode_equivalence.py compares .nib files with it).

ibtool doesn't always write the same nib twice for the same document. Each
variation below was seen in repeated compiles of one document; nothing else
may differ (docs/clean-room/ibtool.md):
- The order of a nib's connections (`UINibConnectionsKey`; an AppKit nib's
  `NSConnections` and `NSOidsKeys`/`NSOidsValues`) and of an AppKit nib's
  accessibility connectors, and the object ids that follow from it: a
  corpus tab bar controller's storyboard outlets (six compiles), a
  storyboard with two accessibility identifiers (six compiles).
- A scroller without `wantsLayer` is a layer tree host in an ssh session's
  first compile and not in later ones, and that first compile tiles
  overlay scrollers over the clip view where later ones tile legacy
  scrollers 17 points beside it (three and six compiles).
- A root view Xcode resizes to its constraints, its top kept, lays each of
  its subviews out either in its new frame or in its old one moved with it
  (six compiles of one xib, four outputs). This needs the document, for
  the roots' frames.
- Below iOS 14 a date picker's date is the time Xcode compiles it (eight
  probes).
- A Mac storyboard's window controllers' and their class swappers'
  `uniqueIdentifierForStoryboardCompilation`, a UUID new each run, wherever
  the nib holds it.

Each nib is put in one canonical form, and the two compared whole: its
archive's versions; every object (reachable or not), numbered from the root as it is reached, with
only the containers above in a fixed order; its class and the class's
fallbacks; each value with its type; references by their canonical
number; the UUIDs above renamed one to one in order of appearance.

  python3 tools/nib_equivalence.py [--document=PATH] [--compile-time-dates] XCODE.nib OURS.nib
"""

import hashlib
import re
import struct
import sys
import xml.etree.ElementTree as ET

# References compared by the class they point to when ordering objects (back
# references and the like, which would otherwise cycle); in the canonical
# form they are references like any other.
_WEAK = {"NSFirstItem", "NSSecondItem", "NSMenu", "NSNextResponder", "NSSuperview", "NSControlView", "NSWindow", "NSSource_", "NSNextKeyView"}
_UNORDERED = {"NSConnections", "NSOidsKeys", "NSOidsValues", "UINibConnectionsKey",
              "NSAccessibilityConnectors", "NSAccessibilityOidsKeys", "NSAccessibilityOidsValues"}
_UUID = re.compile(r"b'[0-9A-F]{8}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{12}'")
_UUID_KEY = "uniqueIdentifierForStoryboardCompilation"
_FRAME = re.compile(r"b'\{\{(-?[0-9.]+), (-?[0-9.]+)\}, \{([0-9.]+), ([0-9.]+)\}\}'")
_SIZE = re.compile(r"b'\{([0-9.]+), ([0-9.]+)\}'")


def _varint(b, i):
    r = s = 0
    while True:
        c = b[i]
        i += 1
        r |= (c & 0x7F) << s
        s += 7
        if c & 0x80:
            return r, i


class _Section:
    """A nib's objects {index: (class, [(key, value text)])}, its classes'
    fallbacks {class: names}, and the nibs embedded in it (a cell's
    prototype nib)."""

    def __init__(self):
        self.objs, self.fallbacks, self.nibs, self.versions = {}, {}, [], ()
        self.memo, self.unordered = {}, set()


def _parse(b):
    """A NIBArchive as a _Section. Value texts: `@N` an object, `NIBn` an
    embedded nib, `nil`, a data value's repr, or a number or boolean with
    its type after `#`."""
    if b[:10] != b"NIBArchive":
        raise ValueError("not a NIBArchive")
    header = struct.unpack("<10I", b[10:50])
    oc, oo, kc, ko, vc, vo, cc, co = header[2:]
    sec = _Section()
    sec.versions = header[:2]
    keys, i = [], ko
    for _ in range(kc):
        n, i = _varint(b, i)
        keys.append(b[i:i + n].decode("utf8", "replace"))
        i += n
    classes, i = [], co
    for _ in range(cc):
        n, i = _varint(b, i)
        x, i = _varint(b, i)
        fallback = struct.unpack("<%di" % x, b[i:i + 4 * x])
        i += 4 * x
        # Compared as stored, its terminator too; named without it.
        raw = b[i:i + n]
        classes.append((raw.rstrip(b"\0").decode() + ("" if raw.endswith(b"\0") else "!unterminated"), fallback))
        i += n
    values, i = [], vo
    for _ in range(vc):
        k, i = _varint(b, i)
        t = b[i]
        i += 1
        if t in (0, 1, 2, 3):
            size, fmt = {0: (1, "<B"), 1: (2, "<h"), 2: (4, "<i"), 3: (8, "<q")}[t]
            v, i = "%d#i%d" % (struct.unpack(fmt, b[i:i + size])[0], t), i + size
        elif t in (4, 5):
            v = "%s#b" % (t == 5)
        elif t == 6:
            v, i = repr(struct.unpack("<f", b[i:i + 4])[0]) + "#f", i + 4
        elif t == 7:
            v, i = repr(struct.unpack("<d", b[i:i + 8])[0]) + "#d", i + 8
        elif t == 8:
            n, i = _varint(b, i)
            d = b[i:i + n]
            i += n
            if d[:10] == b"NIBArchive":
                sec.nibs.append(_parse(d))
                v = "NIB%d" % (len(sec.nibs) - 1)
            else:
                v = repr(d)
        elif t == 9:
            v = "nil"
        elif t == 10:
            v, i = "@%d" % struct.unpack("<I", b[i:i + 4])[0], i + 4
        else:
            raise ValueError("value type %d" % t)
        values.append((keys[k], v))
    i, ranges = oo, []
    for n in range(oc):
        c, i = _varint(b, i)
        vi, i = _varint(b, i)
        vn, i = _varint(b, i)
        ranges.append((vi, vn))
        name, fallback = classes[c]
        sec.objs[n] = (name, values[vi:vi + vn])
        sec.fallbacks[name] = tuple(classes[f][0] for f in fallback)
    # Every value is one object's (Xcode doesn't always store them in its
    # objects' order), and every reference is to an object.
    owned = 0
    for vi, vn in sorted(ranges):
        if vi != owned:
            raise ValueError("a value no object has, or two have")
        owned += vn
    if owned != vc or any(v.startswith("@") and int(v[1:]) >= oc for _, v in values):
        raise ValueError("a value no object has, or a reference to no object")
    _prepare(sec)
    return sec


def _prepare(sec):
    """The containers whose order Xcode varies: the connection tables the
    nib's root lists."""
    sec.unordered = {int(v[1:]) for n, (cls, kv) in sec.objs.items() if cls == "NSIBObjectData" or n == 0
                     for k, v in kv if k in _UNORDERED and v.startswith("@")}


def _sections(sec):
    yield sec
    for n in sec.nibs:
        yield from _sections(n)


def _uuids(root):
    """The texts of the UUID strings Xcode makes anew each run."""
    out = set()
    for sec in _sections(root):
        for cls, kv in sec.objs.values():
            for k, v in kv:
                if k == _UUID_KEY and v.startswith("@"):
                    out |= {s for key, s in sec.objs[int(v[1:])][1] if key == "NS.bytes" and _UUID.fullmatch(s)}
    return out


class _Canon:
    def __init__(self, compile_time_dates):
        self.varying = {("NSScroller", "NSViewIsLayerTreeHost")}
        if compile_time_dates:
            self.varying.add(("NSDate", "NS.time"))

    def _entries(self, sec, n):
        """An object's (key, value) entries, those Xcode varies left out, with
        the members of a container whose order varies as its sorted ones
        would be, by `order`'s key."""
        cls, kv = sec.objs[n]
        return [(k, v) for k, v in kv if (cls, k) not in self.varying]

    def _ordered(self, sec, n, entries, rank):
        """`entries` with the members Xcode orders at will sorted by `rank`."""
        if n not in sec.unordered:
            return entries
        at = [i for i, (k, v) in enumerate(entries) if k == "UINibEncoderEmptyKey" and v.startswith("@")]
        out = list(entries)
        for i, (k, v) in zip(at, sorted((entries[i] for i in at), key=lambda e: rank(int(e[1][1:])))):
            out[i] = (out[i][0], v)
        return out

    def sig(self, sec, n, uuids, stack=()):
        """A content hash of an object and what it refers to, for ordering."""
        if n in sec.memo:
            return sec.memo[n]
        if n in stack:
            return "cycle:" + sec.objs[n][0]
        parts = [sec.objs[n][0]]
        entries = self._entries(sec, n)
        for k, v in self._ordered(sec, n, entries, lambda c: self.sig(sec, c, uuids, stack + (n,))):
            if v.startswith("@"):
                c = int(v[1:])
                s = "w:" + sec.objs[c][0] if k in _WEAK else self.sig(sec, c, uuids, stack + (n,))
            elif re.fullmatch(r"NIB\d+", v):
                s = "nib:" + self.form(sec.nibs[int(v[3:])], uuids)[1]
            else:
                s = "UUID" if v in uuids else v
            parts.append((k, s))
        h = hashlib.sha1(repr(parts).encode()).hexdigest()
        sec.memo[n] = h
        return h

    def form(self, sec, uuids, names=None):
        """(canonical text, its hash) of a section; `names` the UUIDs'
        renaming, shared with the sections embedded in it."""
        names = {} if names is None else names
        rank = lambda c: self.sig(sec, c, uuids)
        number, queue = {}, []

        def reach(c):
            if c not in number:
                number[c] = len(number)
                queue.append(c)

        if sec.objs:
            reach(0)
        while queue:
            n = queue.pop(0)
            for k, v in self._ordered(sec, n, self._entries(sec, n), rank):
                if v.startswith("@"):
                    reach(int(v[1:]))
        for n in sorted((n for n in sec.objs if n not in number), key=rank):
            reach(n)
            while queue:
                m = queue.pop(0)
                for k, v in self._ordered(sec, m, self._entries(sec, m), rank):
                    if v.startswith("@"):
                        reach(int(v[1:]))
        lines = []
        for n in sorted(number, key=number.get):
            cls = sec.objs[n][0]
            out = []
            for k, v in self._ordered(sec, n, self._entries(sec, n), rank):
                if v.startswith("@"):
                    v = "@%d" % number[int(v[1:])]
                elif re.fullmatch(r"NIB\d+", v):
                    v = "nib:" + self.form(sec.nibs[int(v[3:])], uuids, names)[1]
                elif v in uuids:
                    v = names.setdefault(v, "UUID%d" % len(names))
                out.append("%s=%s" % (k, v))
            lines.append("%d %s(%s) %s" % (number[n], cls, ",".join(sec.fallbacks.get(cls, ())), " ".join(out)))
        text = "%d %d\n" % sec.versions + "\n".join(lines)
        return text, hashlib.sha1(text.encode()).hexdigest()


def _clear(sec):
    for s in _sections(sec):
        s.memo.clear()


def _frame(text):
    """A frame, or a size at the origin, from its data value's text, and
    whether it was a frame."""
    m = _FRAME.fullmatch(text)
    if m:
        return tuple(float(g) for g in m.groups()), True
    m = _SIZE.fullmatch(text)
    return ((0.0, 0.0, float(m.group(1)), float(m.group(2))), False) if m else (None, False)


def _scalar(text):
    return float(text.split("#")[0])


def _number(v):
    return str(int(v)) if v == int(v) else repr(v)


def _frame_text(f, full):
    if full:
        return "b'{{%s, %s}, {%s, %s}}'" % tuple(_number(x) for x in f)
    return "b'{%s, %s}'" % (_number(f[2]), _number(f[3]))


def _document_roots(path):
    """The frames of a document's root views: its top-level views and its
    view controllers' views."""
    roots = []
    tree = ET.parse(path)
    parents = {c: p for p in tree.iter() for c in p}
    for v, p in parents.items():
        if v.tag in ("view", "customView") and (p.tag == "objects" or (v.get("key") == "view" and p.tag == "viewController")):
            r = v.find("rect[@key='frame']")
            if r is not None:
                roots.append(tuple(float(r.get(k)) for k in ("x", "y", "width", "height")))
    return roots


def _resized(f, roots):
    """How far a root view of `roots`, resized with its top kept to frame
    `f`, moved down; None if none was."""
    for x0, y0, w0, h0 in roots:
        if x0 == f[0] and w0 == f[2] and h0 != f[3] and y0 + h0 - f[3] == f[1]:
            return f[1] - y0
    return None


def _moved_subviews(sec, roots):
    """(subview, shift) for each subview of a root view resized from one of
    the document's."""
    o, out = sec.objs, []
    for cls, kv in o.values():
        d = dict(kv)
        if "NSSubviews" not in d or "NSSuperview" in d or not d.get("NSFrame", "").startswith("@"):
            continue
        f, full = _frame(dict(o[int(d["NSFrame"][1:])][1]).get("NS.bytes", ""))
        shift = _resized(f, roots) if f and full else None
        if shift is None:
            continue
        for k, v in o[int(d["NSSubviews"][1:])][1]:
            if (k == "UINibEncoderEmptyKey" or k.startswith("NS.object.")) and v.startswith("@"):
                sv = dict(o[int(v[1:])][1])
                if any(sv.get(key, "").startswith("@") for key in ("NSFrame", "NSFrameSize")):
                    out.append((int(v[1:]), shift))
    return out


def _frame_string(sec, n):
    d = dict(sec.objs[n][1])
    return int((d.get("NSFrame") or d["NSFrameSize"])[1:])


def _move(sec, view, original, shift):
    """The view moved down by `shift`, its frame as the nib would have it."""
    (cls, kv), (scls, skv) = original
    key = "NSFrame" if "NSFrame" in dict(kv) else "NSFrameSize"
    sid = int(dict(kv)[key][1:])
    (fx, fy, fw, fh), _ = _frame(dict(skv)["NS.bytes"])
    fy += shift
    full = (fx, fy) != (0.0, 0.0)
    sec.objs[view] = (cls, [(("NSFrame" if full else "NSFrameSize") if k == key else k, v) for k, v in kv])
    sec.objs[sid] = (scls, [(k, _frame_text((fx, fy, fw, fh), full) if k == "NS.bytes" else v) for k, v in skv])


def _overlay_to_legacy(root):
    """The nib's overlay-tiled scroll views tiled as legacy ones: the
    shown scrollers' 17 points taken from the clip view and document view.
    Whether any changed."""
    changed = False
    for sec in _sections(root):
        o = sec.objs

        def get(n):
            return dict(o[n][1])

        def size_of(n):
            d = get(n)
            for k in ("NSFrame", "NSFrameSize"):
                if d.get(k, "").startswith("@"):
                    sid = int(d[k][1:])
                    f, full = _frame(get(sid).get("NS.bytes", ""))
                    if f:
                        return sid, f, full
            return None

        def put(sid, f, full):
            cls, kv = o[sid]
            o[sid] = (cls, [(k, _frame_text(f, full) if k == "NS.bytes" else v) for k, v in kv])

        for n, (cls, kv) in list(o.items()):
            d = dict(kv)
            if cls != "NSScrollView" and "NSsFlags" not in d:
                continue
            if not all(d.get(k, "").startswith("@") for k in ("NSVScroller", "NSHScroller", "NSContentView")):
                continue
            shown = [_scalar(get(int(d[k][1:])).get("NSvFlags", "0")) >= 0 for k in ("NSHScroller", "NSVScroller")]
            if not any(shown):
                continue
            clip = int(d["NSContentView"][1:])
            cf, dv = size_of(clip), get(clip).get("NSDocView", "")
            own = size_of(n)
            if not cf or not own:
                continue
            bars = (17.0 if shown[1] else 0.0, 17.0 if shown[0] else 0.0)
            cs, (cx, cy, cw, ch), cfull = cf
            inset = [0.0, 1.0, 1.0, 2.0][int(_scalar(d["NSsFlags"])) & 3]
            # Only an overlay tiling: the clip view the whole inside of the border.
            if (cw, ch) != (own[1][2] - 2 * inset, own[1][3] - 2 * inset):
                continue
            put(cs, (cx, cy, cw - bars[0], ch - bars[1]), cfull)
            if dv.startswith("@"):
                doc = int(dv[1:])
                df = size_of(doc)
                if df:
                    ds, (dx, dy, dw, dh), dfull = df
                    nw, nh = dw - (bars[0] if dw == cw else 0), dh - (bars[1] if dh == ch else 0)
                    put(ds, (dx, dy, nw, nh), dfull)
                    tc = get(doc).get("NSTextContainer", "")
                    if tc.startswith("@"):
                        t = int(tc[1:])
                        tcls, tkv = o[t]

                        def narrower(v):
                            number, kind = v.split("#")
                            return "%r#%s" % (float(number) - bars[0], kind)

                        o[t] = (tcls, [(k, narrower(v) if k == "NSWidth" and _scalar(v) == dw and nw != dw else v) for k, v in tkv])
            # The scrollers beside each other rather than over the clip view.
            for k, i in (("NSHScroller", 0), ("NSVScroller", 1)):
                sf = size_of(int(d[k][1:]))
                if shown[i] and shown[1 - i] and sf:
                    ss, (sx, sy, sw, sh), sfull = sf
                    put(ss, (sx, sy, sw - (17.0 if i == 0 else 0.0), sh - (17.0 if i == 1 else 0.0)), sfull)
            changed = True
    if changed:
        _clear(root)
    return changed


def _same_moved(form, a, b, roots):
    for x, y in ((a, b), (b, a)):
        moved = list(dict.fromkeys((s, n, sh) for s in _sections(x) for n, sh in _moved_subviews(s, roots)))
        if not moved or len(moved) > 10:
            continue
        original = [(s, n, (s.objs[n], s.objs[_frame_string(s, n)]), _frame_string(s, n)) for s, n, _ in moved]
        want = form(y)
        for mask in range(1, 1 << len(moved)):
            for i, (s, n, sh) in enumerate(moved):
                _, _, (view_obj, string_obj), sid = original[i]
                s.objs[n], s.objs[sid] = view_obj, string_obj
                if mask >> i & 1:
                    _move(s, n, (view_obj, string_obj), sh)
            _clear(x)
            if form(x) == want:
                return True
        for s, n, (view_obj, string_obj), sid in original:
            s.objs[n], s.objs[sid] = view_obj, string_obj
        _clear(x)
    return False


def equivalent(xcode, ours, document=None, compile_time_dates=False):
    """Whether nib `ours` is nib `xcode` up to Xcode's variations; with
    `document`, the xib or storyboard they were compiled from."""
    if xcode == ours:
        return True
    a, b = _parse(xcode), _parse(ours)
    roots = _document_roots(document) if document else []
    canon = _Canon(compile_time_dates)

    def form(sec):
        return canon.form(sec, _uuids(sec))[0]

    if form(a) == form(b):
        return True
    for x, y in ((a, b), (b, a)):
        if _overlay_to_legacy(x) and (form(x) == form(y) or _same_moved(form, x, y, roots)):
            return True
    return _same_moved(form, a, b, roots)


def main(argv):
    document = next((a.split("=", 1)[1] for a in argv if a.startswith("--document=")), None)
    dates = "--compile-time-dates" in argv
    paths = [a for a in argv if not a.startswith("--")]
    if len(paths) != 2:
        print(__doc__.strip().splitlines()[-1].strip(), file=sys.stderr)
        return 2
    with open(paths[0], "rb") as f, open(paths[1], "rb") as g:
        same = equivalent(f.read(), g.read(), document, dates)
    print("equivalent" if same else "different")
    return 0 if same else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
