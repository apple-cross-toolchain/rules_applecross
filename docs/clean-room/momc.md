# momc and mapc (not reproduced)

Core Data models (`.xcdatamodeld`, `.xcdatamodel`) and mapping models
(`.xcmappingmodel`) have no Linux version yet: building a target with one
fails off a Mac. These notes record what black-box probing of Xcode 27.2
found, for whoever takes it further.

## What momc writes

`momc` compiles a model to `<Model>.momd/` holding `<Model>.mom` and
`VersionInfo.plist`, both binary property lists.

- `<Model>.mom` is an `NSKeyedArchiver` archive of the model: an
  `NSManagedObjectModel` (`NSEntities`, `NSFetchRequestTemplates`,
  `NSVersionIdentifiers`, `CDMCompilerToolVersion`,
  `CDMCompilerXcodeVersion`), its `NSEntityDescription`s (class and entity
  names, properties, subentities, superentity, user info, renaming
  identifier, version hash modifier) and their `NSAttributeDescription`s
  and `NSRelationshipDescription`s. Its dictionaries are
  `NSMutableDictionary`s, whose order is CoreFoundation's hash-table
  order, as `crates/cf` reproduces for other plists.
- `VersionInfo.plist` holds each entity's version hash
  (`NSManagedObjectModel_VersionHashes`, 32 bytes) and a checksum of the
  model (`NSManagedObjectModel_VersionChecksums`, a base64 SHA-256).

## Why it stops there

The version hashes are SHA-256 digests whose input isn't established.
Probing `NSAttributeDescription.versionHash` through Core Data's public
API shows that an attribute's hash depends on its name, type, optionality,
transience and version hash modifier (an empty modifier counts as none),
and on one more input that `allowsExternalBinaryDataStorage` and
`preservesValueInHistoryOnDeletion` set identically. It doesn't depend on
the default value, renaming identifier, user info, Spotlight indexing,
value class name, value transformer name or cloud encryption.

These inputs were tried, all without a match:

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

Without the hashes `VersionInfo.plist` can't be written, and a mapping
model (`mapc`) records its models' hashes too. The input could still be
found by more probing, or, in the private fork, by reading the
implementation.

## Generated classes

`momc --action generate`, which rules_apple runs for
`apple_core_data_model`, writes Swift classes that don't depend on the
hashes, but whose header names the user building them and the date
(`Created by <full name> on 11/4/26.`), so Xcode's own output varies
between machines and days.
