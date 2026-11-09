// outline: reads cov-style path then "s WIDTH JOIN CAP"; prints copy(strokingWithWidth:) elements.
import CoreGraphics
import Foundation
var path = CGMutablePath()
while let line = readLine() {
    let t = line.split(separator: " ").map(String.init); if t.isEmpty { continue }
    let v = t.dropFirst().map { CGFloat(Double($0)!) }
    switch t[0] {
    case "m": path.move(to: CGPoint(x: v[0], y: v[1]))
    case "l": path.addLine(to: CGPoint(x: v[0], y: v[1]))
    case "c": path.addCurve(to: CGPoint(x: v[4], y: v[5]), control1: CGPoint(x: v[0], y: v[1]), control2: CGPoint(x: v[2], y: v[3]))
    case "h": path.closeSubpath()
    case "s":
        let o = path.copy(strokingWithWidth: v[0], lineCap: CGLineCap(rawValue: Int32(v[2]))!, lineJoin: CGLineJoin(rawValue: Int32(v[1]))!, miterLimit: 10)
        o.applyWithBlock { e in
            let el = e.pointee; let n = [CGPathElementType.moveToPoint: 1, .addLineToPoint: 1, .addQuadCurveToPoint: 2, .addCurveToPoint: 3, .closeSubpath: 0][el.type]!
            let name = [CGPathElementType.moveToPoint: "m", .addLineToPoint: "l", .addQuadCurveToPoint: "q", .addCurveToPoint: "c", .closeSubpath: "h"][el.type]!
            print(name + (0..<n).map { " \(Double(el.points[$0].x)) \(Double(el.points[$0].y))" }.joined())
        }
        print("--"); path = CGMutablePath()
    default: break
    }
}
