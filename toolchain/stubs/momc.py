#!/usr/bin/env python3
"""momc off a Mac: a Core Data model (`.xcdatamodeld`, or one
`.xcdatamodel`) compiled as rules_apple runs it (`momc
--<platform>-deployment-target V --module M MODEL OUTPUT`); see
docs/clean-room/momc.md.

Each model version compiles to the `.mom` Xcode 27.2's momc writes, a
keyed archive of the model. What `VersionInfo.plist` records besides the
current version, each entity's version hash and each version's checksum,
isn't established, so the compile is refused unless
RULES_APPLECROSS_RELAX_MOMC=1 (--config=relax_momc, or all of
--config=relaxed), which writes VersionInfo.plist without them and leaves
out the `.omo` Xcode adds beside a versioned model's current version: Core
Data loads, saves and migrates stores from the `.mom` and computes the
hashes itself. A model using anything not established against Xcode
(predicates, fetched and derived properties, composite attributes, fetch
indexes, among others) is refused either way.
"""

import math
import os
import plistlib
import re
import struct
import sys
import xml.etree.ElementTree as ET

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
    "NSDecimalNumberPlaceholder", "NSDecimalNumber", "NSValue",
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
                               "NSDefaultValue", "NSFlagsKey", "NSValueTransformerName", "$class"],
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

    def encode_property(self, a, f, extra):
        f["NSEntity"] = a.encode(self.entity, conditional=True)
        f["NSPropertyName"] = a.encode(self.name)
        f["NSRenamingIdentifier"] = a.encode(self.renaming)
        if self.user_info:
            f["NSUserInfo"] = a.encode(self.user_info)
        if self.modifier:
            f["NSVersionHashModifier"] = a.encode(self.modifier)
        f["_P"] = Num("i32", 1 | 16 | 32 | (2 if self.optional else 0) | (8 if self.transient else 0)
                      | (0 if self.user_info else 64) | (0 if self.modifier else 128) | extra)
        if self.optional:
            f["NSIsOptional"] = Bool(True)
        if self.transient:
            f["NSIsTransient"] = Bool(True)


class Attribute(Property):
    class_name = "NSAttributeDescription"
    ATTRIBUTES = {"name", "attributeType", "optional", "transient", "defaultValueString", "defaultDateTimeInterval",
                  "usesScalarValueType", "spotlightIndexingEnabled", "allowsExternalBinaryDataStorage",
                  "preserveAfterDeletion", "allowsCloudEncryption", "valueTransformerName", "customClassName",
                  "versionHashModifier", "elementID", "syncable"}

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

    @staticmethod
    def _default(kind, element):
        text = element.get("defaultValueString")
        interval = element.get("defaultDateTimeInterval")
        if interval is not None:
            if kind != "Date" or text is not None:
                unestablished("a defaultDateTimeInterval for a %s attribute" % kind)
            try:
                seconds = float(interval)
            except ValueError:
                unestablished("the default date %r" % interval)
            # Whole minutes, earlier ones for a time between.
            return NSDate(float(math.floor(seconds / 60) * 60))
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
                value = struct.unpack(">f", struct.pack(">f", value))[0]
            # A whole number of magnitude below 2^55 is one object for its
            # value (and type); any other is an object of its own.
            shared = value == int(value) and abs(value) < 1 << 55 if math.isfinite(value) else False
            return Num("f32" if kind == "Float" else "f64", value, None if shared else fresh())
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
        known(element, self.ATTRIBUTES, ("attribute", "relationship", "userInfo", "uniquenessConstraints"))
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
        attributes = [Attribute(self, e) for e in element.findall("attribute")]
        relationships = [Relationship(self, e) for e in element.findall("relationship")]
        self.properties = sorted(attributes, key=lambda p: natural_key(p.name.value)) + \
            sorted(relationships, key=lambda p: natural_key(p.name.value))
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
        known(element, {"name", "entity", "fetchLimit", "fetchBatchSize"})
        super().__init__()
        self.entity = entity
        try:
            self.limit = int(element.get("fetchLimit", "0"))
            self.batch = int(element.get("fetchBatchSize", "0"))
        except ValueError:
            unestablished("the fetch request %s's limits" % element.get("name"))

    def encode_with(self, a, f):
        f["NSEntity"] = a.encode(self.entity)
        for k in ("NSPredicate", "NSSortDescriptors", "NSValuesToFetch", "NSValuesToGroupBy", "NSHavingPredicate",
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
    """One .xcdatamodel's .mom."""
    contents = os.path.join(path, "contents")
    try:
        root = ET.parse(contents).getroot()
    except (OSError, ET.ParseError) as e:
        raise Invalid("can't read %s: %s" % (contents, e))
    model = Model(root, versions)
    return Writer().to_bytes(Archiver().archive(model))


def version_name(path):
    return os.path.splitext(os.path.basename(path.rstrip("/")))[0]


def compile_models(source, output, versions):
    source = source.rstrip("/")
    if source.endswith(".xcdatamodel"):
        out = output if output.endswith(".mom") else os.path.join(output, version_name(source) + ".mom")
        mom = compile_version(source, versions)
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
    moms = {version_name(m): compile_version(os.path.join(source, m), versions) for m in models}
    out = output if output.endswith(".momd") else os.path.join(output, version_name(source) + ".momd")
    os.makedirs(out, exist_ok=True)
    for name, mom in moms.items():
        with open(os.path.join(out, name + ".mom"), "wb") as f:
            f.write(mom)
    info = {}
    if len(models) > 1:
        info["NSManagedObjectModel_CurrentVersionName"] = version_name(current)
    with open(os.path.join(out, "VersionInfo.plist"), "wb") as f:
        plistlib.dump(info, f, fmt=plistlib.FMT_BINARY)


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
    if action != "compile":
        print("error: momc: --action %s isn't supported off a Mac: only models are compiled" % action, file=sys.stderr)
        return 1
    if len(positional) != 2:
        print("error: momc: expected `momc [options] MODEL OUTPUT`", file=sys.stderr)
        return 1
    source, output = positional
    if not RELAXED:
        print("error: momc: %s: VersionInfo.plist's entity version hashes aren't reproduced off a Mac; "
              "--config=relax_momc writes the model without them (Core Data computes them itself)" % source, file=sys.stderr)
        return 1
    try:
        compile_models(source, output, MOMC_VERSIONS[xcode_version()])
    except Unestablished as e:
        print("error: momc: %s: %s is not compiled as Xcode compiles it" % (source, e), file=sys.stderr)
        return 1
    except Invalid as e:
        print("error: momc: %s: %s" % (source, e), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
