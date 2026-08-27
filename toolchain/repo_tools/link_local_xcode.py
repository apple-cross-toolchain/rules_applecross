#!/usr/bin/env python3
"""Build a cross-compilable Xcode tree out of symlinks into a local install.

Selects the same content tools/package-sdks.sh copies, but materialises a real
directory only where something later has to write into the tree; everything else
becomes a symlink into the installed Xcode. The result is a few tens of thousands
of filesystem objects instead of ~140k copied files.

The file set must stay identical to what package-sdks.sh produces. Actions consume
these paths as directory artifacts, so any extra or missing file changes the
directory's digest, and a build from a local Xcode would stop sharing the remote
cache with one from a packaged archive.

Usage: link_local_xcode.py <developer-dir> <destination-Xcode.app>
"""

import fnmatch
import json
import os
import subprocess
import sys

PLATFORMS = [
    "MacOSX",
    "iPhoneOS",
    "iPhoneSimulator",
    "WatchOS",
    "WatchSimulator",
    "AppleTVOS",
    "AppleTVSimulator",
    "XROS",
    "XRSimulator",
]

# Mirrors SDK_EXCLUDES in tools/package-sdks.sh. Applied to the SDK and Developer
# framework trees only; see _INCLUDES for which excludes each subtree gets.
_SDK_EXCLUDE_NAMES = frozenset([
    "share",  # usr/share: documentation, and man pages whose names are not
              # valid Bazel labels
    "_CodeSignature",
    "Ruby.framework",
    "Perl.framework",
    "Python.framework",
    "Python3.framework",
])
_SDK_EXCLUDE_GLOBS = ("*.lproj", "*.swiftdoc", "*.dylib")

# Only Mach-O payloads are dropped from the library directories.
_LIB_EXCLUDE_GLOBS = ("*.dylib",)

_TOOLCHAIN = "Toolchains/XcodeDefault.xctoolchain"


def _sdk_excluded(name, parent_rel):
    if name in _SDK_EXCLUDE_NAMES:
        # `share` is only dropped directly under a `usr` directory.
        return name != "share" or os.path.basename(parent_rel) == "usr"
    return any(fnmatch.fnmatch(name, g) for g in _SDK_EXCLUDE_GLOBS)


def _lib_excluded(name, parent_rel):
    del parent_rel
    return any(fnmatch.fnmatch(name, g) for g in _LIB_EXCLUDE_GLOBS)


def _never_excluded(name, parent_rel):
    del name, parent_rel
    return False


def _includes():
    """(relative path, exclude predicate) pairs, mirroring package-sdks.sh."""
    items = []
    for p in PLATFORMS:
        plat = "Platforms/{}.platform".format(p)
        items.append((plat + "/Info.plist", _never_excluded))
        items.append((plat + "/Developer/SDKs", _sdk_excluded))
        items.append((plat + "/usr/lib", _lib_excluded))
        items.append((plat + "/Developer/usr/lib", _lib_excluded))
        items.append((plat + "/Developer/Library/Frameworks", _sdk_excluded))
        items.append((plat + "/Developer/Library/PrivateFrameworks", _sdk_excluded))
    items.append((_TOOLCHAIN + "/ToolchainInfo.plist", _never_excluded))
    items.append((_TOOLCHAIN + "/usr/include", _never_excluded))
    items.append((_TOOLCHAIN + "/usr/lib/arc", _never_excluded))
    items.append((_TOOLCHAIN + "/usr/lib/clang", _never_excluded))
    items.append((_TOOLCHAIN + "/usr/lib/swift", _lib_excluded))
    items.append((_TOOLCHAIN + "/usr/lib/swift-5.0", _lib_excluded))
    return items


class Linker(object):
    def __init__(self, source, xcode_app):
        self.source = source
        self.xcode_app = xcode_app
        # Everything grafted from the install lands under Contents/Developer,
        # mirroring the layout of the bundle it was read from.
        self.destination = os.path.join(xcode_app, "Contents", "Developer")
        # Directories that must be real because something writes into them.
        self.writable = set()
        # Mach-O binaries replaced by a generated .tbd, so never linked through.
        self.dropped = set()

    # -- tree construction --------------------------------------------------

    def _real_dir(self, rel):
        path = os.path.join(self.destination, rel)
        os.makedirs(path, exist_ok=True)
        return path

    def _link(self, rel):
        """Symlink one source entry into the destination."""
        src = os.path.join(self.source, rel)
        dst = os.path.join(self.destination, rel)
        if os.path.lexists(dst):
            return
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        if os.path.islink(src):
            # Preserve the link itself. Its target is interpreted relative to a
            # directory that exists in both trees, so it keeps resolving.
            os.symlink(os.readlink(src), dst)
        else:
            os.symlink(src, dst)

    def _graft(self, rel, excluded):
        """Recreate one source subtree, symlinking wherever nothing writes."""
        src = os.path.join(self.source, rel)
        if not os.path.lexists(src):
            return
        if not os.path.isdir(src) or os.path.islink(src):
            self._link(rel)
            return
        if not self._contains_writable(rel) and not self._contains_excluded(rel, excluded):
            self._link(rel)
            return
        self._real_dir(rel)
        for name in sorted(os.listdir(src)):
            child = os.path.normpath(os.path.join(rel, name))
            if excluded(name, rel) or child in self.dropped:
                continue
            self._graft(child, excluded)

    def _contains_writable(self, rel):
        prefix = rel + os.sep
        return any(w == rel or w.startswith(prefix) for w in self.writable)

    def _contains_excluded(self, rel, excluded):
        src = os.path.join(self.source, rel)
        for dirpath, dirnames, filenames in os.walk(src, followlinks=False):
            here = os.path.join(rel, os.path.relpath(dirpath, src)) if dirpath != src else rel
            here = os.path.normpath(here)
            for name in list(dirnames) + filenames:
                if excluded(name, here):
                    return True
        return False

    # -- writable-directory discovery ---------------------------------------

    def _mark(self, rel):
        if os.path.isdir(os.path.join(self.source, rel)):
            self.writable.add(os.path.normpath(rel))

    def _mark_stub_targets(self):
        """Directories that gain a .tbd generated from a Mach-O binary."""
        targets = []
        # Framework bundles carry their payload as an extensionless Mach-O file.
        # Packaging stubifies every one it can find anywhere under a .framework
        # directory, including bundles nested inside the SDKs, and removes the
        # binary either way.
        for rel_root, excluded in _includes():
            for dirpath, rel in self._walk(rel_root, excluded):
                if ".framework" + os.sep not in rel + os.sep:
                    continue
                for name in sorted(os.listdir(dirpath)):
                    full = os.path.join(dirpath, name)
                    if "." in name or excluded(name, rel) or os.path.islink(full):
                        continue
                    if not os.path.isfile(full) or not _is_macho(full):
                        continue
                    self._mark(rel)
                    self.dropped.add(os.path.normpath(os.path.join(rel, name)))
                    targets.append((full, os.path.join(rel, name + ".tbd")))
        # Library directories keep only text stubs, so every real .dylib needs
        # one generated. Symlinked .dylibs are skipped, as packaging skips them.
        for p in PLATFORMS:
            plat = "Platforms/{}.platform".format(p)
            for sub in ("usr/lib", "Developer/usr/lib"):
                targets += self._dylib_stubs(os.path.join(plat, sub))
        for sub in ("usr/lib/swift", "usr/lib/swift-5.0"):
            targets += self._dylib_stubs(os.path.join(_TOOLCHAIN, sub))
        return targets

    def _walk(self, rel_root, excluded):
        """Yield (absolute, relative) directories of one include root."""
        root = os.path.join(self.source, rel_root)
        if not os.path.isdir(root) or os.path.islink(root):
            return
        stack = [(root, rel_root)]
        while stack:
            dirpath, rel = stack.pop()
            yield dirpath, rel
            try:
                entries = sorted(os.listdir(dirpath))
            except OSError:
                continue
            for name in entries:
                full = os.path.join(dirpath, name)
                if excluded(name, rel) or os.path.islink(full):
                    continue
                if os.path.isdir(full):
                    stack.append((full, os.path.normpath(os.path.join(rel, name))))

    def _dylib_stubs(self, rel_root):
        targets = []
        root = os.path.join(self.source, rel_root)
        if not os.path.isdir(root):
            return targets
        for dirpath, _, filenames in os.walk(root, followlinks=False):
            for name in filenames:
                full = os.path.join(dirpath, name)
                if not name.endswith(".dylib") or os.path.islink(full):
                    continue
                rel = os.path.relpath(dirpath, self.source)
                tbd = os.path.join(rel, name[: -len(".dylib")] + ".tbd")
                if os.path.exists(os.path.join(self.source, tbd)):
                    continue
                self._mark(rel)
                targets.append((full, tbd))
        return targets

    def _mark_sdk_fixups(self):
        """Directories the repository rule's SDK fixups write into."""
        for p in PLATFORMS:
            sdk = "Platforms/{0}.platform/Developer/SDKs/{0}.sdk".format(p)
            # normalize_sdk_modulemaps.py deletes a duplicate module map here.
            self._mark(os.path.join(sdk, "usr/include/libxml2"))
            # patch_swiftinterfaces.py writes an arm64 interface beside an
            # arm64e-only one.
            frameworks = os.path.join(self.source, sdk, "System/Library/Frameworks")
            for dirpath, dirnames, _ in os.walk(frameworks, followlinks=False):
                if not dirpath.endswith(".swiftmodule"):
                    continue
                dirnames[:] = []
                if os.path.exists(os.path.join(dirpath, "arm64e-apple-ios.swiftinterface")) \
                        and not os.path.exists(os.path.join(dirpath, "arm64-apple-ios.swiftinterface")):
                    self._mark(os.path.relpath(dirpath, self.source))

    # -- entry point --------------------------------------------------------

    def run(self):
        stub_targets = self._mark_stub_targets()
        self._mark_sdk_fixups()
        # The toolchain's usr directory receives the Linux-hosted tools, the
        # prebuilt LLVM, and the clang resource-directory bridge, all of which
        # write into these. They can never point back at the Xcode install:
        # extracting or copying through a symlink would modify the install.
        for sub in ("/usr/include", "/usr/lib", "/usr/lib/clang"):
            self._mark(_TOOLCHAIN + sub)
        self.writable.add(_TOOLCHAIN + "/usr/bin")

        for rel, excluded in _includes():
            self._graft(rel, excluded)

        self._real_dir(_TOOLCHAIN + "/usr/bin")
        _copy_plists(self.source, self.xcode_app)
        _stubify(stub_targets, self.destination)
        self._retarget_stub_aliases()
        return len(stub_targets)

    def _retarget_stub_aliases(self):
        """Point a versioned framework's alias link at the generated stub.

        A macOS framework reaches its payload through Foo -> Versions/Current/Foo.
        Dropping the binary for a stub leaves that link dangling, so it becomes
        Foo.tbd -> Versions/Current/Foo.tbd, matching what packaging produces.
        """
        for dirpath, dirnames, filenames in os.walk(self.destination, followlinks=False):
            dirnames[:] = [d for d in dirnames
                           if not os.path.islink(os.path.join(dirpath, d))]
            for name in list(dirnames) + filenames:
                link = os.path.join(dirpath, name)
                if not os.path.islink(link) or os.path.exists(link):
                    continue
                target = os.readlink(link)
                stub = os.path.join(dirpath, target + ".tbd")
                if os.path.isfile(stub) and not os.path.lexists(link + ".tbd"):
                    os.symlink(target + ".tbd", link + ".tbd")
                os.unlink(link)


def _is_macho(path):
    try:
        with open(path, "rb") as f:
            magic = f.read(4)
    except OSError:
        return False
    return magic in (b"\xca\xfe\xba\xbe", b"\xcf\xfa\xed\xfe",
                     b"\xce\xfa\xed\xfe", b"\xbe\xba\xfe\xca")


def _copy_plists(source, xcode_app):
    for name in ("Info.plist", "version.plist"):
        src = os.path.join(os.path.dirname(source), name)
        dst = os.path.join(xcode_app, "Contents", name)
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        with open(src, "rb") as f:
            data = f.read()
        with open(dst, "wb") as f:
            f.write(data)


def _stubify(targets, destination):
    """Generate the .tbd stubs that no Xcode ships."""
    for source_binary, rel_tbd in targets:
        out = os.path.join(destination, rel_tbd)
        if os.path.exists(out):
            continue
        os.makedirs(os.path.dirname(out), exist_ok=True)
        subprocess.run(
            ["xcrun", "tapi", "stubify", source_binary, "-o", out],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False,
        )
        if _is_developer_framework(rel_tbd):
            _strip_reexports(out)


def _is_developer_framework(rel):
    return (os.sep + "Developer" + os.sep + "Library" + os.sep) in rel and \
        ".framework" + os.sep in rel


def _strip_reexports(path):
    """Drop a stub's re-export chain.

    tapi records that, say, XCTest re-exports XCTestCore, but ld64.lld cannot
    resolve the @rpath reference into PrivateFrameworks. Cross-compilation only
    needs the stub to exist, not the chain.
    """
    try:
        with open(path) as f:
            tbd = json.load(f)
    except (OSError, ValueError):
        return
    changed = False
    for key in ("main_library", "libraries"):
        obj = tbd.get(key)
        items = [obj] if isinstance(obj, dict) else (obj if isinstance(obj, list) else [])
        for item in items:
            if "reexported_libraries" in item:
                del item["reexported_libraries"]
                changed = True
    if changed:
        with open(path, "w") as f:
            json.dump(tbd, f)


def main() -> int:
    if len(sys.argv) != 3:
        print("usage: link_local_xcode.py <developer-dir> <destination-Xcode.app>",
              file=sys.stderr)
        return 2
    source = os.path.realpath(sys.argv[1])
    destination = os.path.abspath(sys.argv[2])

    if not os.path.isdir(os.path.join(source, "Platforms")):
        print("error: {} has no Platforms directory, so it is not a full Xcode "
              "install.".format(source), file=sys.stderr)
        return 1
    if os.path.commonpath([source, destination]) == source:
        print("error: destination {} is inside the source Xcode at {}".format(
            destination, source), file=sys.stderr)
        return 1

    stubs = Linker(source, destination).run()
    print("linked {} from {} ({} stubs generated)".format(
        destination, source, stubs), file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
