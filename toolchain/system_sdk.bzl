"""The Apple SDKs' modules as rules_swift's explicit module targets.

rules_swift's `system_sdk` extension describes an Xcode's modules (each SDK's
Clang modules, Swift modules and cross-import overlays) as targets that
precompile them, which `swift.use_c_modules` needs: with implicit modules off,
the compiler finds no module, not even SwiftShims, unless it is given one.
That extension asks `xcode-select` and `xcrun` for the Xcode, so it has none
off a Mac and no staged tree on one. This rule runs the same scanner
(rules_swift's tools/explicit_modules/scan.py, through
repo_tools/scan_system_sdk.py) on the toolchain repository's staged tree, with
swift.org's swiftc for `xcrun swiftc`, and points the targets at that tree's
files, which every Swift action of the toolchain has as inputs.
"""

_PLATFORMS = [
    "AppleTVOS",
    "AppleTVSimulator",
    "MacOSX",
    "WatchOS",
    "WatchSimulator",
    "XROS",
    "XRSimulator",
    "iPhoneOS",
    "iPhoneSimulator",
]

# Modules left out, as rules_swift's own MODULE.bazel leaves them out of its
# system_sdk (less those it notes are only for Xcodes before 26.4), with the
# scanner's defaults: they don't compile from their interfaces, or import a
# framework their platform lacks.
_EXCLUDED_MODULES = {
    "AppleTVOS": ["MobileGestaltPrivate", "_AppIntents_HealthKit", "_Contacts_AppIntents"],
    "AppleTVSimulator": ["CoreAudio_Private", "MobileGestaltPrivate", "_AppIntents_HealthKit", "_Contacts_AppIntents"],
    "MacOSX": ["MobileGestaltPrivate"],
    "WatchOS": ["BrowserEngineKit", "MobileGestaltPrivate"],
    "WatchSimulator": ["BrowserEngineKit", "CoreAudio_Private", "MobileGestaltPrivate"],
    "XROS": ["AccessoryTransportExtension", "MobileGestaltPrivate"],
    "XRSimulator": ["AccessoryTransportExtension", "MobileGestaltPrivate"],
    "iPhoneOS": ["MobileGestaltPrivate"],
    "iPhoneSimulator": ["CoreAudio_Private", "MobileGestaltPrivate"],
}

# Stand-ins for the two Xcode tools the scanner runs. swiftc is given no
# prebuilt modules: Xcode's are its own compiler's, and some cover only some of
# a staged SDK's architectures (the staged iPhoneOS SDK has arm64 interfaces
# where Xcode built arm64e alone), which the scanner can't describe. Every
# Swift module is then compiled from its interface, by swift_tools' Swift. xcodebuild names the
# directory the paths are written against, which is replaced below.
_XCODEBUILD = """\
#!/bin/sh
echo "Xcode 0"
echo "Build version applecross"
"""

_DEVELOPER_DIR_NAME = "__bazel_developer_dir_0_0_0_applecross"

_XCRUN = """\
#!/bin/sh
# xcrun --sdk <sdk> --show-sdk-path, and xcrun swiftc <args>.
if [ "$1" = "--sdk" ] && [ "$3" = "--show-sdk-path" ]; then
  for platform in {platforms}; do
    lower=$(echo "$platform" | tr '[:upper:]' '[:lower:]')
    if [ "$lower" = "$2" ]; then
      echo "$DEVELOPER_DIR/Platforms/$platform.platform/Developer/SDKs/$platform.sdk"
      exit 0
    fi
  done
  echo "xcrun: unknown SDK $2" >&2
  exit 1
fi
if [ "$1" = "swiftc" ]; then
  shift
  exec "{swiftc}" \\
    -Xfrontend -prebuilt-module-cache-path -Xfrontend "{no_prebuilt_modules}" \\
    -resource-dir "$DEVELOPER_DIR/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift" \\
    "$@"
fi
echo "xcrun: unsupported: $*" >&2
exit 1
"""

def _host_key(rctx):
    if rctx.os.name.startswith("mac"):
        return "macos"
    arch = rctx.os.arch
    if arch in ("aarch64", "arm64"):
        return "aarch64"
    if arch in ("amd64", "x86_64"):
        return "x86_64"
    fail("Unsupported host CPU: " + arch)

def _apple_cross_system_sdk_impl(rctx):
    host = _host_key(rctx)
    swiftc = rctx.attr.swiftc.get(host)
    if not swiftc:
        fail("No Swift for this host ({}) to scan the Apple SDKs with: configure() needs swift_tools{} for it.".format(
            host,
            "_macos" if host == "macos" else "",
        ))
    swiftc = rctx.path(swiftc)
    toolchain = rctx.path(rctx.attr.toolchain).dirname
    developer_dir = toolchain.get_child("Xcode.app").get_child("Contents").get_child("Developer")
    xctoolchain_usr = developer_dir.get_child("Toolchains").get_child("XcodeDefault.xctoolchain").get_child("usr")
    scan = rctx.path(rctx.attr._scan)
    rctx.watch(scan)
    scan_system_sdk = rctx.path(rctx.attr._scan_system_sdk)
    rctx.watch(scan_system_sdk)

    sdks = [
        platform
        for platform in _PLATFORMS
        if developer_dir.get_child("Platforms").get_child(platform + ".platform").get_child("Developer").get_child("SDKs").get_child(platform + ".sdk").exists
    ]
    if not sdks:
        fail("The staged Xcode at {} has no SDKs to scan".format(developer_dir))

    rctx.file("_scan/bin/xcodebuild", _XCODEBUILD, executable = True)
    rctx.file("_scan/no_prebuilt_modules/.keep", "")
    rctx.file("_scan/bin/xcrun", _XCRUN.format(
        no_prebuilt_modules = str(rctx.path("_scan/no_prebuilt_modules")),
        platforms = " ".join(_PLATFORMS),
        swiftc = str(swiftc),
    ), executable = True)

    python = rctx.which("python3")
    if not python:
        fail("python3 is required to scan the Apple SDKs")
    rctx.report_progress("Scanning the Apple SDKs' modules")
    exclude_args = []
    for sdk in sdks:
        modules = {m: True for m in _EXCLUDED_MODULES.get(sdk, []) + rctx.attr.exclude_modules.get(sdk, [])}
        for module in sorted(modules):
            exclude_args.extend(["--exclude-module", "{}:{}".format(sdk, module)])
    result = rctx.execute(
        [
            str(python),
            str(scan_system_sdk),
            str(scan),
            "--output",
            "BUILD.bazel",
            "--developer-dir",
            str(developer_dir),
        ] + exclude_args + sdks,
        environment = {
            # The staged Ubuntu libraries swift.org's driver needs, as the
            # toolchain's own actions find them.
            "LD_LIBRARY_PATH": str(xctoolchain_usr.get_child("lib")),
            "PATH": str(rctx.path("_scan/bin")) + ":/usr/bin:/bin",
        },
        timeout = 1800,
    )
    if result.return_code != 0:
        fail("Scanning the Apple SDKs' modules failed:\n{}{}".format(result.stdout, result.stderr))
    rctx.delete("_scan")

    # The scanner writes paths against a directory rules_swift's worker links
    # to the Xcode; here they name the staged tree, from the execution root.
    staged = "external/{}/Xcode.app/Contents/Developer".format(rctx.attr.toolchain.repo_name)
    build = rctx.read("BUILD.bazel")
    if build.count(_DEVELOPER_DIR_NAME + "/") != build.count(_DEVELOPER_DIR_NAME):
        fail("Unexpected developer directory reference in the scanned modules")
    rctx.file("BUILD.bazel", build.replace(_DEVELOPER_DIR_NAME + "/", staged + "/"))

apple_cross_system_sdk = repository_rule(
    implementation = _apple_cross_system_sdk_impl,
    attrs = {
        "exclude_modules": attr.string_list_dict(
            doc = "Modules not to describe, by SDK (\"iPhoneOS\"), as rules_swift's " +
                  "system_sdk.configure_sdks(exclude_modules), besides those " +
                  "rules_swift leaves out.",
        ),
        "swiftc": attr.string_keyed_label_dict(
            doc = "swift.org's swiftc by host: \"aarch64\" and \"x86_64\" " +
                  "(Linux), \"macos\".",
        ),
        "toolchain": attr.label(
            mandatory = True,
            doc = "The toolchain repository's BUILD file, beside its staged Xcode.app.",
        ),
        "_scan": attr.label(default = "@rules_swift//tools/explicit_modules:scan.py"),
        "_scan_system_sdk": attr.label(default = "//toolchain:repo_tools/scan_system_sdk.py"),
    },
    doc = "rules_swift's explicit module targets for the Apple SDKs of an " +
          "apple_cross_toolchain repository.",
)
