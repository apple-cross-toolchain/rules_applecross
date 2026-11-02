import XCTest

/// A logic test: no host app, run by `xctest` alone. XCTUnwrap and
/// XCTAssertEqual are libXCTestSwiftSupport's, which a test built on Linux
/// finds on the Mac by its run path into the installed Xcode.
final class LogicTests: XCTestCase {
    func testArithmetic() throws {
        let value: Int? = 21
        XCTAssertEqual(try XCTUnwrap(value) * 2, 42)
    }
}
