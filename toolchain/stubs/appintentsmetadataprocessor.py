#!/usr/bin/env python3
"""appintentsmetadataprocessor off a Mac: App Intents metadata
(Metadata.appintents) from the Swift compiler's constant values, as
rules_apple runs it (`--compile-time-extraction`); see
docs/clean-room/appintentsmetadataprocessor.md.

It writes extract.actionsdata and version.json with the content Xcode 27's
writes, for the intents, parameters, parameter summaries, App Shortcuts and
enums it has been compared on. What hasn't been established against Xcode
is refused, strict (the default); relaxed
(RULES_APPLECROSS_RESOURCE_OUTPUT_MODE=relaxed, --config=relaxed), it is
left out, with a warning.
"""

import json
import os
import sys
import unicodedata

STRICT = os.environ.get("RULES_APPLECROSS_RESOURCE_OUTPUT_MODE") != "relaxed"


class Unestablished(Exception):
    pass


class Invalid(Exception):
    """Input Xcode's processor refuses too, with its message."""


def unestablished(what):
    raise Unestablished(what)


# typeIdentifier of LNPrimitiveValueType for the Swift types that map to one.
PRIMITIVES = {
    "Swift.String": 0,
    "Swift.Bool": 1,
    "Swift.Int": 2,
    "Swift.Double": 7,
    "Foundation.Date": 8,
    "Foundation.URL": 11,
}

# The types a parameter's value can be resolved from, in the order Xcode's
# processor writes them (a set, enumerated as SWIFT_DETERMINISTIC_HASHING=1
# orders it, which rules_apple sets).
RESOLVABLE = {
    "Swift.String": [("primitive", 2), ("array", 2), ("primitive", 0)],
    "Swift.Int": [("primitive", 2), ("primitive", 7), ("primitive", 0)],
    "Swift.Double": [("primitive", 2), ("primitive", 7), ("primitive", 0)],
    "Swift.Bool": [("primitive", 1)],
    "Foundation.Date": [("primitive", 8)],
    "Foundation.URL": [("primitive", 11), ("primitive", 0)],
}

OUTPUT_FLAGS = {
    "AppIntents.IntentResult": 0,
    "AppIntents.OpensIntent": 1,
    "AppIntents.ShowsSnippetView": 2,
    "AppIntents.ProvidesDialog": 4,
    "AppIntents.ReturnsValue": 0,
}

PLATFORMS = {
    "iOS": "LNPlatformNameIOS",
    "macOS": "LNPlatformNameMACOS",
}

AUTHENTICATION = {
    "alwaysAllowed": 0,
    "requiresAuthentication": 1,
}

TILE_COLORS = {
    "red": 0, "orange": 1, "tangerine": 2, "yellow": 3, "lime": 4, "teal": 5, "lightBlue": 6, "blue": 7,
    "navy": 8, "grape": 9, "purple": 10, "pink": 11, "grayBlue": 12, "grayGreen": 13, "grayBrown": 14,
}

KEYBOARD_TYPES = {
    "asciiCapable": 0, "numbersAndPunctuation": 1, "URL": 2, "numberPad": 3, "default": 4,
}

CAPITALIZATION_TYPES = {
    "none": 0, "words": 1, "sentences": 2, "allCharacters": 3,
}

COMPARISON_OPERATORS = {
    "equalTo": 2,
}


def primitive(identifier):
    return {"primitive": {"wrapper": {"typeIdentifier": identifier}}}


def array_of(member):
    return {"array": {"wrapper": {"capabilities": 3, "memberValueType": member}}}


def _generic(type_name, outer):
    """The argument of `outer<...>` in `type_name`, or None."""
    prefix = outer + "<"
    if type_name.startswith(prefix) and type_name.endswith(">"):
        return type_name[len(prefix):-1]
    return None


def swift_escaped(text):
    """`text` escaped as Swift's String.debugDescription escapes it, without
    the quotes: \\0 \\t \\n \\r \\" and \\\\ named, other unprintable scalars
    as \\u{hex}."""
    named = {"\0": "\\0", "\t": "\\t", "\n": "\\n", "\r": "\\r", '"': '\\"', "\\": "\\\\"}
    out = []
    for ch in text:
        if ch in named:
            out.append(named[ch])
        elif ch == " " or ch.isprintable():
            if unicodedata.category(ch) == "Cn":
                unestablished("an unassigned character in an escaped string")
            out.append(ch)
        else:
            out.append("\\u{%x}" % ord(ch))
    return "".join(out)


# --- Values -----------------------------------------------------------------

def _args(value):
    """{label: argument} of an InitCall value, labels as written ("" for none)."""
    if value.get("valueKind") != "InitCall":
        unestablished("a {} value".format(value.get("valueKind")))
    out = {}
    for argument in value["value"]["arguments"]:
        out.setdefault(argument["label"], []).append(argument)
    return out


def _absent(argument):
    return argument["valueKind"] in ("NilLiteral", "Runtime")


def _only(arguments, allowed, what):
    for label, values in arguments.items():
        if label not in allowed and not all(_absent(v) for v in values):
            unestablished("{} with {}".format(what, label or "an unlabelled argument"))


def _string(value):
    if value.get("valueKind") != "RawLiteral" or not isinstance(value.get("value"), str):
        unestablished("a {} where a string literal was expected".format(value.get("valueKind")))
    return value["value"]


def localized(value):
    """A LocalizedStringResource as LNStaticDeferredLocalizedString."""
    if value.get("valueKind") == "RawLiteral":
        return {"alternatives": [], "key": _string(value)}
    if value.get("valueKind") == "InterpolatedStringLiteral":
        unestablished("an interpolated LocalizedStringResource")
    arguments = _args(value)
    if set(arguments) == {"stringLiteral"}:
        return {"alternatives": [], "key": _string(arguments["stringLiteral"][0])}
    _only(arguments, {"", "defaultValue", "table", "comment", "locale", "bundle"}, "a LocalizedStringResource")
    if len(arguments.get("", [])) != 1:
        unestablished("a LocalizedStringResource without a key")
    out = {"alternatives": [], "key": _string(arguments[""][0])}
    for label in ("defaultValue", "table"):
        for argument in arguments.get(label, []):
            if not _absent(argument):
                out[label] = _string(argument)
    return out


def _bool(value):
    if value.get("valueKind") != "RawLiteral" or value.get("value") not in ("true", "false"):
        unestablished("a Bool other than a literal")
    return value["value"] == "true"


def _enum(value):
    if value.get("valueKind") != "Enum":
        unestablished("a {} where an enum case was expected".format(value.get("valueKind")))
    return value["value"]["name"]


def _lookup(table, name, what):
    if name not in table:
        unestablished("{} {}".format(what, name))
    return table[name]


# --- Types --------------------------------------------------------------------

def availability(entry):
    out = {"LNPlatformNameWildcard": {"introducedVersion": "*"}}
    for attribute in entry.get("availabilityAttributes", []):
        if attribute.get("isUnavailable") or attribute.get("isDeprecated") or set(attribute) - {
                "platform", "introducedVersion", "isUnavailable", "isDeprecated"}:
            unestablished("availability other than an introduced version")
        out[_lookup(PLATFORMS, attribute["platform"], "availability on")] = {"introducedVersion": attribute["introducedVersion"]}
    return out


def _properties(entry):
    out = {}
    for prop in entry.get("properties", []):
        out.setdefault(prop["label"], prop)
    return out


class Module:
    def __init__(self, name, entries):
        self.name = name
        self.entries = entries
        self.by_name = {e["typeName"]: e for e in entries}

    def conforms(self, type_name, protocol):
        entry = self.by_name.get(type_name)
        return bool(entry) and protocol in entry.get("conformances", [])

    def value_type(self, type_name):
        """(valueType, resolvableInputTypes, isOptional) of a parameter's type."""
        inner = _generic(type_name, "Swift.Optional")
        if inner is not None:
            value, resolvable, _ = self.value_type(inner)
            return value, resolvable, True
        member = _generic(type_name, "Swift.Array")
        if member is not None:
            value, _, optional = self.value_type(member)
            if optional:
                unestablished("an array of optionals")
            return array_of(value), [], False
        if type_name in PRIMITIVES:
            resolvable = [{"kindValue": 0, "valueType": primitive(i) if kind == "primitive" else array_of(primitive(i))}
                          for kind, i in RESOLVABLE[type_name]]
            return primitive(PRIMITIVES[type_name]), resolvable, False
        if self.conforms(type_name, "AppIntents.AppEnum"):
            return {"linkEnumeration": {"wrapper": {"identifier": type_name.rsplit(".", 1)[1]}}}, [], False
        unestablished("a parameter of type {}".format(type_name))


def _identifier(type_name, module):
    if not type_name.startswith(module + "."):
        unestablished("a type outside the module")
    return type_name.rsplit(".", 1)[1]


def _number(text):
    """A floating-point literal as JSONSerialization writes the double:
    integral values without a fraction."""
    value = float(text)
    return int(value) if value.is_integer() and abs(value) < 2 ** 53 else value


def _default_metadata(value, type_name):
    """A default value's literal as an LNValue for typeSpecificMetadata."""
    text = _string(value) if value.get("valueKind") == "RawLiteral" else unestablished("a default other than a literal")
    if type_name == "Swift.Int":
        return {"int": {"wrapper": int(text)}}
    if type_name == "Swift.Bool":
        return {"int": {"wrapper": 1 if _bool(value) else 0}}
    if type_name == "Swift.Double":
        return {"double": {"wrapper": _number(text)}}
    if type_name == "Swift.String":
        return {"string": {"wrapper": text}}
    unestablished("a default of type {}".format(type_name))


def _flatten(pairs):
    """A dictionary's (key, value) pairs in the order Swift enumerates a
    Dictionary<String, _> of them under deterministic hashing."""
    out = []
    for key in swift_dictionary_order([k for k, _ in pairs]):
        out.extend([key, dict(pairs)[key]])
    return out


def parameter(module, prop):
    value = prop
    type_name = _generic(value["value"]["type"], "AppIntents.IntentParameter")
    if type_name is None:
        unestablished("a parameter that isn't an IntentParameter initializer")
    arguments = _args(value)
    _only(arguments, {"title", "description", "default", "requestValueDialog", "inclusiveRange", "inputOptions"}, "a parameter")
    value_type, resolvable, optional = module.value_type(type_name)
    base = _generic(type_name, "Swift.Optional") or type_name
    out = {
        "capabilities": 0,
        "dynamicOptionsSupport": 0,
        "inputConnectionBehavior": 0,
        "isInput": False,
        "isOptional": optional,
        "name": prop["label"],
        "resolvableInputTypes": resolvable,
        "title": localized(arguments["title"][0]),
        "typeSpecificMetadata": [],
        "valueType": value_type,
    }
    metadata = []
    for argument in arguments.get("description", []):
        if not _absent(argument):
            out["parameterDescription"] = localized(argument)
    for argument in arguments.get("default", []):
        if not _absent(argument):
            out["capabilities"] = 1
            metadata.append(("LNValueTypeSpecificMetadataKeyDefaultValue", _default_metadata(argument, base)))
    for argument in arguments.get("inclusiveRange", []):
        if not _absent(argument):
            if argument["valueKind"] != "Tuple" or base != "Swift.Int":
                unestablished("an inclusiveRange other than an Int tuple")
            low, high = (int(_string(v)) for v in argument["value"])
            metadata += [("LNValueTypeMetadataKeyNumberRangeType", {"int": {"wrapper": 0}}),
                         ("LNValueTypeMetadataKeyNumberLowerBound", {"double": {"wrapper": low}}),
                         ("LNValueTypeMetadataKeyNumberUpperBound", {"double": {"wrapper": high}})]
    for argument in arguments.get("inputOptions", []):
        if not _absent(argument):
            options = _args(argument)
            _only(options, {"keyboardType", "capitalizationType", "multiline"}, "String.IntentInputOptions")
            if set(k for k, v in options.items() if not all(_absent(x) for x in v)) != {"keyboardType", "capitalizationType", "multiline"}:
                unestablished("String.IntentInputOptions without all of keyboardType, capitalizationType and multiline")
            # Inserted in this order: where two keys share a bucket, the first
            # takes it (CapitalizationType before KeyboardType, 3 keys).
            metadata += [
                ("LNValueTypeMetadataKeyStringCapitalizationType", {"int": {"wrapper": _lookup(CAPITALIZATION_TYPES, _enum(options["capitalizationType"][0]), "capitalization type")}}),
                ("LNValueTypeMetadataKeyStringKeyboardType", {"int": {"wrapper": _lookup(KEYBOARD_TYPES, _enum(options["keyboardType"][0]), "keyboard type")}}),
                ("LNValueTypeMetadataKeyStringMultiline", {"int": {"wrapper": 1 if _bool(options["multiline"][0]) else 0}}),
            ]
    out["typeSpecificMetadata"] = _flatten(metadata)
    return out


def _keypath_parameter(value):
    if value.get("valueKind") != "KeyPath":
        unestablished("a parameter summary element other than a key path")
    path = value["value"]["path"]
    if not path.startswith("$") or len(value["value"]["components"]) != 1:
        unestablished("a parameter summary key path other than \\.$parameter")
    return path[1:]


def summary_string(value):
    if value.get("valueKind") == "RawLiteral":
        return {"formatString": swift_escaped(_string(value)), "parameterIdentifiers": []}
    if value.get("valueKind") != "InterpolatedStringLiteral":
        unestablished("a parameter summary string that isn't a literal")
    text, names = [], []
    for segment in value["value"]["segments"]:
        if segment["valueKind"] == "RawLiteral":
            text.append(swift_escaped(segment["value"]))
        else:
            name = _keypath_parameter(segment)
            names.append(name)
            text.append("${" + name + "}")
    return {"formatString": "".join(text), "parameterIdentifiers": names}


def summary(value):
    """An actionConfiguration from a parameterSummary value."""
    kind = value.get("valueKind")
    if kind != "InitCall":
        unestablished("a parameterSummary other than Summary or When")
    type_name = value["value"]["type"]
    arguments = value["value"]["arguments"]
    if type_name.startswith("AppIntents.IntentParameterSummary<"):
        strings = [a for a in arguments if a["label"] == "" and a["valueKind"] != "Builder"]
        builders = [a for a in arguments if a["valueKind"] == "Builder"]
        tables = [a for a in arguments if a["label"] == "table" and not _absent(a)]
        if len(strings) != 1 or len(builders) > 1 or tables or len(arguments) != 2 + len(builders):
            unestablished("a Summary other than a string and its other parameters")
        others = []
        for builder in builders:
            for member in builder["value"]["members"]:
                if member["kind"] != "buildExpression":
                    unestablished("a Summary's other parameters built other than one by one")
                others.append(_keypath_parameter(member["element"]))
        return {"actionSummary": {"wrapper": {"otherParameterIdentifiers": others, "summaryString": summary_string(strings[0])}}}
    if type_name.startswith("AppIntents.ParameterSummaryWhenCondition<"):
        if len(arguments) != 5:
            unestablished("a When other than (parameter, operator, value, then, otherwise)")
        keypath, operator, literal, then, otherwise = arguments
        if literal["type"] != "Swift.Bool":
            unestablished("a When comparing other than a Bool")
        return {"whenClause": {"wrapper": {
            "condition": {
                "comparisonOperator": _lookup(COMPARISON_OPERATORS, _enum(operator), "comparison"),
                "parameterIdentifier": _keypath_parameter(keypath),
                "value": {"bool": {"value": _bool(literal)}},
            },
            "otherwise": summary(_single(otherwise)),
            "when": summary(_single(then)),
        }}}
    unestablished("a parameterSummary of type {}".format(type_name))


def _single(builder):
    if builder.get("valueKind") != "Builder" or len(builder["value"]["members"]) != 1:
        unestablished("a When branch other than one summary")
    member = builder["value"]["members"][0]
    if member["kind"] != "buildExpression":
        unestablished("a When branch built other than from one expression")
    return member["element"]


def action(module, entry):
    props = _properties(entry)
    known = {"title", "description", "openAppWhenRun", "isDiscoverable", "authenticationPolicy", "parameterSummary"}
    parameters = []
    for prop in entry.get("properties", []):
        label = prop["label"]
        if label.startswith("_") or label.startswith("$"):
            continue
        value = prop.get("value") or {}
        if prop.get("valueKind") == "InitCall" and value.get("type", "").startswith("AppIntents.IntentParameter<"):
            parameters.append(parameter(module, prop))
        elif label not in known and prop.get("isStatic") == "true":
            unestablished("an intent's static {}".format(label))
    if "title" not in props:
        unestablished("an intent without a title")
    out = {
        "assistantDefinedSchemaTraits": [],
        "assistantDefinedSchemas": [],
        "authenticationPolicy": 0,
        "availabilityAnnotations": availability(entry),
        "effectiveBundleIdentifiers": [],
        "fullyQualifiedTypeName": entry["typeName"],
        "identifier": _identifier(entry["typeName"], module.name),
        "isAuthPolExplicit": False,
        "isDiscoverable": True,
        "mangledTypeName": entry["mangledTypeName"],
        "mangledTypeNameByBundleIdentifier": {},
        "mangledTypeNameByBundleIdentifierV2": {},
        "mangledTypeNameV2": entry["mangledTypeName"],
        "openAppWhenRun": False,
        "outputFlags": 0,
        "parameters": parameters,
        "presentationStyle": 0,
        "requiredCapabilities": [],
        "supportedModes": 1,
        "systemProtocolMetadata": [],
        "systemProtocolMetadataV2": [],
        "systemProtocols": [],
        "title": localized(props["title"]),
        "typeSpecificMetadata": [],
        "visibilityMetadata": {"assistantOnly": False, "isDiscoverable": True},
    }
    if "description" in props:
        description = props["description"]
        if description.get("valueKind") == "RawLiteral":
            out["descriptionMetadata"] = {"descriptionText": localized(description), "searchKeywords": []}
        else:
            arguments = _args(description)
            _only(arguments, {"", "categoryName", "searchKeywords"}, "an IntentDescription")
            metadata = {"descriptionText": localized(arguments[""][0]), "searchKeywords": []}
            for argument in arguments.get("categoryName", []):
                if not _absent(argument):
                    metadata["categoryName"] = {"title": localized(argument)}
            for argument in arguments.get("searchKeywords", []):
                if not _absent(argument):
                    if argument["valueKind"] != "Array":
                        unestablished("searchKeywords other than an array literal")
                    metadata["searchKeywords"] = [localized(v) for v in argument["value"]]
            out["descriptionMetadata"] = metadata
    if "openAppWhenRun" in props and _bool(props["openAppWhenRun"]):
        out["openAppWhenRun"] = True
        out["supportedModes"] = 2
    if "isDiscoverable" in props and not _bool(props["isDiscoverable"]):
        out["isDiscoverable"] = False
        out["visibilityMetadata"]["isDiscoverable"] = False
    if "authenticationPolicy" in props:
        out["authenticationPolicy"] = _lookup(AUTHENTICATION, _enum(props["authenticationPolicy"]), "authentication policy")
        out["isAuthPolExplicit"] = True
    if "parameterSummary" in props:
        out["actionConfiguration"] = summary(props["parameterSummary"])
    result = [a for a in entry.get("associatedTypeAliases", []) if a["typeAliasName"] == "PerformResult"]
    if len(result) != 1 or not result[0]["substitutedTypeName"].startswith("some "):
        unestablished("a perform() result other than `some IntentResult & ...`")
    for protocol in result[0]["opaqueTypeProtocolRequirements"]:
        out["outputFlags"] |= _lookup(OUTPUT_FLAGS, protocol, "a result conforming to")
    for requirement in result[0]["opaqueTypeSameTypeRequirements"]:
        if requirement["typeAliasName"] != "AppIntents.IntentResult.Value":
            unestablished("a result requirement on {}".format(requirement["typeAliasName"]))
        value = requirement["substitutedTypeName"]
        out["outputType"] = primitive(_lookup(PRIMITIVES, value, "a result value of type"))
    return out


def app_enum(module, entry):
    props = _properties(entry)
    extra = set(props) - {"typeDisplayRepresentation", "caseDisplayRepresentations", "allCases", "rawValue"}
    if extra:
        unestablished("an AppEnum's {}".format(", ".join(sorted(extra))))
    out = {
        "assistantDefinedSchemas": [],
        "availabilityAnnotations": availability(entry),
        "effectiveBundleIdentifiers": [],
        "fullyQualifiedTypeName": entry["typeName"],
        "identifier": _identifier(entry["typeName"], module.name),
        "isSystem": False,
        "mangledTypeName": entry["mangledTypeName"],
        "mangledTypeNameByBundleIdentifier": {},
        "systemProtocolMetadata": [],
        "visibilityMetadata": {"assistantOnly": False, "isDiscoverable": True},
    }
    representation = props["typeDisplayRepresentation"]
    if representation.get("valueKind") == "RawLiteral":
        # A literal TypeDisplayRepresentation is written as escaped source.
        out["displayTypeName"] = {"alternatives": [], "key": swift_escaped(_string(representation))}
    else:
        arguments = _args(representation)
        _only(arguments, {"name", "numericFormat"}, "a TypeDisplayRepresentation")
        out["displayTypeName"] = localized(arguments["name"][0])
        for argument in arguments.get("numericFormat", []):
            if not _absent(argument):
                out["numericFormatTypeName"] = {"alternatives": [], "key": _numeric_format(argument)}
    cases = props["caseDisplayRepresentations"]
    if cases.get("valueKind") != "Dictionary":
        unestablished("caseDisplayRepresentations other than a dictionary literal")
    shown = {}
    for pair in cases["value"]:
        key, value = pair["key"], pair["value"]
        shown[_enum(key)] = display_representation(value)
    out["cases"] = [{"displayRepresentation": shown[c["name"]], "identifier": c["name"]} for c in entry["cases"] if c["name"] in shown]
    if len(out["cases"]) != len(shown):
        unestablished("caseDisplayRepresentations naming cases that aren't the enum's")
    return out


def _numeric_format(value):
    if value.get("valueKind") != "InterpolatedStringLiteral":
        unestablished("a numericFormat that isn't an interpolated literal")
    text = []
    for segment in value["value"]["segments"]:
        if segment["valueKind"] == "RawLiteral":
            text.append(swift_escaped(segment["value"]))
        elif segment["valueKind"] == "Enum" and segment["value"]["name"] == "int":
            text.append("${int}")
        else:
            unestablished("a numericFormat placeholder other than .int")
    return "".join(text)


def display_representation(value):
    if value.get("valueKind") == "RawLiteral":
        return {"title": localized(value)}
    arguments = _args(value)
    _only(arguments, {"title", "subtitle", "image"}, "a DisplayRepresentation")
    out = {"title": localized(arguments["title"][0])}
    for argument in arguments.get("subtitle", []):
        if not _absent(argument):
            out["subtitle"] = localized(argument)
    for argument in arguments.get("image", []):
        if not _absent(argument):
            image = _args(argument)
            _only(image, {"systemName"}, "a DisplayRepresentation.Image")
            name = _string(image["systemName"][0])
            out["image"] = {"systemImageName": {"_0": name}}
            out["imageV2"] = {"systemImageName": {"_0": name}}
    return out


def shortcuts(module, entry):
    props = _properties(entry)
    extra = set(props) - {"appShortcuts", "shortcutTileColor"}
    if extra:
        unestablished("an AppShortcutsProvider's {}".format(", ".join(sorted(extra))))
    color = 14
    if "shortcutTileColor" in props:
        color = _lookup(TILE_COLORS, _enum(props["shortcutTileColor"]), "shortcut tile color")
    builder = props["appShortcuts"]
    if builder.get("valueKind") != "Builder":
        unestablished("appShortcuts other than a builder")
    out = []
    for member in builder["value"]["members"]:
        if member["kind"] != "buildExpression":
            unestablished("appShortcuts built other than one by one")
        arguments = _args(member["element"])
        _only(arguments, {"intent", "phrases", "shortTitle", "systemImageName"}, "an AppShortcut")
        intent = arguments["intent"][0]
        if intent["valueKind"] != "InitCall" or intent["value"]["arguments"]:
            unestablished("an AppShortcut's intent other than Intent()")
        phrases = arguments["phrases"][0]
        if phrases["valueKind"] != "Array":
            unestablished("phrases other than an array literal")
        out.append({
            "actionIdentifier": _identifier(intent["value"]["type"], module.name),
            "availabilityAnnotations": {"LNPlatformNameWildcard": {"introducedVersion": "*"}},
            "phraseTemplates": [{"alternatives": [], "key": _phrase(p)} for p in phrases["value"]],
            "shortTitle": localized(arguments["shortTitle"][0]),
            "systemImageName": _string(arguments["systemImageName"][0]),
        })
    return color, out


def _phrase(value):
    if value.get("valueKind") == "RawLiteral":
        return _string(value)
    if value.get("valueKind") != "InterpolatedStringLiteral":
        unestablished("a phrase that isn't a string literal")
    text = []
    for segment in value["value"]["segments"]:
        if segment["valueKind"] == "RawLiteral":
            text.append(segment["value"])
        elif segment["valueKind"] == "Enum" and segment["value"]["name"] == "applicationName":
            text.append("${applicationName}")
        else:
            unestablished("a phrase interpolating other than .applicationName")
    return "".join(text)


# --- Swift's Dictionary order -------------------------------------------------

_M = (1 << 64) - 1


def _rotl(x, b):
    return ((x << b) | (x >> (64 - b))) & _M


class _SipHash13:
    """SipHash-1-3 as Swift's Hasher computes it (the published stdlib)."""

    def __init__(self, k0=0, k1=0):
        self.v = [0x736f6d6570736575 ^ k0, 0x646f72616e646f6d ^ k1, 0x6c7967656e657261 ^ k0, 0x7465646279746573 ^ k1]
        self.tail = b""
        self.count = 0

    def _round(self):
        v0, v1, v2, v3 = self.v
        v0 = (v0 + v1) & _M; v1 = _rotl(v1, 13); v1 ^= v0; v0 = _rotl(v0, 32)
        v2 = (v2 + v3) & _M; v3 = _rotl(v3, 16); v3 ^= v2
        v0 = (v0 + v3) & _M; v3 = _rotl(v3, 21); v3 ^= v0
        v2 = (v2 + v1) & _M; v1 = _rotl(v1, 17); v1 ^= v2; v2 = _rotl(v2, 32)
        self.v = [v0, v1, v2, v3]

    def _compress(self, m):
        self.v[3] ^= m
        self._round()
        self.v[0] ^= m

    def combine(self, data):
        self.count += len(data)
        buf = self.tail + data
        while len(buf) >= 8:
            self._compress(int.from_bytes(buf[:8], "little"))
            buf = buf[8:]
        self.tail = buf

    def finalize(self):
        self._compress(int.from_bytes(self.tail.ljust(8, b"\0")[:7] + b"\0", "little") | ((self.count & 0xff) << 56))
        self.v[2] ^= 0xff
        for _ in range(3):
            self._round()
        v0, v1, v2, v3 = self.v
        return v0 ^ v1 ^ v2 ^ v3


def _string_hash(text, seed):
    h = _SipHash13(seed)
    h.combine(text.encode("utf-8"))
    h.combine(b"\xff")
    return h.finalize()


def swift_dictionary_order(keys):
    """`keys` (ASCII strings) in the order a Swift Dictionary created with
    them enumerates them under deterministic hashing: buckets of a
    power-of-two table at most 3/4 full, hashed with SipHash-1-3 keyed by
    (the table's scale, 0), linear probing in insertion order."""
    if not keys:
        return []
    for k in keys:
        if not k.isascii():
            unestablished("a non-ASCII dictionary key")
    count = len(keys)
    minimum = max(-(-count * 4 // 3), count + 1)
    scale = (max(minimum, 2) - 1).bit_length()
    size = 1 << scale
    buckets = [None] * size
    for k in keys:
        i = _string_hash(k, scale) & (size - 1)
        while buckets[i] is not None:
            i = (i + 1) & (size - 1)
        buckets[i] = k
    return [k for k in buckets if k is not None]


# --- Main ----------------------------------------------------------------------

# Conformances every intent, enum or provider has, which change nothing.
_IMPLIED = {
    "AppIntents.PersistentlyIdentifiable", "AppIntents._SupportsAppDependencies", "AppIntents._IntentValueRepresentable",
    "AppIntents.TypeDisplayRepresentable", "AppIntents.CaseDisplayRepresentable", "AppIntents.StaticEnumValueRepresentable",
    "AppIntents.InstanceDisplayRepresentable", "AppIntents.CustomLocalizedStringResourceConvertible", "AppIntents.AppValue",
    "AppIntents.StaticDisplayRepresentable", "AppIntents._IntentValue",
}


def _add(out, module, entry, handled):
    conformances = set(entry.get("conformances", []))
    kinds = conformances & handled
    others = {c for c in conformances if c.startswith("AppIntents.")} - handled - _IMPLIED
    if others:
        unestablished("a type conforming to {}".format(", ".join(sorted(others))))
    if "AppIntents.AppIntent" in kinds:
        a = action(module, entry)
        out["actions"][a["identifier"]] = a
    if "AppIntents.AppEnum" in kinds:
        out["enums"].append(app_enum(module, entry))
    if "AppIntents.AppShortcutsProvider" in kinds:
        if "autoShortcutProviderMangledName" in out:
            unestablished("more than one AppShortcutsProvider")
        out["shortcutTileColor"], out["autoShortcuts"] = shortcuts(module, entry)
        out["autoShortcutProviderMangledName"] = entry["mangledTypeName"]
        out["_shortcut_entry"] = entry


def metadata(module_name, entries, version):
    module = Module(module_name, entries)
    out = {
        "actions": {},
        "assistantEntities": [],
        "assistantIntentNegativePhrases": [],
        "assistantIntents": [],
        "autoShortcuts": [],
        "entities": {},
        "enums": [],
        "generator": {"name": "xcode-tools", "version": version},
        "negativePhrases": [],
        "queries": {},
        "shortcutTileColor": 14,
        "version": 1,
    }
    handled = {"AppIntents.AppIntent", "AppIntents.AppEnum", "AppIntents.AppShortcutsProvider"}
    # Providers last, so that the intents their shortcuts name are known.
    entries = sorted(entries, key=lambda e: "AppIntents.AppShortcutsProvider" in e.get("conformances", []))
    for entry in entries:
        try:
            _add(out, module, entry, handled)
        except Unestablished as e:
            if STRICT:
                raise
            print("warning: appintentsmetadataprocessor: {} is not reproduced as Xcode writes it; relaxed: leaving {} out".format(
                e, entry["typeName"]), file=sys.stderr)
    provider = out.pop("_shortcut_entry", None)
    errors = []
    for shortcut in out["autoShortcuts"]:
        a = out["actions"].get(shortcut["actionIdentifier"])
        if a is None:
            unestablished("an App Shortcut of an intent not in the module")
        if not a["isDiscoverable"]:
            errors.append("{}:{}: error: App Intent '{}' must be visible for App Shortcuts use".format(
                provider["file"], provider["line"], shortcut["actionIdentifier"]))
    if errors:
        raise Invalid("\n".join(errors))
    return out


def _list(path):
    with open(path) as f:
        return [line.strip() for line in f if line.strip()]


def main(argv):
    options = {}
    flags = {"--compile-time-extraction", "--force", "--quiet-warnings"}
    i = 0
    while i < len(argv):
        arg = argv[i]
        if arg in flags:
            options[arg] = True
            i += 1
        elif arg.startswith("--") and i + 1 < len(argv):
            options.setdefault(arg, []).append(argv[i + 1])
            i += 2
        else:
            print("error: appintentsmetadataprocessor: unsupported argument {}".format(arg), file=sys.stderr)
            return 1
    for required in ("--output", "--module-name", "--xcode-version", "--swift-const-vals-list"):
        if required not in options:
            print("error: appintentsmetadataprocessor: {} is required".format(required), file=sys.stderr)
            return 1
    if "--compile-time-extraction" not in options:
        print("error: appintentsmetadataprocessor: only --compile-time-extraction is supported off a Mac", file=sys.stderr)
        return 1
    for option in ("--metadata-file-list", "--static-metadata-file-list", "--dependency-file", "--stringsdata-file"):
        if option in options:
            print("error: appintentsmetadataprocessor: {} is not supported off a Mac".format(option), file=sys.stderr)
            return 1
    module_name = options["--module-name"][0]
    version = options["--xcode-version"][0]
    entries = []
    for path in _list(options["--swift-const-vals-list"][0]):
        with open(path) as f:
            entries.extend(json.load(f))
    try:
        result = metadata(module_name, entries, version)
    except Invalid as e:
        print(e, file=sys.stderr)
        return 1
    except Unestablished as e:
        print("error: appintentsmetadataprocessor: {} is not reproduced as Xcode writes it; build with --config=relaxed "
              "(RULES_APPLECROSS_RESOURCE_OUTPUT_MODE=relaxed) to leave it out".format(e), file=sys.stderr)
        return 1
    directory = os.path.join(options["--output"][0], "Metadata.appintents")
    os.makedirs(directory, exist_ok=True)
    with open(os.path.join(directory, "extract.actionsdata"), "w") as f:
        json.dump(result, f, indent=2, sort_keys=True)
    with open(os.path.join(directory, "version.json"), "w") as f:
        json.dump({"toolsVersion": version, "version": "3.0"}, f, indent=2, sort_keys=True)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
