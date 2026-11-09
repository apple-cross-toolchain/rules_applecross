import Metal
import Foundation
let path = CommandLine.arguments[1]
guard let dev = MTLCreateSystemDefaultDevice() else { print("no device"); exit(1) }
print("device", dev.name)
do {
    let data = try Data(contentsOf: URL(fileURLWithPath: path))
    let lib = try dev.makeLibrary(data: data.withUnsafeBytes { DispatchData(bytes: $0) })
    print("functions", lib.functionNames)
    guard let f = lib.makeFunction(name: "k") else { print("no function k"); exit(1) }
    let pso = try dev.makeComputePipelineState(function: f)
    var input: [Float] = (0..<16).map { Float($0) + 0.5 }
    let buf = dev.makeBuffer(bytes: &input, length: 64, options: .storageModeShared)!
    let q = dev.makeCommandQueue()!, cb = q.makeCommandBuffer()!, enc = cb.makeComputeCommandEncoder()!
    enc.setComputePipelineState(pso); enc.setBuffer(buf, offset: 0, index: 0)
    enc.dispatchThreads(MTLSize(width: 16, height: 1, depth: 1), threadsPerThreadgroup: MTLSize(width: 16, height: 1, depth: 1))
    enc.endEncoding(); cb.commit(); cb.waitUntilCompleted()
    let out = buf.contents().bindMemory(to: Float.self, capacity: 16)
    print("out", (0..<16).map { out[$0] })
} catch { print("error", error) }
