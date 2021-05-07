package(default_visibility = ["//visibility:public"])

load("@bazel_tools//tools/osx/crosstool:osx_archs.bzl", "OSX_TOOLS_ARCHS")
load("@rules_cc//cc:defs.bzl", "cc_toolchain_suite", "cc_library")
load(":cc_toolchain_config.bzl", "cc_toolchain_config")

CC_TOOLCHAINS = [(
    cpu + "|compiler",
    ":cc-compiler-" + cpu,
) for cpu in OSX_TOOLS_ARCHS] + [(
    cpu,
    ":cc-compiler-" + cpu,
) for cpu in OSX_TOOLS_ARCHS] + [
    ("k8|compiler", ":cc-compiler-darwin_x86_64"),
    ("darwin|compiler", ":cc-compiler-darwin_x86_64"),
    ("k8", ":cc-compiler-darwin_x86_64"),
    ("darwin", ":cc-compiler-darwin_x86_64"),
]

cc_library(
    name = "malloc",
)

filegroup(
    name = "empty",
    srcs = [],
)

filegroup(
    name = "cc_wrapper",
    srcs = ["cc_wrapper.sh"],
)

filegroup(
    name = "toolchain_files",
    srcs = glob([
        "Xcode.app/Contents/Developer/Toolchains/usr/bin/**",
        "Xcode.app/Contents/Developer/Platforms/*.platform/Info.plist",  # Needed by xcrun to find and invoke tools
        "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/ToolchainInfo.plist",
        "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/**",
        "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/include/**",
        "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/arc/**",
        "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/clang/**",
        "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/**",
        "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-5.0/**",
    ]),
)

alias(
    name = "swift_executable",
    actual = "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/swiftc",
)

cc_toolchain_suite(
    name = "toolchain",
    toolchains = dict(CC_TOOLCHAINS),
)

[
    filegroup(
        name = "osx_tools_" + arch,
        srcs = [
            ":cc_wrapper",
            ":libtool",
            ":libtool_check_unique",
            ":make_hashed_objlist.py",
            ":toolchain_files",
            ":wrapped_clang",
            ":wrapped_clang_pp",
            ":xcrunwrapper.sh",
        ],
    )
    for arch in OSX_TOOLS_ARCHS
]

[
    apple_cc_toolchain(
        name = "cc-compiler-" + arch,
        all_files = ":osx_tools_" + arch,
        ar_files = ":osx_tools_" + arch,
        as_files = ":osx_tools_" + arch,
        compiler_files = ":osx_tools_" + arch,
        dwp_files = ":empty",
        linker_files = ":osx_tools_" + arch,
        objcopy_files = ":empty",
        strip_files = ":osx_tools_" + arch,
        supports_param_files = 1,
        toolchain_config = ":" + (
            arch if arch != "armeabi-v7a" else "stub_armeabi-v7a"
        ),
        toolchain_identifier = (
            arch if arch != "armeabi-v7a" else "stub_armeabi-v7a"
        ),
    )
    for arch in OSX_TOOLS_ARCHS
]

[
    cc_toolchain_config(
        name = (arch if arch != "armeabi-v7a" else "stub_armeabi-v7a"),
        compiler = "compiler",
        cpu = arch,
        cxx_builtin_include_directories = [
        ],
        tool_paths_overrides = {},
    )
    for arch in OSX_TOOLS_ARCHS
]
