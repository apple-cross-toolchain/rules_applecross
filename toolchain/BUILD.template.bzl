load("@apple_support//configs:platforms.bzl", "APPLE_PLATFORMS_CONSTRAINTS")
load("@bazel_features//:features.bzl", "bazel_features")
load("@rules_applecross//toolchain:exec_tool.bzl", "exec_tool")
load("@rules_applecross//toolchain:swift_toolchain.bzl", "swift_toolchain")
load("@rules_cc//cc:defs.bzl", "cc_binary", "cc_library", "cc_toolchain_suite")
load("@rules_cc//cc/toolchains:args.bzl", "cc_args")
load("@rules_cc//cc/toolchains:tool.bzl", "cc_tool")
load("@rules_cc//cc/toolchains:tool_map.bzl", "cc_tool_map")
load("@rules_cc//cc/toolchains:toolchain.bzl", rule_based_cc_toolchain = "cc_toolchain")

package(default_visibility = ["//visibility:public"])

_APPLE_ARCHS = APPLE_PLATFORMS_CONSTRAINTS.keys()

exports_files([
    "cc_wrapper.sh",
    "xcrunwrapper.sh",
])

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
    "@apple_support//toolchain:sysroot_feature",
    "@apple_support//toolchain:headerpad",
    "@rules_cc//cc/toolchains/args/objc_arc_flags:feature",
    "@apple_support//toolchain:user_link_flags",
    "@apple_support_toolchain_env//:linkopts_from_env",
    "@apple_support//toolchain:default_required_flags",
    "@apple_support//toolchain:__apply_simulator_compiler_flags",
    "@apple_support_toolchain_env//:copts_from_env",
    "@apple_support//toolchain:default_link_flags",
    "@apple_support//toolchain:no_deduplicate",
    "@apple_support//toolchain:dead_strip_wrapper",
    "@apple_support//toolchain:apply_implicit_frameworks",
    "@apple_support//toolchain:link_cocoa_wrapper",
    "@apple_support//toolchain:extra_enabled_features",
    "@apple_support//toolchain:user_compile_flags",
    "@apple_support//toolchain:unfiltered_compile_flags",
    "@apple_support//toolchain:__compiler_input_flags_without_header_parsing",
    "@apple_support//toolchain:__header_parsing_input_flags",
    "@rules_cc//cc/toolchains/args/compiler_output_flags:feature",
    "@rules_cc//cc/toolchains/args/linker_param_file:feature",
    "@apple_support//toolchain:set_install_name",
    "@apple_support//toolchain/sanitizers:asan_wrapper",
    "@apple_support//toolchain/sanitizers:tsan_wrapper",
    "@apple_support//toolchain/sanitizers:ubsan_wrapper",
    "@apple_support//toolchain/sanitizers:default_sanitizer_flags",
    "@apple_support//toolchain:suppress_warnings_wrapper",
    "@apple_support//toolchain:treat_warnings_as_errors_wrapper",
    "@apple_support//toolchain:external_include_paths_wrapper",
    "@apple_support_toolchain_env//:off_by_default_layering_check_enabled_features",
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
    "@apple_support_toolchain_env//:off_by_default_layering_check_known_features",
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

filegroup(
    name = "ported_tools",
    srcs = glob(["Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/*"]),
)

filegroup(
    name = "sdk_tool_files",
    srcs = glob(
        include = [
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/*",
            "Xcode.app/Contents/Developer/Platforms/*.platform/Info.plist",
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/SDKs/*.sdk/SDKSettings.json",
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/SDKs/*.sdk/SDKSettings.plist",
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/SDKs/*.sdk/System/Library/CoreServices/SystemVersion.plist",
            "Xcode.app/Contents/Developer/version.plist",
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
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/SDKs/*.sdk/usr/**",
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/SDKs/*.sdk/System/**",
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/SDKs/*.sdk/SDKSettings.json",
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/SDKs/*.sdk/SDKSettings.plist",
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/Library/**",
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/usr/**",
            "Xcode.app/Contents/Developer/Platforms/*.platform/usr/**",
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/ToolchainInfo.plist",
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/**",
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/include/**",
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/*.so*",
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/arc/**",
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/clang/**",
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/**",
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-5.0/**",
        ],
        allow_empty = True,
        exclude = [
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/SDKs/*.sdk/System/Library/Frameworks/Ruby.framework/**",
        ],
    ),
)

filegroup(
    name = "swift_toolchain_files",
    srcs = glob(
        include = [
            "Xcode.app/Contents/Developer/Platforms/*.platform/Info.plist",
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/SDKs/*.sdk/usr/**",
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/SDKs/*.sdk/System/**",
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/SDKs/*.sdk/SDKSettings.json",
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/SDKs/*.sdk/SDKSettings.plist",
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/Library/**",
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/usr/**",
            "Xcode.app/Contents/Developer/Platforms/*.platform/usr/**",
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/ToolchainInfo.plist",
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/include/**",
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/arc/**",
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/clang/**",
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/**",
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-5.0/**",
        ],
        allow_empty = True,
        exclude = [
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/SDKs/*.sdk/System/Library/Frameworks/Ruby.framework/**",
        ],
    ),
)

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
    data = [":toolchain_files"],
)

cc_tool(
    name = "clangpp_tool",
    src = ":wrapped_clang_pp",
    data = [":toolchain_files"],
)

cc_tool(
    name = "libtool_tool",
    src = ":libtool",
    data = [
        ":toolchain_files",
        ":xcrunwrapper.sh",
    ],
)

cc_tool(
    name = "strip_tool",
    src = "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/llvm-strip",
    data = [":toolchain_files"],
)

cc_tool(
    name = "llvm_profdata_tool",
    src = "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/llvm-profdata",
    data = [":toolchain_files"],
)

cc_tool(
    name = "llvm_cov_tool",
    src = "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/llvm-cov",
    data = [":toolchain_files"],
)

cc_tool_map(
    name = "applecross_tools",
    tools = {
        "@rules_cc//cc/toolchains/actions:ar_actions": ":libtool_tool",
        "@rules_cc//cc/toolchains/actions:assembly_actions": ":clang_tool",
        "@rules_cc//cc/toolchains/actions:c_compile_actions": ":clang_tool",
        "@rules_cc//cc/toolchains/actions:cpp_compile": ":clangpp_tool",
        "@rules_cc//cc/toolchains/actions:cpp_header_parsing": ":clangpp_tool",
        "@rules_cc//cc/toolchains/actions:cpp_link_dynamic_library": ":clang_tool",
        "@rules_cc//cc/toolchains/actions:cpp_link_executable": ":clang_tool",
        "@rules_cc//cc/toolchains/actions:cpp_module_compile": ":clangpp_tool",
        "@rules_cc//cc/toolchains/actions:linkstamp_compile": ":clang_tool",
        "@rules_cc//cc/toolchains/actions:llvm_cov": ":llvm_cov_tool",
        "@rules_cc//cc/toolchains/actions:llvm_profdata": ":llvm_profdata_tool",
        "@rules_cc//cc/toolchains/actions:objc_compile": ":clang_tool",
        "@rules_cc//cc/toolchains/actions:objc_executable": ":clang_tool",
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
        toolchain_files = ":swift_toolchain_files",
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
