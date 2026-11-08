import SwiftSyntax
import SwiftSyntaxMacros
import SwiftSyntaxMacroExpansion
import SwiftSyntaxMacrosTestSupport
import XCTest

@testable import AppIntentsMacros

final class AppIntentsMacrosTests: XCTestCase {
    private func specs(_ conformances: [TypeSyntax]) -> [String: MacroSpec] {
        [
            "AppIntent": MacroSpec(type: AppIntentMacro.self, conformances: conformances),
            "AppEnum": MacroSpec(type: AppEnumMacro.self, conformances: conformances),
            "AppEntity": MacroSpec(type: AppEntityMacro.self, conformances: conformances),
            "UnionValue": MacroSpec(type: _UnionValueMacro.self, conformances: conformances),
        ]
    }

    func testIntentWithSchemaConformance() {
        assertMacroExpansion(
            """
            @available(iOS 27.0, *)
            @AppIntent(schema: .audio.playAudio)
            public struct PlayIntent {
            }
            """,
            expandedSource: """
            @available(iOS 27.0, *)
            public struct PlayIntent {
            }

            @available(iOS 27.0, *)
            extension PlayIntent: AssistantSchemaIntent {
                public static let __appSchemaIntent = "audio.playAudio"
            }

            @available(iOS 27.0, *)
            extension PlayIntent: AppIntent {
            }

            @available(iOS 27.0, *)
            extension PlayIntent: AudioPlaybackIntent {
            }
            """,
            macroSpecs: specs(["AssistantSchemaIntent", "AppIntent", "AudioPlaybackIntent"])
        )
    }

    func testEnumWithoutAvailability() {
        assertMacroExpansion(
            """
            @AppEnum(schema: .audio.affinityState)
            enum Affinity: String {
                case liked
            }
            """,
            expandedSource: """
            enum Affinity: String {
                case liked
            }

            @available(iOS 27.0, macOS 27.0, tvOS 27.0, watchOS 27.0, visionOS 27.0, *)
            extension Affinity: AssistantSchemaEnum {
                static let __appSchemaEnum = "audio.affinityState"
            }

            extension Affinity: AppEnum {
            }
            """,
            macroSpecs: specs(["AssistantSchemaEnum", "AppEnum"])
        )
    }

    func testUnknownSchemaIsRefused() {
        assertMacroExpansion(
            "@AppIntent(schema: .photos.openAsset)\nstruct OpenAsset {}",
            expandedSource: "struct OpenAsset {}",
            diagnostics: [DiagnosticSpec(message: "'@AppIntent(schema: .photos.openAsset)' is not supported by this AppIntentsMacros yet", line: 1, column: 1)],
            macroSpecs: specs(["AssistantSchemaIntent", "AppIntent"])
        )
    }

    func testPublicUnionValue() {
        assertMacroExpansion(
            """
            @UnionValue
            public enum NoAvail {
                case a(String)
                case b(Int)
            }
            """,
            expandedSource: """
            public enum NoAvail {
                case a(String)
                case b(Int)
            }

            nonisolated extension NoAvail: _IntentValueRepresentable {
                public static var allIntentValueTypes: [any _IntentValue.Type] {
                    [String.self, Int.self]
                }
                public var asIntentValue: any _IntentValue {
                    switch self {
                    case let .a(_1):
                        return _1
                    case let .b(_1):
                        return _1
                    }
                }
                @ResolverSpecificationBuilder<Self> public static var defaultResolverSpecification: some ResolverSpecification {
                    StringResolver()
                    IntResolver()
                }
                struct StringResolver: AppIntents.Resolver {
                    func resolve(from input: String, context: IntentParameterContext<NoAvail>) async throws -> NoAvail? {
                        .a(input)
                    }
                }
                struct IntResolver: AppIntents.Resolver {
                    func resolve(from input: Int, context: IntentParameterContext<NoAvail>) async throws -> NoAvail? {
                        .b(input)
                    }
                }
            }

            @available(macOS 27.0, *) @available(iOS 27.0, *) @available(watchOS 27.0, *) @available(tvOS 27.0, *) @available(visionOS 27.0, *) nonisolated extension NoAvail: AppUnionValue {
                public typealias Cases = NoAvailCases
                public enum NoAvailCases: String, AppUnionValueCasesProviding {
                    public typealias UnionValue = NoAvail
                    case `a`
                    case `b`
                }
            }
            """,
            macroSpecs: specs(["_IntentValueRepresentable", "AppUnionValue"])
        )
    }

    func testUnionValueWithAvailability() {
        assertMacroExpansion(
            """
            @available(iOS 27.0, *)
            @UnionValue
            enum Audio {
                case song(Track)
            }
            """,
            expandedSource: """
            @available(iOS 27.0, *)
            enum Audio {
                case song(Track)
            }

            @available(iOS 27.0, *)
            nonisolated extension Audio: _IntentValueRepresentable {
                static var allIntentValueTypes: [any _IntentValue.Type] {
                    [Track.self]
                }
                var asIntentValue: any _IntentValue {
                    switch self {
                    case let .song(_1):
                        return _1
                    }
                }
                @ResolverSpecificationBuilder<Self> static var defaultResolverSpecification: some ResolverSpecification {
                    TrackResolver()
                }
                struct TrackResolver: AppIntents.Resolver {
                    func resolve(from input: Track, context: IntentParameterContext<Audio>) async throws -> Audio? {
                        .song(input)
                    }
                }
            }

            @available(macOS 27.0, *) @available(iOS 27.0, *) @available(watchOS 27.0, *) @available(tvOS 27.0, *) @available(visionOS 27.0, *) nonisolated extension Audio: AppUnionValue {
                typealias Cases = AudioCases
                enum AudioCases: String, AppUnionValueCasesProviding {
                    typealias UnionValue = Audio
                    case `song`
                }
            }
            """,
            macroSpecs: specs(["_IntentValueRepresentable", "AppUnionValue"])
        )
    }

    func testEntitySchemaPropertiesBecomeProperties() {
        assertMacroExpansion(
            """
            @available(iOS 27.0, *)
            @AppEntity(schema: .audio.album)
            struct Album {
                let id: String
                @Property
                var title: String
                let universalProductCode: String?
                var artists: [Artist]
                var extra: String
            }
            """,
            expandedSource: """
            @available(iOS 27.0, *)
            struct Album {
                let id: String
                @Property
                var title: String
                @Property
                let universalProductCode: String?
                @Property
                var artists: [Artist]
                var extra: String
            }

            @available(iOS 27.0, *)
            extension Album: AssistantSchemaEntity {
                static let __appSchemaEntity = "audio.album"
            }

            @available(iOS 27.0, *)
            extension Album: AppEntity {
            }
            """,
            macroSpecs: specs(["AssistantSchemaEntity", "AppEntity"])
        )
    }
}
