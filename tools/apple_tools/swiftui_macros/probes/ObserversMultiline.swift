import SwiftUI

struct MultilineObservers: View {
    @State private var a = 0 {
        willSet {
            print(1)
            if newValue > 1 {
                print(2)
            }
        }
        didSet {
            print(oldValue)
            print(3)
        }
    }
    @State var b = 0 {
        didSet { print("b") }
    }
    var body: some View { Text("") }
}
