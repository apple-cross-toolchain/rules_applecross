import Foundation

public func greetWithDate() -> String {
    let formatter = DateFormatter()
    formatter.dateStyle = .short
    return "Hello at \(formatter.string(from: Date()))"
}
