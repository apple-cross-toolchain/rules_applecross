// tm SIZE BOLD(0/1) TEXT: the text's width and the font's default line height, in the system font.
import AppKit
let size = CGFloat(Double(CommandLine.arguments[1])!)
let font = CommandLine.arguments[2] == "1" ? NSFont.boldSystemFont(ofSize: size) : NSFont.systemFont(ofSize: size)
let t = CommandLine.arguments[3]
print((t as NSString).size(withAttributes: [.font: font]).width, NSLayoutManager().defaultLineHeight(for: font))
