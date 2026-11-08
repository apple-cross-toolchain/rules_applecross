// SwiftUI's #Preview of a view, written from the expansions Xcode's
// compiler prints with -dump-macro-expansions; see
// docs/clean-room/previews-macros.md.

import SwiftCompilerPlugin
import SwiftSyntax
import SwiftSyntaxMacros

@main
struct PreviewsMacrosPlugin: CompilerPlugin {
    let providingMacros: [Macro.Type] = [
        SwiftUIView.self,
    ]
}

struct MacroError: Error, CustomStringConvertible {
    let description: String
}

/// `#Preview(_ name: String? = nil, body:)`: a preview registry type, named
/// uniquely, that makes the preview of the body's view.
public struct SwiftUIView: DeclarationMacro {
    public static func expansion(
        of node: some FreestandingMacroExpansionSyntax,
        in context: some MacroExpansionContext
    ) throws -> [DeclSyntax] {
        var name: ExprSyntax?
        var body = node.trailingClosure
        for (index, argument) in node.arguments.enumerated() {
            switch argument.label?.text {
            case nil where index == 0:
                name = argument.expression.trimmed
            case "body" where body == nil:
                guard let closure = argument.expression.as(ClosureExprSyntax.self) else {
                    throw MacroError(description: "#Preview's body must be a closure written in place")
                }
                body = closure
            default:
                throw MacroError(description: "#Preview with traits is not supported by this PreviewsMacros")
            }
        }
        guard let body, node.additionalTrailingClosures.isEmpty else {
            throw MacroError(description: "#Preview needs a body")
        }

        let location = context.location(of: node, at: .afterLeadingTrivia, filePathMode: .fileID)
        let file = location?.file.as(StringLiteralExprSyntax.self)?.representedLiteralValue ?? ""
        let line = location?.line.as(IntegerLiteralExprSyntax.self)?.literal.text ?? "1"
        let column = location?.column.as(IntegerLiteralExprSyntax.self)?.literal.text ?? "1"
        let registry = context.makeUniqueName("PreviewRegistry")
        let preview = name.map { "DeveloperToolsSupport.Preview(\($0))" } ?? "DeveloperToolsSupport.Preview"
        let statements = body.statements.map { $0.trimmedDescription }.joined(separator: "\n")

        return [
            """
            @available(iOS 17.0, macOS 14.0, tvOS 17.0, visionOS 1.0, watchOS 10.0, *)
            nonisolated struct \(registry): DeveloperToolsSupport.PreviewRegistry {
                static var fileID: String {
                    \(literal: file)
                }
                static var line: Int {
                    \(raw: line)
                }
                static var column: Int {
                    \(raw: column)
                }

                @MainActor static func makePreview() throws -> DeveloperToolsSupport.Preview {
                    \(raw: preview) {
                        func __b_buildView(@SwiftUI.ViewBuilder body: () -> any SwiftUI.View) -> any SwiftUI.View {
                            body()
                        }
                        return __b_buildView {
                            \(raw: statements)
                        }
                    }
                }
            }
            """,
        ]
    }
}
