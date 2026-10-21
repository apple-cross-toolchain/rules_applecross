@testable import SwiftUIMacros
import XCTest

final class Base64Tests: XCTestCase {
    func testRoundTrip() {
        // From Xcode's expansion of `@State private var a = 0`.
        XCTAssertEqual(base64("SwiftUICore.State._makeStorage({ 0 })"), "U3dpZnRVSUNvcmUuU3RhdGUuX21ha2VTdG9yYWdlKHsgMCB9KQ==")
        for s in ["", "a", "ab", "abc", "abcd", "{ let x: Int? = nil\nreturn x }()"] {
            XCTAssertEqual(unbase64(base64(s)), s)
        }
    }
}
