// cov W H: reads path commands from stdin (one per line: "m x y", "l x y", "c x1 y1 x2 y2 x y", "h", "f" fill nonzero,
// "e" fill even-odd, "n" new page), fills in black on an 8-bit alpha-only context W x H (y up, as CoreGraphics),
// and prints each row of alpha values, top row first.
import CoreGraphics
import Foundation

let w = Int(CommandLine.arguments[1])!, h = Int(CommandLine.arguments[2])!
func page() -> CGContext {
    let c = CGContext(data: nil, width: w, height: h, bitsPerComponent: 8, bytesPerRow: w, space: CGColorSpaceCreateDeviceGray(), bitmapInfo: CGImageAlphaInfo.alphaOnly.rawValue)!
    return c
}
func dump(_ c: CGContext) {
    let p = c.data!.assumingMemoryBound(to: UInt8.self)
    for y in 0..<h { print((0..<w).map { String(p[y * c.bytesPerRow + $0]) }.joined(separator: " ")) }
    print("--")
}
var ctx = page()
while let line = readLine() {
    let t = line.split(separator: " ").map(String.init)
    if t.isEmpty { continue }
    let v = t.dropFirst().map { CGFloat(Double($0)!) }
    switch t[0] {
    case "m": ctx.move(to: CGPoint(x: v[0], y: v[1]))
    case "l": ctx.addLine(to: CGPoint(x: v[0], y: v[1]))
    case "c": ctx.addCurve(to: CGPoint(x: v[4], y: v[5]), control1: CGPoint(x: v[0], y: v[1]), control2: CGPoint(x: v[2], y: v[3]))
    case "h": ctx.closePath()
    case "f": ctx.fillPath(using: .winding)
    case "e": ctx.fillPath(using: .evenOdd)
    case "a": ctx.setAlpha(v[0])
    case "n": dump(ctx); ctx = page()
    default: break
    }
}
dump(ctx)
