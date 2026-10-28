# xcstringstool

Reproduced: `xcstringstool compile X.xcstrings --output-directory DIR`, as
rules_apple runs it, for String Catalogs: each language's plain strings
become `<lang>.lproj/X.strings`, and its plural, device and substitution
variations become `<lang>.lproj/X.stringsdict`. Both are XML property lists,
byte-identical to Xcode 27.0's. `toolchain/stubs/xcstringstool.py` is the
implementation. It's pure Python, run by the ported tools' Python, so it
behaves the same on any host.

Provenance: black-box, as [README.md](README.md) describes. Everything below
comes from Xcode 27.0's `xcstringstool` compiling catalogs written for the
purpose and from its `--help`. No published source was needed, and no Apple
binary was read.

Method: Xcode 27.0's `xcstringstool` compiled constructed catalogs (about
380 written to isolate one behaviour each, and about 1,400 generated at
random), and this one compiled the same. Where both succeeded, every output
matched byte for byte, empty `.lproj` directories included; where Xcode's
failed, so did this one. `tests/xcstringstool` keeps 372 of those catalogs
with Xcode's output.

## Reading the catalog

- JSON, with a byte order mark and trailing commas allowed. Comments, single
  quotes and unquoted keys are malformed. Of duplicate keys, the first counts.
- `sourceLanguage` and `version` are required; a `stringUnit` needs `state`
  and `value`. A missing one fails with Xcode's message.

## The output

- Property lists as CFPropertyList writes them: the XML header and doctype,
  tab indentation, keys sorted by their UTF-8 bytes, `<dict/>` for an empty
  dictionary. Only `&`, `<` and `>` are escaped; control characters, quotes
  and newlines are written as they are.
- One directory per language that localizes something, `Base` as
  `base.lproj`. A language whose only localizations were left out has none;
  one whose localizations compiled to nothing has an empty one. A catalog
  with no localizations writes nothing.
- States don't matter (`new`, `needs_review`, `stale` and `translated` are
  all compiled), except that a plain source-language string in state `new`
  is left out, unless the string was added by hand (below).
- `shouldTranslate` and `comment` change nothing.
- A localization with `variations` ignores its `stringUnit`.

## Plurals

A plural becomes `{NSStringLocalizedFormatKey: "%#@value@", value: {
NSStringFormatSpecTypeKey: NSStringPluralRuleType,
NSStringFormatValueTypeKey: <type>, <category>: <form>...}}`, every
category kept, known or not. An empty plural writes nothing.

- **The number:** the forms' numeric specifier.
  - Numeric conversions: `d i o u x X e E f F g G a A D U O`.
  - Length modifiers: `hh h l ll q z t j L`.
  - Specifier syntax: an optional position, the flags `-`, `+` and `0`, width and precision. A space, `#`, `'` or `*` doesn't start a specifier.
  - The value type is the length and conversion (`%05.2f` is `f`).
- **Errors:**
  - No form has a number: "Plural variation requires referencing the number in the string".
  - A form has two different numeric specifiers: "Cannot infer format specifier for string group because more than one numerical specifier was found".
  - A form that is itself varied: "Plural case cannot be further varied".
- **Position:** the number's argument position, counting every specifier in the form.
  - At position 1: the format key is `%#@value@` and the forms are kept.
  - At position N > 1: the format key is `%N$#@value@`, and every unpositioned specifier in every form is given its position.
- **Disagreement:** forms that disagree on the position, or one that uses the number twice.
  - If the source language's plural has an `other` form and agrees on a position, that position is used, without a warning.
  - Otherwise Xcode warns ("Cannot reliably infer argument number…") and uses `%#@value@`, the forms kept.

## Devices

A device variation becomes `{NSStringDeviceSpecificRuleType: {<device>:
...}}` in the `.stringsdict`, each device's string or plural (as above, a
nested `NSStringLocalizedFormatKey` and `value`). Its `other` goes, plain, to
the `.strings`, and may not be varied: "Fallback value cannot be further
varied". Substitutions can't be referenced from a device variant.

## Substitutions

A string with substitutions becomes `{NSStringLocalizedFormatKey: <value>,
<name>: {plural rule}}` with `%arg` replaced by `%<formatSpecifier>`. With
an `argNum`, the reference becomes `%N$#@name@` and `%arg` becomes
`%N$<formatSpecifier>`. A reference to a missing substitution fails. When
none is referenced, Xcode warns and writes the value as a plain string in
the `.stringsdict`.

## Strings added by hand

A string whose `extractionState` is `manual` or `migrated` and which the
source language doesn't localize gets its key as its source-language string.

A `manual` one may also get its key in every other language of the catalog
that doesn't localize it. Some argument has to be involved: in the key when
the source language doesn't localize the string, or in any localization of
it. A key with a specifier String(localized:) doesn't produce rules the
fallback out. String(localized:) produces `d u f g ld lu lld llu lf @`, with
flags, width and precision; `%x`, `%s` and `%i`, for example, aren't among
them. Positions with a gap (`%2$d` alone) count as no specifier.

That's established where every text involved has at most one argument, all
of one kind (integer, floating point or object). Beyond that, Xcode's choice
depends on how the arguments compare across languages in ways not yet
pinned down. Such a string is refused strict, and given the fallback when
some text has an argument when relaxed.

## Not reproduced

Refused strict, compiled as the nearest established form relaxed:

- manual strings beyond the case above;
- unknown keys in the catalog, a string or a localization;
- variations other than `plural` and `device`, and a substitution varied
  other than by plural;
- variations together with substitutions, and positioned substitution
  references (`%1$#@name@`) in the value.

Other subcommands (`sync`, `extract`, `generate-symbols`, ...) and options
(`--format`, `--language`, `--serialization-format binary`) aren't
implemented; rules_apple uses none of them.
