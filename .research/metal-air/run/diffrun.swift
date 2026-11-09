// Run each kernel from two metallibs on the same inputs and compare outputs.
// usage: diffrun ours.metallib xcode.metallib spec.json
import Metal
import Foundation

struct Buf: Decodable { let index: Int; let bytes: Int; let fill: String; let hex: String? }
struct Tex: Decodable { let index: Int; let width: Int; let height: Int; let fill: String }
struct Const: Decodable { let index: Int; let type: String; let value: Double }
struct Spec: Decodable {
    let function: String; let grid: [Int]; let group: [Int]
    let buffers: [Buf]; let threadgroup: [[Int]]?; let textures: [Tex]?; let constants: [Const]?
}

var seed: UInt32 = 12345
func rnd() -> UInt32 { seed = seed &* 1664525 &+ 1013904223; return seed }
func fill(_ p: UnsafeMutableRawPointer, _ n: Int, _ how: String) {
    switch how {
    case "float": for i in 0..<n/4 { p.storeBytes(of: Float(rnd() >> 8) / Float(1 << 24) * 2 - 0.5, toByteOffset: i * 4, as: Float.self) }
    case "bytes": for i in 0..<n { p.storeBytes(of: UInt8(truncatingIfNeeded: rnd() >> 24), toByteOffset: i, as: UInt8.self) }
    default: memset(p, 0, n)
    }
}

func run(_ dev: MTLDevice, _ libPath: String, _ s: Spec) throws -> [Data] {
    seed = 12345
    let data = try Data(contentsOf: URL(fileURLWithPath: libPath))
    let lib = try dev.makeLibrary(data: data.withUnsafeBytes { DispatchData(bytes: $0) })
    let fc = MTLFunctionConstantValues()
    for c in s.constants ?? [] {
        switch c.type {
        case "bool": var v = c.value != 0; fc.setConstantValue(&v, type: .bool, index: c.index)
        case "float": var v = Float(c.value); fc.setConstantValue(&v, type: .float, index: c.index)
        case "int": var v = Int32(c.value); fc.setConstantValue(&v, type: .int, index: c.index)
        case "uint": var v = UInt32(c.value); fc.setConstantValue(&v, type: .uint, index: c.index)
        case "half": var v = Float16(c.value); fc.setConstantValue(&v, type: .half, index: c.index)
        case "short": var v = Int16(c.value); fc.setConstantValue(&v, type: .short, index: c.index)
        case "uchar": var v = UInt8(c.value); fc.setConstantValue(&v, type: .uchar, index: c.index)
        case "float4": var v = SIMD4<Float>(repeating: Float(c.value)); fc.setConstantValue(&v, type: .float4, index: c.index)
        case "int2": var v = SIMD2<Int32>(repeating: Int32(c.value)); fc.setConstantValue(&v, type: .int2, index: c.index)
        default: fatalError("constant type \(c.type)")
        }
    }
    let f = try lib.makeFunction(name: s.function, constantValues: fc)
    let pso = try dev.makeComputePipelineState(function: f)
    let q = dev.makeCommandQueue()!, cb = q.makeCommandBuffer()!, enc = cb.makeComputeCommandEncoder()!
    enc.setComputePipelineState(pso)
    var bufs: [MTLBuffer] = []
    for b in s.buffers {
        let mb = dev.makeBuffer(length: b.bytes, options: .storageModeShared)!
        if let hex = b.hex {
            var bytes = [UInt8](); var i = hex.startIndex
            while i < hex.endIndex { let j = hex.index(i, offsetBy: 2); bytes.append(UInt8(hex[i..<j], radix: 16)!); i = j }
            memcpy(mb.contents(), bytes, bytes.count)
        } else { fill(mb.contents(), b.bytes, b.fill) }
        enc.setBuffer(mb, offset: 0, index: b.index); bufs.append(mb)
    }
    for t in s.threadgroup ?? [] { enc.setThreadgroupMemoryLength(t[1], index: t[0]) }
    var texs: [MTLTexture] = []
    for t in s.textures ?? [] {
        let d = MTLTextureDescriptor.texture2DDescriptor(pixelFormat: .rgba32Float, width: t.width, height: t.height, mipmapped: false)
        d.usage = [.shaderRead, .shaderWrite]; d.storageMode = .shared
        let tex = dev.makeTexture(descriptor: d)!
        var px = [Float](repeating: 0, count: t.width * t.height * 4)
        px.withUnsafeMutableBytes { fill($0.baseAddress!, $0.count, t.fill) }
        tex.replace(region: MTLRegionMake2D(0, 0, t.width, t.height), mipmapLevel: 0, withBytes: px, bytesPerRow: t.width * 16)
        enc.setTexture(tex, index: t.index); texs.append(tex)
    }
    enc.dispatchThreads(MTLSize(width: s.grid[0], height: s.grid[1], depth: s.grid[2]),
                        threadsPerThreadgroup: MTLSize(width: s.group[0], height: s.group[1], depth: s.group[2]))
    enc.endEncoding(); cb.commit(); cb.waitUntilCompleted()
    if let e = cb.error { throw e }
    var out = bufs.map { Data(bytes: $0.contents(), count: $0.length) }
    for t in texs {
        var px = [Float](repeating: 0, count: t.width * t.height * 4)
        t.getBytes(&px, bytesPerRow: t.width * 16, from: MTLRegionMake2D(0, 0, t.width, t.height), mipmapLevel: 0)
        out.append(px.withUnsafeBytes { Data($0) })
    }
    return out
}

let a = CommandLine.arguments
let dev = MTLCreateSystemDefaultDevice()!
let specs = try JSONDecoder().decode([Spec].self, from: Data(contentsOf: URL(fileURLWithPath: a[3])))
var failed = 0
for s in specs {
    do {
        let x = try run(dev, a[1], s), y = try run(dev, a[2], s)
        var worst: Float = 0; var same = true
        for (p, q) in zip(x, y) where p != q {
            same = false
            p.withUnsafeBytes { pb in q.withUnsafeBytes { qb in
                let pf = pb.bindMemory(to: Float.self), qf = qb.bindMemory(to: Float.self)
                for i in 0..<pf.count { worst = max(worst, abs(pf[i] - qf[i])) }
            } }
        }
        let nz = x.map { $0.reduce(0) { $1 != 0 ? $0 + 1 : $0 } }
        print(same ? "SAME   " : String(format: "DIFFER %.3g", worst), s.function, "nonzero bytes per output:", nz)
        if !same { failed += 1 }
    } catch { print("ERROR  ", s.function, error); failed += 1 }
}
exit(failed == 0 ? 0 : 1)
