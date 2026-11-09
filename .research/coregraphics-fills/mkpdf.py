"""mkpdf.py OUT.pdf W H CONTENT: a one-page PDF of size WxH points with CONTENT as its stream."""
import sys
out, w, h, content = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4].replace('\\n', '\n').encode()
objs = [b'<< /Type /Catalog /Pages 2 0 R >>', b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
        ('<< /Type /Page /Parent 2 0 R /MediaBox [0 0 %s %s] /Contents 4 0 R >>' % (w, h)).encode(),
        b'<< /Length %d >>\nstream\n' % len(content) + content + b'\nendstream']
pdf = b'%PDF-1.4\n'; offs = []
for i, o in enumerate(objs):
    offs.append(len(pdf)); pdf += b'%d 0 obj\n' % (i + 1) + o + b'\nendobj\n'
x = len(pdf)
pdf += b'xref\n0 %d\n0000000000 65535 f \n' % (len(objs) + 1) + b''.join(b'%010d 00000 n \n' % o for o in offs)
pdf += b'trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n' % (len(objs) + 1, x)
open(out, 'wb').write(pdf)
