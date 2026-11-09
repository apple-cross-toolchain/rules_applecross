// tw SIZE TEXT...: each text's width in the system font of SIZE points (AppKit's NSString size).
import AppKit
let size = CGFloat(Double(CommandLine.arguments[1])!)
let font = NSFont.systemFont(ofSize: size)
for t in CommandLine.arguments.dropFirst(2) {
    let w = (t as NSString).size(withAttributes: [.font: font]).width
    print(t, w)
}
