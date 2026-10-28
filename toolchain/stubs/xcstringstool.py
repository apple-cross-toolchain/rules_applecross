#!/usr/bin/env python3
"""xcstringstool off a Mac: `xcstringstool compile` of a String Catalog, as
rules_apple runs it (`compile --output-directory DIR X.xcstrings`); see
docs/clean-room/xcstringstool.md.

Each language's plain strings go to <lang>.lproj/X.strings and its plural,
device and substitution variations to <lang>.lproj/X.stringsdict, both XML
property lists written as Xcode 27's xcstringstool writes them. What hasn't
been established against it is refused, strict (the default); relaxed
(RULES_APPLECROSS_RESOURCE_OUTPUT_MODE=relaxed, --config=relaxed), it is
written as the nearest established form, with a warning.
"""

import json
import os
import re
import sys

STRICT = os.environ.get("RULES_APPLECROSS_RESOURCE_OUTPUT_MODE") != "relaxed"

# A format specifier: an optional position, the flags -, + and 0, width,
# precision, a length and a conversion. Others (space, #, ', *) are not
# specifiers to xcstringstool.
SPECIFIER = re.compile(r"%(?:(\d+)\$)?([-+0]*)(\d*)(?:\.(\d*))?(hh|h|ll|l|q|z|t|j|L)?([diouxXeEfFgGaADUO@scCSp])")
NUMERIC = set("diouxXeEfFgGaADUO")
SUBSTITUTION = re.compile(r"%(?:(\d+)\$)?#@([^@]*)@")
PLURAL_CATEGORIES = ("zero", "one", "two", "few", "many", "other")


class Failure(Exception):
    pass


class Context:
    def __init__(self, path):
        self.path = path
        self.errors = []

    def where(self, language, key, *path):
        return "({}: {})".format(language, "|==|".join((key,) + path) if path else key)

    def error(self, message, language, key, *path):
        self.errors.append("{}: error: {} {}".format(self.path, message, self.where(language, key, *path)))

    def warning(self, message, language, key, *path):
        print("{}: warning: {} {}".format(self.path, message, self.where(language, key, *path)), file=sys.stderr)

    def unestablished(self, what, language, key, *path):
        """Input whose output hasn't been compared with Xcode's."""
        message = "{} is not reproduced as Xcode compiles it".format(what)
        if STRICT:
            self.errors.append("{}: error: {}; build with --config=relaxed (RULES_APPLECROSS_RESOURCE_OUTPUT_MODE=relaxed) to compile it anyway {}".format(
                self.path, message, self.where(language, key, *path)))
            raise Failure()
        self.warning(message + "; relaxed: compiling it anyway", language, key, *path)


def specifiers(text):
    """(position or None, the specifier, its value type, numeric) of each
    format specifier in `text`, `%%` skipped."""
    out = []
    i = 0
    while True:
        i = text.find("%", i)
        if i < 0:
            return out
        if text.startswith("%%", i):
            i += 2
            continue
        m = SPECIFIER.match(text, i)
        if m:
            position, _, _, _, length, conversion = m.groups()
            out.append((int(position) if position else None, m.group(0), (length or "") + conversion, conversion in NUMERIC))
            i = m.end()
        else:
            i += 1


def positional(text):
    """`text` with each unpositioned specifier given its position."""
    pieces = []
    i = last = 0
    index = 0
    while True:
        i = text.find("%", i)
        if i < 0:
            break
        if text.startswith("%%", i):
            i += 2
            continue
        m = SPECIFIER.match(text, i)
        if not m:
            i += 1
            continue
        index += 1
        if m.group(1) is None:
            pieces.append(text[last:i + 1] + "{}$".format(index))
            last = i + 1
        i = m.end()
    pieces.append(text[last:])
    return "".join(pieces)


def string_unit(ctx, unit, language, key, *path):
    if not isinstance(unit, dict):
        ctx.unestablished("A stringUnit that isn't an object", language, key, *path)
    for field in ("state", "value"):
        if field not in unit:
            ctx.errors.append("{}: error: Missing required key '{}' inside 'strings' -> '{}' -> 'localizations' -> '{}' -> {}".format(
                ctx.path, field, key, language, " -> ".join("'{}'".format(p) for p in path + ("stringUnit",))))
            raise Failure()
    extra = set(unit) - {"state", "value"}
    if extra:
        ctx.unestablished("A stringUnit with {}".format(", ".join(sorted(extra))), language, key, *path)
    return unit["value"]


def plural_rule(ctx, forms, language, key, path, source_position=None):
    """The rule dictionary and format key of a plural variation. Where its
    forms don't agree on the number's position, the source language's
    plural, if it has one that does, decides."""
    values = {}
    for category, form in forms.items():
        if category not in PLURAL_CATEGORIES:
            # Xcode keeps a category it doesn't know; established for one.
            pass
        if set(form) != {"stringUnit"}:
            ctx.error("Plural case cannot be further varied", language, key, *path, "plural." + category)
            raise Failure()
        values[category] = string_unit(ctx, form["stringUnit"], language, key, *path, "plural." + category)
    numeric = []  # (form, position, value type)
    ambiguous = False
    for category, value in values.items():
        found = [(p, t, n) for p, _, t, n in specifiers(value)]
        positions = []
        for index, (p, t, n) in enumerate(found, 1):
            if n:
                positions.append((p or index, t))
        types = {t for _, t in positions}
        if len(types) > 1:
            ctx.error("Cannot infer format specifier for string group because more than one numerical specifier was found", language, key, *path)
            raise Failure()
        if len(positions) > 1:
            ambiguous = True
        numeric.extend(positions)
    if not numeric:
        ctx.error("Plural variation requires referencing the number in the string. To maintain grammatical correctness for strings that do not reference the number of items, use separate top-level strings for one and greater than one.", language, key, *path)
        raise Failure()
    types = {t for _, t in numeric}
    if len(types) > 1:
        ctx.unestablished("A plural whose forms use different numeric specifiers", language, key, *path)
    value_type = numeric[0][1]
    positions = {p for p, _ in numeric}
    if (ambiguous or len(positions) > 1) and source_position is not None:
        position = source_position
    elif ambiguous or len(positions) > 1:
        ctx.warning("Cannot reliably infer argument number for plural variation because %{} is used in multiple positions. Use an explicit substitution instead.".format(value_type), language, key, *path)
        position = 1
    else:
        position = positions.pop()
    rule = {"NSStringFormatSpecTypeKey": "NSStringPluralRuleType", "NSStringFormatValueTypeKey": value_type}
    if position == 1:
        rule.update(values)
        return "%#@value@", rule
    rule.update({c: positional(v) for c, v in values.items()})
    return "%{}$#@value@".format(position), rule


def source_plural_position(localizations, source):
    """The number's position in the source language's plural, when it has
    an `other` form and its forms agree on it."""
    localization = localizations.get(source, {})
    forms = localization.get("variations", {}).get("plural")
    if not forms or "other" not in forms or "substitutions" in localization:
        return None
    positions = set()
    for form in forms.values():
        if set(form) != {"stringUnit"}:
            return None
        found = [(p or i, n) for i, (p, _, _, n) in enumerate(specifiers(form["stringUnit"].get("value", "")), 1)]
        numeric = [p for p, n in found if n]
        if len(numeric) > 1:
            return None
        positions.update(numeric)
    return positions.pop() if len(positions) == 1 else None


def substituted(ctx, localization, language, key):
    """The stringsdict entry of a stringUnit with substitutions."""
    value = string_unit(ctx, localization["stringUnit"], language, key)
    substitutions = localization["substitutions"]
    referenced = []
    for m in SUBSTITUTION.finditer(value):
        if m.group(2) not in substitutions:
            ctx.error("Referencing undefined substitution '{}'".format(m.group(2)), language, key)
            raise Failure()
        if m.group(1):
            ctx.unestablished("A positioned substitution reference", language, key)
        referenced.append(m.group(2))
    unused = [name for name in substitutions if name not in referenced]
    for name in unused:
        ctx.warning("Substitution '{}' was never referenced. To reference, use %#@{}@ in some other string".format(name, name), language, key, "substitutions." + name)
    if not referenced:
        return value
    if unused:
        ctx.unestablished("Substitutions some of which are unused", language, key)
    entry = {}
    for name in referenced:
        sub = substitutions[name]
        extra = set(sub) - {"argNum", "formatSpecifier", "variations"}
        if extra or set(sub.get("variations", {})) != {"plural"}:
            ctx.unestablished("A substitution other than a plural one", language, key, "substitutions." + name)
        spec = sub["formatSpecifier"]
        arg_num = sub.get("argNum")
        prefix = "%{}$".format(arg_num) if arg_num is not None else "%"
        rule = {"NSStringFormatSpecTypeKey": "NSStringPluralRuleType", "NSStringFormatValueTypeKey": spec}
        for category, form in sub["variations"]["plural"].items():
            if set(form) != {"stringUnit"}:
                ctx.unestablished("A substitution's varied plural case", language, key, "substitutions." + name)
            rule[category] = string_unit(ctx, form["stringUnit"], language, key, "substitutions." + name, "plural." + category).replace("%arg", prefix + spec)
        entry[name] = rule
    numbered = {name: substitutions[name].get("argNum") for name in referenced}

    def reference(m):
        n = numbered[m.group(2)]
        return "%{}$#@{}@".format(n, m.group(2)) if n is not None else m.group(0)

    entry["NSStringLocalizedFormatKey"] = SUBSTITUTION.sub(reference, value)
    return entry


def device_rule(ctx, devices, language, key, source_position=None):
    """(the device rule dictionary, the plain fallback or None)."""
    rule = {}
    fallback = None
    for device, variant in devices.items():
        path = ("device." + device,)
        if device == "other":
            if set(variant) != {"stringUnit"}:
                if "substitutions" in variant:
                    for m in SUBSTITUTION.finditer(variant.get("stringUnit", {}).get("value", "")):
                        ctx.error("Referencing undefined substitution '{}'".format(m.group(2)), language, key, *path)
                ctx.error("Fallback value cannot be further varied", language, key, *path)
                raise Failure()
            fallback = string_unit(ctx, variant["stringUnit"], language, key, *path)
            continue
        if "substitutions" in variant:
            for m in SUBSTITUTION.finditer(variant.get("stringUnit", {}).get("value", "")):
                ctx.error("Referencing undefined substitution '{}'".format(m.group(2)), language, key, *path)
                raise Failure()
            ctx.unestablished("A device variant with substitutions", language, key, *path)
        if "variations" in variant:
            kinds = set(variant["variations"])
            if kinds != {"plural"}:
                ctx.unestablished("A device variant varied by {}".format(", ".join(sorted(kinds))), language, key, *path)
            forms = variant["variations"]["plural"]
            if not forms:
                ctx.unestablished("A device variant with an empty plural", language, key, *path)
            format_key, plural = plural_rule(ctx, forms, language, key, path, source_position)
            rule[device] = {"NSStringLocalizedFormatKey": format_key, "value": plural}
        elif set(variant) == {"stringUnit"}:
            rule[device] = string_unit(ctx, variant["stringUnit"], language, key, *path)
        else:
            ctx.unestablished("A device variant with {}".format(", ".join(sorted(variant))), language, key, *path)
    return rule, fallback


# The value types of String(localized:)'s interpolations, which a manual
# string's key may use and still fall back to.
INTERPOLATION_TYPES = {"d", "u", "f", "g", "ld", "lu", "lld", "llu", "lf", "@"}


def _values(localization):
    """Every string a localization holds."""
    if "stringUnit" in localization:
        yield localization["stringUnit"].get("value", "")
    for kind in localization.get("variations", {}).values():
        for variant in kind.values():
            yield from _values(variant)
    for sub in localization.get("substitutions", {}).values():
        for value in _values(sub):
            yield value.replace("%arg", "%" + sub.get("formatSpecifier", ""))


def _format_types(text):
    """The value types of `text`'s specifiers, or None if their positions
    don't make a format string (a gap, as `%2$d` alone)."""
    found = specifiers(text)
    positions = [p for p, _, _, _ in found if p is not None]
    if positions and sorted(set(positions)) != list(range(1, max(positions) + 1)):
        return None
    return [t for _, _, t, _ in found]


_INTEGER = {"d", "u", "ld", "lu", "lld", "llu"}
_FLOAT = {"f", "g", "lf"}


def _kind(value_type):
    return "integer" if value_type in _INTEGER else "float" if value_type in _FLOAT else "object"


def manual_fallback(ctx, key, localizations, source):
    """Whether a string added by hand falls back to its key in the languages
    without a localization of it: when its key, if the source language
    doesn't localize it, or a localization of it takes an argument. Xcode's
    choice is established where each of those takes at most one argument,
    all of one kind, and when the key uses a specifier String(localized:)
    doesn't (never); otherwise it is refused."""
    key_types = _format_types(key)
    if key_types and any(t not in INTERPOLATION_TYPES for t in key_types):
        return False
    texts = [v for loc in localizations.values() for v in _values(loc)]
    if source not in localizations:
        texts.append(key)
    kinds = set()
    for text in texts:
        types = _format_types(text)
        if types is None:
            types = []
        if len(types) > 1 or any(t not in INTERPOLATION_TYPES for t in types):
            ctx.unestablished("Whether a manual string with several or other arguments falls back to its key", "", key)
            return False
        kinds.update(_kind(t) for t in types)
    if len(kinds) > 1:
        ctx.unestablished("Whether a manual string whose arguments differ in kind falls back to its key", "", key)
        return False
    return bool(kinds)


EXTRACTION_STATES = (None, "extracted_with_value", "stale", "manual", "migrated", "translated")


def compile_catalog(ctx, catalog):
    """{language: (strings, stringsdict)}."""
    extra = set(catalog) - {"sourceLanguage", "strings", "version"}
    if extra:
        ctx.unestablished("A catalog with {}".format(", ".join(sorted(extra))), "", "")
    source = catalog.get("sourceLanguage", "en")
    strings_by_key = catalog.get("strings", {})
    # Every language the catalog localizes into, and its source.
    languages = {source}
    for entry in strings_by_key.values():
        languages.update(entry.get("localizations", {}))
    out = {}

    def tables(language):
        return out.setdefault(language, ({}, {}))

    def touch(language):
        """A language whose localization was compiled, if to nothing, has
        its directory; one whose only localizations were left out has none."""
        tables(language)

    for key, entry in strings_by_key.items():
        extra = set(entry) - {"comment", "extractionState", "localizations", "shouldTranslate"}
        if extra:
            ctx.unestablished("A string with {}".format(", ".join(sorted(extra))), "", key)
        state = entry.get("extractionState")
        if state not in EXTRACTION_STATES:
            ctx.unestablished("extractionState {}".format(state), source, key)
        # A string added by hand (manual) or migrated from a .strings file has
        # its key as its source-language value.
        keyed = state in ("manual", "migrated")
        localizations = entry.get("localizations", {})
        source_position = source_plural_position(localizations, source)
        for language, localization in localizations.items():
            try:
                if not (language == source and set(localization) == {"stringUnit"}
                        and localization["stringUnit"].get("state") == "new" and not keyed):
                    touch(language)
                extra = set(localization) - {"stringUnit", "variations", "substitutions"}
                if extra:
                    ctx.unestablished("A localization with {}".format(", ".join(sorted(extra))), language, key)
                strings, stringsdict = out.get(language, ({}, {}))
                if "variations" in localization:
                    if "substitutions" in localization:
                        ctx.unestablished("Variations with substitutions", language, key)
                    variations = localization["variations"]
                    if set(variations) == {"plural"}:
                        if not variations["plural"]:
                            continue
                        format_key, rule = plural_rule(ctx, variations["plural"], language, key, (), source_position)
                        stringsdict[key] = {"NSStringLocalizedFormatKey": format_key, "value": rule}
                    elif set(variations) == {"device"}:
                        rule, fallback = device_rule(ctx, variations["device"], language, key, source_position)
                        stringsdict[key] = {"NSStringDeviceSpecificRuleType": rule}
                        if fallback is not None:
                            strings[key] = fallback
                    else:
                        ctx.unestablished("Variations by {}".format(", ".join(sorted(variations))), language, key)
                elif "substitutions" in localization:
                    stringsdict[key] = substituted(ctx, localization, language, key)
                elif "stringUnit" in localization:
                    value = string_unit(ctx, localization["stringUnit"], language, key)
                    # The source language's new strings are left out, unless
                    # added by hand.
                    if language == source and localization["stringUnit"]["state"] == "new" and not keyed:
                        if state == "migrated":
                            ctx.unestablished("A migrated string new in the source language", language, key)
                        continue
                    strings[key] = value
                else:
                    ctx.unestablished("An empty localization", language, key)
            except Failure:
                pass
        if keyed and source not in localizations:
            tables(source)[0][key] = key
        # A string added by hand also falls back to its key in every language
        # without a localization of it, when its key is a format string or
        # some localization of it a plural or a substitution, unless it uses
        # a specifier Swift's String(localized:) doesn't.
        if state == "manual":
            try:
                fallback = manual_fallback(ctx, key, localizations, source)
            except Failure:
                fallback = False
            if fallback:
                for language in languages:
                    if language not in localizations and language != source:
                        tables(language)[0][key] = key
    return out


def _escape(text):
    return text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def _plist_value(value, depth, lines):
    indent = "\t" * depth
    if isinstance(value, dict):
        if not value:
            lines.append(indent + "<dict/>")
            return
        lines.append(indent + "<dict>")
        # Keys in the order of their UTF-8 bytes, as CFPropertyList writes them.
        for k in sorted(value, key=lambda k: k.encode("utf-8")):
            lines.append(indent + "\t<key>" + _escape(k) + "</key>")
            _plist_value(value[k], depth + 1, lines)
        lines.append(indent + "</dict>")
    else:
        lines.append(indent + "<string>" + _escape(value) + "</string>")


def plist(value):
    lines = [
        '<?xml version="1.0" encoding="UTF-8"?>',
        '<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">',
        '<plist version="1.0">',
    ]
    _plist_value(value, 0, lines)
    lines.append("</plist>")
    return ("\n".join(lines) + "\n").encode("utf-8")


def lproj(language):
    return ("base" if language == "Base" else language) + ".lproj"


def _without_trailing_commas(text):
    """`text` without the commas that end an object or array."""
    out = []
    in_string = escaped = False
    pending = None  # index in `out` of a comma that may be trailing
    for ch in text:
        if in_string:
            out.append(ch)
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == '"':
                in_string = False
            continue
        if ch in "}]" and pending is not None:
            del out[pending]
        if ch == ",":
            pending = len(out)
        elif not ch.isspace():
            pending = None
        if ch == '"':
            in_string = True
        out.append(ch)
    return "".join(out)


def _first_wins(pairs):
    out = {}
    for k, v in pairs:
        out.setdefault(k, v)
    return out


def parse(data):
    """The catalog as Xcode reads it: JSON, a byte order mark and trailing
    commas allowed, the first of duplicate keys kept; None if malformed."""
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError:
        return None
    if text.startswith("\ufeff"):
        text = text[1:]
    try:
        catalog = json.loads(_without_trailing_commas(text), object_pairs_hook=_first_wins)
    except ValueError:
        return None
    return catalog if isinstance(catalog, dict) else None


def compile_xcstrings(args):
    output_dir = None
    input_file = None
    i = 0
    while i < len(args):
        if args[i] in ("--output-directory", "-o") and i + 1 < len(args):
            output_dir = args[i + 1]
            i += 2
        elif args[i].startswith("-"):
            print("error: xcstringstool compile: unsupported option {}".format(args[i]), file=sys.stderr)
            return 1
        else:
            input_file = args[i]
            i += 1
    if not output_dir or not input_file:
        print("usage: xcstringstool compile <input-file> --output-directory <output-directory>", file=sys.stderr)
        return 1

    ctx = Context(os.path.abspath(input_file))
    with open(input_file, "rb") as f:
        catalog = parse(f.read())
    if catalog is None:
        print("{}: error: The data couldn\u2019t be read because it isn\u2019t in the correct format.".format(ctx.path), file=sys.stderr)
        return 1
    for field in ("sourceLanguage", "version"):
        if field not in catalog:
            print("{}: error: Missing required key '{}'".format(ctx.path, field), file=sys.stderr)
            return 1
    try:
        compiled = compile_catalog(ctx, catalog)
    except Failure:
        compiled = None
    if ctx.errors:
        for e in ctx.errors:
            print(e, file=sys.stderr)
        return 1

    base_name = os.path.splitext(os.path.basename(input_file))[0]
    for language, (strings, stringsdict) in sorted(compiled.items()):
        # Every language with a localization gets its directory, even one
        # left empty, as Xcode's does.
        directory = os.path.join(output_dir, lproj(language))
        os.makedirs(directory, exist_ok=True)
        for table, suffix in ((strings, ".strings"), (stringsdict, ".stringsdict")):
            if not table:
                continue
            with open(os.path.join(directory, base_name + suffix), "wb") as f:
                f.write(plist(table))
    return 0


def main():
    if len(sys.argv) < 2:
        print("usage: xcstringstool compile <input-file> --output-directory <output-directory>", file=sys.stderr)
        return 1
    if sys.argv[1] == "compile":
        return compile_xcstrings(sys.argv[2:])
    print("error: xcstringstool: only `compile` is supported off a Mac, not `{}`".format(sys.argv[1]), file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
