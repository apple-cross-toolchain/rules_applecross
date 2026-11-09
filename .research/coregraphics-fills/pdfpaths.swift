// pdfpaths FILE SCALE: prints each filled path of page 1 in device pixels (y up, origin at the media box's corner) as cov
// commands, one fill per block ending with "f"/"e" then "n". Handles q Q cm m l c v y h re f F f* B B* b b* n; strokes and
// clips are reported on stderr and skipped.
import CoreGraphics
import Foundation
let url = URL(fileURLWithPath: CommandLine.arguments[1]) as CFURL
let scale = CGFloat(Double(CommandLine.arguments[2])!)
guard let doc = CGPDFDocument(url), let page = doc.page(at: 1) else { exit(1) }
let box = page.getBoxRect(.mediaBox)
final class St { var lw: CGFloat = 1; var lws = [CGFloat](); var join: Int32 = 0; var cap: Int32 = 0; var joins = [Int32](); var caps = [Int32](); var ctm = [CGAffineTransform](); var cur = CGAffineTransform.identity; var ops = [String](); var last = CGPoint.zero; var start = CGPoint.zero; var skipped = [String: Int]() }
let st = St()
st.cur = CGAffineTransform(translationX: -box.minX, y: -box.minY).concatenating(CGAffineTransform(scaleX: scale, y: scale))
func dev(_ p: CGPoint) -> String { let q = p.applying(st.cur); return "\(Double(q.x)) \(Double(q.y))" }
func nums(_ s: CGPDFScannerRef, _ n: Int) -> [CGFloat] {
    var out = [CGFloat](repeating: 0, count: n)
    for i in (0..<n).reversed() { var v: CGPDFReal = 0; CGPDFScannerPopNumber(s, &v); out[i] = v }
    return out
}
let table = CGPDFOperatorTableCreate()!
func op(_ name: String, _ f: @escaping CGPDFOperatorCallback) { CGPDFOperatorTableSetCallback(table, name, f) }
op("q") { _, _ in st.ctm.append(st.cur); st.lws.append(st.lw); st.joins.append(st.join); st.caps.append(st.cap) }
op("Q") { _, _ in if let c = st.ctm.popLast() { st.cur = c }; if let l = st.lws.popLast() { st.lw = l }; if let j = st.joins.popLast() { st.join = j }; if let c = st.caps.popLast() { st.cap = c } }
op("w") { s, _ in st.lw = nums(s, 1)[0] }
op("j") { s, _ in st.join = Int32(nums(s, 1)[0]) }
op("J") { s, _ in st.cap = Int32(nums(s, 1)[0]) }
op("cm") { s, _ in let v = nums(s, 6); st.cur = CGAffineTransform(a: v[0], b: v[1], c: v[2], d: v[3], tx: v[4], ty: v[5]).concatenating(st.cur) }
op("m") { s, _ in let v = nums(s, 2); let p = CGPoint(x: v[0], y: v[1]); st.ops.append("m \(dev(p))"); st.last = p; st.start = p }
op("l") { s, _ in let v = nums(s, 2); let p = CGPoint(x: v[0], y: v[1]); st.ops.append("l \(dev(p))"); st.last = p }
op("c") { s, _ in let v = nums(s, 6); st.ops.append("c \(dev(CGPoint(x: v[0], y: v[1]))) \(dev(CGPoint(x: v[2], y: v[3]))) \(dev(CGPoint(x: v[4], y: v[5])))"); st.last = CGPoint(x: v[4], y: v[5]) }
op("v") { s, _ in let v = nums(s, 4); st.ops.append("c \(dev(st.last)) \(dev(CGPoint(x: v[0], y: v[1]))) \(dev(CGPoint(x: v[2], y: v[3])))"); st.last = CGPoint(x: v[2], y: v[3]) }
op("y") { s, _ in let v = nums(s, 4); let e = CGPoint(x: v[2], y: v[3]); st.ops.append("c \(dev(CGPoint(x: v[0], y: v[1]))) \(dev(e)) \(dev(e))"); st.last = e }
op("h") { _, _ in st.ops.append("h"); st.last = st.start }
op("re") { s, _ in let v = nums(s, 4); let x = v[0], y = v[1], w = v[2], h = v[3]
    st.ops += ["m \(dev(CGPoint(x: x, y: y)))", "l \(dev(CGPoint(x: x + w, y: y)))", "l \(dev(CGPoint(x: x + w, y: y + h)))", "l \(dev(CGPoint(x: x, y: y + h)))", "h"] }
func devw() -> Double { let t = st.cur; return Double(st.lw) * sqrt(abs(Double(t.a * t.d - t.b * t.c))) }
func emitStroke() { if !st.ops.isEmpty && strokes { print(st.ops.joined(separator: "\n")); print("S \(devw()) \(st.join) \(st.cap)"); print("n") } }
let strokes = CommandLine.arguments.count > 3
func emit(_ rule: String) { if !st.ops.isEmpty && !strokes { print(st.ops.joined(separator: "\n")); print(rule); print("n") }; st.ops = [] }
op("f") { _, _ in emit("f") }; op("F") { _, _ in emit("f") }; op("f*") { _, _ in emit("e") }
op("B") { _, _ in let o = st.ops; emit("f"); st.ops = o; emitStroke(); st.ops = [] }; op("B*") { _, _ in emit("e"); st.skipped["stroke", default: 0] += 1 }
op("b") { _, _ in st.ops.append("h"); emit("f"); st.skipped["stroke", default: 0] += 1 }; op("b*") { _, _ in st.ops.append("h"); emit("e"); st.skipped["stroke", default: 0] += 1 }
op("S") { _, _ in emitStroke(); st.ops = [] }; op("s") { _, _ in st.ops.append("h"); emitStroke(); st.ops = [] }
op("n") { _, _ in if !st.ops.isEmpty { st.skipped["clip/n", default: 0] += 1 }; st.ops = [] }
op("W") { _, _ in st.skipped["clip", default: 0] += 1 }; op("W*") { _, _ in st.skipped["clip", default: 0] += 1 }
var streams = [CGPDFContentStreamRef]()
op("Do") { s, _ in
    var name: UnsafePointer<CChar>? = nil
    guard CGPDFScannerPopName(s, &name), let name else { return }
    let cs = CGPDFScannerGetContentStream(s)
    guard let obj = CGPDFContentStreamGetResource(cs, "XObject", name) else { st.skipped["xobject?", default: 0] += 1; return }
    var strm: CGPDFStreamRef? = nil
    guard CGPDFObjectGetValue(obj, .stream, &strm), let strm, let dict = CGPDFStreamGetDictionary(strm) else { return }
    var sub: UnsafePointer<CChar>? = nil
    guard CGPDFDictionaryGetName(dict, "Subtype", &sub), let sub, String(cString: sub) == "Form" else { st.skipped["image", default: 0] += 1; return }
    st.ctm.append(st.cur)
    var arr: CGPDFArrayRef? = nil
    if CGPDFDictionaryGetArray(dict, "Matrix", &arr), let arr {
        var m = [CGFloat](repeating: 0, count: 6)
        for i in 0..<6 { var v: CGPDFReal = 0; CGPDFArrayGetNumber(arr, i, &v); m[i] = v }
        st.cur = CGAffineTransform(a: m[0], b: m[1], c: m[2], d: m[3], tx: m[4], ty: m[5]).concatenating(st.cur)
    }
    var res: CGPDFDictionaryRef? = nil
    CGPDFDictionaryGetDictionary(dict, "Resources", &res)
    let child = CGPDFContentStreamCreateWithStream(strm, res ?? CGPDFContentStreamGetResource(cs, "XObject", name).flatMap { _ in nil } ?? dict, cs)
    let sc = CGPDFScannerCreate(child, table, nil)
    CGPDFScannerScan(sc)
    if let c = st.ctm.popLast() { st.cur = c }
}; op("sh") { _, _ in st.skipped["shading", default: 0] += 1 }
op("BT") { _, _ in st.skipped["text", default: 0] += 1 }
let stream = CGPDFContentStreamCreateWithPage(page)
let scanner = CGPDFScannerCreate(stream, table, nil)
CGPDFScannerScan(scanner)
FileHandle.standardError.write("size \(Double(box.width * scale)) \(Double(box.height * scale)) skipped \(st.skipped)\n".data(using: .utf8)!)
