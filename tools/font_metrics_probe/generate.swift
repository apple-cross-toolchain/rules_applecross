// Writes crates/ibtool/src/font_metrics.bin (tools/apple_tools): the Mac's system fonts' text metrics as AppKit measures a
// string (NSString.size(withAttributes:)), for ibtool's Auto Layout of content-sized
// controls. A string's width is the sum of its characters' widths and of each adjacent
// pair's adjustment (its kerning), which holds for the characters below (measured).
//
// Run on a Mac (a leased VM): xcrun swift generate.swift font_metrics.bin
//
// Format, little-endian: "IBFM" u32 version (2), u32 table count; per table: u8 name
// length, name (a PostScript name, UTF-8), f64 size, f64 line height, f64 a label's first
// baseline from its top and f64 its last baseline from its bottom (an NSTextField's
// firstBaselineOffsetFromTop and lastBaselineOffsetFromBottom in the font), u32 character
// count, the characters (u32 scalars), their widths (f64), u32 pair count, pairs (u16
// first index, u16 second index, f64 adjustment).
import AppKit

let characters: [Character] = (32...126).map { Character(UnicodeScalar(UInt8($0))) }
    + ["\u{2019}", "\u{2018}", "\u{201C}", "\u{201D}", "\u{2026}", "\u{2013}", "\u{2014}", "\u{2022}", "\u{00A0}", "\u{00A9}", "\u{00AE}", "\u{00B7}"]

// The fonts AppKit documents name by metaFont, at their sizes: the system font and its
// bold at the sizes documents give, the small system font (11) and the menu font.
var fonts: [NSFont] = []
for size in [9.0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 20, 22, 24, 26, 28, 30, 36] {
    fonts.append(.systemFont(ofSize: size))
    fonts.append(.boldSystemFont(ofSize: size))
}
fonts.append(.menuFont(ofSize: 0))

var out = Data()
func put<T>(_ v: T) { withUnsafeBytes(of: v) { out.append(contentsOf: $0) } }
out.append(contentsOf: Array("IBFM".utf8))
put(UInt32(2).littleEndian)
put(UInt32(fonts.count).littleEndian)
for font in fonts {
    func width(_ s: String) -> Double { Double((s as NSString).size(withAttributes: [.font: font]).width) }
    let name = Array((font.fontDescriptor.postscriptName ?? font.fontName).utf8)
    out.append(UInt8(name.count))
    out.append(contentsOf: name)
    put(Double(font.pointSize).bitPattern.littleEndian)
    put(Double((("X" as NSString).size(withAttributes: [.font: font])).height).bitPattern.littleEndian)
    let label = NSTextField(labelWithString: "X")
    label.font = font
    put(Double(label.firstBaselineOffsetFromTop).bitPattern.littleEndian)
    put(Double(label.lastBaselineOffsetFromBottom).bitPattern.littleEndian)
    put(UInt32(characters.count).littleEndian)
    for c in characters { put(c.unicodeScalars.first!.value.littleEndian) }
    let widths = characters.map { width(String($0)) }
    for w in widths { put(w.bitPattern.littleEndian) }
    var pairs: [(UInt16, UInt16, Double)] = []
    for (i, a) in characters.enumerated() {
        for (j, b) in characters.enumerated() {
            let d = width(String(a) + String(b)) - widths[i] - widths[j]
            if d != 0 { pairs.append((UInt16(i), UInt16(j), d)) }
        }
    }
    put(UInt32(pairs.count).littleEndian)
    for (i, j, d) in pairs {
        put(i.littleEndian)
        put(j.littleEndian)
        put(d.bitPattern.littleEndian)
    }
}
try! out.write(to: URL(fileURLWithPath: CommandLine.arguments[1]))
print("wrote \(fonts.count) tables, \(out.count) bytes")
