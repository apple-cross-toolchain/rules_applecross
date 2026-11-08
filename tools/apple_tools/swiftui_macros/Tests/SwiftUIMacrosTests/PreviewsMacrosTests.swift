import SwiftSyntaxMacros
import SwiftSyntaxMacrosTestSupport
import XCTest

@testable import PreviewsMacros

final class PreviewsMacrosTests: XCTestCase {
    func testNamedPreview() {
        assertMacroExpansion(
            """
            #Preview("Named") {
                V()
            }
            """,
            expandedSource: """
            @available(iOS 17.0, macOS 14.0, tvOS 17.0, visionOS 1.0, watchOS 10.0, *)
            nonisolated struct __macro_local_15PreviewRegistryfMu_: DeveloperToolsSupport.PreviewRegistry {
                static var fileID: String {
                    "TestModule/test.swift"
                }
                static var line: Int {
                    1
                }
                static var column: Int {
                    1
                }

                @MainActor static func makePreview() throws -> DeveloperToolsSupport.Preview {
                    DeveloperToolsSupport.Preview("Named") {
                        func __b_buildView(@SwiftUI.ViewBuilder body: () -> any SwiftUI.View) -> any SwiftUI.View {
                            body()
                        }
                        return __b_buildView {
                            V()
                        }
                    }
                }
            }
            """,
            macros: ["Preview": SwiftUIView.self]
        )
    }

    func testTraitsAreRefused() {
        assertMacroExpansion(
            "#Preview(traits: .landscapeLeft) { V() }",
            expandedSource: "#Preview(traits: .landscapeLeft) { V() }",
            diagnostics: [DiagnosticSpec(message: "#Preview with traits is not supported by this PreviewsMacros", line: 1, column: 1)],
            macros: ["Preview": SwiftUIView.self]
        )
    }
}
