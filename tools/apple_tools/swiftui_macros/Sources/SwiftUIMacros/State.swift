// SwiftUI's @State macro and the helper macros its expansion uses; see
// docs/clean-room/swiftui-macros.md.

import SwiftSyntax
import SwiftSyntaxMacros

struct MacroError: Error, CustomStringConvertible {
    let description: String
}

/// What an @State property declares.
struct StateProperty {
    let name: String
    /// The declared type, with `T!` written `T?`.
    let type: TypeSyntax?
    /// The initial value, and whether it was written (rather than the
    /// implicit nil of an optional).
    let initialValue: ExprSyntax?
    let initialValueIsExplicit: Bool
    /// Access modifiers (without `(set)` details) and `static`/`class`, for
    /// the projected value.
    let modifiers: String
    let isStatic: Bool
    let isPrivate: Bool
    /// The property's observers, which the setter runs around the write.
    let willSet: AccessorDeclSyntax?
    let didSet: AccessorDeclSyntax?

    var usesLazyStorage: Bool { isPrivate && !isStatic && initialValue != nil }

    init(_ declaration: some DeclSyntaxProtocol, attribute: AttributeSyntax) throws {
        guard let v = declaration.as(VariableDeclSyntax.self), v.bindingSpecifier.tokenKind == .keyword(.var) else {
            throw MacroError(description: "'@State' can only be applied to a 'var' declaration")
        }
        guard v.bindings.count == 1, let binding = v.bindings.first,
              let pattern = binding.pattern.as(IdentifierPatternSyntax.self) else {
            throw MacroError(description: "'@State' can only be applied to a 'var' declaration with a simple name")
        }
        name = pattern.identifier.trimmedDescription

        var type = binding.typeAnnotation?.type
        var implicitNil = false
        if let iuo = type?.as(ImplicitlyUnwrappedOptionalTypeSyntax.self) {
            type = TypeSyntax(OptionalTypeSyntax(wrappedType: iuo.wrappedType.trimmed))
            implicitNil = true
        } else if type?.is(OptionalTypeSyntax.self) == true {
            implicitNil = true
        }
        self.type = type?.trimmed

        var initial = binding.initializer?.value
        if initial == nil, case .argumentList(let args) = attribute.arguments, let first = args.first,
           ["initialValue", "wrappedValue"].contains(first.label?.text) {
            initial = first.expression
        }
        initialValueIsExplicit = initial != nil
        if initial == nil, implicitNil {
            initial = "nil"
        }
        initialValue = initial?.trimmed

        switch binding.accessorBlock?.accessors {
        case nil:
            willSet = nil
            didSet = nil
        case .accessors(let list) where list.allSatisfy({ ["willSet", "didSet"].contains($0.accessorSpecifier.text) }):
            willSet = list.first { $0.accessorSpecifier.text == "willSet" }
            didSet = list.first { $0.accessorSpecifier.text == "didSet" }
        default:
            throw MacroError(description: "'@State' cannot be applied to a computed property")
        }

        var kept: [String] = []
        var getterAccess = "internal"
        var isStatic = false
        for m in v.modifiers {
            let word = m.name.text
            switch word {
            case "private", "fileprivate", "internal", "public", "package", "open":
                if m.detail == nil {
                    kept.append(word)
                    getterAccess = word
                }
            case "static", "class":
                kept.append(word)
                isStatic = true
            default:
                break
            }
        }
        modifiers = kept.map { $0 + " " }.joined()
        self.isStatic = isStatic
        isPrivate = getterAccess == "private" || getterAccess == "fileprivate"
    }

    /// `SwiftUICore.State(initialValue: …)`, generic when a type is declared.
    var stateInit: String {
        let generic = type.map { "<\($0.trimmedDescription)>" } ?? ""
        return "SwiftUICore.State\(generic)(initialValue: \(initialValue!.trimmedDescription))"
    }

    /// The lazy storage's initializer: the initial value in a closure.
    var makeStorageThunk: String {
        if let type {
            return "SwiftUICore.State._makeStorage(({ let value: \(type.trimmedDescription) = \(initialValue!.trimmedDescription)\nreturn value }))"
        }
        return "SwiftUICore.State._makeStorage({ \(initialValue!.trimmedDescription) })"
    }

    /// The storage to start from: the initial value made into storage.
    var makeStorageValue: String {
        if let type {
            return "SwiftUICore.State._makeStorage(initialValue: { let x: \(type.trimmedDescription) = \(initialValue!.trimmedDescription)\nreturn x }())"
        }
        return "SwiftUICore.State._makeStorage(initialValue: \(initialValue!.trimmedDescription))"
    }
}

private let base64Alphabet = Array("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".utf8)

/// Standard base64 with padding, as the expansions carry source text.
func base64(_ s: String) -> String {
    let bytes = Array(s.utf8)
    var out: [UInt8] = []
    for i in stride(from: 0, to: bytes.count, by: 3) {
        let chunk = bytes[i..<min(i + 3, bytes.count)]
        let n = chunk.enumerated().reduce(UInt32(0)) { $0 | UInt32($1.element) << (16 - 8 * UInt32($1.offset)) }
        for k in 0..<4 {
            out.append(k <= chunk.count ? base64Alphabet[Int(n >> (18 - 6 * UInt32(k)) & 63)] : UInt8(ascii: "="))
        }
    }
    return String(decoding: out, as: UTF8.self)
}

func unbase64(_ s: String) -> String? {
    var n: UInt32 = 0
    var bits = 0
    var out: [UInt8] = []
    for c in s.utf8 where c != UInt8(ascii: "=") {
        guard let v = base64Alphabet.firstIndex(of: c) else { return nil }
        n = n << 6 | UInt32(v)
        bits += 6
        if bits >= 8 {
            bits -= 8
            out.append(UInt8(n >> UInt32(bits) & 255))
        }
    }
    let text = String(decoding: out, as: UTF8.self)
    return Array(text.utf8) == out ? text : nil
}

func stringArgument(_ attribute: AttributeSyntax) throws -> String {
    guard case .argumentList(let args) = attribute.arguments, let first = args.first,
          let literal = first.expression.as(StringLiteralExprSyntax.self),
          let text = literal.representedLiteralValue else {
        throw MacroError(description: "expected a string literal argument")
    }
    return text
}

func decodedArgument(_ attribute: AttributeSyntax) throws -> String {
    guard let text = unbase64(try stringArgument(attribute)) else {
        throw MacroError(description: "expected a base64-encoded argument")
    }
    return text
}

/// The name of the property a helper macro is attached to.
func propertyName(_ declaration: some DeclSyntaxProtocol) throws -> String {
    guard let v = declaration.as(VariableDeclSyntax.self), let binding = v.bindings.first,
          let pattern = binding.pattern.as(IdentifierPatternSyntax.self) else {
        throw MacroError(description: "expected a 'var' declaration with a simple name")
    }
    return pattern.identifier.text
}

public struct StateMacro: PeerMacro, AccessorMacro {
    public static func expansion(
        of node: AttributeSyntax,
        providingPeersOf declaration: some DeclSyntaxProtocol,
        in context: some MacroExpansionContext
    ) throws -> [DeclSyntax] {
        let p = try StateProperty(declaration, attribute: node)
        let n = p.name
        guard p.usesLazyStorage else {
            if p.initialValue != nil {
                return [
                    "private \(raw: p.isStatic ? "static " : "")var _\(raw: n) = \(raw: p.stateInit)",
                    "@SwiftUICore._PropertyWrapperProjectedValue\n\(raw: p.modifiers)var $\(raw: n) = \(raw: p.stateInit).projectedValue",
                ]
            }
            guard let type = p.type else {
                throw MacroError(description: "'@State' needs a type or an initial value")
            }
            return [
                "private \(raw: p.isStatic ? "static " : "")var _\(raw: n): SwiftUICore.State<\(type)>",
                "\(raw: p.modifiers)var $\(raw: n): SwiftUICore.Binding<\(type)> {\n_\(raw: n).projectedValue\n}",
            ]
        }
        let unique = context.makeUniqueName("_initialStoredValue_").text
        let initialStored = p.initialValueIsExplicit ? p.makeStorageThunk : p.makeStorageValue
        return [
            "private var __\(raw: n) = \(raw: p.makeStorageThunk)",
            """
            @SwiftUICore._StatePropertyWrapperStorage(initialValue: "\(raw: base64(unique))")
            private var _\(raw: n): SwiftUICore.State<_>! = SwiftUICore._stateNil(of: { \(raw: p.stateInit) })
            """,
            "@SwiftUICore._StateProjectedValue\n\(raw: p.modifiers)var $\(raw: n) = \(raw: p.stateInit).projectedValue",
            """
            @SwiftUICore._StateInitialStoredValue("\(raw: base64(initialStored))")
            private static var \(raw: unique) = (\(raw: p.makeStorageValue))
            """,
        ]
    }

    public static func expansion(
        of node: AttributeSyntax,
        providingAccessorsOf declaration: some DeclSyntaxProtocol,
        in context: some MacroExpansionContext
    ) throws -> [AccessorDeclSyntax] {
        let p = try StateProperty(declaration, attribute: node)
        let storage = p.usesLazyStorage ? "__\(p.name)" : "_\(p.name)"
        var accessors: [AccessorDeclSyntax] = []
        // Static properties have no init accessor; lazy storage has one only
        // for the implicit nil of an optional (an initial value written out
        // is the storage's own).
        if !p.isStatic && (!p.usesLazyStorage || !p.initialValueIsExplicit) {
            let value = p.usesLazyStorage
                ? "SwiftUICore.State._makeStorage(initialValue: initialValue)"
                : "SwiftUICore.State(initialValue: initialValue)"
            accessors.append("""
                @storageRestrictions(initializes: \(raw: storage))
                init(initialValue) {
                \(raw: storage) = \(raw: value)
                }
                """)
        }
        accessors.append("get {\n\(raw: storage).wrappedValue\n}")
        accessors.append("\(raw: p.isStatic ? "" : "nonmutating ")set {\n\(raw: setter(p, storage: storage, in: context))\n}")
        return accessors
    }

    /// The write, with the observers' bodies run as closures around it: a
    /// willSet's given the new value, a didSet's the old one (read first)
    /// when it uses it.
    static func setter(_ p: StateProperty, storage: String, in context: some MacroExpansionContext) -> String {
        // As Apple's expansion lays it out: statements at 4; an observer's
        // body starts at 4 when its closure is the setter's first statement
        // and at 8 otherwise, later lines keeping their place in the source
        // relative to that; the closures end at 8. Every line carries its
        // own indentation, which the expansion's formatting keeps.
        func closure(_ head: String, _ accessor: AccessorDeclSyntax, first: Bool, call: String) -> String {
            let indent = first ? 4 : 8
            let text = (accessor.body?.statements.description ?? "").drop { $0 == " " || $0 == "\n" }
            var bodyLines = text.split(separator: "\n", omittingEmptySubsequences: false).map(String.init)
            while let last = bodyLines.last, last.allSatisfy({ $0 == " " }) {
                bodyLines.removeLast()
            }
            let shifted = bodyLines.enumerated().map { k, line in
                let trimmed = line.reversed().drop { $0 == " " }.reversed()
                return String(repeating: " ", count: k == 0 ? indent : indent - 4) + String(trimmed)
            }
            return (["    " + head] + shifted + ["        })" + call]).joined(separator: "\n")
        }
        var lines: [String] = []
        var oldValue: (name: String, local: String)?
        if let didSet = p.didSet {
            let name = didSet.parameters?.name.text ?? "oldValue"
            let uses = didSet.body?.tokens(viewMode: .sourceAccurate).contains { $0.tokenKind == .identifier(name) } ?? false
            if didSet.parameters != nil || uses {
                let local = context.makeUniqueName("_oldValue_").text
                oldValue = (name, local)
                lines.append("    let \(local) = \(storage).wrappedValue")
            }
        }
        if let willSet = p.willSet {
            let name = willSet.parameters?.name.text ?? "newValue"
            lines.append(closure("({ \(name) in", willSet, first: lines.isEmpty, call: "(newValue)"))
        }
        lines.append("    \(storage).wrappedValue = newValue")
        if let didSet = p.didSet {
            if let oldValue {
                lines.append(closure("({ \(oldValue.name) in", didSet, first: false, call: "(\(oldValue.local))"))
            } else {
                lines.append(closure("({", didSet, first: false, call: "()"))
            }
        }
        return lines.joined(separator: "\n")
    }
}

/// On `_name` of lazy storage: a `State` read from and written to `__name`.
public struct StatePropertyWrapperStorageMacro: AccessorMacro {
    public static func expansion(
        of node: AttributeSyntax,
        providingAccessorsOf declaration: some DeclSyntaxProtocol,
        in context: some MacroExpansionContext
    ) throws -> [AccessorDeclSyntax] {
        let storage = "_" + (try propertyName(declaration))
        let initialStored = try decodedArgument(node)
        return [
            """
            @storageRestrictions(initializes: \(raw: storage))
            init(initialValue) {
            if initialValue == nil {
            \(raw: storage) = Self.\(raw: initialStored)
            } else {
            \(raw: storage) = SwiftUICore.State._makeStorage(initialValue: initialValue.wrappedValue)
            }
            }
            """,
            "get {\nSwiftUICore.State(initialValue: \(raw: storage).wrappedValue)\n}",
            """
            set {
            if newValue != nil {
            \(raw: storage) = SwiftUICore.State._makeStorage(initialValue: newValue.wrappedValue)
            }
            }
            """,
        ]
    }
}

/// On `$name` of lazy storage: the binding `__name` projects.
public struct StateProjectedValueMacro: AccessorMacro {
    public static func expansion(
        of node: AttributeSyntax,
        providingAccessorsOf declaration: some DeclSyntaxProtocol,
        in context: some MacroExpansionContext
    ) throws -> [AccessorDeclSyntax] {
        let name = try propertyName(declaration)
        guard name.hasPrefix("$") else { throw MacroError(description: "expected a projected value ($name)") }
        return ["get {\n__\(raw: name.dropFirst()).projectedValue\n}"]
    }
}

/// On the static property holding the storage to start from: computed from
/// the expression its argument encodes.
public struct StateInitialStoredValueMacro: AccessorMacro {
    public static func expansion(
        of node: AttributeSyntax,
        providingAccessorsOf declaration: some DeclSyntaxProtocol,
        in context: some MacroExpansionContext
    ) throws -> [AccessorDeclSyntax] {
        ["get {\n\(raw: try decodedArgument(node))\n}"]
    }
}

/// On `$name` of plain storage: the binding `_name` projects.
public struct ProjectedValueMacro: AccessorMacro {
    public static func expansion(
        of node: AttributeSyntax,
        providingAccessorsOf declaration: some DeclSyntaxProtocol,
        in context: some MacroExpansionContext
    ) throws -> [AccessorDeclSyntax] {
        let name = try propertyName(declaration)
        guard name.hasPrefix("$") else { throw MacroError(description: "expected a projected value ($name)") }
        return ["get {\n_\(raw: name.dropFirst()).projectedValue\n}"]
    }
}
