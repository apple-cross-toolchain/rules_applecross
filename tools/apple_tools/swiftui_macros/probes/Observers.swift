import SwiftUI

struct Observers: View {
    @State private var a = 0 { willSet { print(newValue) } }
    @State private var b = 0 { didSet { print(oldValue, b) } }
    @State var c = 0 { didSet { print(c) } }
    @State private var d = 0 { willSet { print("w") } didSet { print("d") } }
    @State private static var s = 0
    var body: some View { Text("") }
}

struct MoreObservers: View {
    @State private var named = 0 { willSet(next) { print(next) } didSet(previous) { print(previous) } }
    @State var optional: Int? { didSet { print(oldValue as Any) } }
    var body: some View { Text("") }
}
