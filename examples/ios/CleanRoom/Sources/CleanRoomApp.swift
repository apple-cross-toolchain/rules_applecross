import SwiftUI

@main
struct CleanRoomApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}

// Exercises both expansions of the @State macro.
struct ContentView: View {
    @State private var taps = 0
    @State var label = "Built by clean-room Apple tools"

    var body: some View {
        Button("\(label) (\(taps))") { taps += 1 }
            .padding()
    }
}
