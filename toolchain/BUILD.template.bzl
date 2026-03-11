package(default_visibility = ["//visibility:public"])

load("@rules_cc//cc:defs.bzl", "cc_toolchain", "cc_toolchain_suite", "cc_library")
load("@build_bazel_apple_support//configs:platforms.bzl", "APPLE_PLATFORMS_CONSTRAINTS")
load(":cc_toolchain_config.bzl", "cc_toolchain_config")
load(":swift_toolchain.bzl", "swift_toolchain")
load("@build_bazel_rules_swift//swift/toolchains:swift_toolchain.bzl", linux_swift_toolchain = "swift_toolchain")

exports_files([
    "wrapped_clang",
    "wrapped_clang_pp",
    "cc_wrapper.sh",
    "libtool",
    "xcrunwrapper.sh",
])

_APPLE_ARCHS = APPLE_PLATFORMS_CONSTRAINTS.keys()

CC_TOOLCHAINS = [(
    cpu + "|clang",
    ":cc-compiler-" + cpu,
) for cpu in _APPLE_ARCHS] + [(
    cpu,
    ":cc-compiler-" + cpu,
) for cpu in _APPLE_ARCHS] + [
    ("k8|clang", ":cc-compiler-darwin_x86_64"),
    ("darwin|clang", ":cc-compiler-darwin_x86_64"),
    ("k8", ":cc-compiler-darwin_x86_64"),
    ("darwin", ":cc-compiler-darwin_x86_64"),
]

cc_library(
    name = "link_extra_lib",
)

cc_library(
    name = "malloc",
)

filegroup(
    name = "empty",
    srcs = [],
)

# Cross-compile: bundled SDK and toolchain files

# Expose the toolchain's bin directory (xcrun, PlistBuddy, plutil, sw_vers, etc.)
filegroup(
    name = "ported_tools",
    srcs = glob(["Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/*"]),
)

# Minimal set of SDK files needed by rules_apple actions (environment_plist,
# plisttool, etc.) in sandboxed and remote execution.  Includes the ported
# tools plus platform/SDK metadata plists — NOT the full SDK.
filegroup(
    name = "sdk_tool_files",
    srcs = glob(
        include = [
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/*",
            "Xcode.app/Contents/Developer/Platforms/*.platform/Info.plist",
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/SDKs/*.sdk/SDKSettings.json",
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/SDKs/*.sdk/SDKSettings.plist",
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/SDKs/*.sdk/System/Library/CoreServices/SystemVersion.plist",
            "Xcode.app/Contents/version.plist",
        ],
        allow_empty = True,
    ),
)

filegroup(
    name = "toolchain_files",
    srcs = glob(
        include = [
            "Xcode.app/Contents/Developer/Toolchains/usr/bin/**",
            "Xcode.app/Contents/Developer/Platforms/*.platform/Info.plist",
            # SDK headers, module maps, and TBD stubs for compilation and linking
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/SDKs/*.sdk/usr/**",
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/SDKs/*.sdk/System/**",
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/SDKs/*.sdk/SDKSettings.json",
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/SDKs/*.sdk/SDKSettings.plist",
            # Platform developer frameworks and libraries
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/Library/**",
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/usr/**",
            "Xcode.app/Contents/Developer/Platforms/*.platform/usr/**",
            # Xcode toolchain files
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/ToolchainInfo.plist",
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/**",
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/include/**",
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/arc/**",
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/clang/**",
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/**",
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-5.0/**",
        ],
        exclude = [
            # Exclude paths with known symlink loops
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/SDKs/*.sdk/System/Library/Frameworks/Ruby.framework/**",
        ],
        allow_empty = True,
    ),
)

cc_toolchain_suite(
    name = "toolchain",
    toolchains = dict(CC_TOOLCHAINS),
)

filegroup(
    name = "tools",
    srcs = [
        ":libtool",
        ":toolchain_files",
        ":wrapped_clang",
        ":wrapped_clang_pp",
        ":xcrunwrapper.sh",
        "cc_wrapper.sh",
    ],
)

[
    cc_toolchain(
        name = "cc-compiler-" + arch,
        all_files = ":tools",
        ar_files = ":tools",
        as_files = ":tools",
        compiler_files = ":tools",
        dwp_files = ":empty",
        linker_files = ":tools",
        objcopy_files = ":empty",
        strip_files = ":tools",
        supports_header_parsing = 1,
        supports_param_files = 1,
        toolchain_config = ":" + arch,
        toolchain_identifier = arch,
    )
    for arch in _APPLE_ARCHS
]

[
    cc_toolchain_config(
        name = arch,
        cpu = arch,
        cxx_builtin_include_directories = [
            "%{cxx_builtin_include_directories}",
        ],
        libtool = ":libtool",
        tool_paths_overrides = {},
    )
    for arch in _APPLE_ARCHS
]

[
    toolchain(
        name = "cc-toolchain-" + arch,
        exec_compatible_with = [
            "@platforms//os:linux",
            "@platforms//cpu:x86_64",
        ],
        target_compatible_with = APPLE_PLATFORMS_CONSTRAINTS[arch],
        toolchain = ":cc-compiler-" + arch,
        toolchain_type = "@bazel_tools//tools/cpp:toolchain_type",
    )
    for arch in _APPLE_ARCHS
]

# Swift toolchains (one per target platform)

[
    swift_toolchain(
        name = "swift-compiler-" + arch,
        cpu = arch,
    )
    for arch in _APPLE_ARCHS
]

[
    toolchain(
        name = "swift-toolchain-" + arch,
        exec_compatible_with = [
            "@platforms//os:linux",
            "@platforms//cpu:x86_64",
        ],
        target_compatible_with = APPLE_PLATFORMS_CONSTRAINTS[arch],
        toolchain = ":swift-compiler-" + arch,
        toolchain_type = "@build_bazel_rules_swift//toolchains:toolchain_type",
    )
    for arch in _APPLE_ARCHS
]

# Linux Swift toolchain (for exec/host Swift compilation)

linux_swift_toolchain(
    name = "swift-compiler-linux_x86_64",
    arch = "x86_64",
    os = "linux",
    root = "%{toolchain_path_prefix}Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr",
    version_file = ":swift_version",
)

toolchain(
    name = "swift-toolchain-linux_x86_64",
    exec_compatible_with = [
        "@platforms//os:linux",
        "@platforms//cpu:x86_64",
    ],
    target_compatible_with = [
        "@platforms//os:linux",
        "@platforms//cpu:x86_64",
    ],
    toolchain = ":swift-compiler-linux_x86_64",
    toolchain_type = "@build_bazel_rules_swift//toolchains:toolchain_type",
)
