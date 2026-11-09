// glyphs FONT SIZE X Y TEXT: prints the text's glyph outlines as cov path commands (quadratics raised to cubics), then "f".
import CoreText
import CoreGraphics
import Foundation
let a = CommandLine.arguments
let font = CTFontCreateWithName(a[1] as CFString, CGFloat(Double(a[2])!), nil)
let ox = Double(a[3])!, oy = Double(a[4])!
let s = a[5]
var chars = Array(s.utf16); var glyphs = [CGGlyph](repeating: 0, count: chars.count)
CTFontGetGlyphsForCharacters(font, &chars, &glyphs, chars.count)
var adv = [CGSize](repeating: .zero, count: glyphs.count)
CTFontGetAdvancesForGlyphs(font, .horizontal, glyphs, &adv, glyphs.count)
var x = ox
for (i, g) in glyphs.enumerated() {
    if let p = CTFontCreatePathForGlyph(font, g, nil) {
        var cur = CGPoint.zero
        p.applyWithBlock { e in
            let el = e.pointee; let pts = el.points
            func P(_ q: CGPoint) -> String { "\(Double(q.x) + x) \(Double(q.y) + oy)" }
            switch el.type {
            case .moveToPoint: print("m \(P(pts[0]))"); cur = pts[0]
            case .addLineToPoint: print("l \(P(pts[0]))"); cur = pts[0]
            case .addQuadCurveToPoint:
                let c = pts[0], e2 = pts[1]
                let c1 = CGPoint(x: cur.x + 2.0/3.0*(c.x-cur.x), y: cur.y + 2.0/3.0*(c.y-cur.y))
                let c2 = CGPoint(x: e2.x + 2.0/3.0*(c.x-e2.x), y: e2.y + 2.0/3.0*(c.y-e2.y))
                print("c \(P(c1)) \(P(c2)) \(P(e2))"); cur = e2
            case .addCurveToPoint: print("c \(P(pts[0])) \(P(pts[1])) \(P(pts[2]))"); cur = pts[2]
            case .closeSubpath: print("h")
            @unknown default: break
            }
        }
    }
    x += Double(adv[i].width)
}
print("f")
