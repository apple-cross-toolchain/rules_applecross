load("@apple_support//configs:platforms.bzl", "APPLE_PLATFORMS_CONSTRAINTS")
load("@bazel_features//:features.bzl", "bazel_features")
load("@rules_applecross//toolchain:builtin_include_directory.bzl", "builtin_include_directory")
load("@rules_applecross//toolchain:exec_tool.bzl", "exec_tool")
load("@rules_applecross//toolchain:swift_toolchain.bzl", "swift_toolchain")
load("@rules_cc//cc:defs.bzl", "cc_binary", "cc_library", "cc_toolchain_suite")
load("@rules_cc//cc/toolchains:args.bzl", "cc_args")
load("@rules_cc//cc/toolchains:tool.bzl", "cc_tool")
load("@rules_cc//cc/toolchains:tool_map.bzl", "cc_tool_map")
load("@rules_cc//cc/toolchains:toolchain.bzl", rule_based_cc_toolchain = "cc_toolchain")

package(default_visibility = ["//visibility:public"])

_APPLE_ARCHS = APPLE_PLATFORMS_CONSTRAINTS.keys()

cc_binary(
    name = "_libtool",
    srcs = ["libtool.cc"],
    copts = [
        "-std=c++17",
        "-O3",
    ],
)

exec_tool(
    name = "libtool",
    binary = ":_libtool",
)

cc_binary(
    name = "_wrapped_clang",
    srcs = ["wrapped_clang.cc"],
    copts = [
        "-std=c++11",
        "-O3",
    ],
)

exec_tool(
    name = "wrapped_clang",
    binary = ":_wrapped_clang",
)

exec_tool(
    name = "wrapped_clang_pp",
    binary = ":_wrapped_clang",
)

CC_TOOLCHAINS = [
    (cpu + "|clang", ":cc-compiler-" + cpu)
    for cpu in _APPLE_ARCHS
] + [
    (cpu, ":cc-compiler-" + cpu)
    for cpu in _APPLE_ARCHS
] + [
    ("k8|clang", ":cc-compiler-darwin_x86_64"),
    ("darwin|clang", ":cc-compiler-darwin_x86_64"),
    ("k8", ":cc-compiler-darwin_x86_64"),
    ("darwin", ":cc-compiler-darwin_x86_64"),
]

APPLE_SUPPORT_MARKER_FEATURES = [
    "@apple_support//toolchain:archive_param_file",
    "@apple_support//toolchain:compile_all_modules",
    "@apple_support//toolchain:compiler_param_file",
    "@apple_support//toolchain:compiler_param_file_on_demand",
    "@apple_support//toolchain:exclude_private_headers_in_module_maps",
    "@apple_support//toolchain:gcc_quoting_for_param_files",
    "@apple_support//toolchain:no_dotd_file",
    "@apple_support//toolchain:no_legacy_features",
    "@apple_support//toolchain:only_doth_headers_in_module_maps",
    "@apple_support//toolchain:parse_headers",
    "@apple_support//toolchain:parse_headers_as_c",
    "@apple_support//toolchain:sanitize_pwd",
    "@apple_support//toolchain:set_soname",
]

APPLE_SUPPORT_ENABLED_MARKERS = [
    "@apple_support//toolchain:archive_param_file",
    "@apple_support//toolchain:sanitize_pwd",
    "@apple_support//toolchain:set_soname",
] + (
    ["@apple_support//toolchain:gcc_quoting_for_param_files"] if bazel_features.cc.fixed_dsym_path_quoting else []
)

APPLE_SUPPORT_ENABLED_FEATURES = APPLE_SUPPORT_ENABLED_MARKERS + [
    "@rules_cc//cc/toolchains/args/layering_check:module_maps",
    "@rules_cc//cc/toolchains/args/strip_flags:feature",
    "@apple_support//toolchain/objc:__objc_executable_feature",
    "@apple_support//toolchain/objc:__objc_fully_link_feature",
    "@apple_support//toolchain:__header_parsing_flags",
    "@apple_support//toolchain:link_libc++",
    "@apple_support//toolchain:default_compile_flags_feature",
    "@apple_support//toolchain:ns_block_assertions",
    "@apple_support//toolchain:debug_prefix_map_pwd_is_dot",
    "@apple_support//toolchain:remap_xcode_path",
    "@apple_support//toolchain:generate_dsym_file_wrapper",
    "@apple_support//toolchain:generate_linkmap_wrapper",
    "@apple_support//toolchain:oso_prefix_is_pwd",
    "@apple_support//toolchain:strip_debug_symbols",
    "@rules_cc//cc/toolchains/args/shared_flag:feature",
    "@apple_support//toolchain:kernel_extension_wrapper",
    "@apple_support//toolchain:output_execpath_flags",
    "@rules_cc//cc/toolchains/args/runtime_library_search_directories:feature",
    "@rules_cc//cc/toolchains/args/library_search_directories:feature",
    "@rules_cc//cc/toolchains/args/libraries_to_link:feature",
    "@apple_support//toolchain/objc:objc_link_flag",
    "@apple_support//toolchain:pch",
    "@apple_support//toolchain/objc:__apple_default_warnings",
    "@apple_support//toolchain:__archiver_flags",
    "@rules_cc//cc/toolchains/args/include_flags:feature",
    "@rules_cc//cc/toolchains/args/dependency_file:feature",
    "@apple_support//toolchain:serialized_diagnostics_file_wrapper",
    "@rules_cc//cc/toolchains/args/pic_flags:feature",
    "@rules_cc//cc/toolchains/args/preprocessor_defines:feature",
    "@apple_support//toolchain/pgo:fdo_instrument_wrapper",
    "@apple_support//toolchain/pgo:fdo_optimize_wrapper",
    "@apple_support//toolchain/pgo:autofdo_wrapper",
    "@apple_support//toolchain:lto_object_path",
    "@apple_support//toolchain/coverage:llvm_coverage_map_format_wrapper",
    "@apple_support//toolchain/coverage:gcc_coverage_map_format_wrapper",
    "@apple_support//toolchain/coverage:coverage_prefix_map",
    "@apple_support//toolchain/coverage:_coverage_prefix_map_absolute_sources_non_hermetic_wrapper",
    "@apple_support//toolchain/objc:__apple_default_compiler_flags",
    "@apple_support//toolchain:headerpad",
    "@rules_cc//cc/toolchains/args/objc_arc_flags:feature",
    "@apple_support//toolchain:user_link_flags",
    "@apple_support_toolchain_env//:linkopts_from_env",
    "@apple_support//toolchain:default_required_flags",
    "@apple_support//toolchain:__apply_simulator_compiler_flags",
    "@apple_support//toolchain/sanitizers:asan_wrapper",
    "@apple_support//toolchain/sanitizers:tsan_wrapper",
    "@apple_support//toolchain/sanitizers:ubsan_wrapper",
    "@apple_support//toolchain/sanitizers:default_sanitizer_flags",
    "@apple_support_toolchain_env//:copts_from_env",
    "@apple_support//toolchain:default_link_flags",
] + select({
    "@apple_support//toolchain:opt_mode": [
        "@apple_support//toolchain:dead_strip",
    ],
    "//conditions:default": [],
}) + [
    "@apple_support//toolchain:no_deduplicate",
    "@apple_support//toolchain:function_sections",
    "@apple_support//toolchain:dead_strip_wrapper",
    "@apple_support//toolchain:apply_implicit_frameworks",
    "@apple_support//toolchain:link_cocoa_wrapper",
    "@apple_support//toolchain:extra_enabled_features",
    "@rules_cc//cc/toolchains/args/compile_flags:user_compile_flags_feature",
    "@apple_support//toolchain:unfiltered_compile_flags",
    "@rules_cc//cc/toolchains/args/compiler_input_flags:feature",
    "@rules_cc//cc/toolchains/args/compiler_output_flags:feature",
    "@rules_cc//cc/toolchains/args/linker_param_file:feature",
    "@rules_cc//cc/toolchains/args/soname_flags:feature",
    "@apple_support//toolchain:suppress_warnings_wrapper",
    "@apple_support//toolchain:treat_warnings_as_errors_wrapper",
    "@apple_support//toolchain:external_include_paths_wrapper",
    # no_warn_duplicate_libraries and reproducible_linker_flag are intentionally
    # absent because the Linux-ported Apple linker does not support their flags.
    # NOTE: @apple_support_toolchain_env//:off_by_default_layering_check_enabled_features
    # is intentionally absent: it depends on @apple_support//crosstool:generate_layering_check_modulemap,
    # an apple_genrule that can only execute on a macOS host, which this
    # Linux-only cross toolchain never has.
]

APPLE_SUPPORT_KNOWN_FEATURES = APPLE_SUPPORT_MARKER_FEATURES + [
    "@apple_support//toolchain:opt",
    "@apple_support//toolchain:dbg",
    "@apple_support//toolchain:fastbuild",
    "@apple_support//toolchain/coverage:coverage",
    "@apple_support//toolchain:kernel_extension",
    "@apple_support//toolchain:serialized_diagnostics_file",
    "@apple_support//toolchain/coverage:llvm_coverage_map_format",
    "@apple_support//toolchain/coverage:gcc_coverage_map_format",
    "@apple_support//toolchain/coverage:_coverage_prefix_map_absolute_sources_non_hermetic",
    "@apple_support//toolchain/sanitizers:asan",
    "@apple_support//toolchain/sanitizers:tsan",
    "@apple_support//toolchain/sanitizers:ubsan",
    "@apple_support//toolchain:generate_dsym_file",
    "@apple_support//toolchain:generate_linkmap",
    "@apple_support//toolchain:link_cocoa",
    "@apple_support//toolchain:dead_strip",
    "@apple_support//toolchain:suppress_warnings",
    "@apple_support//toolchain:treat_warnings_as_errors",
    "@apple_support//toolchain:external_include_paths",
    "@apple_support//toolchain/pgo:fdo_instrument",
    "@apple_support//toolchain/pgo:autofdo",
    "@apple_support//toolchain/pgo:fdo_optimize",
    "@apple_support//toolchain:extra_known_features",
    "@rules_cc//cc/toolchains/args/layering_check:use_module_maps",
] + select({
    "@platforms//os:macos": [
        "@apple_support//toolchain:dynamic_linking_mode",
    ],
    "//conditions:default": [],
})

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

_XCODE_DEVELOPER_DIR = "Xcode.app/Contents/Developer"
_XCODE_TOOLCHAIN_DIR = _XCODE_DEVELOPER_DIR + "/Toolchains/XcodeDefault.xctoolchain"
_XCODE_TOOLCHAIN_BIN = _XCODE_TOOLCHAIN_DIR + "/usr/bin"
_XCODE_TOOLCHAIN_LIB = _XCODE_TOOLCHAIN_DIR + "/usr/lib"

filegroup(
    name = "ported_tools",
    srcs = glob([_XCODE_TOOLCHAIN_BIN + "/*"]),
)

filegroup(
    name = "sdk_tool_files",
    srcs = glob(
        include = [
            _XCODE_TOOLCHAIN_BIN + "/*",
            # xcrun refuses to run without a toolchain descriptor.
            _XCODE_TOOLCHAIN_DIR + "/ToolchainInfo.plist",
            _XCODE_DEVELOPER_DIR + "/Platforms/*.platform/Info.plist",
            _XCODE_DEVELOPER_DIR + "/Platforms/*.platform/Developer/SDKs/*.sdk/Library/Application Support/WatchKit/*",
            _XCODE_DEVELOPER_DIR + "/Platforms/*.platform/Developer/SDKs/*.sdk/SDKSettings.json",
            _XCODE_DEVELOPER_DIR + "/Platforms/*.platform/Developer/SDKs/*.sdk/SDKSettings.plist",
            _XCODE_DEVELOPER_DIR + "/Platforms/*.platform/Developer/SDKs/*.sdk/System/Library/CoreServices/SystemVersion.plist",
            _XCODE_DEVELOPER_DIR + "/version.plist",
        ],
        allow_empty = True,
    ),
)

filegroup(
    name = "toolchain_files",
    srcs = [":sdk_tool_files"],
)

filegroup(
    name = "clang_resource_files",
    srcs = [
        _XCODE_TOOLCHAIN_LIB + "/clang",
    ],
)

builtin_include_directory(
    name = "clang_resource_include_directory",
    path = _XCODE_TOOLCHAIN_LIB + "/clang",
)

filegroup(
    name = "clang_tool_files",
    srcs = [
        _XCODE_TOOLCHAIN_BIN + "/clang",
        _XCODE_TOOLCHAIN_BIN + "/llvm",
        ":clang_resource_files",
    ],
)

filegroup(
    name = "clangpp_tool_files",
    srcs = [
        _XCODE_TOOLCHAIN_BIN + "/clang++",
        _XCODE_TOOLCHAIN_BIN + "/llvm",
        ":clang_resource_files",
    ],
)

_SWIFT_COMPATIBILITY_ARCHIVES = glob([
    _XCODE_TOOLCHAIN_LIB + "/swift/iphoneos/libswiftCompatibility*.a",
    _XCODE_TOOLCHAIN_LIB + "/swift/iphonesimulator/libswiftCompatibility*.a",
    _XCODE_TOOLCHAIN_LIB + "/swift/macosx/libswiftCompatibility*.a",
])

filegroup(
    name = "link_tool_files",
    srcs = [
        ":clang_tool_files",
        _XCODE_TOOLCHAIN_BIN + "/dsymutil",
        _XCODE_TOOLCHAIN_BIN + "/ld",
        _XCODE_TOOLCHAIN_BIN + "/ld64.lld",
        _XCODE_TOOLCHAIN_BIN + "/lld",
        _XCODE_TOOLCHAIN_BIN + "/llvm-strip",
        _XCODE_TOOLCHAIN_BIN + "/strip",
        _XCODE_TOOLCHAIN_LIB + "/libtapi.so",
        _XCODE_TOOLCHAIN_LIB + "/libtapi.so.8svn",
    ] + _SWIFT_COMPATIBILITY_ARCHIVES,
)

filegroup(
    name = "libtool_files",
    srcs = [
        _XCODE_TOOLCHAIN_BIN + "/llvm",
        _XCODE_TOOLCHAIN_BIN + "/llvm-libtool-darwin",
    ],
)

_SWIFT_COMMON_TOOLCHAIN_INPUTS = [
    "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/ToolchainInfo.plist",
    "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/include",
    "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib",
]

_SWIFT_PLATFORM_INPUTS = {
    "AppleTVOS": [_XCODE_DEVELOPER_DIR + "/Platforms/AppleTVOS.platform"],
    "AppleTVSimulator": [_XCODE_DEVELOPER_DIR + "/Platforms/AppleTVSimulator.platform"],
    "MacOSX": [_XCODE_DEVELOPER_DIR + "/Platforms/MacOSX.platform"],
    "WatchOS": [_XCODE_DEVELOPER_DIR + "/Platforms/WatchOS.platform"],
    "WatchSimulator": [_XCODE_DEVELOPER_DIR + "/Platforms/WatchSimulator.platform"],
    "XROS": [_XCODE_DEVELOPER_DIR + "/Platforms/XROS.platform"],
    "XRSimulator": [_XCODE_DEVELOPER_DIR + "/Platforms/XRSimulator.platform"],
    "iPhoneOS": [_XCODE_DEVELOPER_DIR + "/Platforms/iPhoneOS.platform"],
    "iPhoneSimulator": [_XCODE_DEVELOPER_DIR + "/Platforms/iPhoneSimulator.platform"],
}

[
    filegroup(
        name = "swift_toolchain_files_" + platform,
        srcs = platform_inputs + _SWIFT_COMMON_TOOLCHAIN_INPUTS,
    )
    for platform, platform_inputs in _SWIFT_PLATFORM_INPUTS.items()
]

_SWIFT_TOOLCHAIN_FILES_BY_ARCH = {
    "darwin_arm64": ":swift_toolchain_files_MacOSX",
    "darwin_arm64e": ":swift_toolchain_files_MacOSX",
    "darwin_x86_64": ":swift_toolchain_files_MacOSX",
    "ios_arm64": ":swift_toolchain_files_iPhoneOS",
    "ios_arm64e": ":swift_toolchain_files_iPhoneOS",
    "ios_sim_arm64": ":swift_toolchain_files_iPhoneSimulator",
    "ios_x86_64": ":swift_toolchain_files_iPhoneSimulator",
    "tvos_arm64": ":swift_toolchain_files_AppleTVOS",
    "tvos_sim_arm64": ":swift_toolchain_files_AppleTVSimulator",
    "tvos_x86_64": ":swift_toolchain_files_AppleTVSimulator",
    "visionos_arm64": ":swift_toolchain_files_XROS",
    "visionos_sim_arm64": ":swift_toolchain_files_XRSimulator",
    "watchos_arm64": ":swift_toolchain_files_WatchSimulator",
    "watchos_arm64_32": ":swift_toolchain_files_WatchOS",
    "watchos_device_arm64": ":swift_toolchain_files_WatchOS",
    "watchos_device_arm64e": ":swift_toolchain_files_WatchOS",
    "watchos_x86_64": ":swift_toolchain_files_WatchSimulator",
}

filegroup(
    name = "swift_toolchain_files",
    srcs = _SWIFT_COMMON_TOOLCHAIN_INPUTS + [
        _XCODE_DEVELOPER_DIR + "/Platforms",
    ],
)

_APPLE_SDK_PLATFORMS_BY_ARCH = {
    "darwin_arm64": "MacOSX",
    "darwin_arm64e": "MacOSX",
    "darwin_x86_64": "MacOSX",
    "ios_arm64": "iPhoneOS",
    "ios_arm64e": "iPhoneOS",
    "ios_sim_arm64": "iPhoneSimulator",
    "ios_x86_64": "iPhoneSimulator",
    "tvos_arm64": "AppleTVOS",
    "tvos_sim_arm64": "AppleTVSimulator",
    "tvos_x86_64": "AppleTVSimulator",
    "visionos_arm64": "XROS",
    "visionos_sim_arm64": "XRSimulator",
    "watchos_arm64": "WatchSimulator",
    "watchos_arm64_32": "WatchOS",
    "watchos_device_arm64": "WatchOS",
    "watchos_device_arm64e": "WatchOS",
    "watchos_x86_64": "WatchSimulator",
}

_APPLE_SDK_PLATFORMS = {
    sdk_platform: None
    for sdk_platform in _APPLE_SDK_PLATFORMS_BY_ARCH.values()
}.keys()

[
    filegroup(
        name = "sdk_" + sdk_platform.lower(),
        srcs = [
            _XCODE_DEVELOPER_DIR + "/Platforms/" + sdk_platform + ".platform/Developer/SDKs/" + sdk_platform + ".sdk",
        ],
    )
    for sdk_platform in _APPLE_SDK_PLATFORMS
]

[
    filegroup(
        name = "platform_frameworks_" + sdk_platform.lower(),
        srcs = [
            _XCODE_DEVELOPER_DIR + "/Platforms/" + sdk_platform + ".platform/Developer/Library/Frameworks",
        ],
    )
    for sdk_platform in _APPLE_SDK_PLATFORMS
]

[
    filegroup(
        name = "platform_libs_" + sdk_platform.lower(),
        srcs = [
            _XCODE_DEVELOPER_DIR + "/Platforms/" + sdk_platform + ".platform/Developer/usr/lib",
        ],
    )
    for sdk_platform in _APPLE_SDK_PLATFORMS
]

[
    builtin_include_directory(
        name = "sdk_usr_include_directory_" + sdk_platform.lower(),
        path = _XCODE_DEVELOPER_DIR + "/Platforms/" + sdk_platform + ".platform/Developer/SDKs/" + sdk_platform + ".sdk/usr/include",
    )
    for sdk_platform in _APPLE_SDK_PLATFORMS
]

[
    builtin_include_directory(
        name = "sdk_frameworks_directory_" + sdk_platform.lower(),
        path = _XCODE_DEVELOPER_DIR + "/Platforms/" + sdk_platform + ".platform/Developer/SDKs/" + sdk_platform + ".sdk/System/Library/Frameworks",
    )
    for sdk_platform in _APPLE_SDK_PLATFORMS
]

[
    builtin_include_directory(
        name = "platform_frameworks_directory_" + sdk_platform.lower(),
        path = _XCODE_DEVELOPER_DIR + "/Platforms/" + sdk_platform + ".platform/Developer/Library/Frameworks",
    )
    for sdk_platform in _APPLE_SDK_PLATFORMS
]

[
    cc_args(
        name = "applecross_sysroot_" + arch,
        actions = [
            "@rules_cc//cc/toolchains/actions:compile_actions",
            "@rules_cc//cc/toolchains/actions:link_actions",
            "@rules_cc//cc/toolchains/actions:objc_executable",
        ],
        args = [
            "-isysroot",
            "{sysroot}",
            "-F{sysroot}/System/Library/Frameworks",
            "-F{platform_frameworks}",
        ],
        allowlist_include_directories = [
            ":clang_resource_include_directory",
            ":sdk_usr_include_directory_" + sdk_platform.lower(),
            ":sdk_frameworks_directory_" + sdk_platform.lower(),
            ":platform_frameworks_directory_" + sdk_platform.lower(),
        ],
        data = [
            ":sdk_" + sdk_platform.lower(),
            ":platform_frameworks_" + sdk_platform.lower(),
        ],
        env = {
            "APPLECROSS_SYSROOT": "{sysroot}",
            "SDKROOT": "{sysroot}",
        },
        format = {
            "platform_frameworks": ":platform_frameworks_" + sdk_platform.lower(),
            "sysroot": ":sdk_" + sdk_platform.lower(),
        },
    )
    for arch, sdk_platform in _APPLE_SDK_PLATFORMS_BY_ARCH.items()
]

[
    cc_args(
        name = "applecross_platform_link_paths_" + arch,
        actions = [
            "@rules_cc//cc/toolchains/actions:link_actions",
            "@rules_cc//cc/toolchains/actions:objc_executable",
        ],
        # The ported cctools linker predates visionOS. Select the bundled
        # LLVM Mach-O linker for xros targets while retaining cctools for the
        # platforms it already supports.
        args = (["-fuse-ld=lld"] if arch.startswith("visionos_") else []) + [
            "-L{platform_libs}",
        ],
        data = [
            ":platform_libs_" + sdk_platform.lower(),
        ],
        format = {
            "platform_libs": ":platform_libs_" + sdk_platform.lower(),
        },
    )
    for arch, sdk_platform in _APPLE_SDK_PLATFORMS_BY_ARCH.items()
]

cc_args(
    name = "applecross_env",
    actions = ["@rules_cc//cc/toolchains/actions:all_actions"],
    env = {
        "DEVELOPER_DIR": "%{toolchain_path_prefix}Xcode.app/Contents/Developer",
        "LD_LIBRARY_PATH": "%{toolchain_path_prefix}Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib",
    },
)

cc_tool(
    name = "clang_tool",
    src = ":wrapped_clang",
    data = [":clang_tool_files"],
)

cc_tool(
    name = "clangpp_tool",
    src = ":wrapped_clang_pp",
    data = [":clangpp_tool_files"],
)

cc_tool(
    name = "clang_link_tool",
    src = ":wrapped_clang",
    data = [":link_tool_files"],
)

cc_tool(
    name = "libtool_tool",
    src = ":libtool",
    data = [":libtool_files"],
)

cc_tool(
    name = "strip_tool",
    src = "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/llvm-strip",
    data = [
        _XCODE_TOOLCHAIN_BIN + "/llvm",
    ],
)

cc_tool(
    name = "llvm_profdata_tool",
    src = "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/llvm-profdata",
    data = [
        _XCODE_TOOLCHAIN_BIN + "/llvm",
    ],
)

cc_tool(
    name = "llvm_cov_tool",
    src = "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/llvm-cov",
    data = [
        _XCODE_TOOLCHAIN_BIN + "/llvm",
    ],
)

cc_tool_map(
    name = "applecross_tools",
    tools = {
        "@rules_cc//cc/toolchains/actions:ar_actions": ":libtool_tool",
        "@rules_cc//cc/toolchains/actions:assembly_actions": ":clang_tool",
        "@rules_cc//cc/toolchains/actions:c_compile_actions": ":clang_tool",
        "@rules_cc//cc/toolchains/actions:cpp_compile": ":clangpp_tool",
        "@rules_cc//cc/toolchains/actions:cpp_header_parsing": ":clangpp_tool",
        "@rules_cc//cc/toolchains/actions:cpp_link_dynamic_library": ":clang_link_tool",
        "@rules_cc//cc/toolchains/actions:cpp_link_executable": ":clang_link_tool",
        "@rules_cc//cc/toolchains/actions:cpp_module_compile": ":clangpp_tool",
        "@rules_cc//cc/toolchains/actions:linkstamp_compile": ":clang_tool",
        "@rules_cc//cc/toolchains/actions:llvm_cov": ":llvm_cov_tool",
        "@rules_cc//cc/toolchains/actions:llvm_profdata": ":llvm_profdata_tool",
        "@rules_cc//cc/toolchains/actions:objc_compile": ":clang_tool",
        "@rules_cc//cc/toolchains/actions:objc_executable": ":clang_link_tool",
        "@rules_cc//cc/toolchains/actions:objcpp_compile": ":clangpp_tool",
        "@rules_cc//cc/toolchains/actions:strip": ":strip_tool",
    },
)

[
    rule_based_cc_toolchain(
        name = "cc-compiler-" + arch,
        args = [
            "@apple_support_toolchain_env//:include_directories_from_xcode",
            ":applecross_env",
            ":applecross_sysroot_" + arch,
            ":applecross_platform_link_paths_" + arch,
            "@apple_support//toolchain:xcode_env",
        ],
        artifact_name_patterns = ["@apple_support//toolchain:dylib_pattern"],
        compiler = "clang",
        enabled_features = APPLE_SUPPORT_ENABLED_FEATURES,
        known_features = APPLE_SUPPORT_KNOWN_FEATURES,
        legacy_tools = ["@apple_support//toolchain:gcov"],
        make_variables = ["@apple_support//toolchain:stack_frame_variable"],
        module_map = "@apple_support//crosstool:module.modulemap",
        supports_header_parsing = True,
        supports_param_files = True,
        target_system_name = "$(TARGET)",
        tool_map = ":applecross_tools",
        toolchains = ["@apple_support//toolchain:dynamic_toolchain_info"],
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

cc_toolchain_suite(
    name = "toolchain",
    toolchains = dict(CC_TOOLCHAINS),
)

[
    swift_toolchain(
        name = "swift-compiler-" + arch,
        cpu = arch,
        swift_tools = "%{swift_tools}",
        toolchain_files = _SWIFT_TOOLCHAIN_FILES_BY_ARCH[arch],
        toolchain_path_prefix = "%{toolchain_path_prefix}",
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
