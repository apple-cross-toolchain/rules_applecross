// App Intents' @UnionValue, written from the expansions Xcode's compiler
// prints; see docs/clean-room/appintents-macros.md.

import SwiftSyntax
import SwiftSyntaxMacros

public struct _UnionValueMacro: ExtensionMacro {
    // Laid out here, as Xcode's expansion is.
    public static var formatMode: FormatMode { .disabled }

    public static func expansion(
        of node: AttributeSyntax,
        attachedTo declaration: some DeclGroupSyntax,
        providingExtensionsOf type: some TypeSyntaxProtocol,
        conformingTo protocols: [TypeSyntax],
        in context: some MacroExpansionContext
    ) throws -> [ExtensionDeclSyntax] {
        guard let union = declaration.as(EnumDeclSyntax.self) else {
            throw MacroError(description: "@UnionValue can only be applied to an enum")
        }
        var cases: [(name: String, type: String)] = []
        for member in union.memberBlock.members {
            guard let caseDecl = member.decl.as(EnumCaseDeclSyntax.self) else { continue }
            for element in caseDecl.elements {
                guard let parameters = element.parameterClause?.parameters, parameters.count == 1,
                      let parameter = parameters.first, parameter.firstName == nil,
                      let valueType = parameter.type.as(IdentifierTypeSyntax.self), valueType.genericArgumentClause == nil else {
                    throw MacroError(description: "@UnionValue cases with other than one unlabelled associated value of a named type are not supported by this AppIntentsMacros")
                }
                cases.append((element.name.text, valueType.name.text))
            }
        }
        guard !cases.isEmpty else {
            throw MacroError(description: "@UnionValue needs at least one case")
        }
        guard Set(cases.map(\.type)).count == cases.count else {
            throw MacroError(description: "@UnionValue requires that the associated values all have distinct types")
        }

        let name = type.trimmedDescription
        let access = union.modifiers.contains { ["public", "open"].contains($0.name.text) } ? "public " : ""
        let availability = union.attributes.compactMap { element -> String? in
            guard let attribute = element.as(AttributeSyntax.self),
                  attribute.attributeName.trimmedDescription == "available" else { return nil }
            return attribute.trimmedDescription + "\n"
        }.joined()
        let missing = Set(protocols.map { $0.trimmedDescription.split(separator: ".").last.map(String.init) ?? "" })

        var extensions: [DeclSyntax] = []
        if missing.contains("_IntentValueRepresentable") {
            var text = "\(availability)nonisolated extension \(name): _IntentValueRepresentable {\n"
            text += "    \(access)static var allIntentValueTypes: [any _IntentValue.Type] {\n"
            text += "        [\(cases.map { "\($0.type).self" }.joined(separator: ", "))]\n    }\n"
            text += "    \(access)var asIntentValue: any _IntentValue {\n        switch self {\n"
            for c in cases {
                text += "        case let .\(c.name)(_1):\n            return _1\n"
            }
            text += "        }\n    }\n"
            text += "    @ResolverSpecificationBuilder<Self> \(access)static var defaultResolverSpecification: some ResolverSpecification {\n"
            for c in cases {
                text += "        \(c.type)Resolver()\n"
            }
            text += "    }\n"
            for c in cases {
                text += "    struct \(c.type)Resolver: AppIntents.Resolver {\n"
                text += "        func resolve(from input: \(c.type), context: IntentParameterContext<\(name)>) async throws -> \(name)? {\n"
                text += "            .\(c.name)(input)\n        }\n    }\n"
            }
            text += "}"
            extensions.append(DeclSyntax(stringLiteral: text))
        }
        if missing.contains("AppUnionValue") {
            // Xcode's availability for the protocol, not the enum's (two probes).
            var text = "@available(macOS 27.0, *) @available(iOS 27.0, *) @available(watchOS 27.0, *) @available(tvOS 27.0, *) @available(visionOS 27.0, *) nonisolated extension \(name): AppUnionValue {\n"
            text += "    \(access)typealias Cases = \(name)Cases\n"
            text += "    \(access)enum \(name)Cases: String, AppUnionValueCasesProviding {\n"
            text += "        \(access)typealias UnionValue = \(name)\n"
            for c in cases {
                text += "        case `\(c.name)`\n"
            }
            text += "    }\n}"
            extensions.append(DeclSyntax(stringLiteral: text))
        }
        return extensions.compactMap { $0.as(ExtensionDeclSyntax.self) }
    }
}
