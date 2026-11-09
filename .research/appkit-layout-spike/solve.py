"""solve.py DOC: lays out a document's constrained views as a prototype of Xcode's Auto Layout pass; prints, per
view id, the document's frame and the predicted one (x, y from bottom, w, h, in its superview), or BLOCKED reason."""
import math, subprocess, sys, functools
import xml.etree.ElementTree as ET
import kiwisolver as kiwi

TM = '/tmp/lay/tm'
@functools.lru_cache(maxsize=None)
def metrics(size, bold, text):
    w, lh = subprocess.run([TM, str(size), '1' if bold else '0', text], capture_output=True, text=True).stdout.split()
    return float(w), float(lh)
def text_width(size, text, bold=False):
    return metrics(size, bold, text)[0]
# Labels' heights, probed (the font's default line height but at 11 points).
LABEL_HEIGHT = {11: 14, 13: 16, 18: 21}
def label_height(size, bold):
    return LABEL_HEIGHT.get(size, math.ceil(metrics(size, bold, 'x')[1]))
FONT = {'system': 13, 'menu': 13, 'smallSystem': 11, 'message': 13, 'controlContent': 13}

class Blocked(Exception): pass

def intrinsic(el):
    """(width or None, height or None, frame insets (left, right, bottom, top)) of a content-sized control."""
    tag = el.tag
    cell = next((c for c in el if c.get('key') == 'cell'), None)
    def font(c):
        f = c.find('font') if c is not None else None
        if f is None or f.get('usesAppearanceFont') == 'YES': return 13, False
        if f.get('name'): raise Blocked('named font')
        meta = f.get('metaFont')
        bold = meta in ('systemBold', 'boldSystem')
        base = {'systemBold': 13, 'boldSystem': 13}.get(meta, FONT.get(meta))
        if base is None: raise Blocked('font %s' % meta)
        return (float(f.get('size')) if f.get('size') else base), bold
    def font_size(c):
        size, bold = font(c)
        if bold: raise Blocked('bold control font')
        return size
    if tag == 'button' and cell is not None:
        t = cell.get('type'); size = cell.get('controlSize', 'regular'); title = cell.get('title', '')
        if cell.find('../image') is not None or cell.get('image'): raise Blocked('button image')
        if t == 'push' and cell.get('bezelStyle') == 'rounded':
            if size == 'regular': return math.ceil(text_width(font_size(cell), title)) + 24, 24, (0, 0, 0, 0)
            if size == 'small': return math.ceil(text_width(font_size(cell), title)) + 20, 20, (0, 0, 0, 0)
        if t == 'check' and size == 'regular':
            return math.ceil(text_width(font_size(cell), title)) + 22, 16, (0, 0, 0, 0)
        raise Blocked('button %s %s %s' % (t, cell.get('bezelStyle'), size))
    if tag == 'textField' and cell is not None:
        editable = cell.get('editable') == 'YES' or cell.get('borderStyle') == 'bezel'
        if editable:
            if cell.get('controlSize', 'regular') != 'regular': raise Blocked('small text field')
            return None, 24, (0, 0, 0, 0)
        title = cell.get('title', '')
        if '\n' in title or cell.get('lineBreakMode') in ('wordWrapping', 'charWrapping') or el.get('preferredMaxLayoutWidth'):
            raise Blocked('wrapping label')
        fs, bold = font(cell)
        if fs == int(fs): fs = int(fs)
        return math.ceil(text_width(fs, title, bold)), label_height(fs, bold), (2, 2, 0, 0)
    if tag == 'popUpButton' and cell is not None:
        if cell.get('controlSize', 'regular') != 'regular' or cell.get('type') != 'push' or cell.get('pullsDown') == 'YES': raise Blocked('pop-up kind')
        title = cell.get('title') or ''
        return math.ceil(text_width(font_size(cell), title)) + 48, 24, (0, 0, 0, 0)
    if tag == 'slider':
        return None, 16, (2, 2, 0, 0)
    raise Blocked('content-sized <%s>' % tag)

CONTENT = ('button', 'textField', 'popUpButton', 'slider', 'imageView', 'searchField', 'progressIndicator', 'comboBox', 'segmentedControl', 'datePicker', 'stepper', 'colorWell', 'levelIndicator', 'pathControl')

def frame(el):
    r = next((c for c in el if c.tag == 'rect' and c.get('key') == 'frame'), None)
    if r is None: return None
    return [float(r.get(k)) for k in ('x', 'y', 'width', 'height')]

def layout(doc):
    root = ET.parse(doc).getroot()
    parent = {c: p for p in root.iter() for c in p}
    byid = {e.get('id'): e for e in root.iter() if e.get('id')}
    results = {}
    # Every view with constraints owns a solve over itself and its descendants that constraints mention.
    for owner in root.iter():
        cons = owner.find('constraints')
        subs = owner.find('subviews')
        if cons is None and subs is None: continue
        if owner.tag in ('stackView', 'scrollView', 'clipView', 'tableView', 'outlineView', 'collectionView', 'splitView', 'tabView'):
            if cons is not None and len(cons): raise Blocked('constraints in <%s>' % owner.tag)
            continue
        if owner.tag == 'box':
            continue
        if owner.get('id') is None or frame(owner) is None: continue
        # Views: the owner and its direct subviews (deeper ones are solved by their own owners).
        views = [owner] + ([v for v in subs] if subs is not None else [])
        views = [v for v in views if v.get('id') and frame(v) is not None]
        ox, oy, ow, oh = frame(owner)
        # Its size as solved by its own superview's pass, if it was; a box's content view is the box inset by its border.
        if owner.get('id') in results:
            ow, oh = results[owner.get('id')][1][2:]
        elif owner.get('key') == 'contentView' and parent.get(owner) is not None and parent[owner].tag == 'box':
            bx = parent[owner]
            if bx.get('id') in results:
                inset = 0 if bx.get('borderType') == 'none' else float(bx.get('borderWidth', 1))
                ow, oh = results[bx.get('id')][1][2] - 2 * inset, results[bx.get('id')][1][3] - 2 * inset
        V = {}
        s = kiwi.Solver()
        for v in views:
            l, b, w, h = (kiwi.Variable(v.get('id') + n) for n in 'lbwh')
            V[v.get('id')] = (l, b, w, h, v)
        def add(c, strength='required'):
            s.addConstraint(c | strength)
        o = V[owner.get('id')]
        for c in (o[0] == 0, o[1] == 0, o[2] == ow, o[3] == oh): add(c)
        for vid, (l, b, w, h, v) in V.items():
            if v is owner: continue
            x, y, fw, fh = frame(v)
            if v.tag in ('stackView', 'tableView', 'tabView', 'splitView') and v.get('translatesAutoresizingMaskIntoConstraints') == 'NO':
                raise Blocked('constrained <%s>' % v.tag)
            if v.get('translatesAutoresizingMaskIntoConstraints') != 'NO':
                for c in (l == x, b == oh - y - fh, w == fw, h == fh): add(c)
                continue
            ins = (0, 0, 0, 0)
            # A separator box's alignment rect is its line, 2 points inside its frame across it.
            if v.tag == 'box' and v.get('boxType') == 'separator':
                ins = (2, 2, 0, 0) if fw < fh else (0, 0, 2, 2)
            if v.tag in CONTENT:
                iw, ih, ins = intrinsic(v)
                hh = float(v.get('horizontalHuggingPriority', 250)); vh = float(v.get('verticalHuggingPriority', 250))
                hc = float(v.get('horizontalCompressionResistancePriority', 750)); vc = float(v.get('verticalCompressionResistancePriority', 750))
                if iw is not None:
                    add(w <= iw, kiwi.strength.create(0, hh, 0)); add(w >= iw, kiwi.strength.create(0, hc, 0))
                if ih is not None:
                    add(h <= ih, kiwi.strength.create(0, vh, 0)); add(h >= ih, kiwi.strength.create(0, vc, 0))
            # Stay where the document has it when nothing decides (alignment rect from the frame and insets).
            add(l == x + ins[0], 'weak'); add(b == oh - (y + fh) + ins[3], 'weak'); add(w == fw - ins[0] - ins[1], 'weak'); add(h == fh - ins[2] - ins[3], 'weak')
            V[vid] = (l, b, w, h, v, ins)
        def attr(item, a):
            if item is None: return 0
            if item not in V: raise Blocked('constraint to a view elsewhere')
            l, b, w, h = V[item][:4]
            return {'left': l, 'leading': l, 'right': l + w, 'trailing': l + w, 'top': b, 'bottom': b + h,
                    'width': w, 'height': h, 'centerX': l + w * 0.5, 'centerY': b + h * 0.5}[a] if a in ('left', 'leading', 'right', 'trailing', 'bottom', 'top', 'width', 'height', 'centerX', 'centerY') else (_ for _ in ()).throw(Blocked('attribute %s' % a))
        for c in (cons if cons is not None else []):
            fi = c.get('firstItem', owner.get('id')); si = c.get('secondItem')
            fa = c.get('firstAttribute'); sa = c.get('secondAttribute')
            m = float(c.get('multiplier', '1').split(':')[0]) / (float(c.get('multiplier').split(':')[1]) if ':' in c.get('multiplier', '') else 1)
            k = float(c.get('constant', 0))
            lhs = attr(fi, fa)
            rhs = (attr(si, sa) * m + k) if si else k
            rel = c.get('relation')
            expr = (lhs >= rhs) if rel == 'greaterThanOrEqual' else (lhs <= rhs) if rel == 'lessThanOrEqual' else (lhs == rhs)
            p = float(c.get('priority', 1000))
            add(expr, 'required' if p >= 1000 else kiwi.strength.create(0, p, 0))
        s.updateVariables()
        for vid, t in V.items():
            v = t[4]
            if v is owner: continue
            l, b, w, h = (round(q.value(), 6) for q in t[:4])
            ins = t[5] if len(t) > 5 else (0, 0, 0, 0)
            # Frames snap to whole points, each edge rounded half up (top-down unless ROUND_UP is set).
            up = lambda q: math.floor(q + 0.5 + 1e-9)
            x0, x1 = up(l - ins[0]), up(l + w + ins[1])
            if __import__('os').environ.get('ROUND_UP'):
                y0, y1 = up(oh - (b + h) - ins[2]), up(oh - b + ins[3])
            else:
                t0, t1 = up(b - ins[3]), up(b + h + ins[2])
                y0, y1 = oh - t1, oh - t0
            results[vid] = (frame(v), [float(x0), float(y0), float(x1 - x0), float(y1 - y0)])
            if v.tag == 'scrollView' and results[vid][0][2:] != results[vid][1][2:] and not __import__('os').environ.get('NOBLOCK'):
                raise Blocked('a scroll view resized (its contents are retiled)')
    return results

if __name__ == '__main__':
    try:
        for vid, (doc, new) in layout(sys.argv[1]).items():
            print(vid, doc, new)
    except Blocked as e:
        print('BLOCKED', e)
