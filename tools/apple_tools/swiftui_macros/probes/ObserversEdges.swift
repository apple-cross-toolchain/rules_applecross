import SwiftUI

struct EdgeObservers: View {
    @State private var a = 0 { willSet { print(1)
            print(2) } }
    @State private var b = 0 {
        willSet(x) {
            print(x)
            print(2)
        }
    }
    @State private var c = 0 { didSet { print(oldValue)
            print(3) } }
    @State private var d = 0 { willSet {print(newValue)} }
    @State private var e = 0 {
        willSet {
            print(newValue)
        }
    }
    var body: some View { Text("") }
}
