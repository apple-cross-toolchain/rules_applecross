// App Intents' schema macros (@AppIntent, @AppEntity, @AppEnum with a
// schema), written from the expansions Xcode's compiler prints with
// -dump-macro-expansions; see docs/clean-room/appintents-macros.md.

import SwiftCompilerPlugin
import SwiftSyntax
import SwiftSyntaxMacros

@main
struct AppIntentsMacrosPlugin: CompilerPlugin {
    let providingMacros: [Macro.Type] = [
        AppIntentMacro.self,
        AppEntityMacro.self,
        AppEnumMacro.self,
        _UnionValueMacro.self,
    ]
}

struct MacroError: Error, CustomStringConvertible {
    let description: String
}

/// The schemas whose expansions have been observed: each one's extra
/// conformances, after the kind's own, in Xcode's order.
private let schemaConformances: [String: [String: [String]]] = [
    "AppIntent": [
        "audio.addToLibrary": [],
        "audio.addToPlaylist": [],
        "audio.createStation": [],
        "audio.playAudio": ["AudioPlaybackIntent"],
        "audio.updateAudioAffinity": [],
        "audio.warmupAudioQueue": [],
        "system.open": ["OpenIntent"],
    ],
    "AppEntity": [
        "audio.album": [],
        "audio.algorithmicRadioStation": [],
        "audio.artist": [],
        "audio.audiobook": [],
        "audio.playlist": [],
        "audio.podcastEpisode": [],
        "audio.podcastShow": [],
        "audio.song": [],
        "audio.warmupAudioQueueResult": [],
    ],
    "AppEnum": [
        "audio.affinityState": [],
        "audio.playbackAttributes": [],
        "audio.queueInsertionLocation": [],
    ],
]

/// The schemas' required properties (entities) and parameters (intents),
/// which the member-attribute role makes `@Property` and `@Parameter()` when
/// declared without (Xcode's processor's own list of them, one probe each).
private let schemaMembers: [String: [String: Set<String>]] = [
    "AppIntent": [
        "audio.addToLibrary": ["audioEntity"],
        "audio.addToPlaylist": ["audioEntity", "playlist"],
        "audio.createStation": [],
        "audio.playAudio": ["audioEntity", "playbackAttributes", "queueLocation", "warmupAudioQueueResult"],
        "audio.updateAudioAffinity": ["affinityState", "target"],
        "audio.warmupAudioQueue": ["audioEntity", "playbackAttributes"],
        "system.open": ["target"],
    ],
    "AppEntity": [
        "audio.album": ["title", "artistName", "universalProductCode", "artists"],
        "audio.algorithmicRadioStation": ["title", "curatedForMe"],
        "audio.artist": ["name"],
        "audio.audiobook": ["title", "author", "genre", "narrator", "publisher", "seriesTitle", "releaseDate", "purchaseDate"],
        "audio.playlist": ["title", "createdByMe", "curatedForMe", "owner"],
        "audio.podcastEpisode": ["title", "showName", "show", "releaseDate", "duration"],
        "audio.podcastShow": ["title", "showDescription"],
        "audio.song": ["title", "artistName", "albumTitle", "composerName", "internationalStandardRecordingCode", "album", "artists", "composers"],
        "audio.warmupAudioQueueResult": [],
    ],
]

/// The schema a macro names, as "domain.name".
private func schemaName(_ node: AttributeSyntax, kind: String) throws -> String {
    guard case let .argumentList(arguments) = node.arguments, arguments.count == 1,
          let argument = arguments.first, argument.label?.text == "schema",
          let member = argument.expression.as(MemberAccessExprSyntax.self),
          let domain = member.base?.as(MemberAccessExprSyntax.self), domain.base == nil else {
        throw MacroError(description: "'@\(kind)' needs a schema such as .audio.playAudio")
    }
    return "\(domain.declName.baseName.text).\(member.declName.baseName.text)"
}

/// `wrapper` for a schema member declared without it, nothing otherwise.
private func memberAttributes(
    kind: String,
    wrapper: String,
    of node: AttributeSyntax,
    providingAttributesFor member: some DeclSyntaxProtocol
) throws -> [AttributeSyntax] {
    let schema = try schemaName(node, kind: kind)
    guard let names = schemaMembers[kind]?[schema] else {
        throw MacroError(description: "'@\(kind)(schema: .\(schema))' is not supported by this AppIntentsMacros yet")
    }
    guard let variable = member.as(VariableDeclSyntax.self), variable.bindings.count == 1,
          let name = variable.bindings.first?.pattern.as(IdentifierPatternSyntax.self)?.identifier.text,
          names.contains(name) else {
        return []
    }
    let wrapped = variable.attributes.contains { element in
        guard let attribute = element.as(AttributeSyntax.self) else { return false }
        return ["Property", "Parameter"].contains(attribute.attributeName.trimmedDescription.split(separator: ".").last.map(String.init))
    }
    return wrapped ? [] : [AttributeSyntax(stringLiteral: wrapper)]
}

/// A schema macro's expansion: the schema's constant on the AssistantSchema*
/// protocol, then each conformance the type doesn't already have.
private func schemaExtensions(
    kind: String,
    schemaProtocol: String,
    constant: String,
    of node: AttributeSyntax,
    attachedTo declaration: some DeclGroupSyntax,
    providingExtensionsOf type: some TypeSyntaxProtocol,
    conformingTo protocols: [TypeSyntax]
) throws -> [ExtensionDeclSyntax] {
    let schema = try schemaName(node, kind: kind)
    guard let extra = schemaConformances[kind]?[schema] else {
        throw MacroError(description: "'@\(kind)(schema: .\(schema))' is not supported by this AppIntentsMacros yet")
    }

    // The declaration's own availability, or, without any, the schemas' for
    // the schema extension alone (iOS 27's audio schemas, one probe).
    let availability = declaration.attributes.compactMap { element -> String? in
        guard let attribute = element.as(AttributeSyntax.self),
              attribute.attributeName.trimmedDescription == "available" else { return nil }
        return attribute.trimmedDescription + "\n"
    }.joined()
    let schemaAvailability = availability.isEmpty
        ? "@available(iOS 27.0, macOS 27.0, tvOS 27.0, watchOS 27.0, visionOS 27.0, *)\n"
        : availability
    let access = declaration.modifiers.contains { ["public", "open"].contains($0.name.text) } ? "public " : ""

    let missing = Set(protocols.map { $0.trimmedDescription.split(separator: ".").last.map(String.init) ?? "" })
    let name = type.trimmedDescription
    var extensions: [DeclSyntax] = []
    if missing.contains(schemaProtocol) {
        extensions.append("\(raw: schemaAvailability)extension \(raw: name): \(raw: schemaProtocol) {\n    \(raw: access)static let \(raw: constant) = \(literal: schema)\n}")
    }
    for conformance in [kind] + extra where missing.contains(conformance) {
        extensions.append("\(raw: availability)extension \(raw: name): \(raw: conformance) {\n}")
    }
    return extensions.compactMap { $0.as(ExtensionDeclSyntax.self) }
}

public struct AppIntentMacro: ExtensionMacro, MemberAttributeMacro {
    public static func expansion(
        of node: AttributeSyntax,
        attachedTo declaration: some DeclGroupSyntax,
        providingExtensionsOf type: some TypeSyntaxProtocol,
        conformingTo protocols: [TypeSyntax],
        in context: some MacroExpansionContext
    ) throws -> [ExtensionDeclSyntax] {
        try schemaExtensions(kind: "AppIntent", schemaProtocol: "AssistantSchemaIntent", constant: "__appSchemaIntent",
                             of: node, attachedTo: declaration, providingExtensionsOf: type, conformingTo: protocols)
    }

    public static func expansion(
        of node: AttributeSyntax,
        attachedTo declaration: some DeclGroupSyntax,
        providingAttributesFor member: some DeclSyntaxProtocol,
        in context: some MacroExpansionContext
    ) throws -> [AttributeSyntax] {
        try memberAttributes(kind: "AppIntent", wrapper: "@Parameter()", of: node, providingAttributesFor: member)
    }
}

public struct AppEntityMacro: ExtensionMacro, MemberAttributeMacro {
    public static func expansion(
        of node: AttributeSyntax,
        attachedTo declaration: some DeclGroupSyntax,
        providingExtensionsOf type: some TypeSyntaxProtocol,
        conformingTo protocols: [TypeSyntax],
        in context: some MacroExpansionContext
    ) throws -> [ExtensionDeclSyntax] {
        try schemaExtensions(kind: "AppEntity", schemaProtocol: "AssistantSchemaEntity", constant: "__appSchemaEntity",
                             of: node, attachedTo: declaration, providingExtensionsOf: type, conformingTo: protocols)
    }

    public static func expansion(
        of node: AttributeSyntax,
        attachedTo declaration: some DeclGroupSyntax,
        providingAttributesFor member: some DeclSyntaxProtocol,
        in context: some MacroExpansionContext
    ) throws -> [AttributeSyntax] {
        try memberAttributes(kind: "AppEntity", wrapper: "@Property", of: node, providingAttributesFor: member)
    }
}

public struct AppEnumMacro: ExtensionMacro {
    public static func expansion(
        of node: AttributeSyntax,
        attachedTo declaration: some DeclGroupSyntax,
        providingExtensionsOf type: some TypeSyntaxProtocol,
        conformingTo protocols: [TypeSyntax],
        in context: some MacroExpansionContext
    ) throws -> [ExtensionDeclSyntax] {
        try schemaExtensions(kind: "AppEnum", schemaProtocol: "AssistantSchemaEnum", constant: "__appSchemaEnum",
                             of: node, attachedTo: declaration, providingExtensionsOf: type, conformingTo: protocols)
    }
}
