// thin W H: "m/l" path then "s WIDTH": prints stroke alpha at WIDTH, and the fill of the 1-px outline, as two rows-blocks.
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
        for ww in [1.0, Double(v[0])] {
            let b = ctx(); b.addPath(path.copy(strokingWithWidth: CGFloat(ww), lineCap: .butt, lineJoin: .miter, miterLimit: 10)); b.fillPath(); dump(b)
        }
        path = CGMutablePath()
    default: break
    }
}
