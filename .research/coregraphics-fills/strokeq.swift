// strokeq W H: reads paths like cov ("m","l","c","h"), then "s WIDTH JOIN CAP" strokes the path into one alpha context and
// fills copy(strokingWithWidth:) into another; prints "same" or the number of differing pixels and the outline's element count.
import CoreGraphics
import Foundation
let w = Int(CommandLine.arguments[1])!, h = Int(CommandLine.arguments[2])!
func ctx() -> CGContext { CGContext(data: nil, width: w, height: h, bitsPerComponent: 8, bytesPerRow: w, space: CGColorSpaceCreateDeviceGray(), bitmapInfo: CGImageAlphaInfo.alphaOnly.rawValue)! }
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
        let join = CGLineJoin(rawValue: Int32(v[1]))!, cap = CGLineCap(rawValue: Int32(v[2]))!
        let a = ctx(); a.addPath(path); a.setLineWidth(v[0]); a.setLineJoin(join); a.setLineCap(cap); a.strokePath()
        let outline = path.copy(strokingWithWidth: v[0], lineCap: cap, lineJoin: join, miterLimit: 10)
        let b = ctx(); b.addPath(outline); b.fillPath(using: .winding)
        let pa = a.data!.assumingMemoryBound(to: UInt8.self), pb = b.data!.assumingMemoryBound(to: UInt8.self)
        var diff = 0; for i in 0..<(w * h) where pa[i] != pb[i] { diff += 1 }
        var n = 0; outline.applyWithBlock { _ in n += 1 }
        print(diff == 0 ? "same \(n)" : "diff \(diff) \(n)")
        path = CGMutablePath()
    default: break
    }
}
