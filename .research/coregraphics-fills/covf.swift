// covf W H: like cov, but fills into a 32-bit float RGBA context and prints the alpha of each pixel (%.9g), top row first.
import CoreGraphics
import Foundation
let w = Int(CommandLine.arguments[1])!, h = Int(CommandLine.arguments[2])!
let bits = CommandLine.arguments.count > 3 ? Int(CommandLine.arguments[3])! : 32
func page() -> CGContext {
    if bits == 16 {
        return CGContext(data: nil, width: w, height: h, bitsPerComponent: 16, bytesPerRow: w * 8, space: CGColorSpace(name: CGColorSpace.sRGB)!, bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue | CGBitmapInfo.byteOrder16Little.rawValue)!
    }
    return CGContext(data: nil, width: w, height: h, bitsPerComponent: 32, bytesPerRow: w * 16, space: CGColorSpace(name: CGColorSpace.sRGB)!, bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue | CGBitmapInfo.floatComponents.rawValue | CGBitmapInfo.byteOrder32Little.rawValue)!
}
func dump(_ c: CGContext) {
    if bits == 16 {
        let p = c.data!.assumingMemoryBound(to: UInt16.self)
        for y in 0..<h { print((0..<w).map { String(p[(y * c.bytesPerRow / 2) + $0 * 4 + 3]) }.joined(separator: " ")) }
    } else {
        let p = c.data!.assumingMemoryBound(to: Float.self)
        for y in 0..<h { print((0..<w).map { String(format: "%.9g", p[(y * c.bytesPerRow / 4) + $0 * 4 + 3]) }.joined(separator: " ")) }
    }
    print("--")
}
var ctx = page(); ctx.setFillColor(CGColor(srgbRed: 0, green: 0, blue: 0, alpha: 1))
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
    case "n": dump(ctx); ctx = page(); ctx.setFillColor(CGColor(srgbRed: 0, green: 0, blue: 0, alpha: 1))
    default: break
    }
}
dump(ctx)
