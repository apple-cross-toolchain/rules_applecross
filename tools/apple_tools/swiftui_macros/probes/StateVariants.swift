import SwiftUI

struct W: View {
    @State private var p1: Int = 0
    @State fileprivate var p2 = 0
    @State private(set) var p3 = 0
    @State var p4: Int?
    @State private var p5: Int? = nil
    @State private var p6: Optional<Int>
    @State private var p7 = Model()
    @State private var p8 = [1, 2].map { $0 * 2 }
    @State private var p9: Int!
    @State private var p10: [String: Int] = [:]
    @State public var p11 = 1
    @State private static var p12 = 1
    var body: some View { Text("") }
}
final class Model {}
