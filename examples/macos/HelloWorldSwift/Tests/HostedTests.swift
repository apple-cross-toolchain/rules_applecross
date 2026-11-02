import Foundation
import XCTest

/// A hosted test: injected into the app, which is running, so the main
/// bundle is the app's.
final class HostedTests: XCTestCase {
    func testItRunsInTheApp() {
        XCTAssertEqual(Bundle.main.bundleIdentifier, "com.example.hello-world-macos-swift")
    }
}
