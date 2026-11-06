# Core Data's version hashes, probed

`toolchain/stubs/momc.py` writes a model's `VersionInfo.plist` as Xcode's
momc does: each entity's version hash and the model's checksum
(docs/clean-room/momc.md, VersionInfo.plist). What Core Data hashes was
read in a process of our own: `hashes` builds models with Core Data's
public API and asks for their `versionHash` and `versionChecksum`, and
`sha256_log.dylib`, inserted into that process, interposes CommonCrypto's
public `CC_SHA256*` functions to log their input. Nothing Apple ships is
disassembled, decompiled or patched; only our own process's calls are
observed.

    clang -dynamiclib sha256_log.c -o sha256_log.dylib
    clang -fobjc-arc -framework Foundation -framework CoreData hashes.m -o hashes
    DYLD_INSERT_LIBRARIES=$PWD/sha256_log.dylib ./hashes

Each `== label` is followed by the `INIT`, `UPDATE <length> <hex>` and
`FINAL <digest>` of the SHA-256 contexts Core Data uses for it. A compiled
model's hashes can be traced the same way by loading its `.mom` with
`NSManagedObjectModel`'s `initWithContentsOfURL:` and asking for
`entityVersionHashesByName` and `versionChecksum`.
