#!/usr/bin/env python3
"""momc off a Mac: a Core Data model (`.xcdatamodeld`, or one
`.xcdatamodel`) compiled as rules_apple runs it (`momc
--<platform>-deployment-target V --module M MODEL OUTPUT`); see
docs/clean-room/momc.md.

Each model version compiles to the `.mom` Xcode 27.2's momc writes, a
keyed archive of the model, and a single version's `VersionInfo.plist` to
Xcode's, with each entity's version hash and the version's checksum. A
model with several versions (whose current one Xcode gives an `.omo` too)
or a derived attribute is refused unless RULES_APPLECROSS_RELAX_MOMC=1
(--config=relax_momc, or all of --config=relaxed), which writes
VersionInfo.plist without the hashes and leaves out the `.omo`: Core Data
loads, saves and migrates stores from the `.mom` and computes the hashes
itself. `--action generate` writes the Swift classes Xcode's does
but for the header's line naming the user and the date, which no build
reproduces, so it too needs RULES_APPLECROSS_RELAX_MOMC=1. A model using
anything not established against Xcode
(composite attributes, fetch indexes, predicates beyond NSPredicate's
established syntax, among others) is refused either way.
"""

import base64
import hashlib
import math
import os
import plistlib
import re
import struct
import sys
import unicodedata
import xml.etree.ElementTree as ET

# corefoundation.py is installed beside this script; its directory isn't on
# the path when Python runs with PYTHONSAFEPATH (as rules_python's tests do).
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from corefoundation import cf_created_order, cf_dictionary_order

RELAXED = os.environ.get("RULES_APPLECROSS_RELAX_MOMC") == "1" or \
    os.environ.get("RULES_APPLECROSS_RESOURCE_OUTPUT_MODE") == "relaxed"


class Unestablished(Exception):
    pass


def unestablished(what):
    raise Unestablished(what)


class Invalid(Exception):
    """A model Xcode's momc refuses too."""


# The momc of each Xcode: its own version (IDEPlugIns) and Xcode's, both
# recorded in every model.
MOMC_VERSIONS = {
    "27.2": (25436, 270200),  # 27B52
}


def xcode_version():
    """The Xcode whose SDK the build targets, from the version.plist beside
    DEVELOPER_DIR. One whose momc this doesn't know is refused, unless
    RULES_APPLECROSS_RELAX_XCODE_VERSION=1 (--config=relax_xcode_version,
    or --config=relaxed) records the nearest known one's."""
    developer = os.environ.get("DEVELOPER_DIR", "/Applications/Xcode.app/Contents/Developer")
    try:
        with open(os.path.join(developer, "..", "version.plist"), "rb") as f:
            info = plistlib.load(f)
    except (OSError, plistlib.InvalidFileException):
        info = {}
    version = info.get("CFBundleShortVersionString", "?")
    if version in MOMC_VERSIONS:
        return version
    if os.environ.get("RULES_APPLECROSS_RELAX_XCODE_VERSION") == "1" or \
            os.environ.get("RULES_APPLECROSS_RESOURCE_OUTPUT_MODE") == "relaxed":
        return "27.2"
    sys.exit("momc: Xcode %s (%s) is not supported by this momc: it writes the models of Xcode 27.2 (--config=relax_xcode_version records its)"
             % (version, info.get("ProductBuildVersion", "?")))


# --- Archived values -----------------------------------------------------------
#
# Apple's keyed archiver gives each object one entry, found again by
# identity, and its property list writer shares a string or number only
# with the same object written again. So every value here carries the
# identity of the object Core Data would hold: one string for each ASCII
# value the model's XML holds (equal ones are the same string), a new one
# for each other attribute value, constants shared.

_fresh = [0]


def fresh():
    _fresh[0] += 1
    return ("fresh", _fresh[0])


class Str:
    def __init__(self, value, ident=None):
        self.value = value
        self.ident = ident or ("str", value)


def xml_str(value):
    """A string read from the model's XML."""
    if value is None:
        return None
    return Str(value, None if value.isascii() else fresh())


class Key:
    """A key: archived classes' keys are constants of their own."""

    def __init__(self, value):
        self.value = value


class Num:
    """A number of one CoreFoundation type ("i32", "i64", "f32" or
    "f64"): equal integers of one type are one object, equal values of
    different types aren't, and a floating-point number in an object of its
    own is that object alone."""

    def __init__(self, kind, value, ident=None):
        self.kind = kind
        self.value = value
        self.ident = ident or ("num", kind, value)


class Bool:
    def __init__(self, value):
        self.value = value
        self.ident = ("bool", value)


class Data:
    def __init__(self, value):
        self.value = value
        self.ident = fresh()


class UID:
    def __init__(self, value):
        self.value = value


class Dict:
    def __init__(self, pairs):
        self.pairs = pairs


class Array:
    def __init__(self, items):
        self.items = items


# Class names that are a new string each time they're asked for; any other
# class's name is one string, shared with an equal string value.
FRESH_CLASS_NAMES = {
    "NSManagedObjectModel", "NSEntityDescription", "NSPropertyDescription", "NSAttributeDescription",
    "NSRelationshipDescription", "NSFetchRequest", "NSPersistentStoreRequest",
    "NSDecimalNumberPlaceholder", "NSDecimalNumber", "NSValue", "NSComparisonPredicateOperator",
    "NSStringPredicateOperator", "NSSubstringPredicateOperator", "NSLikePredicateOperator", "NSVariableExpression",
    "NSTruePredicate", "NSFalsePredicate", "NSAggregateExpression", "NSBetweenPredicateOperator",
    "NSDerivedAttributeDescription", "NSFetchedPropertyDescription",
}

def class_name_string(name):
    """A class's name, as the archiver and Core Data get it."""
    return Str(name, fresh()) if name in FRESH_CLASS_NAMES else Str(name)


# The order each archived class's keys go into its dictionary, which sets
# the order a CoreFoundation dictionary enumerates them in.
KEY_ORDER = {
    "NSManagedObjectModel": ["NSEntities", "NSConfigurations", "NSFetchRequestTemplates", "NSVersionIdentifiers", "$class",
                             "CDMCompilerToolVersion", "CDMCompilerXcodeVersion"],
    "NSEntityDescription": ["NSManagedObjectModel", "NSSuperentity", "NSClassNameForEntity", "NSIsAbstract",
                            "NSEntityName", "NSSubentities", "NSProperties", "NSUserInfo", "NSUniqueTuples",
                            "NSRenamingIdentifier", "NSVersionHashModifier", "$class"],
    "NSAttributeDescription": ["NSVersionHashModifier", "NSUserInfo", "NSIsOptional", "NSPropertyName", "_P",
                               "NSEntity", "NSAllowsCloudEncryption", "NSAttributeType", "NSRenamingIdentifier",
                               "NSAttributeValueClassName", "NSIndexedBySpotlight", "NSIsTransient",
                               "NSValidationWarnings", "NSDefaultValue", "NSFlagsKey", "NSValueTransformerName",
                               "NSValidationPredicates", "$class"],
    "NSRelationshipDescription": ["NSIsOrdered", "NSIsOptional", "NSEntity", "NSMaxCount", "_P", "NSDeleteRule",
                                  "NSMinCount", "NSVersionHashModifier", "NSDestinationEntity", "NSPropertyName",
                                  "NSRenamingIdentifier", "NSUserInfo", "_NSDestinationEntityName",
                                  "NSInverseRelationship", "_NSInverseRelationshipName", "$class"],
    "NSFetchRequest": ["NSEntity", "NSValuesToFetch", "NSBatchSize", "NSPredicate", "NSFetchLimit", "NSAllocationType",
                       "NSHavingPredicate", "NSSortDescriptors", "NSRelationshipKeyPathsForPrefetching", "NSValuesToGroupBy",
                       "NSResultType", "NSFetchOffset", "NSHasFetchRequestFlags", "NSFetchRequestFlags", "$class"],
    "NSDictionary": ["$class", "NS.keys", "NS.objects"],
    "NSMutableDictionary": ["$class", "NS.keys", "NS.objects"],
    "NSArray": ["NS.objects", "$class"],
    "NSMutableArray": ["$class", "NS.objects"],
    "NSSet": ["NS.objects", "$class"],
    "NSDate": ["NS.time", "$class"],
    "NSDecimalNumberPlaceholder": ["NS.mantissa", "NS.exponent", "NS.length", "NS.compact", "NS.negative",
                                   "NS.mantissa.bo", "$class"],
    "NSComparisonPredicate": ["NSLeftExpression", "NSRightExpression", "NSPredicateOperator", "$class"],
    "NSComparisonPredicateOperator": ["NSOperatorType", "NSOptions", "NSVariant", "$class", "NSModifier"],
    "NSMatchingPredicateOperator": ["NSOperatorType", "$class", "NSModifier", "NSFlags"],
    "NSSelfExpression": ["NSExpressionType", "$class"],
    "NSConstantValueExpression": ["NSExpressionType", "NSConstantValueClassName", "$class", "NSConstantValue"],
    "NSKeyPathExpression": ["NSExpressionType", "NSSelectorName", "NSOperand", "$class", "NSArguments"],
    "NSKeyPathSpecifierExpression": ["NSExpressionType", "NSKeyPath", "$class"],
    "NSEqualityPredicateOperator": ["NSNegate", "NSOperatorType", "NSOptions", "$class", "NSModifier"],
    "NSSubstringPredicateOperator": ["NSModifier", "NSOperatorType", "NSPosition", "$class", "NSFlags"],
    "NSInPredicateOperator": ["NSOperatorType", "$class", "NSModifier", "NSFlags"],
    "NSLikePredicateOperator": ["NSOperatorType", "$class", "NSModifier", "NSFlags"],
    "NSBetweenPredicateOperator": ["NSModifier", "NSOperatorType", "$class"],
    "NSCompoundPredicate": ["NSCompoundPredicateType", "NSSubpredicates", "$class"],
    "NSTruePredicate": ["$class"],
    "NSFalsePredicate": ["$class"],
    "NSVariableExpression": ["NSExpressionType", "NSVariable", "$class"],
    "NSAggregateExpression": ["NSCollection", "NSExpressionType", "$class"],
    "NSFunctionExpression": ["NSExpressionType", "NSSelectorName", "NSOperand", "$class", "NSArguments"],
    "NSDerivedAttributeDescription": ["NSAttributeValueClassName", "NSDefaultValue", "NSRenamingIdentifier", "NSPropertyName",
                                      "NSIsOptional", "NSAttributeType", "NSEntity", "NSFlagsKey", "_P", "NSIsReadOnly",
                                      "NSValueTransformerName", "NSDerivationExpression", "NSFilteringPredicate", "$class"],
    "NSFetchedPropertyDescription": ["NSUserInfo", "NSVersionHashModifier", "NSRenamingIdentifier", "NSIsOptional",
                                     "_NSFetchRequestEntityName", "NSIsTransient", "$class", "NSIsReadOnly", "_P",
                                     "NSFetchRequest", "NSEntity", "NSPropertyName"],
}

CLASS_CHAINS = {
    "NSManagedObjectModel": ["NSManagedObjectModel", "NSObject"],
    "NSEntityDescription": ["NSEntityDescription", "NSObject"],
    "NSAttributeDescription": ["NSAttributeDescription", "NSPropertyDescription", "NSObject"],
    "NSRelationshipDescription": ["NSRelationshipDescription", "NSPropertyDescription", "NSObject"],
    "NSFetchRequest": ["NSFetchRequest", "NSPersistentStoreRequest", "NSObject"],
    "NSDictionary": ["NSDictionary", "NSObject"],
    "NSMutableDictionary": ["NSMutableDictionary", "NSDictionary", "NSObject"],
    "NSArray": ["NSArray", "NSObject"],
    "NSMutableArray": ["NSMutableArray", "NSArray", "NSObject"],
    "NSSet": ["NSSet", "NSObject"],
    "NSDate": ["NSDate", "NSObject"],
    "NSDecimalNumberPlaceholder": ["NSDecimalNumberPlaceholder", "NSDecimalNumber", "NSNumber", "NSValue", "NSObject"],
    "NSComparisonPredicate": ["NSComparisonPredicate", "NSPredicate", "NSObject"],
    "NSComparisonPredicateOperator": ["NSComparisonPredicateOperator", "NSPredicateOperator", "NSObject"],
    "NSMatchingPredicateOperator": ["NSMatchingPredicateOperator", "NSStringPredicateOperator", "NSPredicateOperator",
                                    "NSObject"],
    "NSSelfExpression": ["NSSelfExpression", "NSExpression", "NSObject"],
    "NSConstantValueExpression": ["NSConstantValueExpression", "NSExpression", "NSObject"],
    "NSKeyPathExpression": ["NSKeyPathExpression", "NSFunctionExpression", "NSExpression", "NSObject"],
    "NSKeyPathSpecifierExpression": ["NSKeyPathSpecifierExpression", "NSExpression", "NSObject"],
    "NSEqualityPredicateOperator": ["NSEqualityPredicateOperator", "NSPredicateOperator", "NSObject"],
    "NSSubstringPredicateOperator": ["NSSubstringPredicateOperator", "NSStringPredicateOperator", "NSPredicateOperator",
                                     "NSObject"],
    "NSInPredicateOperator": ["NSInPredicateOperator", "NSPredicateOperator", "NSObject"],
    "NSLikePredicateOperator": ["NSLikePredicateOperator", "NSMatchingPredicateOperator", "NSStringPredicateOperator",
                                "NSPredicateOperator", "NSObject"],
    "NSBetweenPredicateOperator": ["NSBetweenPredicateOperator", "NSPredicateOperator", "NSObject"],
    "NSCompoundPredicate": ["NSCompoundPredicate", "NSPredicate", "NSObject"],
    "NSTruePredicate": ["NSTruePredicate", "NSPredicate", "NSObject"],
    "NSFalsePredicate": ["NSFalsePredicate", "NSPredicate", "NSObject"],
    "NSVariableExpression": ["NSVariableExpression", "NSExpression", "NSObject"],
    "NSAggregateExpression": ["NSAggregateExpression", "NSExpression", "NSObject"],
    "NSFunctionExpression": ["NSFunctionExpression", "NSExpression", "NSObject"],
    "NSDerivedAttributeDescription": ["NSDerivedAttributeDescription", "NSAttributeDescription", "NSPropertyDescription",
                                      "NSObject"],
    "NSFetchedPropertyDescription": ["NSFetchedPropertyDescription", "NSPropertyDescription", "NSObject"],
}


class Archiver:
    """NSKeyedArchiver: objects numbered as they're first encoded, an
    object encoded conditionally numbered then and written when it's
    encoded outright, its class described after its fields."""

    def __init__(self):
        self.objects = [Str("$null")]
        self.numbers = {}
        self.pending = {}
        self.classes = {}

    def encode(self, obj, conditional=False):
        if obj is None:
            return UID(0)
        n = self.numbers.get(obj.ident)
        if n is not None:
            if not conditional and n in self.pending:
                del self.pending[n]
                self._write(n, obj)
            return UID(n)
        n = len(self.objects)
        self.objects.append(None)
        self.numbers[obj.ident] = n
        if conditional:
            self.pending[n] = obj
        else:
            self._write(n, obj)
        return UID(n)

    def _write(self, n, obj):
        if isinstance(obj, (Str, Num, Bool, Data)):
            self.objects[n] = obj
            return
        fields = {}
        obj.encode_with(self, fields)
        fields["$class"] = self._class(obj.class_name)
        order = [k for k in KEY_ORDER[obj.class_name] if k in fields]
        assert len(order) == len(fields), (obj.class_name, sorted(fields))
        self.objects[n] = Dict([(Key(k), fields[k]) for k in cf_dictionary_order(order)])

    def _class(self, name):
        if name not in self.classes:
            n = len(self.objects)
            self.objects.append(None)
            self.classes[name] = n

            self.objects[n] = Dict([(Key("$classname"), class_name_string(name)),
                                    (Key("$classes"), Array([class_name_string(c) for c in CLASS_CHAINS[name]]))])
        return UID(self.classes[name])

    def archive(self, root):
        self.encode(root)
        if self.pending:
            unestablished("an object only referred to conditionally")
        return Dict([(Key("$version"), Num("i32", 100000)), (Key("$archiver"), Str("NSKeyedArchiver")),
                     (Key("$top"), Dict([(Key("root"), UID(1))])), (Key("$objects"), Array(self.objects))])


class Archived:
    """An archived object, the same object wherever it's referred to."""

    def __init__(self):
        self.ident = fresh()


class NSDictionary(Archived):
    """A dictionary of string keys, enumerated in CoreFoundation's order for
    the order `pairs` were inserted in."""

    def __init__(self, pairs, class_name="NSDictionary"):
        super().__init__()
        values = {k.value: (k, v) for k, v in pairs}
        self.pairs = [values[k] for k in cf_dictionary_order([k.value for k, _ in pairs])]
        self.class_name = class_name

    def encode_with(self, a, f):
        f["NS.keys"] = [a.encode(k) for k, _ in self.pairs]
        f["NS.objects"] = [a.encode(v) for _, v in self.pairs]


class NSArray(Archived):
    def __init__(self, items, class_name="NSArray"):
        super().__init__()
        self.items = items
        self.class_name = class_name

    def encode_with(self, a, f):
        f["NS.objects"] = [a.encode(x) for x in self.items]


class NSSet(Archived):
    """A set created from all its members at once, enumerated in
    CoreFoundation's order for their hashes (`names`, each member's)."""

    def __init__(self, members, names):
        super().__init__()
        by_name = dict(zip(names, members))
        self.items = [by_name[n] for n in cf_created_order(names)]
        self.class_name = "NSSet"

    def encode_with(self, a, f):
        f["NS.objects"] = [a.encode(x) for x in self.items]


class NSDate(Archived):
    """A date, one object for each time."""
    class_name = "NSDate"

    def __init__(self, time):
        super().__init__()
        self.time = time
        self.ident = ("date", time)

    def encode_with(self, a, f):
        f["NS.time"] = Num("f64", self.time)


class NSDecimalNumber(Archived):
    class_name = "NSDecimalNumberPlaceholder"

    def __init__(self, text):
        super().__init__()
        m = re.fullmatch(r"(-?)(\d+)(?:\.(\d+))?", text)
        if not m:
            unestablished("the decimal default value %r" % text)
        digits = (m.group(2) + (m.group(3) or "")).lstrip("0") or "0"
        exponent = -len(m.group(3) or "")
        mantissa = int(digits)
        # Compact: no trailing zeros in the mantissa.
        while mantissa and mantissa % 10 == 0:
            mantissa //= 10
            exponent += 1
        if mantissa >= 1 << 64 or not -128 <= exponent <= 127:
            unestablished("the decimal default value %r" % text)
        self.negative = bool(m.group(1)) and mantissa != 0
        self.mantissa = mantissa
        self.exponent = exponent if mantissa else 0

    def encode_with(self, a, f):
        words = []
        m = self.mantissa
        while m:
            words.append(m & 0xFFFF)
            m >>= 16
        f["NS.mantissa"] = Data(struct.pack("<8H", *(words + [0] * (8 - len(words)))))
        f["NS.exponent"] = Num("i32", self.exponent)
        f["NS.length"] = Num("i32", len(words))
        f["NS.compact"] = Bool(self.mantissa != 0)
        f["NS.negative"] = Bool(self.negative)
        f["NS.mantissa.bo"] = Num("i32", 1)


EMPTY_USER_INFO = NSDictionary([])


def whole_minutes(text):
    """A date in a model (seconds since 2001) as Core Data holds it: whole
    minutes, earlier ones for a time between."""
    try:
        seconds = float(text)
    except ValueError:
        unestablished("the date %r" % text)
    if not math.isfinite(seconds):
        unestablished("the date %r" % text)
    return float(math.floor(seconds / 60) * 60)


def floating(value, kind="f64"):
    """A floating-point NSNumber: a whole number of magnitude below 2^55 is
    one object for its value (and type); any other is an object of its own."""
    shared = math.isfinite(value) and value == int(value) and abs(value) < 1 << 55
    return Num(kind, value, None if shared else fresh())


class SelfExpression(Archived):
    """SELF: one object in a model."""
    class_name = "NSSelfExpression"

    def __init__(self):
        self.ident = ("NSSelfExpression",)

    def encode_with(self, a, f):
        f["NSExpressionType"] = Num("i64", 1)


SELF = SelfExpression()


class ConstantValueExpression(Archived):
    class_name = "NSConstantValueExpression"

    def __init__(self, value):
        super().__init__()
        self.value = value

    def encode_with(self, a, f):
        if self.value is None:
            f["NSConstantValueClassName"] = UID(0)
        else:
            f["NSConstantValue"] = a.encode(self.value)
        f["NSExpressionType"] = Num("i64", 0)


class KeyPathSpecifierExpression(Archived):
    class_name = "NSKeyPathSpecifierExpression"

    def __init__(self, key):
        super().__init__()
        self.key = key

    def encode_with(self, a, f):
        f["NSExpressionType"] = Num("i64", 10)
        f["NSKeyPath"] = a.encode(self.key)


class KeyPathExpression(Archived):
    """A key path of SELF (`SELF.key`, type 3) or of a variable
    (`$VAR.key`, type 4): one key by `valueForKey:`, several by
    `valueForKeyPath:`."""
    class_name = "NSKeyPathExpression"

    def __init__(self, key, operand=None):
        super().__init__()
        self.key = key
        self.operand = operand

    def encode_with(self, a, f):
        selector = "valueForKeyPath:" if "." in self.key.value or self.operand else "valueForKey:"
        f["NSExpressionType"] = Num("i64", 4 if self.operand else 3)
        f["NSSelectorName"] = a.encode(Str(selector, ("selector", selector)))
        f["NSOperand"] = a.encode(self.operand or SELF)
        f["NSArguments"] = a.encode(NSArray([KeyPathSpecifierExpression(self.key)], "NSMutableArray"))


class VariableExpression(Archived):
    class_name = "NSVariableExpression"

    def __init__(self, name):
        super().__init__()
        self.name = name

    def encode_with(self, a, f):
        f["NSExpressionType"] = Num("i64", 2)
        f["NSVariable"] = a.encode(self.name)


class AggregateExpression(Archived):
    class_name = "NSAggregateExpression"

    def __init__(self, items):
        super().__init__()
        self.items = items

    def encode_with(self, a, f):
        f["NSCollection"] = a.encode(NSArray(self.items, "NSMutableArray"))
        f["NSExpressionType"] = Num("i64", 14)


class Operator(Archived):
    """A predicate's operator: its class, its type, ANY (2) or ALL (1), and
    its options ([c] 1, [d] 2, [n] 4)."""

    def __init__(self, class_name, kind, modifier=0, options=0):
        super().__init__()
        self.class_name, self.kind, self.modifier, self.options = class_name, kind, modifier, options

    def encode_with(self, a, f):
        f["NSOperatorType"] = Num("i64", self.kind)
        f["NSModifier"] = Num("i64", self.modifier)
        if self.class_name == "NSComparisonPredicateOperator":
            f["NSOptions"] = Num("i64", self.options)
            f["NSVariant"] = Num("i64", self.kind)
        elif self.class_name == "NSEqualityPredicateOperator":
            f["NSOptions"] = Num("i64", self.options)
            f["NSNegate"] = Bool(self.kind == NOT_EQUAL)
        elif self.class_name == "NSSubstringPredicateOperator":
            f["NSFlags"] = Num("i64", self.options)
            f["NSPosition"] = Num("i64", 0 if self.kind == BEGINS_WITH else 1)
        elif self.class_name != "NSBetweenPredicateOperator":
            f["NSFlags"] = Num("i64", self.options)


def ComparisonOperator(kind):
    return Operator("NSComparisonPredicateOperator", kind)


def MatchingOperator():
    return Operator("NSMatchingPredicateOperator", MATCHES)


LESS, LESS_OR_EQUAL, GREATER, GREATER_OR_EQUAL, EQUAL, NOT_EQUAL = 0, 1, 2, 3, 4, 5
MATCHES, LIKE, BEGINS_WITH, ENDS_WITH, IN, CONTAINS, BETWEEN = 6, 7, 8, 9, 10, 99, 100
# Each operator's class.
OPERATOR_CLASSES = {
    LESS: "NSComparisonPredicateOperator", LESS_OR_EQUAL: "NSComparisonPredicateOperator",
    GREATER: "NSComparisonPredicateOperator", GREATER_OR_EQUAL: "NSComparisonPredicateOperator",
    EQUAL: "NSEqualityPredicateOperator", NOT_EQUAL: "NSEqualityPredicateOperator",
    MATCHES: "NSMatchingPredicateOperator", LIKE: "NSLikePredicateOperator",
    BEGINS_WITH: "NSSubstringPredicateOperator", ENDS_WITH: "NSSubstringPredicateOperator",
    IN: "NSInPredicateOperator", CONTAINS: "NSInPredicateOperator", BETWEEN: "NSBetweenPredicateOperator",
}


class ComparisonPredicate(Archived):
    class_name = "NSComparisonPredicate"

    def __init__(self, left, right, operator):
        super().__init__()
        self.left, self.right, self.operator = left, right, operator

    def encode_with(self, a, f):
        f["NSLeftExpression"] = a.encode(self.left)
        f["NSRightExpression"] = a.encode(self.right)
        f["NSPredicateOperator"] = a.encode(self.operator)


NOT, AND, OR = 0, 1, 2


class CompoundPredicate(Archived):
    class_name = "NSCompoundPredicate"

    def __init__(self, kind, subpredicates):
        super().__init__()
        self.kind, self.subpredicates = kind, subpredicates

    def encode_with(self, a, f):
        f["NSCompoundPredicateType"] = Num("i64", self.kind)
        f["NSSubpredicates"] = a.encode(NSArray(self.subpredicates))


class ConstantPredicate(Archived):
    """TRUEPREDICATE or FALSEPREDICATE: one object each in a model."""

    def __init__(self, value):
        self.class_name = "NSTruePredicate" if value else "NSFalsePredicate"
        self.ident = (self.class_name,)

    def encode_with(self, a, f):
        pass


class PredicateParser:
    """NSPredicate's format, as far as it's established against Xcode's
    momc (docs/clean-room/momc.md#predicates): comparisons of key paths,
    SELF, variables, constants and {aggregates}, with ANY, ALL, SOME and
    NONE and [cdn] options; NOT, AND and OR; TRUEPREDICATE and
    FALSEPREDICATE. Anything else is refused."""

    TOKEN = re.compile(r"""\s*(?:
        (?P<string>"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*')
      | (?P<number>-?(?:\d+\.\d*|\.\d+|\d+)(?:[eE][+-]?\d+)?)
      | (?P<variable>\$[A-Za-z_][A-Za-z0-9_]*)
      | (?P<word>[A-Za-z_@][A-Za-z0-9_@]*)
      | (?P<punct>==|=<|<=|=>|>=|!=|<>|&&|\|\||[()<>=!{},.\[\]])
    )""", re.X)
    OPERATORS = {"==": EQUAL, "=": EQUAL, "!=": NOT_EQUAL, "<>": NOT_EQUAL, "<": LESS, "<=": LESS_OR_EQUAL,
                 "=<": LESS_OR_EQUAL, ">": GREATER, ">=": GREATER_OR_EQUAL, "=>": GREATER_OR_EQUAL,
                 "BEGINSWITH": BEGINS_WITH, "ENDSWITH": ENDS_WITH, "CONTAINS": CONTAINS, "LIKE": LIKE,
                 "MATCHES": MATCHES, "IN": IN, "BETWEEN": BETWEEN}
    # Options apply to these operators (an equality's as NSOptions).
    OPTIONED = {EQUAL, NOT_EQUAL, BEGINS_WITH, ENDS_WITH, CONTAINS, LIKE, MATCHES, IN}
    RESERVED = {"AND", "OR", "NOT", "ANY", "ALL", "SOME", "NONE", "IN", "BETWEEN", "BEGINSWITH", "ENDSWITH",
                "CONTAINS", "LIKE", "MATCHES", "TRUEPREDICATE", "FALSEPREDICATE", "YES", "NO", "TRUE", "FALSE",
                "NIL", "NULL", "SELF", "SUBQUERY", "FUNCTION", "CAST", "FIRST", "LAST", "SIZE", "ANYKEY", "UTI-CONFORMS-TO",
                "UTI-EQUALS"}

    def __init__(self, text):
        self.text = text
        self.tokens = []
        pos = 0
        while pos < len(text):
            if text[pos:].strip() == "":
                break
            m = self.TOKEN.match(text, pos)
            if not m or m.end() == pos:
                self.refuse("its text at %r" % text[pos:pos + 10])
            kind = m.lastgroup
            self.tokens.append((kind, m.group(kind)))
            pos = m.end()
        self.at = 0

    def refuse(self, what):
        unestablished("the predicate %r (%s)" % (self.text, what))

    def peek(self, offset=0):
        i = self.at + offset
        return self.tokens[i] if i < len(self.tokens) else (None, None)

    def word(self, offset=0):
        kind, value = self.peek(offset)
        return value.upper() if kind == "word" else value if kind == "punct" else None

    def take(self):
        token = self.peek()
        self.at += 1
        return token

    def parse(self):
        p = self.disjunction()
        if self.at != len(self.tokens):
            self.refuse("its text from %r" % (self.peek()[1],))
        return p

    @staticmethod
    def join(kind, left, right):
        # A run of ANDs, or of ORs, is one compound, however parenthesized.
        parts = []
        for p in (left, right):
            parts += p.subpredicates if isinstance(p, CompoundPredicate) and p.kind == kind else [p]
        return CompoundPredicate(kind, parts)

    def disjunction(self):
        p = self.conjunction()
        while self.word() in ("OR", "||"):
            self.take()
            p = self.join(OR, p, self.conjunction())
        return p

    def conjunction(self):
        p = self.negation()
        while self.word() in ("AND", "&&"):
            self.take()
            p = self.join(AND, p, self.negation())
        return p

    def negation(self):
        if self.word() in ("NOT", "!"):
            self.take()
            p = self.negation()
            # NOT NOT p is p.
            if isinstance(p, CompoundPredicate) and p.kind == NOT:
                return p.subpredicates[0]
            return CompoundPredicate(NOT, [p])
        return self.primary()

    def primary(self):
        w = self.word()
        if w == "(":
            self.take()
            p = self.disjunction()
            if self.take()[1] != ")":
                self.refuse("an unclosed parenthesis")
            return p
        if w in ("TRUEPREDICATE", "FALSEPREDICATE"):
            self.take()
            return ConstantPredicate(w == "TRUEPREDICATE")
        modifier, negate = 0, False
        if w in ("ANY", "SOME", "ALL", "NONE"):
            self.take()
            modifier = 1 if w == "ALL" else 2
            negate = w == "NONE"
        left = self.expression()
        op = self.word()
        if op not in self.OPERATORS:
            self.refuse("the operator %r" % (self.peek()[1],))
        self.take()
        kind = self.OPERATORS[op]
        options = 0
        if self.word() == "[":
            self.take()
            kind_, flags = self.take()
            if kind_ != "word" or self.take()[1] != "]" or not set(flags.lower()) <= set("cdn") or kind not in self.OPTIONED:
                self.refuse("its options")
            options = sum({"c": 1, "d": 2, "n": 4}[c] for c in set(flags.lower()))
        right = self.expression()
        p = ComparisonPredicate(left, right, Operator(OPERATOR_CLASSES[kind], kind, modifier, options))
        return CompoundPredicate(NOT, [p]) if negate else p

    def expression(self):
        kind, value = self.take()
        if kind == "string":
            body = value[1:-1]
            if "\\" in body.replace('\\"', "").replace("\\'", ""):
                self.refuse("an escape other than a quote's")
            return ConstantValueExpression(parsed_str(body.replace('\\"', '"').replace("\\'", "'")))
        if kind == "number":
            if re.fullmatch(r"-?\d+", value):
                if not -1 << 63 <= int(value) < 1 << 63:
                    self.refuse("an integer beyond 64 bits")
                # 32-bit where it fits.
                return ConstantValueExpression(Num("i32" if -1 << 31 <= int(value) < 1 << 31 else "i64", int(value)))
            return ConstantValueExpression(floating(float(value)))
        if kind == "variable":
            variable = VariableExpression(parsed_str(value[1:]))
            if self.word() == ".":
                self.take()
                return KeyPathExpression(self.key_path(), variable)
            return variable
        if kind == "punct" and value == "{":
            items = []
            while self.word() != "}":
                items.append(self.expression())
                if self.word() == ",":
                    self.take()
                elif self.word() != "}":
                    self.refuse("an aggregate")
            self.take()
            return AggregateExpression(items)
        if kind == "word":
            upper = value.upper()
            if upper in ("YES", "TRUE", "NO", "FALSE"):
                return ConstantValueExpression(Bool(upper in ("YES", "TRUE")))
            if upper in ("NIL", "NULL"):
                return ConstantValueExpression(None)
            if upper == "SELF":
                if self.word() == ".":
                    self.take()
                    return KeyPathExpression(self.key_path())
                return SELF
            if upper in self.RESERVED:
                self.refuse("the word %s" % value)
            self.at -= 1
            return KeyPathExpression(self.key_path())
        self.refuse("the expression %r" % (value,))

    def key_path(self):
        keys = []
        while True:
            kind, value = self.take()
            if kind != "word" or value.upper() in self.RESERVED:
                self.refuse("the key path at %r" % (value,))
            keys.append(value)
            if self.word() != ".":
                return parsed_str(".".join(keys))
            self.take()


# The characters short strings of 8 to 11 characters can be held in a
# pointer with (Foundation's tagged pointer strings, as published): 6 bits
# for up to 9, the first 32 at 5 bits for up to 11.
TAGGED_6 = "eilotrm.apdnsIc ufkMShjTRxgC4013bDNvwyUL2O856P-B79AFKEWV_zGJ/HYX"
TAGGED_5 = TAGGED_6[:32]


def parsed_str(value):
    """A string a predicate's parser makes: one object for its value where
    it fits in a pointer (as any equal string), else an object of its own
    (`parent.parent.n`, `kids.@count`, as a date rule's key). The empty
    string is one object of the parser's own."""
    if value == "":
        return Str("", ("parsed", ""))
    tagged = value.isascii() and (len(value) <= 7 or (len(value) <= 9 and all(c in TAGGED_6 for c in value))
                                  or (len(value) <= 11 and all(c in TAGGED_5 for c in value)))
    return Str(value) if tagged else Str(value, fresh())


def predicate(text):
    return PredicateParser(text).parse()


class FunctionExpression(Archived):
    """A function of NSExpression's (`now()`, `uppercase:(name)`): its
    selector sent to the class that implements them."""
    class_name = "NSFunctionExpression"

    def __init__(self, selector, arguments, arguments_class="NSMutableArray"):
        super().__init__()
        self.selector, self.arguments, self.arguments_class = selector, arguments, arguments_class

    def encode_with(self, a, f):
        f["NSExpressionType"] = Num("i64", 4)
        # A selector's name, one object for its value.
        f["NSSelectorName"] = a.encode(Str(self.selector, ("selector", self.selector)))
        f["NSOperand"] = a.encode(ClassExpression("_NSPredicateUtilities"))
        f["NSArguments"] = a.encode(NSArray(self.arguments, self.arguments_class))


class ClassExpression(Archived):
    """A constant that is a class: its name, a new string each time."""
    class_name = "NSConstantValueExpression"

    def __init__(self, name):
        super().__init__()
        self.name = name

    def encode_with(self, a, f):
        f["NSExpressionType"] = Num("i64", 0)
        f["NSConstantValueClassName"] = a.encode(Str(self.name, fresh()))


KEY_PATH = r"[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*"


def derivation(text):
    """A derived attribute's expression: a key path, `now()`, a string
    function of a key path, or a to-many key path's @count or @sum (as
    count: and sum: of the key path before it)."""
    text = text.strip()
    if text == "now()":
        return FunctionExpression("now", [])
    m = re.fullmatch(r"(canonical|uppercase|lowercase):\((%s)\)" % KEY_PATH, text)
    if m:
        return FunctionExpression(m.group(1) + ":", [KeyPathExpression(parsed_str(m.group(2)))])
    m = re.fullmatch(r"(%s)\.@(count|sum)" % KEY_PATH, text)
    if m:
        return FunctionExpression(m.group(2) + ":", [KeyPathExpression(parsed_str(m.group(1)))], "NSArray")
    if re.fullmatch(KEY_PATH, text) and text.upper() not in PredicateParser.RESERVED:
        return KeyPathExpression(parsed_str(text))
    unestablished("the derivation expression %r" % text)


def user_info(element):
    """An element's userInfo entries, as Core Data holds them."""
    entries = [(xml_str(e.get("key")), xml_str(e.get("value"))) for u in element.findall("userInfo") for e in u.findall("entry")]
    if len({k.value for k, _ in entries}) != len(entries):
        unestablished("a userInfo key given twice")
    # Filled in name order (case-insensitive and numeric, or bytewise: the
    # models examined don't tell them apart).
    return NSDictionary(sorted(entries, key=lambda e: natural_key(e[0].value))) if entries else None


# --- The model -----------------------------------------------------------------

ATTRIBUTE_TYPES = {
    "Integer 16": (100, "NSNumber"), "Integer 32": (200, "NSNumber"), "Integer 64": (300, "NSNumber"),
    "Decimal": (400, "NSDecimalNumber"), "Double": (500, "NSNumber"), "Float": (600, "NSNumber"),
    "String": (700, "NSString"), "Boolean": (800, "NSNumber"), "Date": (900, "NSDate"), "Binary": (1000, "NSData"),
    "UUID": (1100, "NSUUID"), "URI": (1200, "NSURL"), "Transformable": (1800, None),
}

DELETE_RULES = {"No Action": 0, "Nullify": 1, "Cascade": 2, "Deny": 3}


def yes(element, name):
    value = element.get(name)
    if value not in (None, "YES", "NO"):
        unestablished("%s=%r" % (name, value))
    return value == "YES"


def known(element, attributes, children=()):
    for name in element.attrib:
        if name not in attributes:
            unestablished("the %s attribute %s" % (element.tag, name))
    for child in element:
        if child.tag not in children:
            unestablished("a model's <%s> in <%s>" % (child.tag, element.tag))


def natural_key(name):
    """Xcode's order for names: case-insensitive, with runs of digits as
    numbers (equal names fall back to their bytes, which isn't
    established). Its order for other than ASCII names is a localized
    collation this doesn't have."""
    if not name.isascii():
        unestablished("the name %r (Xcode orders names by a collation of its own)" % name)
    parts = re.split(r"(\d+)", name.lower())
    return ([int(p) if p.isdigit() else p for p in parts], name)


def flags(**bits):
    return sum(bit for bit, on in bits.values() if on)


def copied_dictionary(pairs):
    """A mutable dictionary filled from another dictionary, itself filled
    in name order: the entities, subentities and fetch requests by name.
    (An entity's subentities whose names collide in its table are in
    Xcode's order for 71 of the 74 examined; the rest are swapped.)"""
    pairs = sorted(pairs, key=lambda p: natural_key(p[0].value))
    by_name = {k.value: (k, v) for k, v in pairs}
    return NSDictionary([by_name[n] for n in cf_dictionary_order([k.value for k, _ in pairs])], "NSMutableDictionary")


class Property(Archived):
    def __init__(self, entity, element):
        super().__init__()
        self.entity = entity
        self.name = xml_str(element.get("name"))
        renaming = element.get("elementID")
        self.renaming = xml_str(renaming) if renaming else self.name
        self.user_info = user_info(element)
        modifier = element.get("versionHashModifier")
        self.modifier = xml_str(modifier) if modifier else None
        self.optional = yes(element, "optional")
        self.transient = yes(element, "transient")
        self.read_only = False
        self.validation = []

    def encode_property(self, a, f, extra):
        f["NSEntity"] = a.encode(self.entity, conditional=True)
        f["NSPropertyName"] = a.encode(self.name)
        f["NSRenamingIdentifier"] = a.encode(self.renaming)
        if self.validation:
            f["NSValidationPredicates"] = a.encode(NSArray([p for p, _ in self.validation]))
            f["NSValidationWarnings"] = a.encode(NSArray([Num("i64", w) for _, w in self.validation]))
        if self.user_info:
            f["NSUserInfo"] = a.encode(self.user_info)
        if self.modifier:
            f["NSVersionHashModifier"] = a.encode(self.modifier)
        f["_P"] = Num("i32", 1 | (0 if self.validation else 16 | 32) | (2 if self.optional else 0) | (8 if self.transient else 0)
                      | (4 if self.read_only else 0)
                      | (0 if self.user_info else 64) | (0 if self.modifier else 128) | extra)
        if self.optional:
            f["NSIsOptional"] = Bool(True)
        if self.transient:
            f["NSIsTransient"] = Bool(True)
        if self.read_only:
            f["NSIsReadOnly"] = Bool(True)


class Attribute(Property):
    class_name = "NSAttributeDescription"
    ATTRIBUTES = {"name", "attributeType", "optional", "transient", "defaultValueString", "defaultDateTimeInterval",
                  "usesScalarValueType", "spotlightIndexingEnabled", "allowsExternalBinaryDataStorage",
                  "preserveAfterDeletion", "allowsCloudEncryption", "valueTransformerName", "customClassName",
                  "versionHashModifier", "elementID", "syncable", "minValueString", "maxValueString",
                  "regularExpressionString", "minDateTimeInterval", "maxDateTimeInterval"}

    def __init__(self, entity, element):
        known(element, self.ATTRIBUTES, ("userInfo",))
        super().__init__(entity, element)
        kind = element.get("attributeType")
        if kind not in ATTRIBUTE_TYPES:
            unestablished("the attribute type %r" % kind)
        self.type, value_class = ATTRIBUTE_TYPES[kind]
        custom = element.get("customClassName")
        if custom is not None and kind != "Transformable":
            unestablished("a custom class for a %s attribute" % kind)
        self.value_class = xml_str(custom) if custom is not None else class_name_string(value_class) if value_class else None
        transformer = element.get("valueTransformerName")
        if transformer is not None and kind != "Transformable":
            unestablished("a value transformer for a %s attribute" % kind)
        self.transformer = xml_str(transformer)
        self.spotlight = yes(element, "spotlightIndexingEnabled") and not self.transient
        self.cloud_encryption = yes(element, "allowsCloudEncryption")
        self.flags = (4 if yes(element, "allowsExternalBinaryDataStorage") else 0) | \
            (16 if yes(element, "preserveAfterDeletion") else 0)
        self.default = self._default(kind, element)
        self.validation = self._validation(kind, element)

    # Each rule's warning code: a number's, a date's and a string's minimum
    # and maximum, and a string's pattern.
    WARNINGS = {"number": (1620, 1610), "date": (1640, 1630), "string": (1670, 1660)}
    PATTERN_WARNING = 1680

    @staticmethod
    def _validation(kind, element):
        """The validation rules: predicates on the value, each with its
        warning code, the minimum's before the maximum's before a pattern's."""
        if kind == "Date":
            bounds = [element.get("minDateTimeInterval"), element.get("maxDateTimeInterval")]
            given = [element.get("minValueString"), element.get("maxValueString")]
        else:
            bounds = [element.get("minValueString"), element.get("maxValueString")]
            given = [element.get("minDateTimeInterval"), element.get("maxDateTimeInterval")]
        pattern = element.get("regularExpressionString")
        if any(x is not None for x in given) or (pattern is not None and kind != "String"):
            unestablished("a validation rule of another type's for a %s attribute" % kind)
        if all(x is None for x in bounds + [pattern]) or kind in ("Boolean", "Binary", "UUID", "URI", "Transformable"):
            # These types' are left out.
            return []
        if kind in ("Integer 16", "Integer 32", "Integer 64", "Decimal", "Double", "Float"):
            group = "number"
            value = Attribute._double_value if kind in ("Double", "Float") else Attribute._long_long
        elif kind == "Date":
            group, value = "date", lambda text: floating(whole_minutes(text))
        else:
            group, value = "string", Attribute._long_long
        rules = []
        for text, operator, warning in zip(bounds, (GREATER_OR_EQUAL, LESS_OR_EQUAL), Attribute.WARNINGS[group]):
            if text is None:
                continue
            if group == "date":
                # A new key string each time.
                subject = KeyPathExpression(Str("timeIntervalSinceReferenceDate", fresh()))
            elif group == "string":
                subject = KeyPathExpression(Str("length"))
            else:
                subject = SELF
            rules.append((ComparisonPredicate(subject, ConstantValueExpression(value(text)),
                                              ComparisonOperator(operator)), warning))
        if pattern is not None:
            rules.append((ComparisonPredicate(SELF, ConstantValueExpression(xml_str(pattern)), MatchingOperator()),
                          Attribute.PATTERN_WARNING))
        return rules

    @staticmethod
    def _long_long(text):
        """The string's longLongValue: the integer after any white space,
        anything after it ignored, 0 for none, and the nearest 64-bit integer
        for one outside them."""
        m = re.match(r"\s*([+-]?\d+)", text)
        value = int(m.group(1)) if m else 0
        return Num("i64", max(-1 << 63, min(value, (1 << 63) - 1)))

    @staticmethod
    def _double_value(text):
        """The string's doubleValue: the decimal number after any white
        space, anything after it ignored, and 0 for none ("inf" too)."""
        if re.match(r"\s*[+-]?0[xX]", text):
            unestablished("the validation bound %r" % text)
        m = re.match(r"\s*([+-]?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?)", text)
        value = float(m.group(1)) if m else 0.0
        if not math.isfinite(value):
            unestablished("the validation bound %r" % text)
        return floating(value)

    @staticmethod
    def _default(kind, element):
        text = element.get("defaultValueString")
        interval = element.get("defaultDateTimeInterval")
        if interval is not None:
            if kind != "Date" or text is not None:
                unestablished("a defaultDateTimeInterval for a %s attribute" % kind)
            return NSDate(whole_minutes(interval))
        if text is None:
            return None
        if kind in ("Integer 16", "Integer 32", "Integer 64"):
            if not re.fullmatch(r"-?\d+", text):
                unestablished("the integer default value %r" % text)
            return Num("i64", int(text))
        if kind in ("Double", "Float"):
            try:
                value = float(text)
            except ValueError:
                unestablished("the default value %r" % text)
            if kind == "Float":
                return floating(struct.unpack(">f", struct.pack(">f", value))[0], "f32")
            return floating(value)
        if kind == "Boolean":
            if text not in ("YES", "NO"):
                unestablished("the Boolean default value %r" % text)
            return Bool(text == "YES")
        if kind == "String":
            return xml_str(text)
        if kind == "Decimal":
            return NSDecimalNumber(text)
        unestablished("a default value for a %s attribute" % kind)

    def encode_with(self, a, f):
        self.encode_property(a, f, (256 if self.spotlight else 0) | (2048 if self.cloud_encryption else 0))
        if self.spotlight:
            f["NSIndexedBySpotlight"] = Bool(True)
        if self.cloud_encryption:
            f["NSAllowsCloudEncryption"] = Bool(True)
        f["NSAttributeType"] = Num("i32", self.type)
        f["NSAttributeValueClassName"] = a.encode(self.value_class)
        f["NSDefaultValue"] = a.encode(self.default)
        f["NSValueTransformerName"] = a.encode(self.transformer)
        f["NSFlagsKey"] = Num("i32", self.flags)


class DerivedAttribute(Attribute):
    """An attribute Core Data derives (`derived="YES"`), read-only. Others'
    settings beside it (user info, a modifier, flags) aren't established."""
    class_name = "NSDerivedAttributeDescription"
    ATTRIBUTES = {"name", "attributeType", "optional", "derived", "derivationExpression", "usesScalarValueType", "syncable"}

    def __init__(self, entity, element):
        known(element, self.ATTRIBUTES)
        attrs = {k: v for k, v in element.attrib.items() if k not in ("derived", "derivationExpression")}
        plain = ET.Element(element.tag, attrs)
        super().__init__(entity, plain)
        if element.get("derived") != "YES":
            unestablished("derived=%r" % element.get("derived"))
        self.read_only = True
        self.expression = derivation(element.get("derivationExpression") or "")

    def encode_with(self, a, f):
        super().encode_with(a, f)
        f["NSDerivationExpression"] = a.encode(self.expression)
        f["NSFilteringPredicate"] = UID(0)


class FetchedProperty(Property):
    """A fetched property: transient and read-only, its fetch request's
    entity named beside the request."""
    class_name = "NSFetchedPropertyDescription"
    ATTRIBUTES = {"name", "optional", "versionHashModifier", "elementID", "syncable"}

    def __init__(self, entity, element):
        known(element, self.ATTRIBUTES, ("userInfo", "fetchRequest"))
        super().__init__(entity, element)
        self.transient = True
        self.read_only = True
        requests = element.findall("fetchRequest")
        if len(requests) != 1:
            unestablished("a fetched property without one fetch request")
        self.request_element = requests[0]
        self.request = None

    def resolve(self, by_name):
        name = self.request_element.get("entity")
        if name not in by_name:
            unestablished("the fetched property's entity %s" % name)
        self.request = FetchRequest(self.request_element, by_name[name])

    def encode_with(self, a, f):
        self.encode_property(a, f, 0)
        f["_NSFetchRequestEntityName"] = a.encode(self.request.entity.name)
        f["NSFetchRequest"] = a.encode(self.request)


class Relationship(Property):
    class_name = "NSRelationshipDescription"
    ATTRIBUTES = {"name", "optional", "toMany", "ordered", "minCount", "maxCount", "deletionRule", "destinationEntity",
                  "inverseName", "inverseEntity", "versionHashModifier", "elementID", "syncable"}

    def __init__(self, entity, element):
        known(element, self.ATTRIBUTES, ("userInfo",))
        super().__init__(entity, element)
        self.to_many = yes(element, "toMany")
        self.ordered = yes(element, "ordered")
        if self.ordered and not self.to_many:
            unestablished("an ordered to-one relationship")
        try:
            self.min_count = int(element.get("minCount", "0"))
            self.max_count = int(element.get("maxCount", "0" if self.to_many else "1"))
        except ValueError:
            unestablished("the relationship counts of %s" % self.name.value)
        if not self.to_many and self.max_count != 1:
            unestablished("a to-one relationship with maxCount=%d" % self.max_count)
        rule = element.get("deletionRule", "Nullify")
        if rule not in DELETE_RULES:
            unestablished("the deletion rule %r" % rule)
        self.delete_rule = DELETE_RULES[rule]
        self.destination_name = element.get("destinationEntity")
        if self.destination_name is None:
            unestablished("a relationship without a destination")
        self.inverse_name = element.get("inverseName")
        self.inverse_entity = element.get("inverseEntity")
        if (self.inverse_name is None) != (self.inverse_entity is None) or \
                (self.inverse_entity is not None and self.inverse_entity != self.destination_name):
            unestablished("the inverse of %s" % self.name.value)
        self.destination = None
        self.inverse = None

    def encode_with(self, a, f):
        self.encode_property(a, f, 1024 if self.ordered else 0)
        if self.ordered:
            f["NSIsOrdered"] = Bool(True)
        f["NSDestinationEntity"] = a.encode(self.destination, conditional=True)
        f["NSInverseRelationship"] = a.encode(self.inverse, conditional=True)
        f["_NSInverseRelationshipName"] = a.encode(self.inverse.name if self.inverse else None)
        f["_NSDestinationEntityName"] = a.encode(self.destination.name)
        f["NSDeleteRule"] = Num("i32", self.delete_rule)
        f["NSMinCount"] = Num("i32", self.min_count)
        f["NSMaxCount"] = Num("i32", self.max_count)


class Entity(Archived):
    class_name = "NSEntityDescription"
    ATTRIBUTES = {"name", "representedClassName", "syncable", "codeGenerationType", "parentEntity", "isAbstract",
                  "versionHashModifier", "elementID"}

    def __init__(self, model, element):
        known(element, self.ATTRIBUTES, ("attribute", "relationship", "fetchedProperty", "userInfo", "uniquenessConstraints"))
        super().__init__()
        self.model = model
        self.name = xml_str(element.get("name"))
        if self.name is None:
            unestablished("an entity without a name")
        self.class_name_value = xml_str(element.get("representedClassName"))
        self.parent_name = element.get("parentEntity")
        self.abstract = yes(element, "isAbstract")
        modifier = element.get("versionHashModifier")
        self.modifier = xml_str(modifier) if modifier else None
        renaming = element.get("elementID")
        self.renaming = xml_str(renaming) if renaming else self.name
        self.user_info = user_info(element)
        attributes = [(DerivedAttribute if e.get("derived") else Attribute)(self, e) for e in element.findall("attribute")]
        relationships = [Relationship(self, e) for e in element.findall("relationship")]
        fetched = [FetchedProperty(self, e) for e in element.findall("fetchedProperty")]
        self.properties = sorted(attributes, key=lambda p: natural_key(p.name.value)) + \
            sorted(relationships, key=lambda p: natural_key(p.name.value)) + \
            sorted(fetched, key=lambda p: natural_key(p.name.value))
        if len({p.name.value for p in self.properties}) != len(self.properties):
            unestablished("two properties of %s named alike" % self.name.value)
        self.unique = []
        for u in element.findall("uniquenessConstraints"):
            known(u, (), ("uniquenessConstraint",))
            for c in u.findall("uniquenessConstraint"):
                known(c, (), ("constraint",))
                self.unique.append([xml_str(x.get("value")) for x in c.findall("constraint")])
        self.parent = None
        self.subentities = []

    def encode_with(self, a, f):
        f["NSEntityName"] = a.encode(self.name)
        f["NSClassNameForEntity"] = a.encode(self.class_name_value)
        f["NSSuperentity"] = a.encode(self.parent, conditional=True)
        f["NSSubentities"] = a.encode(copied_dictionary([(e.name, e) for e in self.subentities]))
        f["NSProperties"] = a.encode(NSDictionary([(p.name, p) for p in self.properties], "NSMutableDictionary"))
        f["NSUserInfo"] = a.encode(self.user_info or EMPTY_USER_INFO)
        f["NSVersionHashModifier"] = a.encode(self.modifier)
        f["NSRenamingIdentifier"] = a.encode(self.renaming)
        if self.unique:
            f["NSUniqueTuples"] = a.encode(NSArray([NSArray(t, "NSMutableArray") for t in self.unique], "NSMutableArray"))
        f["NSManagedObjectModel"] = a.encode(self.model, conditional=True)
        if self.abstract:
            f["NSIsAbstract"] = Bool(True)


class FetchRequest(Archived):
    class_name = "NSFetchRequest"

    def __init__(self, element, entity):
        known(element, {"name", "entity", "fetchLimit", "fetchBatchSize", "predicateString"})
        super().__init__()
        self.entity = entity
        text = element.get("predicateString")
        self.predicate = predicate(text) if text else None
        try:
            self.limit = int(element.get("fetchLimit", "0"))
            self.batch = int(element.get("fetchBatchSize", "0"))
        except ValueError:
            unestablished("the fetch request %s's limits" % element.get("name"))

    def encode_with(self, a, f):
        # Conditionally: a fetched property's entity is written where the
        # model's entities are (templates follow them all).
        f["NSEntity"] = a.encode(self.entity, conditional=True)
        f["NSPredicate"] = a.encode(self.predicate)
        for k in ("NSSortDescriptors", "NSValuesToFetch", "NSValuesToGroupBy", "NSHavingPredicate",
                  "NSRelationshipKeyPathsForPrefetching"):
            f[k] = UID(0)
        f["NSHasFetchRequestFlags"] = Bool(True)
        f["NSFetchRequestFlags"] = Num("i64", 30)
        f["NSResultType"] = Num("i64", 0)
        f["NSAllocationType"] = Num("i64", 0)
        f["NSFetchOffset"] = Num("i32", 0)
        f["NSFetchLimit"] = Num("i32", self.limit)
        f["NSBatchSize"] = Num("i32", self.batch)


class Model(Archived):
    class_name = "NSManagedObjectModel"
    ATTRIBUTES = {"type", "documentVersion", "lastSavedToolsVersion", "systemVersion", "minimumToolsVersion",
                  "sourceLanguage", "userDefinedModelVersionIdentifier", "iOSVersion", "macOSVersion"}

    def __init__(self, root, versions):
        super().__init__()
        if root.tag != "model":
            unestablished("a model whose root is <%s>" % root.tag)
        known(root, self.ATTRIBUTES, ("entity", "configuration", "fetchRequest", "elements"))
        self.versions = versions
        self.entities = [Entity(self, e) for e in root.findall("entity")]
        by_name = {e.name.value: e for e in self.entities}
        if len(by_name) != len(self.entities):
            unestablished("two entities named alike")
        for e in self.entities:
            if e.parent_name is not None:
                if e.parent_name not in by_name:
                    unestablished("the parent entity %s" % e.parent_name)
                e.parent = by_name[e.parent_name]
                e.parent.subentities.append(e)
            for r in e.properties:
                if isinstance(r, FetchedProperty):
                    r.resolve(by_name)
                if isinstance(r, Relationship):
                    if r.destination_name not in by_name:
                        unestablished("the destination entity %s" % r.destination_name)
                    r.destination = by_name[r.destination_name]
        for e in self.entities:
            for r in e.properties:
                if isinstance(r, Relationship) and r.inverse_name is not None:
                    inverse = [p for p in r.destination.properties if p.name.value == r.inverse_name]
                    if not inverse or not isinstance(inverse[0], Relationship):
                        unestablished("the inverse relationship %s.%s" % (r.inverse_entity, r.inverse_name))
                    r.inverse = inverse[0]
        for e in self.entities:
            inherited = set()
            p = e.parent
            while p is not None:
                if p is e:
                    unestablished("the entity %s inheriting from itself" % e.name.value)
                inherited |= {x.name.value for x in p.properties}
                p = p.parent
            clash = inherited & {x.name.value for x in e.properties}
            if clash:
                raise Invalid("%s.%s conflicts with a property inherited from %s" % (e.name.value, min(clash), e.parent.name.value))
            if e.unique:
                for r in e.properties:
                    if isinstance(r, Relationship) and r.inverse and not r.inverse.optional and not r.inverse.to_many:
                        raise Invalid("Entity %s cannot have uniqueness constraints and to-one mandatory inverse relationship %s.%s"
                                      % (e.name.value, r.destination.name.value, r.inverse.name.value))
        self.configurations = []
        for c in root.findall("configuration"):
            known(c, {"name", "usedWithCloudKit"}, ("memberEntity",))
            members = []
            for m in c.findall("memberEntity"):
                known(m, {"name"})
                if m.get("name") not in by_name:
                    unestablished("the configuration member %s" % m.get("name"))
                members.append(by_name[m.get("name")])
            self.configurations.append((xml_str(c.get("name")), members))
        self.fetch_requests = []
        for r in root.findall("fetchRequest"):
            if r.get("entity") not in by_name:
                unestablished("the fetch request entity %s" % r.get("entity"))
            self.fetch_requests.append((xml_str(r.get("name")), FetchRequest(r, by_name[r.get("entity")])))
        for e in root.findall("elements"):
            known(e, (), ("element",))
        self.identifier = xml_str(root.get("userDefinedModelVersionIdentifier", ""))

    def encode_with(self, a, f):
        f["NSEntities"] = a.encode(copied_dictionary([(e.name, e) for e in self.entities]))
        if self.configurations:
            f["NSConfigurations"] = a.encode(NSDictionary(
                [(name, NSSet(members, [m.name.value for m in members]))
                 for name, members in sorted(self.configurations, key=lambda c: natural_key(c[0].value))],
                "NSMutableDictionary"))
        f["NSFetchRequestTemplates"] = a.encode(copied_dictionary(self.fetch_requests) if self.fetch_requests else None)
        f["NSVersionIdentifiers"] = a.encode(NSSet([self.identifier], [self.identifier.value]))
        tool, xcode = self.versions
        f["CDMCompilerToolVersion"] = Num("f64", float(tool))
        f["CDMCompilerXcodeVersion"] = Num("i64", xcode)


# --- Binary property list ------------------------------------------------------

class Writer:
    """Apple's binary property list writer for a keyed archive: a string or
    number object written once, a key once among the keys, a UID or
    Boolean written each time it occurs (referring to the first)."""

    def __init__(self):
        self.items = []
        self.shared = {}

    def _slot(self, kind, payload):
        self.items.append((kind, payload))
        return len(self.items) - 1

    def _once(self, ident, kind, payload):
        if ident not in self.shared:
            self.shared[ident] = self._slot(kind, payload)
        return self.shared[ident]

    def add(self, v):
        if isinstance(v, (UID, Bool)):
            kind = "uid" if isinstance(v, UID) else "bool"
            n = self._slot(kind, v.value)
            return self.shared.setdefault((kind, v.value), n)
        if isinstance(v, list):
            return self.add(Array(v))
        if isinstance(v, Num):
            return self._once(v.ident, v.kind, v.value)
        if isinstance(v, Str):
            return self._once(v.ident, "str", v.value)
        if isinstance(v, Key):
            return self._once(("key", v.value), "str", v.value)
        if isinstance(v, Data):
            return self._once(v.ident, "data", v.value)
        if isinstance(v, Array):
            n = self._slot("array", None)
            self.items[n] = ("array", [self.add(x) for x in v.items])
            return n
        if isinstance(v, Dict):
            n = self._slot("dict", None)
            keys = [self.add(k) for k, _ in v.pairs]
            values = [self.add(x) for _, x in v.pairs]
            self.items[n] = ("dict", (keys, values))
            return n
        raise TypeError(type(v))

    def to_bytes(self, top):
        self.add(top)
        n = len(self.items)
        ref = 1 if n < 256 else 2 if n < 65536 else 4

        def refs(rs):
            return b"".join(r.to_bytes(ref, "big") for r in rs)

        def head(kind, count):
            if count < 15:
                return bytes([kind << 4 | count])
            return bytes([kind << 4 | 0xF]) + integer(count)

        def integer(i):
            if 0 <= i < 1 << 8:
                return b"\x10" + i.to_bytes(1, "big")
            if 0 <= i < 1 << 16:
                return b"\x11" + i.to_bytes(2, "big")
            if 0 <= i < 1 << 32:
                return b"\x12" + i.to_bytes(4, "big")
            return b"\x13" + i.to_bytes(8, "big", signed=True)
        out = bytearray(b"bplist00")
        offsets = []
        for kind, p in self.items:
            offsets.append(len(out))
            if kind == "uid":
                size = 1 if p < 256 else 2 if p < 65536 else 4
                out += bytes([0x80 | (size - 1)]) + p.to_bytes(size, "big")
            elif kind == "bool":
                out += b"\x09" if p else b"\x08"
            elif kind in ("i32", "i64"):
                out += integer(p)
            elif kind == "f32":
                out += b"\x22" + struct.pack(">f", p)
            elif kind == "f64":
                out += b"\x23" + struct.pack(">d", p)
            elif kind == "data":
                out += head(0x4, len(p)) + p
            elif kind == "str":
                if p.isascii():
                    out += head(0x5, len(p)) + p.encode("ascii")
                else:
                    units = p.encode("utf-16-be")
                    out += head(0x6, len(units) // 2) + units
            elif kind == "array":
                out += head(0xA, len(p)) + refs(p)
            else:
                out += head(0xD, len(p[0])) + refs(p[0] + p[1])
        table = len(out)
        size = 1 if table < 256 else 2 if table < 65536 else 4 if table < 1 << 32 else 8
        for o in offsets:
            out += o.to_bytes(size, "big")
        out += bytes(6) + bytes([size, ref]) + n.to_bytes(8, "big") + (0).to_bytes(8, "big") + table.to_bytes(8, "big")
        return bytes(out)


def compile_version(path, versions):
    """One .xcdatamodel's .mom, and its Model."""
    contents = os.path.join(path, "contents")
    try:
        root = ET.parse(contents).getroot()
    except (OSError, ET.ParseError) as e:
        raise Invalid("can't read %s: %s" % (contents, e))
    model = Model(root, versions)
    return Writer().to_bytes(Archiver().archive(model)), model


# --- Version hashes (docs/clean-room/momc.md) ---------------------------------
# What Core Data hashes, read in a process of our own: a model built with
# Core Data's public API, its versionHash and versionChecksum asked for, and
# CommonCrypto's SHA-256 functions interposed there to log their input.

def _sha256(*parts):
    return hashlib.sha256(b"".join(parts)).digest()


def _le32(n):
    return (n & 0xFFFFFFFF).to_bytes(4, "little")


def property_hash(p):
    """A property's version hash: the SHA-256 of its name, a flags byte
    (optional 1, transient 2, read-only 4), a zero byte and its version hash
    modifier; then, for an attribute or a relationship, the SHA-256 of that
    and what it adds."""
    flags = (1 if p.optional else 0) | (2 if p.transient else 0) | (4 if p.read_only else 0)
    common = _sha256(p.name.value.encode(), bytes([flags, 0]), p.modifier.value.encode() if p.modifier else b"")
    if isinstance(p, DerivedAttribute):
        # Its derivation expression's description follows, as NSExpression
        # describes it (`count:(title)`), which isn't reproduced.
        unestablished("a derived attribute's version hash")
    if isinstance(p, Attribute):
        # The type, then a byte for each of external storage and preserving
        # the value in history on deletion.
        return _sha256(common, _le32(p.type), b"\x01" if p.flags & 4 else b"", b"\x01" if p.flags & 16 else b"")
    if isinstance(p, Relationship):
        # The destination's and the inverse's names, the maximum and minimum
        # counts, and two bytes for an ordered one (not the deletion rule).
        return _sha256(common, p.destination.name.value.encode(), p.inverse.name.value.encode() if p.inverse else b"",
                       _le32(p.max_count), _le32(p.min_count), b"\x01\x00" if p.ordered else b"")
    return common


def entity_hash(e):
    """An entity's version hash: the SHA-256 of its name, whether it's
    abstract, its parent's hash, the hashes of all its properties but
    transient ones (fetched ones among them; inherited ones too) in byte
    order of their names, and its version hash modifier."""
    properties = []
    p = e
    while p is not None:
        properties.extend(x for x in p.properties if not x.transient)
        p = p.parent
    parts = [e.name.value.encode(), bytes([1 if e.abstract else 0])]
    if e.parent is not None:
        parts.append(entity_hash(e.parent))
    parts.extend(property_hash(x) for x in sorted(properties, key=lambda x: x.name.value.encode()))
    if e.modifier:
        parts.append(e.modifier.value.encode())
    # Then each uniqueness constraint, its items joined by dots.
    parts.extend(".".join(x.value for x in c).encode() for c in e.unique)
    return _sha256(*parts)


def version_hashes(model):
    """{entity name: hash}, and the model's version checksum: the base64
    SHA-256 of its entities' hashes in byte order of their names."""
    hashes = {e.name.value: entity_hash(e) for e in model.entities}
    checksum = base64.b64encode(_sha256(*(hashes[n] for n in sorted(hashes, key=str.encode)))).decode()
    return hashes, checksum


def version_info(name, model):
    """A single version's VersionInfo.plist, as Xcode's momc writes it: its
    entities' hashes and its checksum, in CoreFoundation dictionaries, the
    version's name a string of its own in each (106 test models)."""
    hashes, checksum = version_hashes(model)
    if name in hashes:
        unestablished("an entity named as its model version")
    # The entities' table as the .mom's (filled in name order), filled again
    # from it, then copied twice, each copy created from the last's order:
    # the order of all 100 test models' tables (a single filled copy fits
    # 98).
    order = cf_dictionary_order(cf_dictionary_order(sorted(hashes, key=natural_key)))
    order = cf_created_order(cf_created_order(order))
    entities = Dict([(Str(n), Data(hashes[n])) for n in order])
    tables = {
        "NSManagedObjectModel_VersionHashes": Dict([(Str(name, fresh()), entities)]),
        "NSManagedObjectModel_VersionChecksums": Dict([(Str(name, fresh()), Str(checksum, fresh()))]),
    }
    return Writer().to_bytes(Dict([(Str(k), tables[k]) for k in cf_dictionary_order(sorted(tables))]))


def version_name(path):
    return os.path.splitext(os.path.basename(path.rstrip("/")))[0]


def compile_models(source, output, versions):
    source = source.rstrip("/")
    if source.endswith(".xcdatamodel"):
        out = output if output.endswith(".mom") else os.path.join(output, version_name(source) + ".mom")
        mom, _ = compile_version(source, versions)
        os.makedirs(os.path.dirname(out) or ".", exist_ok=True)
        with open(out, "wb") as f:
            f.write(mom)
        return
    if not source.endswith(".xcdatamodeld"):
        unestablished("the model %s" % source)
    try:
        models = sorted(n for n in os.listdir(source) if n.endswith(".xcdatamodel"))
    except OSError as e:
        raise Invalid("can't read the model: %s" % e.strerror)
    if not models:
        unestablished("a model bundle without versions")
    current = None
    try:
        with open(os.path.join(source, ".xccurrentversion"), "rb") as f:
            current = plistlib.load(f).get("_XCCurrentVersionName")
    except FileNotFoundError:
        pass
    except plistlib.InvalidFileException:
        unestablished("the model's .xccurrentversion")
    if current is None and len(models) > 1:
        unestablished("a versioned model without a current version")
    if current is not None and current not in models:
        unestablished("the current version %s" % current)
    compiled = {version_name(m): compile_version(os.path.join(source, m), versions) for m in models}
    moms = {name: mom for name, (mom, _) in compiled.items()}
    # A single version's VersionInfo.plist is Xcode's; several versions'
    # also need the .omo beside the current one, which isn't reproduced.
    info_bytes = None
    if len(models) == 1:
        try:
            (name, (_, model)), = compiled.items()
            info_bytes = version_info(name, model)
        except Unestablished:
            if not RELAXED:
                raise
    elif not RELAXED:
        unestablished("a model with several versions (the .omo beside its current one)")
    out = output if output.endswith(".momd") else os.path.join(output, version_name(source) + ".momd")
    os.makedirs(out, exist_ok=True)
    for name, mom in moms.items():
        with open(os.path.join(out, name + ".mom"), "wb") as f:
            f.write(mom)
    if info_bytes is None:
        info = {}
        if len(models) > 1:
            info["NSManagedObjectModel_CurrentVersionName"] = version_name(current)
        info_bytes = plistlib.dumps(info, fmt=plistlib.FMT_BINARY)
    with open(os.path.join(out, "VersionInfo.plist"), "wb") as f:
        f.write(info_bytes)


# --- Generated classes (momc --action generate) -------------------------------

# The line of Xcode's header naming the user and the date it ran, which no
# build reproduces: this one in its place (docs/clean-room/momc.md).
GENERATED_BY = "//  Created by momc."

# Each attribute type's Swift type: as a scalar, and as an object.
SWIFT_TYPES = {
    "Integer 16": ("Int16", "NSNumber?"), "Integer 32": ("Int32", "NSNumber?"), "Integer 64": ("Int64", "NSNumber?"),
    "Double": ("Double", "NSNumber?"), "Float": ("Float", "NSNumber?"), "Boolean": ("Bool", "NSNumber?"),
    "Date": ("TimeInterval", "Date?"), "String": (None, "String?"), "Decimal": (None, "NSDecimalNumber?"),
    "Binary": (None, "Data?"), "UUID": (None, "UUID?"), "URI": (None, "URL?"),
}


def header(file_name):
    return ["//", "//  " + file_name, "//  ", "//", GENERATED_BY, "//",
            "//  This file was automatically generated and should not be edited.", "//", ""]


def swift_class(class_name):
    """The Swift class an entity's class name declares, and its module (""
    for a leading dot, None unqualified)."""
    if "." in class_name:
        module, _, name = class_name.rpartition(".")
        return name, module
    return class_name, None


def generate(source, output):
    """The Swift classes Xcode's momc generates for a model's current
    version: for each entity generated as a class, its class and its
    properties; as a category, its properties; and the model's file."""
    source = source.rstrip("/")
    if source.endswith(".xcdatamodeld"):
        try:
            models = sorted(n for n in os.listdir(source) if n.endswith(".xcdatamodel"))
        except OSError as e:
            raise Invalid("can't read the model: %s" % e.strerror)
        current = None
        try:
            with open(os.path.join(source, ".xccurrentversion"), "rb") as f:
                current = plistlib.load(f).get("_XCCurrentVersionName")
        except FileNotFoundError:
            pass
        if current is None:
            if len(models) != 1:
                unestablished("a model bundle without one current version")
            current = models[0]
        version = os.path.join(source, current)
    elif source.endswith(".xcdatamodel"):
        version = source
    else:
        unestablished("the model %s" % source)
    try:
        root = ET.parse(os.path.join(version, "contents")).getroot()
    except (OSError, ET.ParseError) as e:
        raise Invalid("can't read %s: %s" % (version, e))
    entities = {e.get("name"): e for e in root.findall("entity")}

    def class_of(entity):
        return entity.get("representedClassName") or None

    files = {}
    for e in root.findall("entity"):
        mode = e.get("codeGenerationType")
        if mode is None:
            continue
        if mode not in ("class", "category"):
            unestablished("the code generation type %r" % mode)
        if class_of(e) is None:
            continue
        name, _ = swift_class(class_of(e))
        if not name.isascii():
            unestablished("the class name %r in generated classes" % name)
        parent = entities.get(e.get("parentEntity")) if e.get("parentEntity") else None
        if e.get("parentEntity") and parent is None:
            unestablished("the parent entity %s" % e.get("parentEntity"))
        if parent is not None and class_of(parent) is None:
            unestablished("a parent entity without a class")
        superclass = swift_class(class_of(parent))[0] if parent is not None else "NSManagedObject"
        if mode == "class":
            objc = "" if "." in class_of(e) else "@objc(%s)" % name
            files[name + "+CoreDataClass.swift"] = header(name + "+CoreDataClass.swift") + [
                "public import Foundation", "public import CoreData", "",
                "public typealias %sCoreDataClassSet = NSSet" % name, "", objc,
                "public class %s: %s {" % (name, superclass), "", "}"]
        imports, lines, accessors = [], [], []
        # Attributes, then relationships, each in the document's order.
        for p in [p for p in e if p.tag == "attribute"] + [p for p in e if p.tag != "attribute"]:
            if p.tag == "attribute":
                kind = p.get("attributeType")
                if p.get("attributeType") == "Transformable":
                    swift = (p.get("customClassName") or "NSObject") + "?"
                elif kind in SWIFT_TYPES:
                    scalar, boxed = SWIFT_TYPES[kind]
                    # Numbers are scalars unless the model says not; dates aren't
                    # unless it says so.
                    default = "NO" if kind == "Date" else "YES"
                    swift = scalar if scalar and p.get("usesScalarValueType", default) == "YES" else boxed
                else:
                    unestablished("the generated type of a %s attribute" % kind)
                lines.append("    @NSManaged nonisolated public var %s: %s" % (p.get("name"), swift))
            elif p.tag == "relationship":
                destination = entities.get(p.get("destinationEntity"))
                if destination is None:
                    unestablished("the destination entity %s" % p.get("destinationEntity"))
                target, module = swift_class(class_of(destination)) if class_of(destination) else ("NSManagedObject", None)
                if module and "import %s" % module not in imports:
                    imports.append("import %s" % module)
                rel = p.get("name")
                if p.get("toMany") != "YES":
                    lines.append("    @NSManaged nonisolated public var %s: %s?" % (rel, target))
                    continue
                ordered = p.get("ordered") == "YES"
                lines.append("    @NSManaged nonisolated public var %s: %s?" % (rel, "NSOrderedSet" if ordered else "NSSet"))
                cap = rel[:1].upper() + rel[1:]
                methods = [
                    ("insertObject:in%sAtIndex:" % cap, "insertInto%s(_ value: %s, at idx: Int)" % (cap, target)),
                    ("removeObjectFrom%sAtIndex:" % cap, "removeFrom%s(at idx: Int)" % cap),
                    ("insert%s:atIndexes:" % cap, "insertInto%s(_ values: [%s], at indexes: NSIndexSet)" % (cap, target)),
                    ("remove%sAtIndexes:" % cap, "removeFrom%s(at indexes: NSIndexSet)" % cap),
                    ("replaceObjectIn%sAtIndex:withObject:" % cap, "replace%s(at idx: Int, with value: %s)" % (cap, target)),
                    ("replace%sAtIndexes:with%s:" % (cap, cap), "replace%s(at indexes: NSIndexSet, with values: [%s])" % (cap, target)),
                ] if ordered else []
                collection = "NSOrderedSet" if ordered else "NSSet"
                methods += [
                    ("add%sObject:" % cap, "addTo%s(_ value: %s)" % (cap, target)),
                    ("remove%sObject:" % cap, "removeFrom%s(_ value: %s)" % (cap, target)),
                    ("add%s:" % cap, "addTo%s(_ values: %s)" % (cap, collection)),
                    ("remove%s:" % cap, "removeFrom%s(_ values: %s)" % (cap, collection)),
                ]
                accessors += ["", "// MARK: Generated accessors for %s" % rel, "extension %s {" % name, ""]
                for selector, signature in methods:
                    accessors += ["    @objc(%s)" % selector, "    @NSManaged nonisolated public func %s" % signature, ""]
                accessors.append("}")
            elif p.tag not in ("fetchedProperty", "userInfo", "uniquenessConstraints", "fetchIndex"):
                unestablished("an entity's <%s> in generated classes" % p.tag)
        body = header(name + "+CoreDataProperties.swift") + [
            "public import Foundation", "public import CoreData"] + (imports or [""]) + [
            "", "public typealias %sCoreDataPropertiesSet = NSSet" % name, "", "extension %s {" % name, "",
            "    @nonobjc public class func fetchRequest() -> NSFetchRequest<%s> {" % name,
            "        return NSFetchRequest<%s>(entityName: \"%s\")" % (name, e.get("name")), "    }", ""] + lines + [
            "", "}"] + accessors
        # Identifiable once, on the root of an entity hierarchy.
        if parent is None:
            body += ["", "extension %s : Identifiable {" % name, "", "}"]
        files[name + "+CoreDataProperties.swift"] = body
    if not files:
        return
    # The model's name as macOS's file system gives it, decomposed (NFD).
    model_name = unicodedata.normalize("NFD", os.path.splitext(os.path.basename(source))[0])
    # The model's name as an identifier: its letters alone, accents removed
    # (`My-Model` My Model, `9lives` lives, `Ünï` Uni).
    folded = "".join(c for c in unicodedata.normalize("NFD", model_name) if not unicodedata.combining(c))
    prefix = "".join(c for c in folded if c.isalpha())
    if not prefix or not prefix.isascii():
        unestablished("the model name %r in generated classes" % model_name)
    files[model_name + "+CoreDataModel.swift"] = header(model_name + "+CoreDataModel.swift") + [
        "public import Foundation", "public import CoreData", "",
        "public typealias %sFoundationFrameworkSet = NSSet" % prefix,
        "public typealias %sCoreDataFrameworkManagedObject = NSManagedObject" % prefix, "", ""]
    os.makedirs(output, exist_ok=True)
    for name, lines in files.items():
        with open(os.path.join(output, name), "w") as f:
            f.write("\n".join(lines) + "\n")


def main(argv):
    action = "compile"
    positional = []
    i = 0
    while i < len(argv):
        a = argv[i]
        name, eq, value = a.partition("=")
        takes_value = name in ("--sdkroot", "--module", "--action", "--swift-version") or \
            re.fullmatch(r"--[a-z]+-deployment-target", name)
        if takes_value:
            if not eq:
                i += 1
                if i == len(argv):
                    print("error: momc: %s needs a value" % name, file=sys.stderr)
                    return 1
                value = argv[i]
            if name == "--action":
                action = value
        elif a in ("--no-warnings", "--no-inverse-relationship-warnings", "--no-max-property-count-warnings",
                   "--no-delete-rule-warnings"):
            pass
        elif a.startswith("--"):
            print("error: momc: unsupported option %s" % a, file=sys.stderr)
            return 1
        else:
            positional.append(a)
        i += 1
    if action not in ("compile", "generate"):
        print("error: momc: --action %s isn't supported off a Mac" % action, file=sys.stderr)
        return 1
    if len(positional) != 2:
        print("error: momc: expected `momc [options] MODEL OUTPUT`", file=sys.stderr)
        return 1
    source, output = positional
    if action == "generate":
        if not RELAXED:
            print("error: momc: %s: Xcode's generated classes name the user and the date in their header, which no "
                  "build reproduces; --config=relax_momc writes them with a fixed header" % source, file=sys.stderr)
            return 1
        try:
            generate(source, output)
        except Unestablished as e:
            print("error: momc: %s: %s is not generated as Xcode generates it" % (source, e), file=sys.stderr)
            return 1
        except Invalid as e:
            print("error: momc: %s: %s" % (source, e), file=sys.stderr)
            return 1
        return 0
    try:
        compile_models(source, output, MOMC_VERSIONS[xcode_version()])
    except Unestablished as e:
        print("error: momc: %s: %s is not compiled as Xcode compiles it%s" % (
            source, e, "" if RELAXED else "; --config=relax_momc compiles it without what isn't reproduced"), file=sys.stderr)
        return 1
    except Invalid as e:
        print("error: momc: %s: %s" % (source, e), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
