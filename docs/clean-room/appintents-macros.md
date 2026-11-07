# App Intents' schema macros

Since the iOS 18 SDK, AppIntents declares `@AppIntent(schema:)`,
`@AppEntity(schema:)` and `@AppEnum(schema:)` as macros implemented by
Xcode's `AppIntentsMacros` plugin, which a Swift compiler on Linux cannot load.
`tools/apple_tools/swiftui_macros` builds a clean-room `AppIntentsMacros`
executable that the toolchain passes to Linux compiles as
`-load-plugin-executable .../AppIntentsMacros#AppIntentsMacros`. It was written
from the expansions Xcode's compiler prints with `-Xfrontend
-dump-macro-expansions`; Apple's plugin was not disassembled.

## The expansions

For a type with a schema, the macro adds, in this order:

- an extension conforming it to `AssistantSchemaIntent`, `AssistantSchemaEntity`
  or `AssistantSchemaEnum`, holding `static let __appSchemaIntent` (or
  `__appSchemaEntity`, `__appSchemaEnum`) set to the schema's name, such as
  `"audio.playAudio"`; `public` when the type is `public` or `open`;
- an empty extension conforming it to `AppIntent`, `AppEntity` or `AppEnum`;
- an empty extension for each conformance the schema adds: `AudioPlaybackIntent`
  for `.audio.playAudio`, `OpenIntent` for `.system.open`.

A conformance the type already declares is left out. Every extension carries
the type's own `@available` attributes; a type without any gets the schema's
availability on the schema extension alone (`@available(iOS 27.0, macOS 27.0,
tvOS 27.0, watchOS 27.0, visionOS 27.0, *)` for the audio schemas).

The member-attribute role marks the schema's required members declared
without a wrapper, `var` or `let`: an entity's as `@Property`, an intent's as
`@Parameter()`. The required members are the ones Xcode's processor reports
missing from an empty type of the schema; optional ones (a playlist's
`trackCount`, a song's `duration`) and every other member are left alone.

## What is implemented

The schemas probed (one probe each): intents `.audio.addToLibrary`,
`.audio.addToPlaylist`, `.audio.createStation`, `.audio.playAudio`,
`.audio.updateAudioAffinity`, `.audio.warmupAudioQueue` and `.system.open`;
entities `.audio.album`, `.audio.algorithmicRadioStation`, `.audio.artist`,
`.audio.audiobook`, `.audio.playlist`, `.audio.podcastEpisode`,
`.audio.podcastShow`, `.audio.song` and `.audio.warmupAudioQueueResult`; enums
`.audio.affinityState`, `.audio.playbackAttributes` and
`.audio.queueInsertionLocation`. Any other schema is refused, as are the
deprecated `@AssistantIntent`, `@AssistantEntity` and `@AssistantEnum`, and the
other macros of the plugin (`@ComputedProperty`, `@DeferredProperty`,
`_ModelDelegationIntent`).

## @UnionValue

An enum whose cases each hold one unlabelled value of a distinct named type
(three probes) gets two extensions:

- `nonisolated extension E: _IntentValueRepresentable`, with the enum's own
  `@available` attributes: `allIntentValueTypes` listing the case types,
  `asIntentValue` returning the case's value, and a
  `defaultResolverSpecification` of one `<Type>Resolver` per case, each a
  nested `AppIntents.Resolver` that wraps its input in its case;
- `nonisolated extension E: AppUnionValue`, always with iOS, macOS, tvOS,
  watchOS and visionOS 27.0's availability on one line, and an
  `<E>Cases: String, AppUnionValueCasesProviding` enum of the case names.

Members are `public` when the enum is (the resolvers stay internal). Cases
with labelled, several or generic values are refused, and types that repeat
are refused with Xcode's message.
