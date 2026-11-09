// thin2 W H: "m/l" path then "s WIDTH": prints stroke alpha, then fills of outlines with caps butt/round/square at width max(w,1) and w.
import CoreGraphics
import Foundation
let w = Int(CommandLine.arguments[1])!, h = Int(CommandLine.arguments[2])!
func ctx() -> CGContext { CGContext(data: nil, width: w, height: h, bitsPerComponent: 8, bytesPerRow: w, space: CGColorSpaceCreateDeviceGray(), bitmapInfo: CGImageAlphaInfo.alphaOnly.rawValue)! }
func dump(_ c: CGContext) { let p = c.data!.assumingMemoryBound(to: UInt8.self); print((0..<(w*h)).map { String(p[$0]) }.joined(separator: " ")) }
var path = CGMutablePath()
while let line = readLine() {
    let t = line.split(separator: " ").map(String.init); if t.isEmpty { continue }
    let v = t.dropFirst().map { CGFloat(Double($0)!) }
    switch t[0] {
    case "m": path.move(to: CGPoint(x: v[0], y: v[1]))
    case "l": path.addLine(to: CGPoint(x: v[0], y: v[1]))
    case "s":
        let a = ctx(); a.addPath(path); a.setLineWidth(v[0]); a.strokePath(); dump(a)
        for cap in [CGLineCap.butt, .round, .square] {
            for ww in [1.0, 1.0 + 1.0/256, 1.0 + 1.0/64] {
                let b = ctx(); b.addPath(path.copy(strokingWithWidth: CGFloat(ww), lineCap: cap, lineJoin: .miter, miterLimit: 10)); b.fillPath(); dump(b)
            }
        }
        path = CGMutablePath()
    default: break
    }
}
