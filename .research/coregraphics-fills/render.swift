// render IN.pdf SCALE OUT.png: draws the PDF's first page with CoreGraphics into an 8-bit sRGB
// premultiplied context at SCALE, as a PNG (public API only).
import CoreGraphics
import Foundation
import ImageIO
import UniformTypeIdentifiers

let args = CommandLine.arguments
let doc = CGPDFDocument(URL(fileURLWithPath: args[1]) as CFURL)!
let page = doc.page(at: 1)!
let scale = CGFloat(Double(args[2])!)
let box = page.getBoxRect(.mediaBox)
let w = Int((box.width * scale).rounded(.up)), h = Int((box.height * scale).rounded(.up))
let ctx = CGContext(data: nil, width: w, height: h, bitsPerComponent: 8, bytesPerRow: 0,
                    space: CGColorSpace(name: CGColorSpace.sRGB)!,
                    bitmapInfo: CGImageAlphaInfo.premultipliedFirst.rawValue | CGBitmapInfo.byteOrder32Little.rawValue)!
ctx.scaleBy(x: scale, y: scale)
ctx.drawPDFPage(page)
let img = ctx.makeImage()!
let dest = CGImageDestinationCreateWithURL(URL(fileURLWithPath: args[3]) as CFURL, UTType.png.identifier as CFString, 1, nil)!
CGImageDestinationAddImage(dest, img, nil)
CGImageDestinationFinalize(dest)
