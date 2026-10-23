import Greeting

public final class Greeter {
    public let name: String

    public init(name: String) {
        self.name = name
    }

    public var message: String {
        "Hello, \(name)! \([1, 2, 3].map { $0 * 2 }.reduce(0, +))"
    }

    public func greet() {
        var text = message + "\n"
        text.withUTF8 { greeting_print($0.baseAddress, $0.count) }
    }
}
