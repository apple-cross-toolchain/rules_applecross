#!/usr/bin/env python3

import os
import plistlib
import re
import sys

IOS12 = ("12.0", "11.0", "5.0")
IOS13 = ("13.0", "11.0", "6.0")
IOS13_FOUNDATION = ("13.0", "10.16", "6.0")
IOS14 = ("14.0", "11.0", "7.0")

NUMERIC_REASONS = ["negativeNumbersNotSupported", "greaterThanMaximumValue", "lessThanMinimumValue"]

DEFAULT_REASONS = {
    "CurrencyAmount": NUMERIC_REASONS,
    "Decimal": NUMERIC_REASONS,
    "Distance": NUMERIC_REASONS[:1],
    "Energy": NUMERIC_REASONS[:1],
    "Integer": NUMERIC_REASONS,
    "Mass": NUMERIC_REASONS[:1],
    "Speed": NUMERIC_REASONS[:1],
    "Temperature": NUMERIC_REASONS[:1],
    "Volume": NUMERIC_REASONS[:1],
}

DEFAULT_CODES = [
    "unspecified",
    "ready",
    "continueInApp",
    "inProgress",
    "success",
    "failure",
    "failureRequiringAppLaunch",
]

DEPRECATION = "%s is deprecated. Please use %s instead. You can update this by tapping the warning icon next to this enum in the Intent Definition file"

HIDDEN = '__attribute__((visibility("hidden")))'

DOCS = {
    "protocol": [
        " @abstract Protocol to declare support for handling a %(c)s. By implementing this protocol, a class can provide logic for resolving, confirming and handling the intent.",
        " @discussion The minimum requirement for an implementing class is that it should be able to handle the intent. The confirmation method is optional. The handling method is always called last, after confirming the intent.",
    ],
    "handle": [
        " @abstract Handling method - Execute the task represented by the %(c)s that's passed in",
        " @discussion Called to actually execute the intent. The app must return a response for this intent.",
        "",
        " @param  intent The input intent",
        " @param  completion The response handling block takes a %(c)sResponse containing the details of the result of having executed the intent",
        "",
        " @see  %(c)sResponse",
    ],
    "resolve": [
        " @abstract Resolution methods - Determine if this intent is ready for the next step (confirmation)",
        " @discussion Called to make sure the app extension is capable of handling this intent in its current form. This method is for validating if the intent needs any further fleshing out.",
        "",
        " @param  intent The input intent",
        " @param  completion The response block contains an INIntentResolutionResult for the parameter being resolved",
        "",
        " @see INIntentResolutionResult",
    ],
    "options": [
        " @abstract Dynamic options methods - provide options for the parameter at runtime",
        " @discussion Called to query dynamic options for the parameter and this intent in its current form.",
        "",
        " @param  intent The input intent",
    ],
    "search": [
        " @param  searchTerm The search term",
    ],
    "options_completion": [
        " @param  completion The response block contains options for the parameter",
    ],
    "confirm": [
        " @abstract Confirmation method - Validate that this intent is ready for the next step (i.e. handling)",
        " @discussion Called prior to asking the app to handle the intent. The app should return a response object that contains additional information about the intent, which may be relevant for the system to show the user prior to handling. If unimplemented, the system will assume the intent is valid, and will assume there is no additional information relevant to this intent.",
        "",
        " @param  intent The input intent",
        " @param  completion The response block contains a %(c)sResponse containing additional details about the intent that may be relevant for the system to show the user prior to handling.",
        "",
        " @see %(c)sResponse",
    ],
    "default": [
        " @abstract Default values for parameters with dynamic options",
        " @discussion Called to query the parameter default value.",
    ],
    "deprecated": [
        " @abstract Deprecated dynamic options methods.",
    ],
    "response_code": [
        " @abstract Constants indicating the state of the response.",
    ],
    "code": [
        " @abstract The response code indicating your success or failure in confirming or handling the intent.",
    ],
    "init": [
        " @abstract Initializes the response object with the specified code and user activity object.",
        " @discussion The app extension has the option of capturing its private state as an NSUserActivity and returning it as the 'currentActivity'. If the app is launched, an NSUserActivity will be passed in with the private state. The NSUserActivity may also be used to query the app's UI extension (if provided) for a view controller representing the current intent handling state. In the case of app launch, the NSUserActivity will have its activityType set to the name of the intent. This intent object will also be available in the NSUserActivity.interaction property.",
        "",
        " @param  code The response code indicating your success or failure in confirming or handling the intent.",
        " @param  userActivity The user activity object to use when launching your app. Provide an object if you want to add information that is specific to your app. If you specify nil, the system automatically creates a user activity object for you, sets its type to the class name of the intent being handled, and fills it with an INInteraction object containing the intent and your response.",
    ],
    "code_init": [
        " @abstract Initializes and returns the response object with the %(code)s code.",
    ],
}

SUCCESS_COMMENT = [
    "// This resolution result is for when the app extension wants to tell Siri to proceed, with a given %(c)s. The resolvedValue can be different than the original %(c)s. This allows app extensions to apply business logic constraints.",
    "// Use %(not_required)s to continue with a 'nil' value.",
]
DISAMBIGUATION_COMMENT = ["// This resolution result is to ask Siri to disambiguate between the provided %(c)s."]
CONFIRMATION_COMMENT = ["// This resolution result is to ask Siri to confirm if this is the value with which the user wants to continue."]


def doc(names, indent="", **values):
    lines = ["/*!"]
    for name in names.split():
        lines += DOCS[name]
    lines.append(" */")
    return [indent + line % values if line else "" for line in lines]


def comment(lines, indent="", **values):
    return [indent + line % values for line in lines]


def _type(swift, objc, resolution=None, collection=None, reference=None,
          availability=None, swift_import=None, objc_import=None):
    return {
        "swift": swift,
        "objc": objc,
        "resolution": resolution,
        "collection": collection or swift,
        "reference": reference or collection or swift,
        "availability": availability,
        "swift_import": swift_import,
        "objc_import": objc_import,
    }


def _measurement(unit, resolution, availability=None):
    return _type(
        "Measurement<Unit%s>" % unit,
        "NSMeasurement<NSUnit%s *> *" % unit,
        resolution,
        reference="NSMeasurement",
        availability=availability,
    )


TYPES = {
    "Boolean": _type("NSNumber", "NSNumber *", "INBooleanResolutionResult", availability=IOS13_FOUNDATION),
    "CurrencyAmount": _type("INCurrencyAmount", "INCurrencyAmount *", "INCurrencyAmountResolutionResult", availability=("12.0", "12.0", "5.0")),
    "Data": _type("Data", "NSData *"),
    "Date": _type("Date", "NSDate *"),
    "DateComponents": _type("DateComponents", "NSDateComponents *", "INDateComponentsResolutionResult", reference="NSDateComponents", availability=IOS13_FOUNDATION),
    "DateComponentsRange": _type("INDateComponentsRange", "INDateComponentsRange *", "INDateComponentsRangeResolutionResult"),
    "Decimal": _type("NSNumber", "NSNumber *", "INDoubleResolutionResult", collection="Double"),
    "Dictionary": _type("Dictionary", "NSDictionary *"),
    "Distance": _measurement("Length", "INLengthResolutionResult"),
    "Energy": _measurement("Energy", "INEnergyResolutionResult", IOS13_FOUNDATION),
    "File": _type("INFile", "INFile *", "INFileResolutionResult", availability=IOS13_FOUNDATION),
    "Image": _type("INImage", "INImage *"),
    "Integer": _type("NSNumber", "NSNumber *", "INIntegerResolutionResult", collection="Int"),
    "Location": _type("CLLocation", "CLLocation *", swift_import="CoreLocation", objc_import="<CoreLocation/CoreLocation.h>"),
    "Mass": _measurement("Mass", "INMassResolutionResult", IOS13_FOUNDATION),
    "Object": _type("INObject", "INObject *", "INObjectResolutionResult"),
    "PaymentMethod": _type("INPaymentMethod", "INPaymentMethod *", "INPaymentMethodResolutionResult"),
    "Person": _type("INPerson", "INPerson *", "INPersonResolutionResult"),
    "Placemark": _type("CLPlacemark", "CLPlacemark *", "INPlacemarkResolutionResult", swift_import="CoreLocation", objc_import="<CoreLocation/CoreLocation.h>"),
    "SpeakableString": _type("INSpeakableString", "INSpeakableString *", "INSpeakableStringResolutionResult"),
    "Speed": _measurement("Speed", "INSpeedResolutionResult", IOS13_FOUNDATION),
    "String": _type("String", "NSString *", "INStringResolutionResult", reference="NSString"),
    "Temperature": _measurement("Temperature", "INTemperatureResolutionResult"),
    "TimeInterval": _type("NSNumber", "NSNumber *", "INTimeIntervalResolutionResult", availability=IOS13_FOUNDATION),
    "URL": _type("URL", "NSURL *", "INURLResolutionResult", reference="NSURL"),
    "UUID": _type("UUID", "NSUUID *"),
    "Volume": _measurement("Volume", "INVolumeResolutionResult", IOS13_FOUNDATION),
}


def upper_first(name):
    return name[:1].upper() + name[1:]


def lower_first(name):
    return name[:1].lower() + name[1:]


def swift_availability(availability, tvos=False):
    text = "@available(iOS %s, macOS %s, watchOS %s, *)" % availability
    if tvos:
        text += " @available(tvOS, unavailable)"
    return text


def objc_availability(availability, tvos=False):
    text = "API_AVAILABLE(ios(%s), macos(%s), watchos(%s))" % availability
    if tvos:
        text += " API_UNAVAILABLE(tvos)"
    return text


def banner(name):
    return [
        "//",
        "// %s" % name,
        "//",
        "// This file was automatically generated and should not be edited.",
        "//",
        "",
    ]


class Enum:
    def __init__(self, data, prefix):
        self.name = data["INEnumName"]
        self.class_name = data.get("INEnumClassName") or prefix + self.name
        self.generates_header = bool(data.get("INEnumGeneratesHeader"))
        self.values = []
        index = 0
        for value in data.get("INEnumValues", []):
            index = value.get("INEnumValueIndex", index)
            self.values.append((value["INEnumValueName"], index))
            index += 1


class Type:
    def __init__(self, data, prefix):
        self.name = data["INTypeName"]
        self.class_name = data.get("INTypeClassName") or prefix + self.name
        self.properties = sorted_properties(
            [p for p in data.get("INTypeProperties", []) if not p.get("INTypePropertyDefault")],
            "INTypeProperty",
        )


def sorted_properties(items, key):
    return sorted((Property(data, key) for data in items), key=lambda prop: prop.priority)


class Property:
    def __init__(self, data, key):
        self.name = data[key + "Name"]
        self.priority = data.get(key + "DisplayPriority", 0)
        self.type = data.get(key + "Type", "String")
        self.object_type = data.get(key + "ObjectType")
        self.enum_type = data.get(key + "EnumType")
        self.multiple = bool(data.get(key + "SupportsMultipleValues"))
        self.configurable = bool(data.get(key + "Configurable"))
        self.resolution = bool(data.get(key + "SupportsResolution"))
        self.dynamic = bool(data.get(key + "SupportsDynamicEnumeration")) or self.type == "PaymentMethod"
        self.search = bool(data.get(key + "SupportsSearch"))
        defaults = DEFAULT_REASONS.get(self.type, []) if self.resolution and not self.enum_type else []
        listed = [r[key + "UnsupportedReasonCode"] for r in data.get(key + "UnsupportedReasons", [])]
        self.unsupported_reasons = [r for r in defaults if r not in listed] + listed
        self.confirmation_reasons = [
            r[key + "ConfirmationReasonCode"]
            for r in data.get(key + "ConfirmationReasons", [])
        ]
        self.has_reasons = bool(self.unsupported_reasons or self.confirmation_reasons)
        self.custom_result = self.resolution and self.has_reasons
        self.provides_options = self.dynamic and (self.configurable or self.resolution)


class Code:
    def __init__(self, name, value, parameters, assignments, key_path):
        self.name = name
        self.value = value
        self.parameters = parameters
        self.assignments = assignments
        self.key_path = key_path

    def availability(self, type_availability):
        if self.key_path:
            return IOS13
        result = None
        for parameter in self.parameters:
            availability = type_availability(parameter)
            if availability and (result is None or float(availability[0]) > float(result[0])):
                result = availability
        return result


class Intent:
    def __init__(self, data, prefix):
        self.name = data["INIntentName"]
        self.class_name = data.get("INIntentClassName") or prefix + self.name + "Intent"
        self.category = data.get("INIntentCategory")
        self.widgets = bool(data.get("INIntentEligibleForWidgets"))
        self.parameters = sorted_properties(data.get("INIntentParameters", []), "INIntentParameter")
        response = data.get("INIntentResponse", {})
        self.response_parameters = sorted_properties(response.get("INIntentResponseParameters", []), "INIntentResponseParameter")
        self.codes = self._codes(response.get("INIntentResponseCodes", []))

    def _codes(self, definitions):
        defined = {}
        for code in definitions:
            defined.setdefault(code["INIntentResponseCodeName"], code)
        custom = [name for name in defined if name not in DEFAULT_CODES]
        codes = []
        for index, name in enumerate(DEFAULT_CODES + custom):
            value = None
            if index == 0:
                value = 0
            elif custom and name == custom[0]:
                value = 100
            codes.append(Code(name, value, *self._code_parameters(name, defined.get(name, {}))))
        return codes

    def _code_parameters(self, name, code):
        text = " ".join(code.get(k, "") for k in ("INIntentResponseCodeFormatString", "INIntentResponseCodeConciseFormatString"))
        tokens = re.findall(r"\$\{([A-Za-z0-9_]+)((?:\.[A-Za-z0-9_]+)*)\}", text)
        by_name = {p.name: p for p in self.response_parameters}
        names = []
        assignments = []
        if self.category == "toggle" and "state" in by_name and name != "failure":
            names.append("state")
            assignments.append("state")
        seen = set()
        for token, path in tokens:
            if token not in by_name or (token, path) in seen:
                continue
            seen.add((token, path))
            if token not in names:
                names.append(token)
            if token not in assignments or path:
                assignments.append(token)
        return [by_name[n] for n in names], assignments, any(path for _, path in tokens)


class Model:
    def __init__(self, data, prefix):
        self.prefix = prefix
        version = str(data.get("INIntentDefinitionModelVersion", "1.2"))
        self.legacy = tuple(int(x) for x in re.findall(r"\d+", version)[:2]) < (1, 2)
        self.enums = [Enum(e, prefix) for e in data.get("INEnums", [])]
        self.types = [Type(t, prefix) for t in data.get("INTypes", [])]
        self.intents = [
            Intent(i, prefix)
            for i in data.get("INIntents", [])
            if i.get("INIntentType", "Custom") == "Custom"
        ]
        self.enums_by_name = {e.name: e for e in self.enums}
        self.types_by_name = {t.name: t for t in self.types}


class Generator:
    def __init__(self, model):
        self.model = model
        self.intent = None

    def enum(self, prop):
        if prop.enum_type is None:
            return None
        return self.model.enums_by_name.get(prop.enum_type)

    def object_type(self, prop):
        if prop.type != "Object" or prop.object_type is None:
            return None
        return self.model.types_by_name.get(prop.object_type)

    def info(self, prop):
        if prop.type not in TYPES:
            raise SystemExit("intentbuilderc: unsupported type %s for %s" % (prop.type, prop.name))
        return TYPES[prop.type]

    def enum_name(self, enum):
        if enum.generates_header or self.intent is None:
            return enum.class_name
        return self.model.prefix + self.intent.name + enum.name

    def type_name(self, prop):
        enum = self.enum(prop)
        if enum is not None:
            return enum.name
        object_type = self.object_type(prop)
        if object_type is not None:
            return object_type.name
        return prop.type

    def type_availability(self, prop):
        if self.enum(prop) is not None or self.object_type(prop) is not None:
            return None
        return self.info(prop)["availability"]

    def result_base(self, prop):
        return self.model.prefix + self.intent.name + upper_first(prop.name)

    def result_parent(self, prop):
        enum = self.enum(prop)
        if enum is not None:
            return enum.class_name + "ResolutionResult"
        object_type = self.object_type(prop)
        if object_type is not None:
            return object_type.class_name + "ResolutionResult"
        return self.info(prop)["resolution"]

    def result(self, prop):
        if prop.custom_result:
            return self.result_base(prop) + "ResolutionResult"
        return self.result_parent(prop)

    def legacy_enums(self, intent):
        enums = []
        for prop in intent.parameters + intent.response_parameters:
            enum = self.enum(prop)
            if enum is not None and not enum.generates_header and enum not in enums:
                enums.append(enum)
        return enums

    def files(self):
        out = {}
        for enum in self.model.enums:
            if enum.generates_header:
                out.update(self.enum_files(enum))
        for t in self.model.types:
            out.update(self.type_files(t))
        for intent in self.model.intents:
            self.intent = intent
            out.update(self.intent_files(intent))
            self.intent = None
        return out


class Swift(Generator):
    def __init__(self, model, visibility):
        Generator.__init__(self, model)
        self.visibility = "internal" if visibility == "project" else "public"

    def single_type(self, prop):
        enum = self.enum(prop)
        if enum is not None:
            return self.enum_name(enum)
        object_type = self.object_type(prop)
        if object_type is not None:
            return object_type.class_name
        return self.info(prop)["swift"]

    def item_type(self, prop, key="collection"):
        enum = self.enum(prop)
        if enum is not None:
            return self.enum_name(enum)
        object_type = self.object_type(prop)
        if object_type is not None:
            return object_type.class_name
        return self.info(prop)[key]

    def property_type(self, prop):
        if self.enum(prop) is not None:
            return "[Int]" if prop.multiple else self.single_type(prop)
        if prop.multiple:
            return "[%s]?" % self.item_type(prop)
        return "%s?" % self.single_type(prop)

    def value_type(self, prop):
        if prop.multiple:
            return "[%s]" % self.item_type(prop)
        return self.single_type(prop)

    def header(self, name):
        return banner(name + ".swift") + ["#if canImport(Intents)", ""]

    def imports(self, props):
        lines = ["import Intents"]
        for prop in props:
            module = self.info(prop)["swift_import"]
            if module and "import " + module not in lines:
                lines.append("import " + module)
        return lines

    def properties(self, props, in_type=False):
        lines = []
        for prop in props:
            availability = self.type_availability(prop)
            if in_type and availability is None:
                availability = IOS13
            if availability is not None:
                lines += ["", "    " + swift_availability(availability)]
            lines.append("    @NSManaged %s var %s: %s" % (self.visibility, prop.name, self.property_type(prop)))
        if in_type:
            lines.append("")
        return lines

    def enum_cases(self, enum):
        return ["    case `%s` = %d" % (lower_first(value), index) for value, index in enum.values]

    def enum_files(self, enum):
        v = self.visibility
        c = enum.class_name
        lines = self.header(c)
        lines += [
            "import Intents",
            "",
            swift_availability(IOS12, tvos=True),
            "@objc %s enum %s: Int {" % (v, c),
        ]
        lines += self.enum_cases(enum)
        lines += [
            "}",
            "",
            swift_availability(IOS13, tvos=True),
            "@objc(%sResolutionResult)" % c,
            "%s class %sResolutionResult: INEnumResolutionResult {" % (v, c),
            "",
        ]
        lines += comment(SUCCESS_COMMENT, "    ", c=c, not_required="notRequired()")
        lines += [
            "    @objc(successWithResolved%s:)" % enum.name,
            "    %s class func success(with resolvedValue: %s) -> Self {" % (v, c),
            "        return __success(withResolvedValue: resolvedValue.rawValue)",
            "    }",
            "",
        ]
        lines += comment(CONFIRMATION_COMMENT, "    ")
        lines += [
            "    @objc(confirmationRequiredWith%sToConfirm:)" % enum.name,
            "    %s class func confirmationRequired(with valueToConfirm: %s) -> Self {" % (v, c),
            "        return __confirmationRequiredWithValue(toConfirm: valueToConfirm.rawValue)",
            "    }",
            "}",
            "",
            "#endif",
        ]
        return {c + ".swift": lines}

    def type_files(self, t):
        v = self.visibility
        c = t.class_name
        lines = self.header(c)
        lines += self.imports(t.properties)
        lines += [
            "",
            swift_availability(IOS12, tvos=True),
            "@objc(%s)" % c,
            "%s class %s: INObject {" % (v, c),
        ]
        lines += self.properties(t.properties, in_type=True)
        lines += [
            "    override %s class var supportsSecureCoding: Bool { true }" % v,
            "",
            "}",
            "",
            swift_availability(IOS13, tvos=True),
            "@objc(%sResolutionResult)" % c,
            "%s class %sResolutionResult: INObjectResolutionResult {" % (v, c),
            "",
        ]
        lines += comment(SUCCESS_COMMENT, "    ", c=c, not_required="notRequired()")
        lines += [
            "    @objc(successWithResolved%s:)" % t.name,
            "    %s class func success(with resolvedObject: %s) -> Self {" % (v, c),
            "        return super.success(with: resolvedObject)",
            "    }",
            "",
        ]
        lines += comment(DISAMBIGUATION_COMMENT, "    ", c=c)
        lines += [
            "    @objc(disambiguationWith%ssToDisambiguate:)" % t.name,
            "    %s class func disambiguation(with objectsToDisambiguate: [%s]) -> Self {" % (v, c),
            "        return super.disambiguation(with: objectsToDisambiguate)",
            "    }",
            "",
        ]
        lines += comment(CONFIRMATION_COMMENT, "    ")
        lines += [
            "    @objc(confirmationRequiredWith%sToConfirm:)" % t.name,
            "    %s class func confirmationRequired(with objectToConfirm: %s?) -> Self {" % (v, c),
            "        return super.confirmationRequired(with: objectToConfirm)",
            "    }",
            "",
        ]
        for signature in (
            "success(with resolvedObject: INObject)",
            "disambiguation(with objectsToDisambiguate: [INObject])",
            "confirmationRequired(with objectToConfirm: INObject?)",
        ):
            lines += [
                "    @available(*, unavailable)",
                "    override %s class func %s -> Self {" % (v, signature),
                "        fatalError()",
                "    }",
                "",
            ]
        lines += ["}", "", "#endif"]
        return {c + ".swift": lines}

    def method(self, availability, selector, signature, renamed=None):
        lines = []
        if availability:
            lines.append("    " + availability)
        if renamed:
            lines.append('    @available(*, renamed: "%s")' % renamed)
        lines += ["    @objc(%s)" % selector, "    " + signature, ""]
        return lines

    def handle_methods(self, intent, optional):
        c = intent.class_name
        o = "optional " if optional else ""
        selector = "handle%s:completion:" % intent.name
        handling = doc("handle", "    ", c=c)
        completion = self.method(None, selector, "%sfunc handle(intent: %s, completion: @escaping (%sResponse) -> Swift.Void)" % (o, c, c), "handle(intent:)")
        completion[-1] = "    "
        asynchronous = self.method(swift_availability(IOS13), selector, "%sfunc handle(intent: %s) async -> %sResponse" % (o, c, c))
        if optional:
            return handling + completion + asynchronous
        return completion + handling + asynchronous

    def options_methods(self, intent, prop):
        c = intent.class_name
        n = intent.name
        p = upper_first(prop.name)
        legacy = self.model.legacy
        collection = self.item_type(prop, "reference")
        if prop.search:
            name = ("provide%sOptions" if legacy else "provide%sOptionsCollection") % p
            selector = "%sFor%s:searchTerm:withCompletion:" % (name, n)
            return (
                self.method(swift_availability(IOS14), selector, "func %s(for intent: %s, searchTerm: String?, with completion: @escaping (INObjectCollection<%s>?, Error?) -> Swift.Void)" % (name, c, collection), "%s(for:searchTerm:)" % name)
                + self.method(swift_availability(IOS14), selector, "func %s(for intent: %s, searchTerm: String?) async throws -> INObjectCollection<%s>" % (name, c, collection))
            )
        if legacy:
            item = self.item_type(prop)
            selector = "provide%sOptionsFor%s:withCompletion:" % (p, n)
            return (
                self.method(swift_availability(IOS13), selector, "optional func provide%sOptions(for intent: %s, with completion: @escaping ([%s]?, Error?) -> Swift.Void)" % (p, c, item), "provide%sOptions(for:)" % p)
                + self.method(swift_availability(IOS13), selector, "optional func provide%sOptions(for intent: %s) async throws -> [%s]" % (p, c, item))
            )
        selector = "provide%sOptionsCollectionFor%s:withCompletion:" % (p, n)
        return (
            self.method(swift_availability(IOS14), selector, "func provide%sOptionsCollection(for intent: %s, with completion: @escaping (INObjectCollection<%s>?, Error?) -> Swift.Void)" % (p, c, collection), "provide%sOptionsCollection(for:)" % p)
            + self.method(swift_availability(IOS14), selector, "func provide%sOptionsCollection(for intent: %s) async throws -> INObjectCollection<%s>" % (p, c, collection))
        )

    def protocol(self, intent):
        c = intent.class_name
        n = intent.name
        lines = [
            swift_availability(IOS12, tvos=True),
            "@objc(%sHandling)" % c,
            "%s protocol %sHandling: NSObjectProtocol {" % (self.visibility, c),
            "",
        ]
        if not intent.widgets:
            lines += self.handle_methods(intent, False)
        resolved = [prop for prop in intent.parameters if prop.resolution]
        if resolved:
            lines += doc("resolve", "    ")
        for prop in resolved:
            p = upper_first(prop.name)
            result = self.result(prop)
            if prop.multiple:
                result = "[%s]" % result
            selector = "resolve%sFor%s:withCompletion:" % (p, n)
            lines += self.method(swift_availability(IOS13), selector, "func resolve%s(for intent: %s, with completion: @escaping (%s) -> Swift.Void)" % (p, c, result), "resolve%s(for:)" % p)
            lines += self.method(swift_availability(IOS13), selector, "func resolve%s(for intent: %s) async -> %s" % (p, c, result))
        options = [prop for prop in intent.parameters if prop.provides_options]
        ordered = [prop for prop in options if prop.search] + [prop for prop in options if not prop.search]
        if ordered:
            lines += doc("options search options_completion" if ordered[0].search else "options options_completion", "    ")
        for prop in ordered:
            lines += self.options_methods(intent, prop)
        selector = "confirm%s:completion:" % n
        lines += self.method(None, selector, "optional func confirm(intent: %s, completion: @escaping (%sResponse) -> Swift.Void)" % (c, c), "confirm(intent:)")
        lines += doc("confirm", "    ", c=c)
        lines += self.method(swift_availability(IOS13), selector, "optional func confirm(intent: %s) async -> %sResponse" % (c, c))
        if intent.widgets:
            lines += self.handle_methods(intent, True)
        if options:
            lines += doc("default", "    ")
        for prop in options:
            p = upper_first(prop.name)
            lines += self.method(swift_availability(IOS14), "default%sFor%s:" % (p, n), "optional func default%s(for intent: %s) -> %s?" % (p, c, self.value_type(prop)))
        if not self.model.legacy:
            if options:
                lines += doc("deprecated", "    ")
            for prop in options:
                p = upper_first(prop.name)
                lines += [
                    '    @available(iOS, introduced: 13.0, deprecated: 14.0, message: "")',
                    '    @available(watchOS, introduced: 6.0, deprecated: 7.0, message: "")',
                ]
                lines += self.method(None, "provide%sOptionsFor%s:withCompletion:" % (p, n), "optional func provide%sOptions(for intent: %s, with completion: @escaping ([%s]?, Error?) -> Swift.Void)" % (p, c, self.item_type(prop)))
        lines += ["}", ""]
        return lines

    def response(self, intent):
        c = intent.class_name
        v = self.visibility
        lines = doc("response_code") + [
            swift_availability(IOS12, tvos=True),
            "@objc %s enum %sResponseCode: Int {" % (v, c),
        ]
        for code in intent.codes:
            if code.key_path:
                lines.append("    " + swift_availability(IOS13))
            value = "" if code.value is None else " = %d" % code.value
            lines.append("    case %s%s" % (code.name, value))
        lines += [
            "}",
            "",
            swift_availability(IOS12, tvos=True),
            "@objc(%sResponse)" % c,
            "%s class %sResponse: INIntentResponse {" % (v, c),
            "",
        ]
        lines += self.properties(intent.response_parameters)
        if intent.response_parameters:
            lines.append("")
        lines += doc("code", "    ")
        lines += [
            "    @objc %s fileprivate(set) var code: %sResponseCode = .unspecified" % (v, c),
            "",
        ]
        lines += doc("init", "    ")
        lines += [
            "    @objc(initWithCode:userActivity:)",
            "    %s convenience init(code: %sResponseCode, userActivity: NSUserActivity?) {" % (v, c),
            "        self.init()",
            "        self.code = code",
            "        self.userActivity = userActivity",
            "    }",
        ]
        for code in intent.codes:
            if not code.parameters:
                continue
            lines.append("")
            lines += doc("code_init", "    ", code=code.name)
            availability = code.availability(self.type_availability)
            if availability:
                lines.append("    " + swift_availability(availability))
            first = code.parameters[0]
            selector = "%sIntentResponseWith%s:" % (code.name, upper_first(first.name)) + "".join("%s:" % p.name for p in code.parameters[1:])
            arguments = ", ".join("%s: %s" % (p.name, self.value_type(p)) for p in code.parameters)
            lines += [
                "    @objc(%s)" % selector,
                "    %s static func %s(%s) -> %sResponse {" % (v, code.name, arguments, c),
                "        let intentResponse = %sResponse(code: .%s, userActivity: nil)" % (c, code.name),
            ]
            lines += ["        intentResponse.%s = %s" % (a, a) for a in code.assignments]
            lines += ["        return intentResponse", "    }"]
        lines += ["", "}", ""]
        return lines

    def resolution_results(self, intent):
        v = self.visibility
        lines = []
        for prop in intent.parameters:
            if not prop.custom_result:
                continue
            base = self.result_base(prop)
            for kind, reasons in (("Unsupported", prop.unsupported_reasons), ("Confirmation", prop.confirmation_reasons)):
                if not reasons:
                    continue
                lines += [swift_availability(IOS13), "@objc %s enum %s%sReason: Int {" % (v, base, kind)]
                lines += ["    case %s%s" % (reason, " = 1" if index == 0 else "") for index, reason in enumerate(reasons)]
                lines += ["}", ""]
            lines += [
                swift_availability(IOS13),
                "@objc(%sResolutionResult)" % base,
                "%s class %sResolutionResult: %s {" % (v, base, self.result_parent(prop)),
            ]
            if prop.unsupported_reasons:
                lines += [
                    "    @objc(unsupportedForReason:)",
                    "    %s class func unsupported(forReason reason: %sUnsupportedReason) -> Self {" % (v, base),
                    "        return __unsupported(withReason: reason.rawValue)",
                    "    }",
                ]
            if prop.confirmation_reasons:
                lines += [
                    "    @objc(confirmationRequiredWith%sToConfirm:forReason:)" % self.type_name(prop),
                    "    %s class func confirmationRequired(with itemToConfirm: %s, forReason reason: %sConfirmationReason) -> Self {" % (v, self.value_type(prop), base),
                    "        return __confirmationRequiredWithItem(toConfirm: itemToConfirm, forReason: reason.rawValue)",
                    "    }",
                ]
            lines += ["}", ""]
        return lines

    def intent_files(self, intent):
        c = intent.class_name
        lines = self.header(c)
        lines += self.imports(intent.parameters + intent.response_parameters)
        lines.append("")
        for enum in self.legacy_enums(intent):
            name = self.enum_name(enum)
            message = DEPRECATION % (name, enum.class_name)
            lines += [
                '@available(iOS, introduced: 12.0, deprecated: 12.0, message: "%s", renamed: "%s")' % (message, enum.class_name),
                '@available(watchOS, introduced: 5.0, deprecated: 5.0, message: "%s", renamed: "%s")' % (message, enum.class_name),
                "@objc %s enum %s: Int {" % (self.visibility, name),
            ]
            lines += self.enum_cases(enum)
            lines += ["}", ""]
        lines += [
            swift_availability(IOS12, tvos=True),
            "@objc(%s)" % c,
            "%s class %s: INIntent {" % (self.visibility, c),
            "",
        ]
        lines += self.properties(intent.parameters)
        lines += ["", "}", ""]
        lines += doc("protocol", c=c)
        lines += self.protocol(intent)
        lines += self.response(intent)
        lines += self.resolution_results(intent)
        lines.append("#endif")
        return {c + ".swift": lines}


class ObjC(Generator):
    def __init__(self, model, visibility):
        Generator.__init__(self, model)
        self.hidden = visibility == "project"
        self.enum_attribute = HIDDEN + " " if self.hidden else ""

    def pointer_type(self, prop):
        if self.enum(prop) is not None:
            return "NSNumber *"
        object_type = self.object_type(prop)
        if object_type is not None:
            return object_type.class_name + " *"
        return self.info(prop)["objc"]

    def declaration(self, prop):
        enum = self.enum(prop)
        if enum is not None and not prop.multiple:
            return "assign", self.enum_name(enum) + " "
        if prop.multiple:
            return "copy, nullable", "NSArray<%s> *" % self.pointer_type(prop)
        return "copy, nullable", self.pointer_type(prop)

    def value_type(self, prop):
        return self.declaration(prop)[1].rstrip()

    def interface(self, availability, declaration):
        return [availability] + ([HIDDEN] if self.hidden else []) + [declaration]

    def properties(self, props, in_type=False):
        lines = []
        for prop in props:
            attributes, declared = self.declaration(prop)
            availability = self.type_availability(prop)
            if in_type and availability is None:
                availability = IOS13
            suffix = " " + objc_availability(availability) if availability else ""
            lines.append("@property (readwrite, %s, nonatomic) %s%s%s;" % (attributes, declared, prop.name, suffix))
        return lines

    def imports(self, props, forward_types):
        lines = ["#import <Intents/Intents.h>"]
        forward = []
        for prop in props:
            enum = self.enum(prop)
            object_type = self.object_type(prop)
            line = None
            if enum is not None:
                if enum.generates_header:
                    line = '#import "%s.h"' % enum.class_name
            elif object_type is not None:
                if forward_types:
                    if object_type.class_name not in forward:
                        forward.append(object_type.class_name)
                else:
                    line = '#import "%s.h"' % object_type.class_name
            elif self.info(prop)["objc_import"]:
                line = "#import %s" % self.info(prop)["objc_import"]
            if line and line not in lines:
                lines.append(line)
        return lines, forward

    def enum_body(self, name, values):
        lines = ["typedef NS_ENUM(NSInteger, %s) {" % name]
        for i, (value, index) in enumerate(values):
            comma = "," if i + 1 < len(values) else ""
            lines.append("    %s%s = %d%s" % (name, upper_first(value), index, comma))
        return lines

    def source_header(self, c):
        return banner(c + ".m") + [
            '#import "%s.h"' % c,
            "",
            "#if __has_include(<Intents/Intents.h>) && !TARGET_OS_TV",
            "",
        ]

    def enum_files(self, enum):
        c = enum.class_name
        header = banner(c + ".h")
        header += ["#if __has_include(<Intents/Intents.h>)", "", "#import <Intents/Intents.h>", "", "NS_ASSUME_NONNULL_BEGIN", ""]
        header += self.enum_body(c, enum.values)
        header += ["} %s%s;" % (self.enum_attribute, objc_availability(IOS12, tvos=True)), ""]
        header += self.interface(objc_availability(IOS13, tvos=True), "@interface %sResolutionResult : INEnumResolutionResult" % c)
        header.append("")
        header += comment(SUCCESS_COMMENT, c=c, not_required="+notRequired")
        header += [
            "+ (instancetype)successWithResolved%s:(%s)resolvedValue NS_SWIFT_NAME(success(with:));" % (enum.name, c),
            "",
        ]
        header += comment(CONFIRMATION_COMMENT)
        header += [
            "+ (instancetype)confirmationRequiredWith%sToConfirm:(%s)valueToConfirm NS_SWIFT_NAME(confirmationRequired(with:));" % (enum.name, c),
            "",
            "+ (instancetype)successWithResolvedValue:(NSInteger)resolvedValue NS_UNAVAILABLE;",
            "+ (instancetype)confirmationRequiredWithValueToConfirm:(NSInteger)valueToConfirm NS_UNAVAILABLE;",
            "",
            "@end",
            "",
            "NS_ASSUME_NONNULL_END",
            "",
            "#endif",
        ]
        source = self.source_header(c) + [
            "@implementation %sResolutionResult" % c,
            "",
            "+ (instancetype)successWithResolved%s:(%s)resolvedValue {" % (enum.name, c),
            "    return [super successWithResolvedValue:resolvedValue];",
            "}",
            "",
            "+ (instancetype)confirmationRequiredWith%sToConfirm:(%s)valueToConfirm {" % (enum.name, c),
            "    return [super confirmationRequiredWithValueToConfirm:valueToConfirm];",
            "}",
            "",
            "@end",
            "",
            "#endif",
        ]
        return {c + ".h": header, c + ".m": source}

    def type_files(self, t):
        c = t.class_name
        imports, forward = self.imports(t.properties, True)
        header = banner(c + ".h") + ["#if __has_include(<Intents/Intents.h>)", ""] + imports + [""]
        if forward:
            header += ["@class %s;" % name for name in forward] + [""]
        header += ["NS_ASSUME_NONNULL_BEGIN", ""]
        header += self.interface(objc_availability(IOS12, tvos=True), "@interface %s : INObject" % c)
        header.append("")
        if t.properties:
            header += self.properties(t.properties, in_type=True) + [""]
        header += ["@end", ""]
        header += self.interface(objc_availability(IOS13, tvos=True), "@interface %sResolutionResult : INObjectResolutionResult" % c)
        header.append("")
        header += comment(SUCCESS_COMMENT, c=c, not_required="+notRequired")
        header += [
            "+ (instancetype)successWithResolved%s:(%s *)resolvedObject NS_SWIFT_NAME(success(with:));" % (t.name, c),
            "",
        ]
        header += comment(DISAMBIGUATION_COMMENT, c=c)
        header += [
            "+ (instancetype)disambiguationWith%ssToDisambiguate:(NSArray<%s *> *)objectsToDisambiguate NS_SWIFT_NAME(disambiguation(with:));" % (t.name, c),
            "",
        ]
        header += comment(CONFIRMATION_COMMENT)
        header += [
            "+ (instancetype)confirmationRequiredWith%sToConfirm:(%s *)objectToConfirm NS_SWIFT_NAME(confirmationRequired(with:));" % (t.name, c),
            "",
            "+ (instancetype)successWithResolvedObject:(__kindof INObject *)resolvedObject NS_UNAVAILABLE;",
            "+ (instancetype)disambiguationWithObjectsToDisambiguate:(NSArray<__kindof INObject *> *)objectsToDisambiguate NS_UNAVAILABLE;",
            "+ (instancetype)confirmationRequiredWithObjectToConfirm:(nullable __kindof INObject *)objectToConfirm NS_UNAVAILABLE;",
            "",
            "@end",
            "",
            "NS_ASSUME_NONNULL_END",
            "",
            "#endif",
        ]
        source = self.source_header(c) + ["@implementation %s" % c, ""]
        if t.properties:
            source += ["@dynamic %s;" % ", ".join(p.name for p in t.properties), ""]
        source += [
            "@end",
            "",
            "@implementation %sResolutionResult" % c,
            "",
            "+ (instancetype)successWithResolved%s:(%s *)resolvedObject {" % (t.name, c),
            "    return [super successWithResolvedObject:resolvedObject];",
            "}",
            "",
            "+ (instancetype)disambiguationWith%ssToDisambiguate:(NSArray<%s *> *)objectsToDisambiguate {" % (t.name, c),
            "    return [super disambiguationWithObjectsToDisambiguate:objectsToDisambiguate];",
            "}",
            "",
            "+ (instancetype)confirmationRequiredWith%sToConfirm:(%s *)objectToConfirm {" % (t.name, c),
            "    return [super confirmationRequiredWithObjectToConfirm:objectToConfirm];",
            "}",
            "",
            "+ (BOOL)supportsSecureCoding {",
            "    return YES;",
            "}",
            "",
            "@end",
            "",
            "#endif",
        ]
        return {c + ".h": header, c + ".m": source}

    def handle_method(self, intent):
        c = intent.class_name
        return doc("handle", c=c) + ["- (void)handle%s:(%s *)intent completion:(void (^)(%sResponse *response))completion NS_SWIFT_NAME(handle(intent:completion:));" % (intent.name, c, c), ""]

    def options_method(self, intent, prop):
        c = intent.class_name
        n = intent.name
        p = upper_first(prop.name)
        item = self.pointer_type(prop)
        if prop.search:
            name = ("provide%sOptions" if self.model.legacy else "provide%sOptionsCollection") % p
            return "- (void)%sFor%s:(%s *)intent searchTerm:(nullable NSString *)searchTerm withCompletion:(void (^)(INObjectCollection<%s> * _Nullable %sOptionsCollection, NSError * _Nullable error))completion NS_SWIFT_NAME(%s(for:searchTerm:with:)) %s;" % (name, n, c, item, prop.name, name, objc_availability(IOS14))
        if self.model.legacy:
            return "- (void)provide%sOptionsFor%s:(%s *)intent withCompletion:(void (^)(NSArray<%s> * _Nullable %sOptions, NSError * _Nullable error))completion NS_SWIFT_NAME(provide%sOptions(for:with:)) %s;" % (p, n, c, item, prop.name, p, objc_availability(IOS13))
        return "- (void)provide%sOptionsCollectionFor%s:(%s *)intent withCompletion:(void (^)(INObjectCollection<%s> * _Nullable %sOptionsCollection, NSError * _Nullable error))completion NS_SWIFT_NAME(provide%sOptionsCollection(for:with:)) %s;" % (p, n, c, item, prop.name, p, objc_availability(IOS14))

    def protocol(self, intent):
        c = intent.class_name
        n = intent.name
        required = []
        if not intent.widgets:
            required += self.handle_method(intent)
        resolved = [prop for prop in intent.parameters if prop.resolution]
        if resolved:
            required += [line.lstrip() for line in doc("resolve")]
        for prop in resolved:
            p = upper_first(prop.name)
            if prop.multiple:
                argument = "NSArray<%s *> *resolutionResults" % self.result(prop)
            else:
                argument = "%s *resolutionResult" % self.result(prop)
            required += ["- (void)resolve%sFor%s:(%s *)intent withCompletion:(void (^)(%s))completion NS_SWIFT_NAME(resolve%s(for:with:)) %s;" % (p, n, c, argument, p, objc_availability(IOS13)), ""]
        options = [prop for prop in intent.parameters if prop.provides_options]
        ordered = [prop for prop in options if prop.search] + [prop for prop in options if not prop.search]
        if ordered:
            required += doc("options search options_completion" if ordered[0].search else "options options_completion")
        for prop in ordered:
            required += [self.options_method(intent, prop), ""]
        lines = doc("protocol", c=c) + self.interface(objc_availability(IOS12, tvos=True), "@protocol %sHandling <NSObject>" % c)
        lines.append("")
        if required:
            lines += ["@required", ""] + required
        lines += ["@optional", ""] + doc("confirm", c=c) + [
            "- (void)confirm%s:(%s *)intent completion:(void (^)(%sResponse *response))completion NS_SWIFT_NAME(confirm(intent:completion:));" % (n, c, c),
            "",
        ]
        if intent.widgets:
            lines += self.handle_method(intent)
        if options:
            lines += doc("default")
        for prop in options:
            p = upper_first(prop.name)
            value = self.value_type(prop)
            if not value.endswith("*"):
                value += " "
            lines += ["- (nullable %s)default%sFor%s:(%s *)intent NS_SWIFT_NAME(default%s(for:)) %s;" % (value, p, n, c, p, objc_availability(IOS14)), ""]
        if not self.model.legacy:
            if options:
                lines += doc("deprecated")
            for prop in options:
                p = upper_first(prop.name)
                lines += ['- (void)provide%sOptionsFor%s:(%s *)intent withCompletion:(void (^)(NSArray<%s> * _Nullable %sOptions, NSError * _Nullable error))completion NS_SWIFT_NAME(provide%sOptions(for:with:)) API_DEPRECATED("", ios(13.0, 14.0), watchos(6.0, 7.0));' % (p, n, c, self.pointer_type(prop), prop.name, p), ""]
        lines += ["@end", ""]
        return lines

    def response_code_enum(self, intent):
        c = intent.class_name
        lines = doc("response_code") + ["typedef NS_ENUM(NSInteger, %sResponseCode) {" % c]
        for index, code in enumerate(intent.codes):
            availability = " " + objc_availability(IOS13) if code.key_path else ""
            value = "" if code.value is None else " = %d" % code.value
            comma = "," if index + 1 < len(intent.codes) else ""
            lines.append("    %sResponseCode%s%s%s%s" % (c, upper_first(code.name), availability, value, comma))
        lines += ["} %s%s;" % (self.enum_attribute, objc_availability(IOS12, tvos=True)), ""]
        return lines

    def initializers(self, intent):
        c = intent.class_name
        declarations = []
        implementations = []
        for code in intent.codes:
            if not code.parameters:
                continue
            first = code.parameters[0]
            selector = "%sIntentResponseWith%s:(%s)%s" % (code.name, upper_first(first.name), self.value_type(first), first.name)
            for p in code.parameters[1:]:
                selector += " %s:(%s)%s" % (p.name, self.value_type(p), p.name)
            swift_name = "%s(%s)" % (code.name, "".join("%s:" % p.name for p in code.parameters))
            availability = code.availability(self.type_availability)
            suffix = " " + objc_availability(availability) if availability else ""
            declarations += doc("code_init", code=code.name)
            declarations.append("+ (instancetype)%s NS_SWIFT_NAME(%s)%s;" % (selector, swift_name, suffix))
            implementations += [
                "+ (instancetype)%s {" % selector,
                "    %sResponse *intentResponse = [[%sResponse alloc] initWithCode:%sResponseCode%s userActivity:nil];" % (c, c, c, upper_first(code.name)),
            ]
            implementations += ["    intentResponse.%s = %s;" % (a, a) for a in code.assignments]
            implementations += ["    return intentResponse;", "}", ""]
        if declarations:
            declarations.append("")
        return declarations, implementations

    def resolution_results(self, intent):
        declarations = []
        implementations = []
        for prop in intent.parameters:
            if not prop.custom_result:
                continue
            base = self.result_base(prop)
            for kind, reasons in (("Unsupported", prop.unsupported_reasons), ("Confirmation", prop.confirmation_reasons)):
                if not reasons:
                    continue
                declarations.append("typedef NS_ENUM(NSInteger, %s%sReason) {" % (base, kind))
                declarations += ["    %s%sReason%s%s," % (base, kind, upper_first(reason), " = 1" if index == 0 else "") for index, reason in enumerate(reasons)]
                declarations += ["} %s%s;" % (self.enum_attribute, objc_availability(IOS13)), ""]
            declarations += self.interface(objc_availability(IOS13), "@interface %sResolutionResult : %s" % (base, self.result_parent(prop)))
            declarations.append("")
            implementations += ["@implementation %sResolutionResult" % base, ""]
            if prop.unsupported_reasons:
                selector = "unsupportedForReason:(%sUnsupportedReason)reason" % base
                declarations += ["+ (instancetype)%s;" % selector, ""]
                implementations += ["+ (instancetype)%s {" % selector, "    return [super unsupportedWithReason:reason];", "}", ""]
            if prop.confirmation_reasons:
                selector = "confirmationRequiredWith%sToConfirm:(%s)valueToConfirm forReason:(%sConfirmationReason)reason" % (self.type_name(prop), self.declaration(prop)[1], base)
                declarations += ["+ (instancetype)%s;" % selector, ""]
                implementations += ["+ (instancetype)%s {" % selector, "    return [super confirmationRequiredWithItemToConfirm:valueToConfirm forReason:reason];", "}", ""]
            declarations += ["@end", ""]
            implementations += ["@end", ""]
        return declarations, implementations

    def intent_files(self, intent):
        c = intent.class_name
        imports, _ = self.imports(intent.parameters + intent.response_parameters, False)
        header = banner(c + ".h") + ["#if __has_include(<Intents/Intents.h>)", ""] + imports + [""]
        for enum in self.legacy_enums(intent):
            name = self.enum_name(enum)
            header += self.enum_body(name, enum.values)
            header += ['} %sAPI_DEPRECATED("%s", ios(12.0, 12.0), watchos(5.0, 5.0));' % (self.enum_attribute, DEPRECATION % (name, enum.class_name)), ""]
        header += ["NS_ASSUME_NONNULL_BEGIN", ""]
        header += self.interface(objc_availability(IOS12, tvos=True), "@interface %s : INIntent" % c)
        header.append("")
        header += self.properties(intent.parameters)
        header += ["", "@end", "", "@class %sResponse;" % c]
        header += ["@class %sResolutionResult;" % self.result_base(prop) for prop in intent.parameters if prop.has_reasons]
        header.append("")
        header += self.protocol(intent)
        header += self.response_code_enum(intent)
        header += self.interface(objc_availability(IOS12, tvos=True), "@interface %sResponse : INIntentResponse" % c)
        header += [
            "",
            "- (instancetype)init NS_UNAVAILABLE;",
            "",
        ]
        header += doc("init")
        header += [
            "- (instancetype)initWithCode:(%sResponseCode)code userActivity:(nullable NSUserActivity *)userActivity NS_DESIGNATED_INITIALIZER;" % c,
            "",
        ]
        initializer_declarations, initializer_implementations = self.initializers(intent)
        header += initializer_declarations
        header += self.properties(intent.response_parameters)
        if intent.response_parameters:
            header.append("")
        header += doc("code")
        header += ["@property (readonly, NS_NONATOMIC_IOSONLY) %sResponseCode code;" % c, "", "@end", ""]
        result_declarations, result_implementations = self.resolution_results(intent)
        header += result_declarations
        header += ["NS_ASSUME_NONNULL_END", "", "#endif"]
        source = self.source_header(c) + ["@implementation %s" % c, ""]
        if intent.parameters:
            source += ["@dynamic %s;" % ", ".join(p.name for p in intent.parameters), ""]
        source += [
            "@end",
            "",
            "@interface %sResponse ()" % c,
            "",
            "@property (readwrite, NS_NONATOMIC_IOSONLY) %sResponseCode code;" % c,
            "",
            "@end",
            "",
            "@implementation %sResponse" % c,
            "",
            "@synthesize code = _code;",
            "",
        ]
        if intent.response_parameters:
            source += ["@dynamic %s;" % ", ".join(p.name for p in intent.response_parameters), ""]
        source += [
            "- (instancetype)initWithCode:(%sResponseCode)code userActivity:(nullable NSUserActivity *)userActivity {" % c,
            "    self = [super init];",
            "    if (self) {",
            "        _code = code;",
            "        self.userActivity = userActivity;",
            "    }",
            "    return self;",
            "}",
            "",
        ]
        source += initializer_implementations
        source += ["@end", ""]
        source += result_implementations
        source.append("#endif")
        return {c + ".h": header, c + ".m": source}


def parse_args(argv):
    options = {}
    command = None
    i = 0
    while i < len(argv):
        arg = argv[i]
        if arg.startswith("-") and i + 1 < len(argv):
            options[arg[1:]] = argv[i + 1]
            i += 2
            continue
        if command is None:
            command = arg
        i += 1
    return command, options


def main(argv):
    command, options = parse_args(argv)
    language = options.get("language", "Swift")
    if command != "generate" or "input" not in options or "output" not in options or language not in ("Swift", "Objective-C"):
        sys.stderr.write("usage: intentbuilderc generate -input <file> -output <dir> -language <Swift|Objective-C> [-classPrefix <prefix>] [-swiftVersion <version>] [-visibility <public|private|project>]\n")
        return 1
    with open(options["input"], "rb") as f:
        model = Model(plistlib.load(f), options.get("classPrefix", ""))
    generator = Swift if language == "Swift" else ObjC
    files = generator(model, options.get("visibility", "")).files()
    os.makedirs(options["output"], exist_ok=True)
    for name, lines in files.items():
        with open(os.path.join(options["output"], name), "w") as f:
            f.write("\n".join(lines) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
