load("@build_bazel_apple_support//configs:platforms.bzl", "APPLE_PLATFORMS_CONSTRAINTS")
load("@build_bazel_rules_swift//swift/toolchains:swift_toolchain.bzl", linux_swift_toolchain = "swift_toolchain")
load("@rules_applecross//toolchain:objc_link_config.bzl", "apple_cc_toolchain_config")
load("@rules_applecross//toolchain:swift_toolchain.bzl", "swift_toolchain")
load("@rules_cc//cc:defs.bzl", "cc_library", "cc_toolchain_suite", _native_cc_toolchain = "cc_toolchain")
load("@rules_cc//cc/toolchains:args.bzl", "cc_args")
load("@rules_cc//cc/toolchains:args_list.bzl", "cc_args_list")
load("@rules_cc//cc/toolchains:feature.bzl", "cc_feature")
load("@rules_cc//cc/toolchains:feature_constraint.bzl", "cc_feature_constraint")
load("@rules_cc//cc/toolchains:feature_set.bzl", "cc_feature_set")
load("@rules_cc//cc/toolchains:nested_args.bzl", "cc_nested_args")
load("@rules_cc//cc/toolchains:tool.bzl", "cc_tool")
load("@rules_cc//cc/toolchains:tool_map.bzl", "cc_tool_map")
load("@rules_cc//cc/toolchains:toolchain.bzl", _cc_toolchain_macro = "cc_toolchain")

package(default_visibility = ["//visibility:public"])

_APPLE_ARCHS = APPLE_PLATFORMS_CONSTRAINTS.keys()
_PLATFORMS = "@rules_applecross//toolchain/platforms"

# =============================================================================
# Exported files
# =============================================================================

exports_files([
    "wrapped_clang",
    "wrapped_clang_pp",
    "cc_wrapper.sh",
    "libtool",
    "xcrunwrapper.sh",
])

# =============================================================================
# File groups
# =============================================================================

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
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/arc/**",
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/clang/**",
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/**",
            "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-5.0/**",
        ],
        exclude = [
            "Xcode.app/Contents/Developer/Platforms/*.platform/Developer/SDKs/*.sdk/System/Library/Frameworks/Ruby.framework/**",
        ],
        allow_empty = True,
    ),
)

# =============================================================================
# Tool definitions
# =============================================================================

cc_tool(
    name = "wrapped_clang_tool",
    src = ":wrapped_clang",
    data = [":toolchain_files"],
    allowlist_include_directories = [],
)

cc_tool(
    name = "wrapped_clang_pp_tool",
    src = ":wrapped_clang_pp",
    data = [":toolchain_files"],
    allowlist_include_directories = [],
)

cc_tool(
    name = "cc_wrapper_tool",
    src = ":cc_wrapper.sh",
    data = [
        ":toolchain_files",
        ":wrapped_clang",
        ":wrapped_clang_pp",
        ":xcrunwrapper.sh",
    ],
    allowlist_include_directories = [],
)

cc_tool(
    name = "libtool_tool",
    src = ":libtool",
    data = [":toolchain_files"],
    allowlist_include_directories = [],
)

cc_tool(
    name = "llvm_strip_tool",
    src = "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/llvm-strip",
)

# =============================================================================
# Tool map
# =============================================================================

cc_tool_map(
    name = "tool_map",
    tools = {
        "@rules_cc//cc/toolchains/actions:assembly_actions": ":wrapped_clang_tool",
        "@rules_cc//cc/toolchains/actions:c_compile": ":wrapped_clang_tool",
        "@rules_cc//cc/toolchains/actions:objc_compile": ":wrapped_clang_tool",
        "@rules_cc//cc/toolchains/actions:cpp_compile_actions": ":wrapped_clang_pp_tool",
        "@rules_cc//cc/toolchains/actions:link_actions": ":cc_wrapper_tool",
        "@rules_cc//cc/toolchains/actions:ar_actions": ":libtool_tool",
        "@rules_cc//cc/toolchains/actions:strip": ":llvm_strip_tool",
    },
)

# =============================================================================
# SDK paths (detected at repo rule time)
# =============================================================================

_TOOLCHAIN_PREFIX = "%{toolchain_path_prefix}"
_REPO_PATH = "%{cxx_builtin_include_directories}"
_DEVELOPER_DIR = _TOOLCHAIN_PREFIX + "Xcode.app/Contents/Developer"

# =============================================================================
# Target flags (triple is injected by apple_cc_toolchain_config at analysis time)
# =============================================================================

cc_args(
    name = "target_flags",
    actions = [
        "@rules_cc//cc/toolchains/actions:compile_actions",
        "@rules_cc//cc/toolchains/actions:link_actions",
    ],
    args = ["-no-canonical-prefixes"],
    allowlist_absolute_include_directories = [_REPO_PATH],
)

# =============================================================================
# SDK sysroot and framework paths
# =============================================================================

cc_args(
    name = "sysroot_compile_flags",
    actions = ["@rules_cc//cc/toolchains/actions:compile_actions"],
    args = select({
        _PLATFORMS + ":sdk_iphoneos": ["-isysroot", "%{sdk_path_iphoneos}", "-F%{sdk_fw_iphoneos}", "-F%{plat_fw_iphoneos}"],
        _PLATFORMS + ":sdk_iphonesimulator": ["-isysroot", "%{sdk_path_iphonesimulator}", "-F%{sdk_fw_iphonesimulator}", "-F%{plat_fw_iphonesimulator}"],
        _PLATFORMS + ":sdk_macosx": ["-isysroot", "%{sdk_path_macosx}", "-F%{sdk_fw_macosx}", "-F%{plat_fw_macosx}"],
        _PLATFORMS + ":sdk_appletvos": ["-isysroot", "%{sdk_path_appletvos}", "-F%{sdk_fw_appletvos}", "-F%{plat_fw_appletvos}"],
        _PLATFORMS + ":sdk_appletvsimulator": ["-isysroot", "%{sdk_path_appletvsimulator}", "-F%{sdk_fw_appletvsimulator}", "-F%{plat_fw_appletvsimulator}"],
        _PLATFORMS + ":sdk_xros": ["-isysroot", "%{sdk_path_xros}", "-F%{sdk_fw_xros}", "-F%{plat_fw_xros}"],
        _PLATFORMS + ":sdk_xrsimulator": ["-isysroot", "%{sdk_path_xrsimulator}", "-F%{sdk_fw_xrsimulator}", "-F%{plat_fw_xrsimulator}"],
        _PLATFORMS + ":sdk_watchos": ["-isysroot", "%{sdk_path_watchos}", "-F%{sdk_fw_watchos}", "-F%{plat_fw_watchos}"],
        _PLATFORMS + ":sdk_watchsimulator": ["-isysroot", "%{sdk_path_watchsimulator}", "-F%{sdk_fw_watchsimulator}", "-F%{plat_fw_watchsimulator}"],
    }),
)

cc_args(
    name = "sysroot_link_flags",
    actions = ["@rules_cc//cc/toolchains/actions:link_actions"],
    args = select({
        _PLATFORMS + ":sdk_iphoneos": ["-isysroot", "%{sdk_path_iphoneos}", "-F%{plat_fw_iphoneos}", "-L%{plat_lib_iphoneos}"],
        _PLATFORMS + ":sdk_iphonesimulator": ["-isysroot", "%{sdk_path_iphonesimulator}", "-F%{plat_fw_iphonesimulator}", "-L%{plat_lib_iphonesimulator}"],
        _PLATFORMS + ":sdk_macosx": ["-isysroot", "%{sdk_path_macosx}", "-F%{plat_fw_macosx}", "-L%{plat_lib_macosx}"],
        _PLATFORMS + ":sdk_appletvos": ["-isysroot", "%{sdk_path_appletvos}", "-F%{plat_fw_appletvos}", "-L%{plat_lib_appletvos}"],
        _PLATFORMS + ":sdk_appletvsimulator": ["-isysroot", "%{sdk_path_appletvsimulator}", "-F%{plat_fw_appletvsimulator}", "-L%{plat_lib_appletvsimulator}"],
        _PLATFORMS + ":sdk_xros": ["-isysroot", "%{sdk_path_xros}", "-F%{plat_fw_xros}", "-L%{plat_lib_xros}"],
        _PLATFORMS + ":sdk_xrsimulator": ["-isysroot", "%{sdk_path_xrsimulator}", "-F%{plat_fw_xrsimulator}", "-L%{plat_lib_xrsimulator}"],
        _PLATFORMS + ":sdk_watchos": ["-isysroot", "%{sdk_path_watchos}", "-F%{plat_fw_watchos}", "-L%{plat_lib_watchos}"],
        _PLATFORMS + ":sdk_watchsimulator": ["-isysroot", "%{sdk_path_watchsimulator}", "-F%{plat_fw_watchsimulator}", "-L%{plat_lib_watchsimulator}"],
    }),
)

# =============================================================================
# Default compile flags
# =============================================================================

cc_args(
    name = "default_compile_flags",
    actions = ["@rules_cc//cc/toolchains/actions:compile_actions"],
    args = [
        "-fstack-protector",
        "-fcolor-diagnostics",
        "-Wall",
        "-Wthread-safety",
        "-Wself-assign",
        "-fno-omit-frame-pointer",
    ],
)

cc_args(
    name = "deterministic_compile_flags",
    actions = ["@rules_cc//cc/toolchains/actions:compile_actions"],
    args = [
        "-Wno-builtin-macro-redefined",
        "-D__DATE__=\"redacted\"",
        "-D__TIMESTAMP__=\"redacted\"",
        "-D__TIME__=\"redacted\"",
    ],
)

cc_args(
    name = "debug_prefix_map_flags",
    actions = ["@rules_cc//cc/toolchains/actions:compile_actions"],
    args = [
        "-fdebug-prefix-map=__BAZEL_EXECUTION_ROOT__=.",
        "-fdebug-prefix-map=__BAZEL_XCODE_DEVELOPER_DIR__=/PLACEHOLDER_DEVELOPER_DIR",
    ],
)

cc_args(
    name = "fortify_source_flags",
    actions = ["@rules_cc//cc/toolchains/actions:compile_actions"],
    args = ["-D_FORTIFY_SOURCE=1"],
    requires_any_of = [":not_asan"],
)

# =============================================================================
# ObjC-specific flags
# =============================================================================

cc_args(
    name = "objc_default_flags",
    actions = [
        "@rules_cc//cc/toolchains/actions:objc_compile",
        "@rules_cc//cc/toolchains/actions:objcpp_compile",
    ],
    args = select({
        _PLATFORMS + ":is_ios": ["-DOS_IOS", "-fno-autolink"],
        _PLATFORMS + ":is_watchos": ["-DOS_IOS", "-fno-autolink"],
        _PLATFORMS + ":is_macos": ["-DOS_MACOSX", "-fno-autolink"],
        _PLATFORMS + ":is_tvos": ["-DOS_TVOS", "-fno-autolink"],
        _PLATFORMS + ":is_visionos": ["-fno-autolink"],
    }),
)

cc_args(
    name = "objc_warnings",
    actions = [
        "@rules_cc//cc/toolchains/actions:objc_compile",
        "@rules_cc//cc/toolchains/actions:objcpp_compile",
    ],
    args = [
        "-Werror=incompatible-sysroot",
        "-Wshorten-64-to-32",
        "-Wbool-conversion",
        "-Wconstant-conversion",
        "-Wduplicate-method-match",
        "-Wempty-body",
        "-Wenum-conversion",
        "-Wint-conversion",
        "-Wunreachable-code",
        "-Wmismatched-return-types",
        "-Wundeclared-selector",
        "-Wuninitialized",
        "-Wunused-function",
        "-Wunused-variable",
    ],
)

cc_args(
    name = "objcpp_flags",
    actions = ["@rules_cc//cc/toolchains/actions:objcpp_compile"],
    args = [
        "-stdlib=libc++",
        "-std=gnu++17",
    ],
)

cc_args(
    name = "simulator_compile_flags",
    actions = [
        "@rules_cc//cc/toolchains/actions:objc_compile",
        "@rules_cc//cc/toolchains/actions:objcpp_compile",
    ],
    args = select({
        _PLATFORMS + ":is_simulator": [
            "-fexceptions",
            "-fasm-blocks",
            "-fobjc-abi-version=2",
            "-fobjc-legacy-dispatch",
        ],
        "//conditions:default": [],
    }),
)

# =============================================================================
# Default link flags
# =============================================================================

cc_args(
    name = "default_link_flags",
    actions = ["@rules_cc//cc/toolchains/actions:link_actions"],
    args = [
        "-fuse-ld=lld",
        "--ld-path=%{tools_path_prefix}ld64.lld",
        "-Wl,-force_load_swift_libs",
        "-fobjc-link-runtime",
        "-headerpad_max_install_names",
        "-Wl,-oso_prefix,__BAZEL_EXECUTION_ROOT__/",
    ],
)

cc_args(
    name = "undefined_dynamic_lookup_flags",
    actions = ["@rules_cc//cc/toolchains/actions:link_actions"],
    args = ["-undefined", "dynamic_lookup"],
)

cc_args(
    name = "lto_object_path_flags",
    actions = ["@rules_cc//cc/toolchains/actions:link_actions"],
    args = [
        "-Xlinker",
        "-object_path_lto",
        "-Xlinker",
        "{output_execpath}.lto.o",
    ],
    format = {"output_execpath": "@rules_cc//cc/toolchains/variables:output_execpath"},
    requires_not_none = "@rules_cc//cc/toolchains/variables:output_execpath",
)

cc_args(
    name = "link_libcpp",
    actions = ["@rules_cc//cc/toolchains/actions:link_actions"],
    args = ["-lc++"],
    requires_any_of = [":not_kernel_extension"],
)

# =============================================================================
# Archiver flags override: libtool needs -static and -o for all Apple platforms.
# The built-in archiver_flags only uses libtool flags for @platforms//os:macos.
# =============================================================================

cc_feature(
    name = "apple_archiver_flags_feature",
    overrides = "@rules_cc//cc/toolchains/features/legacy:archiver_flags",
    args = [":apple_archiver_args"],
)

cc_args_list(
    name = "apple_archiver_args",
    args = [
        ":apple_ar_create_flags",
        ":apple_ar_output_execpath",
        ":apple_ar_libraries_to_link",
    ],
)

cc_args(
    name = "apple_ar_create_flags",
    actions = ["@rules_cc//cc/toolchains/actions:ar_actions"],
    args = ["-no_warning_for_no_symbols", "-static"],
)

cc_args(
    name = "apple_ar_output_execpath",
    actions = ["@rules_cc//cc/toolchains/actions:ar_actions"],
    args = ["-o", "{output_execpath}"],
    format = {"output_execpath": "@rules_cc//cc/toolchains/variables:output_execpath"},
    requires_not_none = "@rules_cc//cc/toolchains/variables:output_execpath",
)

cc_nested_args(
    name = "apple_ar_link_obj_file",
    args = ["{object_file}"],
    format = {"object_file": "@rules_cc//cc/toolchains/variables:libraries_to_link.name"},
    requires_equal = "@rules_cc//cc/toolchains/variables:libraries_to_link.type",
    requires_equal_value = "object_file",
)

cc_nested_args(
    name = "apple_ar_link_object_file_group",
    args = ["{object_files}"],
    format = {"object_files": "@rules_cc//cc/toolchains/variables:libraries_to_link.object_files"},
    iterate_over = "@rules_cc//cc/toolchains/variables:libraries_to_link.object_files",
    requires_equal = "@rules_cc//cc/toolchains/variables:libraries_to_link.type",
    requires_equal_value = "object_file_group",
)

cc_nested_args(
    name = "apple_ar_libraries_to_link_expansion",
    iterate_over = "@rules_cc//cc/toolchains/variables:libraries_to_link",
    nested = [
        ":apple_ar_link_obj_file",
        ":apple_ar_link_object_file_group",
    ],
)

cc_args(
    name = "apple_ar_libraries_to_link",
    actions = ["@rules_cc//cc/toolchains/actions:ar_actions"],
    nested = [":apple_ar_libraries_to_link_expansion"],
    requires_not_none = "@rules_cc//cc/toolchains/variables:libraries_to_link",
)

# =============================================================================
# Implicit frameworks
# =============================================================================

cc_args(
    name = "implicit_frameworks",
    actions = ["@rules_cc//cc/toolchains/actions:link_actions"],
    args = select({
        _PLATFORMS + ":is_ios": ["-framework", "Foundation", "-framework", "UIKit"],
        _PLATFORMS + ":is_tvos": ["-framework", "Foundation", "-framework", "UIKit"],
        _PLATFORMS + ":is_visionos": ["-framework", "Foundation", "-framework", "UIKit"],
        _PLATFORMS + ":is_watchos": ["-framework", "Foundation", "-framework", "UIKit"],
        _PLATFORMS + ":is_macos": ["-framework", "Foundation"],
    }),
    requires_any_of = [":not_kernel_extension"],
)

# =============================================================================
# Apple environment variables
# =============================================================================

_ALL_CC_ACTIONS = [
    "@rules_cc//cc/toolchains/actions:compile_actions",
    "@rules_cc//cc/toolchains/actions:link_actions",
    "@rules_cc//cc/toolchains/actions:ar_actions",
]

cc_args(
    name = "apple_env",
    actions = _ALL_CC_ACTIONS,
    env = {
        "DEVELOPER_DIR": _DEVELOPER_DIR,
        "ZERO_AR_DATE": "1",
        "XCODE_VERSION_OVERRIDE": "%{xcode_version}",
        "APPLE_SDK_VERSION_OVERRIDE": "%{sdk_version_override}",
    },
)

_SDK_PLATFORM_MAP = {
    "sdk_macosx": "MacOSX",
    "sdk_iphoneos": "iPhoneOS",
    "sdk_iphonesimulator": "iPhoneSimulator",
    "sdk_appletvos": "AppleTVOS",
    "sdk_appletvsimulator": "AppleTVSimulator",
    "sdk_watchos": "WatchOS",
    "sdk_watchsimulator": "WatchSimulator",
    "sdk_xros": "XROS",
    "sdk_xrsimulator": "XRSimulator",
}

[
    cc_args(
        name = "apple_sdk_platform_" + sdk_name,
        actions = _ALL_CC_ACTIONS,
        env = {"APPLE_SDK_PLATFORM": platform_name},
    )
    for sdk_name, platform_name in _SDK_PLATFORM_MAP.items()
]

cc_args_list(
    name = "apple_sdk_platform",
    args = select({
        _PLATFORMS + ":" + sdk_name: [":apple_sdk_platform_" + sdk_name]
        for sdk_name in _SDK_PLATFORM_MAP.keys()
    }),
)

# =============================================================================
# Feature definitions
# =============================================================================

# Backfill feature that carries the actual feature_name.
cc_feature(
    name = "backfill_legacy_args",
    feature_name = "experimental_replace_legacy_action_config_features",
)

# Inline soname_flags since @rules_cc//cc/toolchains/args/soname_flags:feature is not visible.
cc_feature(
    name = "soname_flags",
    args = [":macos_set_install_name"],
    feature_name = "_soname_flags",
)

cc_args(
    name = "macos_set_install_name",
    actions = ["@rules_cc//cc/toolchains/actions:link_actions"],
    args = ["-Wl,-install_name,@rpath/{runtime_solib_name}"],
    format = {"runtime_solib_name": "@rules_cc//cc/toolchains/variables:runtime_solib_name"},
    requires_not_none = "@rules_cc//cc/toolchains/variables:runtime_solib_name",
)

# The experimental feature set that replaces legacy action configs with rule-based ones.
cc_feature_set(
    name = "experimental_replace_legacy_action_config_features",
    all_of = [
        ":backfill_legacy_args",
        ":apple_archiver_flags_feature",
        "@rules_cc//cc/toolchains/args/pic_flags:feature",
        "@rules_cc//cc/toolchains/args/libraries_to_link:feature",
        "@rules_cc//cc/toolchains/args/linker_param_file:feature",
        "@rules_cc//cc/toolchains/args/preprocessor_defines:feature",
        "@rules_cc//cc/toolchains/args/random_seed:feature",
        "@rules_cc//cc/toolchains/args/runtime_library_search_directories:feature",
        "@rules_cc//cc/toolchains/args/shared_flag:feature",
        "@rules_cc//cc/toolchains/args/strip_debug_symbols:feature",
        "@rules_cc//cc/toolchains/args/strip_flags:feature",
        "@rules_cc//cc/toolchains/args/objc_arc_flags:feature",
        ":soname_flags",
        "@rules_cc//cc/toolchains/args/include_flags:feature",
        "@rules_cc//cc/toolchains/args/compiler_input_flags:feature",
        "@rules_cc//cc/toolchains/args/compiler_output_flags:feature",
        "@rules_cc//cc/toolchains/args/fission_flags:feature",
        "@rules_cc//cc/toolchains/args/link_flags:feature",
        "@rules_cc//cc/toolchains/args/linkstamp_flags:feature",
        "@rules_cc//cc/toolchains/args/library_search_directories:feature",
        "@rules_cc//cc/toolchains/args/dependency_file:feature",
        "@rules_cc//cc/toolchains/args/compile_flags:feature",
    ],
)

# Compilation mode features

cc_feature(
    name = "opt_feature",
    overrides = "@rules_cc//cc/toolchains/features:opt",
    args = [":opt_compile_flags", ":opt_link_flags", ":opt_ns_block_assertions"],
)

cc_args(
    name = "opt_compile_flags",
    actions = ["@rules_cc//cc/toolchains/actions:compile_actions"],
    args = ["-g0", "-O2", "-DNDEBUG"],
)

cc_args(
    name = "opt_link_flags",
    actions = ["@rules_cc//cc/toolchains/actions:link_actions"],
    args = ["-dead_strip"],
)

cc_args(
    name = "opt_ns_block_assertions",
    actions = ["@rules_cc//cc/toolchains/actions:compile_actions"],
    args = ["-DNS_BLOCK_ASSERTIONS=1"],
)

cc_feature(
    name = "dbg_feature",
    overrides = "@rules_cc//cc/toolchains/features:dbg",
    args = [":dbg_compile_flags"],
)

cc_args(
    name = "dbg_compile_flags",
    actions = ["@rules_cc//cc/toolchains/actions:compile_actions"],
    args = ["-g"],
)

cc_feature(
    name = "fastbuild_feature",
    overrides = "@rules_cc//cc/toolchains/features:fastbuild",
    args = [":fastbuild_compile_flags"],
)

cc_args(
    name = "fastbuild_compile_flags",
    actions = ["@rules_cc//cc/toolchains/actions:compile_actions"],
    args = ["-O0", "-DDEBUG"],
)

# Coverage features

cc_feature(
    name = "coverage_feature",
    overrides = "@rules_cc//cc/toolchains/features/legacy:coverage",
)

cc_feature(
    name = "llvm_coverage_map_format_feature",
    overrides = "@rules_cc//cc/toolchains/features/legacy:llvm_coverage_map_format",
    args = [":llvm_coverage_compile_flags", ":llvm_coverage_link_flags"],
    requires_any_of = [":coverage_feature_set"],
)

cc_args(
    name = "llvm_coverage_compile_flags",
    actions = ["@rules_cc//cc/toolchains/actions:compile_actions"],
    args = ["-fprofile-instr-generate", "-fcoverage-mapping", "-g"],
)

cc_args(
    name = "llvm_coverage_link_flags",
    actions = ["@rules_cc//cc/toolchains/actions:link_actions"],
    args = ["-fprofile-instr-generate"],
)

cc_feature(
    name = "coverage_prefix_map_feature",
    feature_name = "coverage_prefix_map",
    args = [":coverage_prefix_map_flags"],
    requires_any_of = [":coverage_feature_set"],
)

cc_args(
    name = "coverage_prefix_map_flags",
    actions = ["@rules_cc//cc/toolchains/actions:compile_actions"],
    args = ["-fcoverage-prefix-map=__BAZEL_EXECUTION_ROOT__=."],
)

# dSYM generation

cc_feature(
    name = "generate_dsym_file_feature",
    overrides = "@rules_cc//cc/toolchains/features/legacy:generate_dsym_file",
    args = [":dsym_compile_flags", ":dsym_link_flags"],
)

cc_args(
    name = "dsym_compile_flags",
    actions = ["@rules_cc//cc/toolchains/actions:compile_actions"],
    args = ["-g"],
)

cc_args(
    name = "dsym_link_flags",
    actions = ["@rules_cc//cc/toolchains/actions:link_actions"],
    args = ["DSYM_HINT_DSYM_PATH={dsym_path}"],
    format = {"dsym_path": "@rules_cc//cc/toolchains/variables:dsym_path"},
    requires_not_none = "@rules_cc//cc/toolchains/variables:dsym_path",
)

# Linkmap

cc_feature(
    name = "generate_linkmap_feature",
    feature_name = "generate_linkmap",
    args = [":linkmap_flags"],
)

cc_args(
    name = "linkmap_flags",
    actions = ["@rules_cc//cc/toolchains/actions:link_actions"],
    args = [
        "-Xlinker",
        "-map",
        "-Xlinker",
        "{output_execpath}.map",
    ],
    format = {"output_execpath": "@rules_cc//cc/toolchains/variables:output_execpath"},
    requires_not_none = "@rules_cc//cc/toolchains/variables:output_execpath",
)

# Install name (for dylibs)

cc_feature(
    name = "set_install_name_feature",
    feature_name = "set_install_name",
    args = [":install_name_flags"],
)

cc_args(
    name = "install_name_flags",
    actions = ["@rules_cc//cc/toolchains/actions:cpp_link_dynamic_library"],
    args = [
        "-Xlinker",
        "-install_name",
        "-Xlinker",
        "@rpath/{runtime_solib_name}",
    ],
    format = {"runtime_solib_name": "@rules_cc//cc/toolchains/variables:runtime_solib_name"},
    requires_not_none = "@rules_cc//cc/toolchains/variables:runtime_solib_name",
)

# Sanitizer features

cc_feature(
    name = "asan_feature",
    feature_name = "asan",
    args = [":asan_compile_flags", ":asan_link_flags"],
)

cc_args(
    name = "asan_compile_flags",
    actions = ["@rules_cc//cc/toolchains/actions:compile_actions"],
    args = ["-fsanitize=address"],
)

cc_args(
    name = "asan_link_flags",
    actions = ["@rules_cc//cc/toolchains/actions:link_actions"],
    args = ["-fsanitize=address"],
)

cc_feature(
    name = "tsan_feature",
    feature_name = "tsan",
    args = [":tsan_compile_flags", ":tsan_link_flags"],
)

cc_args(
    name = "tsan_compile_flags",
    actions = ["@rules_cc//cc/toolchains/actions:compile_actions"],
    args = ["-fsanitize=thread"],
)

cc_args(
    name = "tsan_link_flags",
    actions = ["@rules_cc//cc/toolchains/actions:link_actions"],
    args = ["-fsanitize=thread"],
)

cc_feature(
    name = "ubsan_feature",
    feature_name = "ubsan",
    args = [":ubsan_compile_flags", ":ubsan_link_flags"],
)

cc_args(
    name = "ubsan_compile_flags",
    actions = ["@rules_cc//cc/toolchains/actions:compile_actions"],
    args = ["-fsanitize=undefined"],
)

cc_args(
    name = "ubsan_link_flags",
    actions = ["@rules_cc//cc/toolchains/actions:link_actions"],
    args = ["-fsanitize=undefined"],
)

# Marker features (no args, just enablement)

cc_feature(
    name = "archive_param_file_feature",
    feature_name = "archive_param_file",
)

cc_feature(
    name = "no_legacy_features_feature",
    feature_name = "no_legacy_features",
)

cc_feature(
    name = "parse_headers_feature",
    feature_name = "parse_headers",
)

cc_feature(
    name = "module_maps_feature",
    feature_name = "module_maps",
)

cc_feature(
    name = "dead_strip_feature",
    feature_name = "dead_strip",
    args = [":dead_strip_link_flags"],
)

cc_args(
    name = "dead_strip_link_flags",
    actions = ["@rules_cc//cc/toolchains/actions:link_actions"],
    args = ["-dead_strip"],
)

cc_feature(
    name = "kernel_extension_feature",
    feature_name = "kernel_extension",
)

cc_feature(
    name = "suppress_warnings_feature",
    feature_name = "suppress_warnings",
    args = [":suppress_warnings_flags"],
)

cc_args(
    name = "suppress_warnings_flags",
    actions = ["@rules_cc//cc/toolchains/actions:compile_actions"],
    args = ["-w"],
)

cc_feature(
    name = "treat_warnings_as_errors_feature",
    feature_name = "treat_warnings_as_errors",
    args = [":werror_compile_flags", ":werror_link_flags"],
)

cc_args(
    name = "werror_compile_flags",
    actions = ["@rules_cc//cc/toolchains/actions:compile_actions"],
    args = ["-Werror"],
)

cc_args(
    name = "werror_link_flags",
    actions = ["@rules_cc//cc/toolchains/actions:link_actions"],
    args = ["-Wl,-fatal_warnings"],
)

cc_feature(
    name = "no_warn_duplicate_libraries_feature",
    feature_name = "no_warn_duplicate_libraries",
    args = [":no_warn_duplicate_libraries_flags"],
)

cc_args(
    name = "no_warn_duplicate_libraries_flags",
    actions = ["@rules_cc//cc/toolchains/actions:link_actions"],
    args = ["-Wl,-no_warn_duplicate_libraries"],
)

cc_feature(
    name = "dynamic_linking_mode_feature",
    overrides = "@rules_cc//cc/toolchains/features:dynamic_linking_mode",
)

# Feature constraints used by requires_any_of

cc_feature_constraint(
    name = "not_asan",
    none_of = [":asan_feature"],
)

cc_feature_constraint(
    name = "not_kernel_extension",
    none_of = [":kernel_extension_feature"],
)

cc_feature_set(
    name = "coverage_feature_set",
    all_of = [":coverage_feature"],
)

# gcc quoting for param files
cc_feature(
    name = "gcc_quoting_for_param_files_feature",
    feature_name = "gcc_quoting_for_param_files",
)

# Enables the rules_apple codepath that uses standard C++ variables
# (libraries_to_link, output_execpath) for objc_executable linking instead
# of legacy ObjC variables (filelist, force_load_exec_paths, linked_binary).
cc_feature(
    name = "use_cpp_vars_for_objc_executable",
    feature_name = "use_cpp_variables_for_objc_executable",
)

# =============================================================================
# No-deduplicate (non-opt builds)
# =============================================================================

cc_feature(
    name = "no_deduplicate_feature",
    feature_name = "no_deduplicate",
    args = [":no_deduplicate_flags"],
)

cc_args(
    name = "no_deduplicate_flags",
    actions = ["@rules_cc//cc/toolchains/actions:link_actions"],
    args = [
        "-Xlinker",
        "-no_deduplicate",
    ],
    requires_any_of = [":not_opt"],
)

cc_feature_constraint(
    name = "not_opt",
    none_of = [":opt_feature"],
)

# =============================================================================
# All toolchain args aggregated
# =============================================================================

cc_args_list(
    name = "all_args",
    args = [
        ":target_flags",
        ":sysroot_compile_flags",
        ":sysroot_link_flags",
        ":default_compile_flags",
        ":deterministic_compile_flags",
        ":debug_prefix_map_flags",
        ":fortify_source_flags",
        ":objc_default_flags",
        ":objc_warnings",
        ":objcpp_flags",
        ":simulator_compile_flags",
        ":default_link_flags",
        ":undefined_dynamic_lookup_flags",
        ":lto_object_path_flags",
        ":link_libcpp",
        ":implicit_frameworks",
        ":apple_env",
        ":apple_sdk_platform",
    ],
)

# =============================================================================
# CC toolchains (one per Apple arch)
# =============================================================================

_KNOWN_FEATURES = [
    ":opt_feature",
    ":dbg_feature",
    ":fastbuild_feature",
    ":coverage_feature",
    ":llvm_coverage_map_format_feature",
    ":coverage_prefix_map_feature",
    ":generate_dsym_file_feature",
    ":generate_linkmap_feature",
    ":set_install_name_feature",
    ":asan_feature",
    ":tsan_feature",
    ":ubsan_feature",
    ":archive_param_file_feature",
    ":no_legacy_features_feature",
    ":parse_headers_feature",
    ":module_maps_feature",
    ":dead_strip_feature",
    ":kernel_extension_feature",
    ":suppress_warnings_feature",
    ":treat_warnings_as_errors_feature",
    ":no_warn_duplicate_libraries_feature",
    ":dynamic_linking_mode_feature",
    ":no_deduplicate_feature",
    ":gcc_quoting_for_param_files_feature",
    ":use_cpp_vars_for_objc_executable",
    ":soname_flags",
    ":backfill_legacy_args",
    ":apple_archiver_flags_feature",
    # Non-legacy builtin features
    "@rules_cc//cc/toolchains/features:static_linking_mode",
    "@rules_cc//cc/toolchains/features:static_link_cpp_runtimes",
    # Legacy builtin features
    "@rules_cc//cc/toolchains/features/legacy:legacy_compile_flags",
    "@rules_cc//cc/toolchains/features/legacy:default_compile_flags",
    "@rules_cc//cc/toolchains/features/legacy:dependency_file",
    "@rules_cc//cc/toolchains/features/legacy:pic",
    "@rules_cc//cc/toolchains/features/legacy:preprocessor_defines",
    "@rules_cc//cc/toolchains/features/legacy:includes",
    "@rules_cc//cc/toolchains/features/legacy:include_paths",
    "@rules_cc//cc/toolchains/features/legacy:fdo_instrument",
    "@rules_cc//cc/toolchains/features/legacy:fdo_optimize",
    "@rules_cc//cc/toolchains/features/legacy:cs_fdo_instrument",
    "@rules_cc//cc/toolchains/features/legacy:cs_fdo_optimize",
    "@rules_cc//cc/toolchains/features/legacy:fdo_prefetch_hints",
    "@rules_cc//cc/toolchains/features/legacy:autofdo",
    "@rules_cc//cc/toolchains/features/legacy:shared_flag",
    "@rules_cc//cc/toolchains/features/legacy:linkstamps",
    "@rules_cc//cc/toolchains/features/legacy:output_execpath_flags",
    "@rules_cc//cc/toolchains/features/legacy:runtime_library_search_directories",
    "@rules_cc//cc/toolchains/features/legacy:library_search_directories",
    "@rules_cc//cc/toolchains/features/legacy:archiver_flags",
    "@rules_cc//cc/toolchains/features/legacy:libraries_to_link",
    "@rules_cc//cc/toolchains/features/legacy:force_pic_flags",
    "@rules_cc//cc/toolchains/features/legacy:user_link_flags",
    "@rules_cc//cc/toolchains/features/legacy:random_seed",
    "@rules_cc//cc/toolchains/features/legacy:legacy_link_flags",
    "@rules_cc//cc/toolchains/features/legacy:static_libgcc",
    "@rules_cc//cc/toolchains/features/legacy:fission_support",
    "@rules_cc//cc/toolchains/features/legacy:per_object_debug_info",
    "@rules_cc//cc/toolchains/features/legacy:strip_debug_symbols",
    "@rules_cc//cc/toolchains/features/legacy:gcc_coverage_map_format",
    "@rules_cc//cc/toolchains/features/legacy:fully_static_link",
    "@rules_cc//cc/toolchains/features/legacy:user_compile_flags",
    "@rules_cc//cc/toolchains/features/legacy:unfiltered_compile_flags",
    "@rules_cc//cc/toolchains/features/legacy:linker_param_file",
    "@rules_cc//cc/toolchains/features/legacy:compiler_input_flags",
    "@rules_cc//cc/toolchains/features/legacy:compiler_output_flags",
    ":experimental_replace_legacy_action_config_features",
]

_ENABLED_FEATURES = [
    ":opt_feature",
    ":dbg_feature",
    ":archive_param_file_feature",
    ":no_legacy_features_feature",
    ":module_maps_feature",
    ":set_install_name_feature",
    ":coverage_prefix_map_feature",
    ":no_deduplicate_feature",
    ":gcc_quoting_for_param_files_feature",
    ":use_cpp_vars_for_objc_executable",
    ":apple_archiver_flags_feature",
    ":experimental_replace_legacy_action_config_features",
]

# Inner rule-based cc_toolchain (creates _base_{arch} and __base_{arch}_config)
[
    _cc_toolchain_macro(
        name = "_base_" + arch,
        tool_map = ":tool_map",
        args = [":all_args"],
        known_features = _KNOWN_FEATURES,
        enabled_features = _ENABLED_FEATURES,
        compiler = "clang",
        supports_header_parsing = True,
        supports_param_files = True,
    )
    for arch in _APPLE_ARCHS
]

# Wrap with target triple from XcodeVersionConfig + legacy objc_fully_link
[
    apple_cc_toolchain_config(
        name = "_cc-compiler-" + arch + "_config",
        inner_config = ":__base_" + arch + "_config",
        cpu = arch,
        libtool_path = "libtool",
    )
    for arch in _APPLE_ARCHS
]

[
    _native_cc_toolchain(
        name = "cc-compiler-" + arch,
        toolchain_config = "_cc-compiler-" + arch + "_config",
        all_files = "_cc-compiler-" + arch + "_config",
        ar_files = "_cc-compiler-" + arch + "_config",
        as_files = "_cc-compiler-" + arch + "_config",
        compiler_files = "_cc-compiler-" + arch + "_config",
        coverage_files = "_cc-compiler-" + arch + "_config",
        dwp_files = "_cc-compiler-" + arch + "_config",
        linker_files = "_cc-compiler-" + arch + "_config",
        objcopy_files = "_cc-compiler-" + arch + "_config",
        strip_files = "_cc-compiler-" + arch + "_config",
        supports_header_parsing = True,
        supports_param_files = True,
        exec_transition_for_inputs = False,
    )
    for arch in _APPLE_ARCHS
]

# Toolchain registrations for CC

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

# =============================================================================
# Legacy cc_toolchain_suite (for --crosstool_top compatibility)
# =============================================================================

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

cc_library(
    name = "link_extra_lib",
)

cc_library(
    name = "malloc",
)

cc_toolchain_suite(
    name = "toolchain",
    toolchains = dict(CC_TOOLCHAINS),
)

# =============================================================================
# Swift toolchains
# =============================================================================

[
    swift_toolchain(
        name = "swift-compiler-" + arch,
        cpu = arch,
        toolchain_files = ":toolchain_files",
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
