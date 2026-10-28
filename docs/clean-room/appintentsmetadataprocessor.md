# appintentsmetadataprocessor

Reproduced: `appintentsmetadataprocessor --compile-time-extraction`, as
rules_apple runs it for a bundle's `app_intents`. It reads the Swift
compiler's constant values (`.swiftconstvalues`, from `-emit-const-values-path`
with rules_swift's protocol list) and writes `Metadata.appintents/`
(`extract.actionsdata` and `version.json`) with the same content Xcode 27.0
writes. rules_apple then rewrites both files with their keys sorted, so the
content is what has to match, not the layout.
`toolchain/stubs/appintentsmetadataprocessor.py` is the implementation, in
pure Python.

Method: Swift sources, each built to isolate one behaviour, plus 300
generated at random. Xcode 27.0's `swiftc` compiled them, Xcode's
`appintentsmetadataprocessor` processed the constant values (with
`SWIFT_DETERMINISTIC_HASHING=1`, as rules_apple runs it), and this one
processed the same constant values. All 378 outputs matched, Xcode's errors
included. Swift 6.4 on Linux emits the same constant values as Xcode's
compiler for these sources. An app built entirely on Linux
(`//examples/ios/HelloWorldSwift`) carries metadata identical to what Xcode's
processor writes from the same input. `tests/appintentsmetadataprocessor`
keeps 87 of the cases with Xcode's output.

## What it covers

- **Intents** (`AppIntent`):
  - `title`, and `description` (with `categoryName` and `searchKeywords`);
  - `openAppWhenRun`, `isDiscoverable`, `authenticationPolicy`;
  - `@available` introductions (iOS, macOS);
  - the `perform()` result's protocols (`ProvidesDialog`, `ShowsSnippetView`, `OpensIntent`) and a `ReturnsValue` of a primitive type.
- **Parameters** (`@Parameter`) of `String`, `Int`, `Bool`, `Double`, `Date`, `URL`, an `AppEnum`, and their optionals and arrays:
  - title, description, default value;
  - an `Int`'s `inclusiveRange`;
  - a `String`'s `inputOptions`.
- **Parameter summaries:** `Summary` with interpolated parameters and other parameters, and `When(..., .equalTo, Bool)` with `otherwise`.
- **`AppEnum`:** type and case display representations (title, subtitle, SF Symbol image), and a type's numeric format.
- **`AppEntity`:** its default query, `@Property`s of the primitive types and their optionals, and parameters and results of entity type.
  - Such a parameter's `dynamicOptionsSupport` is 1 when the entity's query is an `EntityQuery` and 2 when it is an `EntityStringQuery`.
- **Queries:** `EntityQuery` (capabilities 66) and `EntityStringQuery` (70), each as its entity's default query.
- **Type names:** `TypeDisplayRepresentation(name:)` with a literal name and no numeric format also gives the type a numeric format, the escaped name; a bare literal or a `LocalizedStringResource` name gives none.
- **`AppShortcutsProvider`:**
  - shortcuts with phrases interpolating `.applicationName`, short titles and symbol names;
  - `shortcutTileColor`;
  - Xcode's error for a shortcut of an intent that isn't discoverable.

## Details

- **Localized strings:**
  - A `LocalizedStringResource` is `{key, alternatives: []}`, with `defaultValue` and `table` when given (not its comment).
  - A parameter summary's text, a literal `TypeDisplayRepresentation` and a numeric format are written as Swift escapes them in source: `String.debugDescription` without its quotes (`\"`, `\\`, `\t`, `\n`, `\r`, `\0`, unprintable scalars as `\u{hex}`). Other strings are written as they are.
- **Resolvable input types:**
  - A parameter's `resolvableInputTypes` is a set, so its order is Swift's.
  - It is fixed per type: `String` [Int, [Int], String]; `Int` and `Double` [Int, Double, String]; `URL` [URL, String]; `Bool`, `Date` themselves; enums and arrays none.
- **Type-specific metadata:** a parameter's `typeSpecificMetadata` is a flattened `Dictionary<String, _>`, in Swift's enumeration order under deterministic hashing:
  - SipHash-1-3 keyed by (the table's scale, 0) over the key's UTF-8 and a 0xFF terminator;
  - a power-of-two table at most 3/4 full;
  - linear probing in insertion order.

  All collision-free four-key orders were predicted exactly. Where keys collide, insertion is default value, range type, lower and upper bounds, capitalization, keyboard type, multiline.
- **Encodings:**
  - default values: `{string}`, `{int}` (Bools as 0/1), `{double}`, with integral doubles written without a fraction, as JSONSerialization writes them;
  - keyboard types: asciiCapable 0, numbersAndPunctuation 1, URL 2, numberPad 3, default 4;
  - capitalization: none 0, words 1, sentences 2, allCharacters 3;
  - tile colors: red 0 through grayBrown 14 (the default).

## Not reproduced

Refused strict; relaxed, the type is left out with a warning:

- `TransientAppEntity`, queries other than `EntityQuery` and `EntityStringQuery` (`EnumerableEntityQuery`, `EntityPropertyQuery`, ...), entity properties of entity or enum type, and the other App Intents protocols;
- parameters of other types, dynamic options, other summary or `When` forms;
- availability other than an introduced version, and platforms other than iOS and macOS;
- values the compiler couldn't extract (computed at run time).

`--metadata-file-list`, `--static-metadata-file-list`, `--dependency-file`
and `--stringsdata-file` are refused; rules_apple passes none of them.

App Shortcuts Flexible Matching (`appintentsnltrainingprocessor`, on by
default in rules_apple through `app_intents_flexible_matching`) trains a
language model on the shortcuts' phrases and isn't reproduced. Off a Mac it
fails, saying to set `app_intents_flexible_matching = False`; relaxed, its
assets are left out.
