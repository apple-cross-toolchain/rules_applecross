import SwiftUI

struct V: View {
    @State private var a = 0
    @State var b: String = "x"
    @State private var c: Bool
    @State private var d: (title: String, body: String)?
    @State var e = [Int]()
    @State(initialValue: 3) var f: Int
    @State(wrappedValue: 4) var g: Int
    init() { _c = State(initialValue: true) }
    var body: some View { Text("\(a)\(b)\(c)\(f)\(g)").onTapGesture { a += 1; $b.wrappedValue = "y" } }
}
