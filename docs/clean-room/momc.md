# momc and mapc

Core Data models (`.xcdatamodeld`, `.xcdatamodel`) compile off a Mac: each
model version to the `.mom` Xcode 27.2's `momc` writes, byte for byte, and a
model with one version to Xcode's `VersionInfo.plist` too, its entities'
version hashes and checksum included (below). A model with several versions,
whose current version also gets an `.omo`, or with a derived attribute,
compiles with `--config=relax_momc`, with a `VersionInfo.plist` that leaves
out what isn't established. `toolchain/stubs/momc.py` is the implementation,
pure Python; `tests/momc` compares it with Xcode's own output for 106 models
(100 of their `VersionInfo.plist`s), and refuses 7 more. Generated classes (`momc --action generate`) are Xcode's
but for their header's dated line, with `--config=relax_momc`; mapping models
(`mapc`) aren't reproduced.

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

### Predicates

Fetch requests' predicates (`predicateString`) and fetched properties' are
archived as Foundation's NSPredicate and NSExpression trees, from a parser of
NSPredicate's format as far as it's established:

- **Comparisons:** `==` (also `=`) and `!=` (`<>`) are an
  `NSEqualityPredicateOperator` (type 4 and 5, `NSNegate`); `<`, `<=` (`=<`),
  `>`, `>=` (`=>`) an `NSComparisonPredicateOperator` (0 to 3, its variant
  the same); `BEGINSWITH` and `ENDSWITH` an `NSSubstringPredicateOperator` (8
  and 9, position 0 and 1); `CONTAINS` and `IN` an `NSInPredicateOperator`
  (99 and 10); `LIKE` an `NSLikePredicateOperator` (7), `MATCHES` an
  `NSMatchingPredicateOperator` (6), `BETWEEN` an
  `NSBetweenPredicateOperator` (100). `[c]`, `[d]` and `[n]` are options 1,
  2 and 4 (`NSOptions` for equality, `NSFlags` for strings). `ANY` and
  `SOME` are modifier 2, `ALL` 1; `NONE` is `NOT` of `ANY`.
- **Compounds:** `NOT` (`!`), `AND` (`&&`) and `OR` (`||`) are an
  `NSCompoundPredicate` of type 0, 1 and 2. AND binds tighter than OR; a run
  of ANDs, or of ORs, is one compound however it's parenthesized, and
  `NOT NOT p` is `p`. `TRUEPREDICATE` and `FALSEPREDICATE` are one object
  each in a model.
- **Expressions:** a key path of SELF is an `NSKeyPathExpression` (type 3)
  by `valueForKey:` for one key and `valueForKeyPath:` for several
  (`SELF.` dropped); of a variable (`$FETCH_SOURCE.name`), type 4. `$X` is
  an `NSVariableExpression`, `{…}` an `NSAggregateExpression`, constants an
  `NSConstantValueExpression`: integers 32-bit where they fit, decimals and
  exponents doubles, `YES`/`NO` Booleans, `nil` a class name of `$null` and
  no value.
- **Strings the parser makes** (key paths, constants, variable names) are one
  object for their value where Foundation would hold them in a pointer (a
  tagged pointer string, as published: up to 7 ASCII characters, 9 of a
  6-bit and 11 of a 5-bit alphabet), else an object each
  (`parent.parent.n`); the empty string is one object of its own. Several
  operator and expression classes' names are new strings each time.
- **Fetched properties** are `NSFetchedPropertyDescription`s, transient and
  read-only (`_P` 4 and 8), naming their request's entity, whose request
  refers to it conditionally; they follow the relationships.
- **Derived attributes** are `NSDerivedAttributeDescription`s, read-only,
  with an `NSDerivationExpression`: a key path; `now()`; `canonical:`,
  `uppercase:` and `lowercase:` of a key path, as `NSFunctionExpression`s
  of `_NSPredicateUtilities`; and a to-many key path's `@count` or `@sum`,
  as `count:` and `sum:` of the key path before it.
- Refused: other syntax (`SUBQUERY`, functions, arithmetic, `ANYKEY`,
  casts), other derivations (`@avg`, …), and derived attributes with user
  info, a modifier or other settings.

1,080 random predicates in 90 models of up to three entities, with fetched
properties and derived attributes among them, compiled as Xcode does.

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

- predicates and derivations beyond those above;
- composite attributes, fetch indexes;
- fetch request options other than a limit and a batch size;
- a userInfo key given twice;
- names other than ASCII, which Xcode orders by a collation of its own;
- an Xcode other than 27.2 unless `--config=relax_xcode_version`, since
  every model records momc's version (27.2's is 25436) and Xcode's.

## VersionInfo.plist

`VersionInfo.plist` holds the current version's name (for a versioned
model), each entity's version hash (`NSManagedObjectModel_VersionHashes`,
32 bytes) and a checksum of each version (`NSManagedObjectModel_VersionChecksums`,
a base64 SHA-256).

What Core Data hashes was read in a process of our own
(`tools/coredata_hash_probe`): models built with Core Data's public API,
their `versionHash` and `versionChecksum` asked for, and CommonCrypto's
public `CC_SHA256_Init`, `_Update` and `_Final` interposed there
(`DYLD_INSERT_LIBRARIES` on our own program) to log their input. Each hash
is a SHA-256 of:

- **A property, first:** its name (UTF-8), a flags byte (optional 1,
  transient 2, read-only 4: a fetched property's is 7), a zero byte, and
  its version hash modifier if it has one. A fetched property's hash is
  that.
- **An attribute:** that digest, its type as a 32-bit little-endian
  integer, then a byte 1 for external binary storage and another for
  preserving its value in history on deletion. Not its default value,
  renaming identifier, user info, Spotlight indexing, value class or
  transformer, or cloud encryption.
- **A derived attribute:** that again, then a hash of its derivation
  expression's description as NSExpression gives it (`count:(title)`),
  which isn't reproduced: refused unless relaxed.
- **A relationship:** that first digest, the destination entity's name,
  the inverse's name if any, the maximum and minimum counts as 32-bit
  little-endian integers, and the bytes 1 0 if ordered. Not its deletion
  rule.
- **An entity:** its name, a byte 1 if abstract (else 0), its parent's
  hash, the hashes of its properties but transient ones (inherited ones
  too) in byte order of their names, its version hash modifier, and each
  uniqueness constraint, its items joined by dots.
- **A version's checksum:** its entities' hashes, in byte order of their
  names; base64.

A single version's `VersionInfo.plist` is a binary property list of two
CoreFoundation dictionaries, the hashes' and the checksums', each keyed by
the version's name (a string of its own each time). The entities' table is
in the order of the `.mom`'s, filled again from it and copied twice (a table
created from the last one's order each time): all 100 test models' orders,
where filling once fits 98. A versioned model's tables share some names
between them in a way not established, and its current version's `.omo` (a
`momv2` binary Core Data reads instead of the `.mom`) isn't reproduced: it
is refused unless relaxed. Relaxed, its `VersionInfo.plist` holds only the
current version's name, and no `.omo` is written, as for a derived
attribute's model, which Core Data handles the same way: it computes the
hashes from the model itself. On a Mac, a store created with either side's
version 1 opened, migrated lightweight and read with either side's `.momd`
(this one's `VersionInfo.plist`, no `.omo`) exactly as with Xcode's.

A mapping model (`mapc`) records its models' hashes too, which are now
known; it isn't reproduced yet.

## Generated classes

`momc --action generate`, which rules_apple runs for
`apple_core_data_model`, writes Swift classes whose header names the user
building them and the date (`Created by <full name> on 11/4/26.`), so
Xcode's own output varies between machines and days. With
`--config=relax_momc` they're generated as Xcode writes them but for that
line, `//  Created by momc.`; without, refused. `tests/momc/generate` has
nine models and their expected files, 67 models checked in all.

- **Files:** for each entity generated as a class, `<Class>+CoreDataClass.swift`
  and `<Class>+CoreDataProperties.swift`; as a category, the second alone;
  none for an entity without a class or code generation; and
  `<model>+CoreDataModel.swift` when any are written, its typealiases
  prefixed by the model's name's letters alone, accents removed (`Ünï`
  `Uni`; the name decomposed, NFD, as macOS's file system gives it). The
  current version of a versioned model. `--swift-version` changes nothing.
- **Classes:** `@objc(<Class>)` for an unqualified class name (an empty
  line for `App.Class` or `.Class`), the parent entity's class as the
  superclass, else `NSManagedObject`.
- **Properties:** attributes, then relationships, each in the document's
  order; fetched properties left out. Integers, doubles, floats and
  Booleans are scalars unless `usesScalarValueType="NO"` (`NSNumber?`), a
  date only when it says so (`TimeInterval`); every other attribute is
  optional, required or not: `String?`, `NSDecimalNumber?`, `Date?`,
  `Data?`, `UUID?`, `URL?`, a transformable's custom class or `NSObject?`.
  Names are written as they are (`var class`). A to-one relationship is its
  destination's class, optional (`NSManagedObject?` without one), and its
  module imported for `Module.Class`; to-many `NSSet?`, ordered
  `NSOrderedSet?`, with Core Data's four and ten accessors.
- `Identifiable` is declared for a root entity alone.
