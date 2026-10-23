#!/usr/bin/env python3
import collections
import gzip
import hashlib
import io
import json
import os
import plistlib
import re
import shutil
import stat
import struct
import subprocess
import sys
import tarfile
import tempfile
import urllib.request
import xml.etree.ElementTree as ElementTree
import zlib

XCODE_VERSION = "27.0"
XCODE_BUILD = "27A0"
SDK_VERSION = "27.0"
SDK_BUILD = "27A0"
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUTPUT = os.path.join(ROOT, "apple-sdks.tar.zst")
SWIFT_URL = "https://download.swift.org/swift-{0}-release/xcode/swift-{0}-RELEASE/swift-{0}-RELEASE-osx.pkg"
ARCHIVE_MTIME = 1672560000

Platform = collections.namedtuple(
    "Platform",
    "sdk name family family_name product sys env archs runtime build_id minimum",
)

PLATFORMS = [
    Platform("MacOSX", "macosx", "macosx", "macOS", "macOS", "macos", "", ["x86_64", "arm64", "arm64e"], "osx", 1, "10.13"),
    Platform("iPhoneOS", "iphoneos", "iphoneos", "iOS", "iPhone OS", "ios", "", ["arm64", "arm64e"], "ios", 2, "12.0"),
    Platform("iPhoneSimulator", "iphonesimulator", "iphoneos", "iOS", "iPhone OS", "ios", "simulator", ["arm64", "x86_64"], "iossim", 7, "12.0"),
    Platform("AppleTVOS", "appletvos", "appletvos", "tvOS", "Apple TVOS", "tvos", "", ["arm64", "arm64e"], "tvos", 3, "12.0"),
    Platform("AppleTVSimulator", "appletvsimulator", "appletvos", "tvOS", "Apple TVOS", "tvos", "simulator", ["arm64", "x86_64"], "tvossim", 8, "12.0"),
    Platform("WatchOS", "watchos", "watchos", "watchOS", "Watch OS", "watchos", "", ["arm64_32", "arm64", "arm64e"], "watchos", 4, "5.0"),
    Platform("WatchSimulator", "watchsimulator", "watchos", "watchOS", "Watch OS", "watchos", "simulator", ["arm64", "x86_64"], "watchossim", 9, "5.0"),
    Platform("XROS", "xros", "xros", "xrOS", "xrOS", "xros", "", ["arm64", "arm64e"], "xros", 11, "1.0"),
    Platform("XRSimulator", "xrsimulator", "xros", "xrOS", "xrOS", "xros", "simulator", ["arm64", "x86_64"], "xrossim", 12, "1.0"),
]

LIBRARIES = {
    "libSystem": (
        "/usr/lib/libSystem.B.dylib",
        [
            "___bzero",
            "___error",
            "___memcpy_chk",
            "___memmove_chk",
            "___memset_chk",
            "___stack_chk_fail",
            "___stack_chk_guard",
            "__availability_version_check",
            "__dyld_get_image_header",
            "_abort",
            "_bzero",
            "_calloc",
            "_dispatch_once_f",
            "_dlsym",
            "_exit",
            "_free",
            "_malloc",
            "_malloc_good_size",
            "_malloc_size",
            "_memcmp",
            "_memcpy",
            "_memmove",
            "_memset",
            "_memset_pattern16",
            "_posix_memalign",
            "_realloc",
            "_strcmp",
            "_strlen",
            "_strncmp",
            "_write",
            "dyld_stub_binder",
        ],
        [],
    ),
    "libobjc": (
        "/usr/lib/libobjc.A.dylib",
        [
            "__objc_empty_cache",
            "_objc_alloc",
            "_objc_allocWithZone",
            "_objc_autorelease",
            "_objc_autoreleaseReturnValue",
            "_objc_msgSend",
            "_objc_opt_self",
            "_objc_release",
            "_objc_retain",
            "_objc_retainAutoreleasedReturnValue",
        ],
        ["NSObject"],
    ),
    "libc++": (
        "/usr/lib/libc++.1.dylib",
        [
            "__ZdlPv",
            "__Znwm",
            "___gxx_personality_v0",
        ],
        [],
    ),
}

SKIPPED_SUFFIXES = (
    ".abi.json",
    ".swiftmodule",
    ".package.swiftinterface",
    ".private.swiftinterface",
    ".swiftdoc",
    ".swiftsourceinfo",
)
TOOLCHAIN_SWIFT_ENTRIES = ("Cxx.swiftmodule", "CxxStdlib.swiftmodule")
TOOLCHAIN_SWIFT_SUFFIXES = (".a", ".h", ".modulemap", ".yaml")
TOOLCHAIN_SWIFT_PLACEHOLDERS = ("libLiveExecutionResultsLogger", "libPlaygrounds")
EXPORT_KEYS = ("symbols", "objc-classes", "objc-eh-types", "objc-ivars", "weak-symbols", "thread-local-symbols")
OBJC_PREFIXES = (("_OBJC_CLASS_$_", "objc-classes"), ("_OBJC_EHTYPE_$_", "objc-eh-types"), ("_OBJC_IVAR_$_", "objc-ivars"))
EXPORT_KIND_MASK = 0x03
EXPORT_THREAD_LOCAL = 0x01
EXPORT_WEAK = 0x04
FAT_MAGIC = 0xCAFEBABE
FAT_MAGIC_64 = 0xCAFEBABF
MH_MAGIC = 0xFEEDFACE
MH_MAGIC_64 = 0xFEEDFACF
MH_APP_EXTENSION_SAFE = 0x02000000
LC_SEGMENT = 0x1
LC_ID_DYLIB = 0xD
LC_SEGMENT_64 = 0x19
LC_DYLD_INFO = 0x22
LC_REEXPORT_DYLIB = 0x8000001F
LC_DYLD_INFO_ONLY = 0x80000022
LC_DYLD_EXPORTS_TRIE = 0x80000033
ARCHS = {
    (0x01000007, 3): "x86_64",
    (0x01000007, 8): "x86_64h",
    (0x0100000C, 0): "arm64",
    (0x0100000C, 1): "arm64",
    (0x0100000C, 2): "arm64e",
    (0x0200000C, 1): "arm64_32",
}


class Region(io.RawIOBase):
    def __init__(self, f, offset, length):
        self.f = f
        self.position = offset
        self.end = offset + length

    def readable(self):
        return True

    def readinto(self, buffer):
        size = min(len(buffer), self.end - self.position)
        if size <= 0:
            return 0
        self.f.seek(self.position)
        data = self.f.read(size)
        buffer[: len(data)] = data
        self.position += len(data)
        return len(data)


class Inflater(io.RawIOBase):
    def __init__(self, raw):
        self.raw = raw
        self.decompressor = zlib.decompressobj()
        self.pending = b""

    def readable(self):
        return True

    def readinto(self, buffer):
        while not self.pending:
            chunk = self.raw.read(1 << 20)
            if not chunk:
                self.pending = self.decompressor.flush()
                break
            self.pending = self.decompressor.decompress(chunk)
        size = min(len(buffer), len(self.pending))
        buffer[:size] = self.pending[:size]
        self.pending = self.pending[size:]
        return size


def xar_entries(f):
    magic, header_size, _, toc_compressed, _, _ = struct.unpack(">4sHHQQI", f.read(28))
    if magic != b"xar!":
        sys.exit("not a xar archive")
    f.seek(header_size)
    toc = ElementTree.fromstring(zlib.decompress(f.read(toc_compressed)))
    heap = header_size + toc_compressed
    entries = {}

    def walk(element, prefix):
        for child in element.findall("file"):
            path = prefix + child.findtext("name")
            data = child.find("data")
            if data is not None:
                entries[path] = (
                    heap + int(data.findtext("offset")),
                    int(data.findtext("length")),
                    data.find("encoding").get("style"),
                )
            walk(child, path + "/")

    walk(toc.find("toc"), "")
    return entries


def read_exact(stream, size):
    chunks = []
    while size:
        chunk = stream.read(min(size, 1 << 20))
        if not chunk:
            sys.exit("truncated cpio archive")
        chunks.append(chunk)
        size -= len(chunk)
    return b"".join(chunks)


def skip(stream, size):
    while size:
        chunk = stream.read(min(size, 1 << 20))
        if not chunk:
            sys.exit("truncated cpio archive")
        size -= len(chunk)


def cpio_entries(stream):
    while True:
        magic = read_exact(stream, 6)
        if magic == b"070707":
            header = read_exact(stream, 70)
            ino = (int(header[0:6], 8), int(header[6:12], 8))
            mode = int(header[12:18], 8)
            namesize = int(header[53:59], 8)
            filesize = int(header[59:70], 8)
            name = read_exact(stream, namesize)[:-1].decode()
            name_pad = data_pad = 0
        elif magic in (b"070701", b"070702"):
            header = read_exact(stream, 104)
            fields = [int(header[i : i + 8], 16) for i in range(0, 104, 8)]
            ino = (fields[7], fields[8], fields[0])
            mode = fields[1]
            filesize = fields[6]
            namesize = fields[11]
            name = read_exact(stream, namesize)[:-1].decode()
            name_pad = (4 - (110 + namesize) % 4) % 4
            data_pad = (4 - filesize % 4) % 4
            skip(stream, name_pad)
        else:
            sys.exit("unsupported cpio format")
        if name == "TRAILER!!!":
            return
        yield name, mode, ino, filesize
        skip(stream, data_pad)


def wanted(path):
    parts = path.split("/")
    if parts[-1].startswith("._") or path.endswith(SKIPPED_SUFFIXES) or len(parts) < 4 or parts[:2] != ["usr", "lib"]:
        return False
    if parts[2] == "swift":
        return parts[3] in ("clang", "shims") or any(parts[3] == p.name for p in PLATFORMS)
    if parts[2] == "clang" and len(parts) >= 6:
        if parts[4] == "include":
            return True
        return parts[4:6] == ["lib", "darwin"] and len(parts) == 7 and not path.endswith(".dylib")
    return False


def extract_payload(pkg, destination):
    with open(pkg, "rb") as f:
        payloads = [entry for path, entry in xar_entries(f).items() if path.endswith(".pkg/Payload")]
        if not payloads:
            sys.exit(pkg + " has no package payload")
        for offset, length, encoding in payloads:
            raw = io.BufferedReader(Region(f, offset, length), 1 << 20)
            if encoding == "application/x-gzip":
                raw = io.BufferedReader(Inflater(raw), 1 << 20)
            elif encoding != "application/octet-stream":
                sys.exit("unsupported payload encoding " + encoding)
            if raw.peek(2)[:2] != b"\x1f\x8b":
                sys.exit("unsupported payload compression")
            extract_cpio(gzip.GzipFile(fileobj=raw), destination)


def extract_cpio(stream, destination):
    links = {}
    for name, mode, ino, size in cpio_entries(stream):
        path = os.path.normpath(name)
        if not wanted(path):
            skip(stream, size)
            continue
        target = os.path.join(destination, path)
        if stat.S_ISDIR(mode):
            os.makedirs(target, exist_ok=True)
            skip(stream, size)
            continue
        os.makedirs(os.path.dirname(target), exist_ok=True)
        if stat.S_ISLNK(mode):
            os.symlink(read_exact(stream, size).decode(), target)
        elif stat.S_ISREG(mode):
            if size == 0 and ino in links:
                shutil.copyfile(links[ino], target)
            else:
                with open(target, "wb") as out:
                    remaining = size
                    while remaining:
                        chunk = stream.read(min(remaining, 1 << 20))
                        if not chunk:
                            sys.exit("truncated cpio archive")
                        out.write(chunk)
                        remaining -= len(chunk)
                if size:
                    links[ino] = target
            os.chmod(target, (mode & 0o755) | 0o644)
        else:
            skip(stream, size)


def write_plist(path, value):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb") as f:
        plistlib.dump(value, f, sort_keys=True)


def write_json(path, value):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        json.dump(value, f, indent=2, sort_keys=True)
        f.write("\n")


def target_names(platform, archs):
    suffix = "-simulator" if platform.env else ""
    return ["{}-{}{}".format(arch, platform.sys, suffix) for arch in archs]


def yaml_list(values, quote=False):
    return "[ {} ]".format(", ".join("'{}'".format(v.replace("'", "''")) if quote else v for v in values))


def write_tbd(path, targets, install_name, exports, versions=None, reexports=(), flags=(), swift_abi=0):
    lines = ["--- !tapi-tbd", "tbd-version: 4", "targets: " + yaml_list(targets)]
    if flags:
        lines.append("flags: " + yaml_list(flags))
    lines.append("install-name: '{}'".format(install_name))
    if versions:
        lines += ["current-version: " + versions[0], "compatibility-version: " + versions[1]]
    if swift_abi:
        lines.append("swift-abi-version: {}".format(swift_abi))
    if reexports:
        lines += ["reexported-libraries:", "  - targets: " + yaml_list(targets), "    libraries: " + yaml_list(reexports, True)]
    exports = [(entry_targets, groups) for entry_targets, groups in exports if any(groups.values())]
    if exports:
        lines.append("exports:")
    for entry_targets, groups in exports:
        lines.append("  - targets: " + yaml_list(entry_targets))
        for key in EXPORT_KEYS:
            if groups.get(key):
                lines.append("    {}: {}".format(key, yaml_list(sorted(groups[key]), True)))
    lines.append("...")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def uleb128(data, offset):
    value = shift = 0
    while True:
        byte = data[offset]
        offset += 1
        value |= (byte & 0x7F) << shift
        shift += 7
        if byte < 0x80:
            return value, offset


def trie_symbols(trie):
    symbols = []
    pending = [(0, b"")] if trie else []
    while pending:
        node, prefix = pending.pop()
        size, offset = uleb128(trie, node)
        if size:
            symbols.append((prefix.decode(), uleb128(trie, offset)[0]))
        offset += size
        count = trie[offset]
        offset += 1
        for _ in range(count):
            end = trie.index(b"\0", offset)
            edge = trie[offset:end]
            child, offset = uleb128(trie, end + 1)
            pending.append((child, prefix + edge))
    return symbols


def export_groups(trie):
    groups = {key: [] for key in EXPORT_KEYS}
    for name, flags in trie_symbols(trie):
        if name.startswith("_OBJC_METACLASS_$_"):
            continue
        for prefix, key in OBJC_PREFIXES:
            if name.startswith(prefix):
                groups[key].append(name[len(prefix) :])
                break
        else:
            if flags & EXPORT_WEAK:
                groups["weak-symbols"].append(name)
            elif flags & EXPORT_KIND_MASK == EXPORT_THREAD_LOCAL:
                groups["thread-local-symbols"].append(name)
            else:
                groups["symbols"].append(name)
    return {key: sorted(values) for key, values in groups.items()}


def packed_version(value):
    return "{}.{}.{}".format(value >> 16, (value >> 8) & 0xFF, value & 0xFF)


def macho_images(data):
    magic = struct.unpack_from(">I", data)[0]
    if magic not in (FAT_MAGIC, FAT_MAGIC_64):
        return [data]
    images = []
    for index in range(struct.unpack_from(">I", data, 4)[0]):
        if magic == FAT_MAGIC_64:
            _, _, offset, size = struct.unpack_from(">iiQQ", data, 8 + index * 32)
        else:
            _, _, offset, size = struct.unpack_from(">iiII", data, 8 + index * 20)
        images.append(data[offset : offset + size])
    return images


def read_dylib(path):
    with open(path, "rb") as f:
        data = f.read()
    dylibs = []
    for image in macho_images(data):
        magic, cputype, cpusubtype, _, ncmds, _, flags = struct.unpack_from("<IiIIIII", image)
        if magic not in (MH_MAGIC, MH_MAGIC_64):
            sys.exit(path + " is not a Mach-O file")
        arch = ARCHS.get((cputype, cpusubtype & 0xFFFFFF))
        if not arch:
            continue
        dylib = {"arch": arch, "reexports": [], "safe": bool(flags & MH_APP_EXTENSION_SAFE), "swift_abi": 0, "trie": b""}
        offset = 32 if magic == MH_MAGIC_64 else 28
        for _ in range(ncmds):
            cmd, size = struct.unpack_from("<II", image, offset)
            if cmd in (LC_ID_DYLIB, LC_REEXPORT_DYLIB):
                name_offset, _, current, compatibility = struct.unpack_from("<IIII", image, offset + 8)
                name = image[offset + name_offset : offset + size].split(b"\0", 1)[0].decode()
                if cmd == LC_ID_DYLIB:
                    dylib["install_name"] = name
                    dylib["versions"] = (packed_version(current), packed_version(compatibility))
                else:
                    dylib["reexports"].append(name)
            elif cmd in (LC_DYLD_INFO, LC_DYLD_INFO_ONLY):
                start, length = struct.unpack_from("<II", image, offset + 40)
                dylib["trie"] = image[start : start + length]
            elif cmd == LC_DYLD_EXPORTS_TRIE:
                start, length = struct.unpack_from("<II", image, offset + 8)
                dylib["trie"] = image[start : start + length]
            elif cmd in (LC_SEGMENT, LC_SEGMENT_64):
                wide = cmd == LC_SEGMENT_64
                section = offset + (72 if wide else 56)
                for _ in range(struct.unpack_from("<I", image, offset + (64 if wide else 48))[0]):
                    if image[section : section + 16].rstrip(b"\0") == b"__objc_imageinfo":
                        start = struct.unpack_from("<I", image, section + (48 if wide else 40))[0]
                        dylib["swift_abi"] = (struct.unpack_from("<I", image, start + 4)[0] >> 8) & 0xFF
                    section += 80 if wide else 68
            offset += size
        dylibs.append(dylib)
    if not dylibs:
        sys.exit(path + " has no slice for a known architecture")
    return dylibs


def version_map(platform):
    if platform.sys != "xros":
        return {}
    pairs = [("17.1", "1.0"), ("18.0", "2.0"), ("26.0", "26.0"), (SDK_VERSION, SDK_VERSION)]
    forward = {ios: xros for ios, xros in pairs}
    backward = {xros: ios for ios, xros in pairs}
    return {
        "VersionMap": {
            "iOS_visionOS": forward,
            "iOS_xrOS": forward,
            "visionOS_iOS": backward,
            "xrOS_iOS": backward,
        },
    }


def sdk_settings(platform):
    display = ("Simulator - " if platform.env else "") + "{} {}".format(platform.family_name, SDK_VERSION)
    target = {
        "Archs": platform.archs,
        "BuildVersionPlatformID": str(platform.build_id),
        "ClangRuntimeLibraryPlatformName": platform.runtime,
        "DefaultDeploymentTarget": SDK_VERSION,
        "LLVMTargetTripleEnvironment": platform.env,
        "LLVMTargetTripleSys": platform.sys,
        "LLVMTargetTripleVendor": "apple",
        "MaximumDeploymentTarget": SDK_VERSION + ".99",
        "MinimumDeploymentTarget": platform.minimum,
        "PlatformFamilyName": platform.family_name,
        "SystemPrefix": "",
    }
    return dict(version_map(platform), **{
        "CanonicalName": platform.name + SDK_VERSION,
        "CustomProperties": {},
        "DefaultDeploymentTarget": SDK_VERSION,
        "DefaultProperties": {"PLATFORM_NAME": platform.name},
        "DisplayName": display,
        "IsBaseSDK": "YES",
        "MaximumDeploymentTarget": SDK_VERSION + ".99",
        "MinimalDisplayName": SDK_VERSION,
        "SupportedTargets": {platform.name: target},
        "Version": SDK_VERSION,
    })


def platform_info(platform):
    return {
        "AdditionalInfo": {
            "CFBundleSupportedPlatforms": [platform.sdk],
            "DTPlatformName": platform.name,
            "DTPlatformVersion": SDK_VERSION,
        },
        "CFBundleIdentifier": "com.apple.platform." + platform.name,
        "CFBundleShortVersionString": SDK_VERSION,
        "CFBundleVersion": SDK_VERSION,
        "DefaultProperties": {
            "DEFAULT_COMPILER": "com.apple.compilers.llvm.clang.1_0",
            "PLATFORM_NAME": platform.name,
        },
        "Description": platform.family_name + (" Simulator" if platform.env else ""),
        "FamilyIdentifier": platform.family,
        "FamilyName": platform.family_name,
        "Identifier": "com.apple.platform." + platform.name,
        "Name": platform.name,
        "Type": "Platform",
        "Version": SDK_VERSION,
    }


def stubify(source, destination, platform):
    if os.path.islink(source):
        os.symlink(os.readlink(source)[: -len(".dylib")] + ".tbd", destination)
        return
    dylibs = read_dylib(source)
    targets = target_names(platform, [dylib["arch"] for dylib in dylibs])
    exports = []
    for target, dylib in zip(targets, dylibs):
        groups = export_groups(dylib["trie"])
        for entry_targets, entry_groups in exports:
            if entry_groups == groups:
                entry_targets.append(target)
                break
        else:
            exports.append(([target], groups))
    first = dylibs[0]
    flags = [] if first["safe"] else ["not_app_extension_safe"]
    write_tbd(
        destination,
        targets,
        first["install_name"],
        exports,
        first["versions"],
        first["reexports"],
        flags,
        first["swift_abi"],
    )


def runtime_module(name, entries):
    library = "Core" if name == "Swift" else name
    return "libswift{}.dylib".format(library) in entries


def place_swift(platform, source, sdk_swift, toolchain_swift):
    os.makedirs(sdk_swift, exist_ok=True)
    os.makedirs(toolchain_swift, exist_ok=True)
    entries = set(os.listdir(source))
    for entry in sorted(entries):
        path = os.path.join(source, entry)
        if entry.startswith("libswift") and entry.endswith(".dylib"):
            stubify(path, os.path.join(sdk_swift, entry[: -len(".dylib")] + ".tbd"), platform)
        elif entry.endswith(".swiftmodule") and runtime_module(entry[: -len(".swiftmodule")], entries):
            shutil.copytree(path, os.path.join(sdk_swift, entry), symlinks=True)
        elif entry in TOOLCHAIN_SWIFT_ENTRIES or entry.endswith(TOOLCHAIN_SWIFT_SUFFIXES):
            if os.path.isdir(path):
                shutil.copytree(path, os.path.join(toolchain_swift, entry), symlinks=True)
            else:
                shutil.copy2(path, os.path.join(toolchain_swift, entry), follow_symlinks=False)


def assemble(extracted, xcode_app):
    developer = os.path.join(xcode_app, "Contents", "Developer")
    toolchain = os.path.join(developer, "Toolchains", "XcodeDefault.xctoolchain")
    swift = os.path.join(extracted, "usr", "lib", "swift")
    write_plist(
        os.path.join(xcode_app, "Contents", "version.plist"),
        {
            "CFBundleShortVersionString": XCODE_VERSION,
            "CFBundleVersion": XCODE_VERSION,
            "ProductBuildVersion": XCODE_BUILD,
        },
    )
    write_plist(
        os.path.join(toolchain, "ToolchainInfo.plist"),
        {"Identifier": "com.apple.dt.toolchain.XcodeDefault"},
    )
    for directory in ("usr/bin", "usr/include", "usr/lib/swift"):
        os.makedirs(os.path.join(toolchain, directory), exist_ok=True)
    shutil.copytree(
        os.path.join(extracted, "usr", "lib", "clang"),
        os.path.join(toolchain, "usr", "lib", "clang"),
        symlinks=True,
    )
    clang = os.path.join(swift, "clang")
    if os.path.islink(clang):
        os.symlink(os.readlink(clang), os.path.join(toolchain, "usr", "lib", "swift", "clang"))
    for platform in PLATFORMS:
        platform_dir = os.path.join(developer, "Platforms", platform.sdk + ".platform")
        sdk = os.path.join(platform_dir, "Developer", "SDKs", platform.sdk + ".sdk")
        write_plist(os.path.join(platform_dir, "Info.plist"), platform_info(platform))
        settings = sdk_settings(platform)
        write_json(os.path.join(sdk, "SDKSettings.json"), settings)
        write_plist(os.path.join(sdk, "SDKSettings.plist"), settings)
        write_plist(
            os.path.join(sdk, "System", "Library", "CoreServices", "SystemVersion.plist"),
            {
                "ProductBuildVersion": SDK_BUILD,
                "ProductName": platform.product,
                "ProductVersion": SDK_VERSION,
            },
        )
        for directory in (
            os.path.join(sdk, "System", "Library", "Frameworks"),
            os.path.join(sdk, "usr", "include"),
            os.path.join(sdk, "usr", "lib"),
            os.path.join(platform_dir, "Developer", "Library", "Frameworks"),
            os.path.join(platform_dir, "Developer", "Library", "PrivateFrameworks"),
            os.path.join(platform_dir, "Developer", "usr", "lib"),
            os.path.join(platform_dir, "usr", "lib"),
        ):
            os.makedirs(directory, exist_ok=True)
        targets = target_names(platform, platform.archs)
        for name, (install_name, symbols, classes) in LIBRARIES.items():
            exports = [(targets, {"symbols": symbols, "objc-classes": classes})]
            write_tbd(os.path.join(sdk, "usr", "lib", name + ".tbd"), targets, install_name, exports)
        for name in ["Foundation"] if platform.name == "macosx" else ["Foundation", "UIKit"]:
            framework = os.path.join(sdk, "System", "Library", "Frameworks", name + ".framework")
            os.makedirs(framework, exist_ok=True)
            install_name = "/System/Library/Frameworks/{0}.framework/{1}{0}".format(
                name,
                "Versions/C/" if platform.name == "macosx" else "",
            )
            write_tbd(os.path.join(framework, name + ".tbd"), targets, install_name, [])
        sdk_swift = os.path.join(sdk, "usr", "lib", "swift")
        place_swift(
            platform,
            os.path.join(swift, platform.name),
            sdk_swift,
            os.path.join(toolchain, "usr", "lib", "swift", platform.name),
        )
        for name in TOOLCHAIN_SWIFT_PLACEHOLDERS:
            path = os.path.join(toolchain, "usr", "lib", "swift", platform.name, name + ".tbd")
            write_tbd(path, targets, "/usr/lib/swift/{}.dylib".format(name), [])
        shutil.copytree(os.path.join(swift, "shims"), os.path.join(sdk_swift, "shims"))


def swift_release():
    with open(os.path.join(ROOT, "MODULE.bazel")) as f:
        tag = re.search(r"apple_cross_toolchain\.swift\((.*?)\n\)", f.read(), re.S)
    version = tag and re.search(r'swift_version = "([^"]+)"', tag.group(1))
    sha256 = tag and re.search(r'"xcode": "([0-9a-f]{64})"', tag.group(1))
    if not version or not sha256:
        sys.exit("MODULE.bazel has no apple_cross_toolchain.swift() with an \"xcode\" sha256")
    return version.group(1), sha256.group(1)


def verify(path, sha256):
    digest = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            digest.update(chunk)
    if digest.hexdigest() != sha256:
        sys.exit("{} does not have MODULE.bazel's sha256 {}".format(path, sha256))


def normalize(info):
    info.uid = info.gid = 0
    info.uname = info.gname = ""
    info.mtime = ARCHIVE_MTIME
    return info


def archive(source, destination):
    with open(destination, "wb") as out:
        zstd = subprocess.Popen(["zstd", "-q", "-T0"], stdin=subprocess.PIPE, stdout=out)
        with tarfile.open(fileobj=zstd.stdin, mode="w|") as tar:
            tar.add(source, os.path.basename(source), filter=normalize)
        zstd.stdin.close()
        if zstd.wait():
            sys.exit("zstd failed")


def main():
    if len(sys.argv) > 2:
        sys.exit("usage: tools/stub-sdks.py [swift.org macOS toolchain .pkg]")
    if os.path.exists(OUTPUT):
        sys.exit(OUTPUT + " exists; move it away first")
    version, sha256 = swift_release()
    with tempfile.TemporaryDirectory() as work:
        pkg = sys.argv[1] if len(sys.argv) == 2 else os.path.join(work, "swift.pkg")
        if len(sys.argv) == 1:
            with urllib.request.urlopen(SWIFT_URL.format(version)) as response, open(pkg, "wb") as out:
                shutil.copyfileobj(response, out, 1 << 20)
        verify(pkg, sha256)
        extracted = os.path.join(work, "swift")
        extract_payload(pkg, extracted)
        assemble(extracted, os.path.join(work, "Xcode.app"))
        partial = OUTPUT + ".partial"
        try:
            archive(os.path.join(work, "Xcode.app"), partial)
            os.replace(partial, OUTPUT)
        finally:
            if os.path.exists(partial):
                os.remove(partial)


if __name__ == "__main__":
    main()
