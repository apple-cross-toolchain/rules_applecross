import Counter
import SwiftUI

struct ContentView: View {
    @State var counter = Counter()

    var body: some View {
        Button("\(counter.value)") { counter.value += 1 }
    }
}
