// SwiftUI's @Entry macro and the helper macros its expansion uses; see
// docs/clean-room/swiftui-macros.md.

import SwiftDiagnostics
import SwiftSyntax
import SwiftSyntaxMacros

/// What an @Entry property declares.
struct EntryProperty {
    let name: String
    let type: TypeSyntax?
    let defaultValue: ExprSyntax?

    init(_ declaration: some DeclSyntaxProtocol) throws {
        guard let v = declaration.as(VariableDeclSyntax.self), v.bindingSpecifier.tokenKind == .keyword(.var),
              v.bindings.count == 1, let binding = v.bindings.first,
              let pattern = binding.pattern.as(IdentifierPatternSyntax.self) else {
            throw MacroError(description: "'@Entry' can only be applied to a 'var' declaration with a simple name")
        }
        guard binding.accessorBlock == nil else {
            throw MacroError(description: "'@Entry' cannot be applied to a computed property")
        }
        name = pattern.identifier.trimmedDescription
        type = binding.typeAnnotation?.type.trimmed
        defaultValue = binding.initializer?.value.trimmed
    }
}

/// The key protocol for the values type an @Entry extends; FocusedValues'
/// keys have no default value.
private func keyProtocol(_ context: some MacroExpansionContext) throws -> (name: String, hasDefault: Bool) {
    for node in context.lexicalContext {
        guard let ext = node.as(ExtensionDeclSyntax.self) else { continue }
        switch ext.extendedType.trimmedDescription.split(separator: ".").last.map(String.init) {
        case "EnvironmentValues": return ("EnvironmentKey", true)
        case "Transaction": return ("TransactionKey", true)
        case "ContainerValues": return ("ContainerValueKey", true)
        case "FocusedValues": return ("FocusedValueKey", false)
        default: break
        }
    }
    throw MacroError(description: "'@Entry' can only be used in an extension of EnvironmentValues, Transaction, ContainerValues or FocusedValues")
}

struct EntryMessage: DiagnosticMessage {
    let message: String
    let diagnosticID = MessageID(domain: "SwiftUIMacros", id: "Entry")
    let severity = DiagnosticSeverity.warning
}

public struct EntryMacro: AccessorMacro, PeerMacro {
    // Laid out here, not by the expansion's formatter: a default is kept as
    // written (`{}` stays on one line).
    public static var formatMode: FormatMode { .disabled }

    public static func expansion(
        of node: AttributeSyntax,
        providingAccessorsOf declaration: some DeclSyntaxProtocol,
        in context: some MacroExpansionContext
    ) throws -> [AccessorDeclSyntax] {
        let key = "__Key_\(try EntryProperty(declaration).name)"
        return [
            "    get {\n        self[\(raw: key).self]\n    }",
            "    set {\n        self[\(raw: key).self] = newValue\n    }",
            "    _modify {\n        yield &self[\(raw: key).self]\n    }",
        ]
    }

    public static func expansion(
        of node: AttributeSyntax,
        providingPeersOf declaration: some DeclSyntaxProtocol,
        in context: some MacroExpansionContext
    ) throws -> [DeclSyntax] {
        let p = try EntryProperty(declaration)
        let key = try keyProtocol(context)
        // Diagnosed on the declaration's own node (not a trimmed copy), so
        // the warning has its place in the source.
        let declaredType = declaration.as(VariableDeclSyntax.self)?.bindings.first?.typeAnnotation?.type
        if let function = declaredType?.as(FunctionTypeSyntax.self) ?? declaredType?.as(AttributedTypeSyntax.self)?.baseType.as(FunctionTypeSyntax.self) {
            context.diagnose(Diagnostic(node: function, message: EntryMessage(message:
                "Storing a closure in '@Entry var \(p.name)' may invalidate dependents on every update because closures may not be comparable.")))
        }
        guard key.hasDefault else {
            guard let type = p.type else { throw MacroError(description: "'@Entry' in FocusedValues needs a type") }
            // A focused value is optional: its key names the wrapped type.
            let value = type.as(OptionalTypeSyntax.self)?.wrappedType.trimmed ?? type
            // FocusedValueKey is SwiftUI's own, not SwiftUICore's, in Apple's.
            return ["private struct \(raw: "__Key_" + p.name): SwiftUI.\(raw: key.name) {\n    typealias Value = \(value)\n}"]
        }
        guard let defaultValue = p.defaultValue ?? (p.type?.is(OptionalTypeSyntax.self) == true ? "nil" : nil) else {
            throw MacroError(description: "'@Entry' needs a default value")
        }
        let typeAnnotation = p.type.map { ": \($0.trimmedDescription)" } ?? ""
        var members = "    @SwiftUICore.__EntryDefaultValue\n    static var defaultValue\(typeAnnotation) = \(defaultValue.trimmedDescription)"
        // A default made by a call is checked: a class's is made again on
        // every read (_SwiftUIEntryTypeCheck's class overload warns).
        if defaultValue.is(FunctionCallExprSyntax.self) {
            let location = context.location(of: declaration, at: .afterLeadingTrivia, filePathMode: .fileID)
            let file = location?.file.as(StringLiteralExprSyntax.self)?.representedLiteralValue ?? ""
            let line = location?.line.as(IntegerLiteralExprSyntax.self)?.literal.text ?? "1"
            members += """

                    #sourceLocation(file: "\(file)", line: \(line))
                    private static let _typeCheck: Void = #_SwiftUIEntryTypeCheck(Value.self, macroName: "Entry", propertyName: "\(p.name)")
                    #sourceLocation()
                """
        }
        return ["private struct \(raw: "__Key_" + p.name): SwiftUICore.\(raw: key.name) {\n\(raw: members)\n}"]
    }
}

/// On a key's `defaultValue`: a getter that makes the value afresh.
public struct EntryDefaultValueMacro: AccessorMacro {
    public static func expansion(
        of node: AttributeSyntax,
        providingAccessorsOf declaration: some DeclSyntaxProtocol,
        in context: some MacroExpansionContext
    ) throws -> [AccessorDeclSyntax] {
        guard let v = declaration.as(VariableDeclSyntax.self), let value = v.bindings.first?.initializer?.value else {
            throw MacroError(description: "'@__EntryDefaultValue' needs a property with an initial value")
        }
        return ["get {\n\(value.trimmed)\n}"]
    }
}

/// A value type's default: nothing to say.
public struct EntryTypeCheckMacro: ExpressionMacro {
    public static func expansion(
        of node: some FreestandingMacroExpansionSyntax,
        in context: some MacroExpansionContext
    ) throws -> ExprSyntax {
        "()"
    }
}

/// A class's default: made again on every read, which may invalidate the
/// views that read it.
public struct EntryTypeCheckClassMacro: ExpressionMacro {
    public static func expansion(
        of node: some FreestandingMacroExpansionSyntax,
        in context: some MacroExpansionContext
    ) throws -> ExprSyntax {
        let arguments = node.arguments
        let macroName = arguments.first { $0.label?.text == "macroName" }?.expression.as(StringLiteralExprSyntax.self)?.representedLiteralValue ?? "Entry"
        let property = arguments.first { $0.label?.text == "propertyName" }?.expression.as(StringLiteralExprSyntax.self)?.representedLiteralValue ?? ""
        context.diagnose(Diagnostic(node: node, message: EntryMessage(message:
            "Storing a class type in '@\(macroName) var \(property)' may invalidate dependents on every update because the default value is reallocated on every access. Initialize the default value elsewhere and reference it from the '@\(macroName)' declaration.")))
        return "()"
    }
}
