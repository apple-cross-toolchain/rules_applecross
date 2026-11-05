# momc and mapc

Core Data models (`.xcdatamodeld`, `.xcdatamodel`) compile off a Mac with
`--config=relax_momc`: each model version to the `.mom` Xcode 27.2's
`momc` writes, byte for byte, with a `VersionInfo.plist` that leaves out what
isn't established (below). `toolchain/stubs/momc.py` is the implementation,
pure Python; `tests/momc` compares it with Xcode's own output for 86 models,
and refuses 7 more. Mapping models (`mapc`) and generated classes
(`momc --action generate`) aren't reproduced.

## What momc writes

`momc` compiles a model bundle to `<Model>.momd/`: a `.mom` for each
version, `VersionInfo.plist` and, beside a versioned model's current version,
an `.omo` (a `momv2` binary of its own, which Core Data reads instead of
the `.mom` when it's there). A single `.xcdatamodel` compiles to one `.mom`.
The deployment target, platform and `--module` don't change any of them.

A `.mom` is an `NSKeyedArchiver` archive of an `NSManagedObjectModel`, its
`NSEntityDescription`s, their `NSAttributeDescription`s and
`NSRelationshipDescription`s, and the fetch requests, configurations,
validation predicates, dictionaries, sets, arrays, dates and decimal
numbers they hold.

### The archive

- **Object numbers.** Objects are numbered as they're first encoded, depth
  first. An object encoded conditionally (an entity's superentity and model, a
  property's entity, a relationship's destination and inverse) is numbered
  where it's first mentioned and written where it's encoded outright. An
  object's class description is numbered after its fields.
- **Field order.** An object's fields are a CoreFoundation dictionary: they
  enumerate in [hash-table order](foundation-hashing.md) for the order they
  went in. That order is fixed for each class (`KEY_ORDER`), fitted to every
  combination of fields seen.
- **Identity.** The archiver finds objects again by identity, and the
  property-list writer shares a string or number only when the object
  written is the same one:
  - Every ASCII string the model's XML holds is one string for its value.
  - Any other string is one for each attribute in the XML it comes from.
  - Integers are one object for each value and CoreFoundation type. The
    archive's keys, counts and flags are 32-bit; default values, Xcode's
    version, a fetch request's flags and types, and predicates' constants,
    types and warnings are 64-bit.
  - A floating-point number is one object for its value and type when it's
    a whole number of magnitude below 2^55. Otherwise it's an object of its
    own.
  - Equal dates are one object.
  - Booleans and UIDs are written each time they occur, referring to the
    first.
  - A decimal's mantissa is written each time.
  - A predicate's `SELF` is one object, and so is its `valueForKey:`
    selector name (apart from an equal string in the XML). A date rule's
    key, `timeIntervalSinceReferenceDate`, is a new string each time; a
    string rule's, `length`, is shared with equal strings.
- **Class names.** These are one string shared with equal values, except
  Core Data's own classes, `NSDecimalNumber` and its placeholder, `NSValue`,
  `NSPredicate`, `NSComparisonPredicateOperator` and
  `NSStringPredicateOperator`, whose names are a new string each time (as
  the value class name of a Decimal attribute is).

### The model

- **Entities.** An entity records its name, class name (`$null` without one),
  superentity, subentities, properties, user info (one empty dictionary
  shared by those without), version hash modifier, renaming identifier (its
  name without one), uniqueness constraints and abstractness.
- **Property fields.** Properties record their entity, name, renaming
  identifier, user info and version hash modifier, then their own fields,
  each Boolean only when true.
- **`_P` flags.** 1, plus 2 if optional, 8 if transient, 16 and 32 without
  validation predicates and warnings, 64 without user info, 128 without a
  version hash modifier, 256 if indexed by Spotlight (not when transient),
  1024 if an ordered relationship, 2048 if cloud encrypted.
- **`NSFlagsKey`.** 4 for external binary storage, 16 to preserve the value
  after deletion.
- **Attribute types.** Integer 16, 32 and 64 are 100, 200 and 300; Decimal
  400; Double 500; Float 600; String 700; Boolean 800; Date 900; Binary
  1000; UUID 1100; URI 1200; Transformable 1800.
- **Delete rules.** No Action 0, Nullify 1, Cascade 2, Deny 3.
- **Validation rules.** An attribute's minimum, maximum and pattern become
  `NSValidationPredicates`, an array of `NSComparisonPredicate`s in that
  order, and `NSValidationWarnings`, an array of their warning codes,
  encoded after the renaming identifier.
  - A number's bound compares `SELF` (1620 for the minimum, 1610 for the
    maximum); a date's compares `SELF.timeIntervalSinceReferenceDate`
    (1640, 1630); a string's compares `SELF.length` (1670, 1660).
  - A minimum is `>=` (operator type and variant 3), a maximum `<=` (1).
  - A pattern is `SELF MATCHES` it (`NSMatchingPredicateOperator`, 1680).
  - Integer, Decimal and string-length bounds are the text's `longLongValue`
    (white space skipped, anything after the integer ignored, 0 for none,
    saturating); Double and Float bounds its `doubleValue` as a 64-bit
    number (`inf` is 0); a date's is whole minutes, rounded down, as for
    default values.
  - A rule given, even empty, is a predicate. Booleans', Binary data's,
    UUIDs', URIs' and Transformables' rules are left out.
- **Default values.**
  - Integers are 64-bit numbers.
  - Floats are 32-bit, doubles 64-bit.
  - A date is whole minutes since the reference date, rounded down.
  - A decimal is compact: no trailing zeros in its mantissa, and not
    compact when it's zero.
- **Dictionary orders.**
  - Properties go in as attributes, then relationships, each in name order
    (case-insensitive, runs of digits as numbers).
  - User info goes in name order.
  - Entities, fetch requests and subentities go into a dictionary in name
    order, and the archived dictionary is filled from that one.
  - A configuration's entities are a set created whole, in the model's
    order.

### Checked against Xcode

During the work, 200 random models were also checked. Each had up to 9
entities, with every supported attribute type, flag, default, relationship
rule, inheritance, constraint, configuration and fetch request. Of the 111
with ASCII names that Xcode compiles:
- 108 compiled the same;
- two were refused for a userInfo key given twice;
- one differed in its subentities' order.

Xcode refuses an entity with uniqueness constraints and a mandatory to-one
inverse relationship, and a property that conflicts with an inherited one;
so does this. The subentity order can differ only when the names collide in
the dictionary: 71 of the 74 such dictionaries examined are in Xcode's
order, and the other three have two subentities swapped.

### Refused

Compiling refuses these, relaxed or not:

- fetch requests with predicates;
- fetched properties, derived and composite attributes, fetch indexes;
- fetch request options other than a limit and a batch size;
- a userInfo key given twice;
- names other than ASCII, which Xcode orders by a collation of its own;
- an Xcode other than 27.2 unless `--config=relax_xcode_version`, since
  every model records momc's version (27.2's is 25436) and Xcode's.

## VersionInfo.plist

`VersionInfo.plist` holds the current version's name (for a versioned
model), each entity's version hash (`NSManagedObjectModel_VersionHashes`,
32 bytes) and a checksum of each version (`NSManagedObjectModel_VersionChecksums`,
a base64 SHA-256). The hashes are SHA-256 digests whose input isn't
established, so without `--config=relax_momc` compiling is refused.
Relaxed, `VersionInfo.plist` holds only the current version's name, and no
`.omo` is written.

Core Data computes the hashes from the model itself. On a Mac, a store
created with either side's version 1 opened, migrated lightweight and read
with either side's `.momd` (this one's `VersionInfo.plist`, no `.omo`)
exactly as with Xcode's.

Probing `NSAttributeDescription.versionHash` through Core Data's public API
shows that an attribute's hash depends on:
- its name, type, optionality and transience;
- its version hash modifier (an empty modifier counts as none);
- one more input, which `allowsExternalBinaryDataStorage` and
  `preservesValueInHistoryOnDeletion` set identically.

It doesn't depend on the default value, renaming identifier, user info,
Spotlight indexing, value class name, value transformer name or cloud
encryption. These inputs were tried, all without a match:

- the UTF-8, UTF-8 with a NUL, and UTF-16 name, followed or preceded by
  the type and flags as 1-, 2-, 4- and 8-byte integers in either byte
  order, in every order of up to five components;
- one flags word holding optionality, transience and that third input at
  any of the first sixteen bit positions, with the type in any width;
- text forms (`1`/`0`, `YES`/`NO`, `true`/`false`, `Y`/`N`) joined by
  any of ten separators, with the type as a number and its value class
  name, in every order of up to six components;
- Foundation's binary and XML property lists, keyed archives,
  descriptions and joined strings of arrays and dictionaries of those
  fields.

A mapping model (`mapc`) records its models' hashes too. The input could
still be found by more probing, or, in the private fork, by reading the
implementation.

## Generated classes

`momc --action generate`, which rules_apple runs for
`apple_core_data_model`, writes Swift classes that don't depend on the
hashes, but whose header names the user building them and the date
(`Created by <full name> on 11/4/26.`), so Xcode's own output varies
between machines and days. It's refused.
