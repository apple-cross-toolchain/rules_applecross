# Copyright 2018 The Bazel Authors. All rights reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#    http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""A Swift toolchain for cross-compiling for Apple platforms from Linux.

Adapted for rules_swift 3.x.
"""

load("@bazel_tools//tools/cpp:toolchain_utils.bzl", "find_cpp_toolchain")
load("@rules_cc//cc/common:cc_common.bzl", "cc_common")
load("@rules_cc//cc/common:cc_info.bzl", "CcInfo")
load(
    "@build_bazel_rules_swift//swift:swift.bzl",
    "SwiftToolchainInfo",
)
load(
    "@build_bazel_rules_swift//swift/toolchains/config:action_config.bzl",
    "ActionConfigInfo",
    "add_arg",
)
load(
    "@build_bazel_rules_swift//swift/toolchains/config:all_actions_config.bzl",
    "all_actions_action_configs",
)
load(
    "@build_bazel_rules_swift//swift/toolchains/config:compile_config.bzl",
    "compile_action_configs",
)
load(
    "@build_bazel_rules_swift//swift/toolchains/config:compile_module_interface_config.bzl",
    "compile_module_interface_action_configs",
)
load(
    "@build_bazel_rules_swift//swift/toolchains/config:modulewrap_config.bzl",
    "modulewrap_action_configs",
)
load(
    "@build_bazel_rules_swift//swift/toolchains/config:symbol_graph_config.bzl",
    "symbol_graph_action_configs",
)
load(
    "@build_bazel_rules_swift//swift/toolchains/config:synthesize_interface_config.bzl",
    "synthesize_interface_action_configs",
)
load(
    "@build_bazel_rules_swift//swift/toolchains/config:tool_config.bzl",
    "ToolConfigInfo",
)

# Path to the toolchain root, substituted by the template engine.
_TOOLCHAIN_ROOT = "%{toolchain_path_prefix}Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr"
_DEVELOPER_DIR = "%{toolchain_path_prefix}Xcode.app/Contents/Developer"

# Swift action names (matching rules_swift constants).
_SWIFT_ACTION_COMPILE = "SwiftCompile"
_SWIFT_ACTION_COMPILE_MODULE_INTERFACE = "SwiftCompileModuleInterface"
_SWIFT_ACTION_DERIVE_FILES = "SwiftDeriveFiles"
_SWIFT_ACTION_DUMP_AST = "SwiftDumpAST"
_SWIFT_ACTION_PRECOMPILE_C_MODULE = "SwiftPrecompileCModule"
_SWIFT_ACTION_SYMBOL_GRAPH_EXTRACT = "SwiftSymbolGraphExtract"
_SWIFT_ACTION_SYNTHESIZE_INTERFACE = "SwiftSynthesizeInterface"

# Map from cpu prefix to platform info.
_PLATFORM_INFO = {
    "darwin": struct(
        platform_type = "macos",
        os_name = "macosx",
        sdk_platform_device = "MacOSX",
        sdk_platform_sim = "MacOSX",
        swift_platform_name = "macosx",
    ),
    "ios": struct(
        platform_type = "ios",
        os_name = "ios",
        sdk_platform_device = "iPhoneOS",
        sdk_platform_sim = "iPhoneSimulator",
        swift_platform_name = "iphoneos",
    ),
    "tvos": struct(
        platform_type = "tvos",
        os_name = "tvos",
        sdk_platform_device = "AppleTVOS",
        sdk_platform_sim = "AppleTVSimulator",
        swift_platform_name = "appletvos",
    ),
    "watchos": struct(
        platform_type = "watchos",
        os_name = "watchos",
        sdk_platform_device = "WatchOS",
        sdk_platform_sim = "WatchSimulator",
        swift_platform_name = "watchos",
    ),
    "visionos": struct(
        platform_type = "visionos",
        os_name = "xros",
        sdk_platform_device = "XROS",
        sdk_platform_sim = "XRSimulator",
        swift_platform_name = "xros",
    ),
}

_SIMULATOR_CPUS = [
    "ios_i386", "ios_x86_64",
    "tvos_x86_64",
    "watchos_i386", "watchos_x86_64",
    "visionos_x86_64",
]

_PLATFORM_TYPE_MAP = {
    "ios": apple_common.platform_type.ios,
    "macos": apple_common.platform_type.macos,
    "tvos": apple_common.platform_type.tvos,
    "watchos": apple_common.platform_type.watchos,
}

def _make_resource_directory_configurator(developer_dir):
    """Configures the -resource-dir flag for Swift compilation."""
    def _resource_directory_configurator(_prerequisites, args):
        args.add(
            "-resource-dir",
            "{developer_dir}/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift".format(
                developer_dir = developer_dir,
            ),
        )

    return _resource_directory_configurator

def _make_tool_configs(additional_tools):
    """Creates tool configurations for Swift actions."""
    def _driver_config(mode):
        return {
            "mode": mode,
            "toolchain_root": _TOOLCHAIN_ROOT,
            "tool_executable_suffix": "",
        }

    # Persistent worker mode for main compile actions (matches upstream).
    persistent_tool_config = ToolConfigInfo(
        additional_tools = additional_tools,
        driver_config = _driver_config("swiftc"),
        use_param_file = True,
        worker_mode = "persistent",
    )

    # Wrap mode for non-compile actions.
    wrap_tool_config = ToolConfigInfo(
        additional_tools = additional_tools,
        driver_config = _driver_config("swiftc"),
        use_param_file = True,
        worker_mode = "wrap",
    )

    return {
        _SWIFT_ACTION_COMPILE: persistent_tool_config,
        _SWIFT_ACTION_DERIVE_FILES: persistent_tool_config,
        _SWIFT_ACTION_DUMP_AST: persistent_tool_config,
        _SWIFT_ACTION_PRECOMPILE_C_MODULE: wrap_tool_config,
        _SWIFT_ACTION_COMPILE_MODULE_INTERFACE: wrap_tool_config,
        _SWIFT_ACTION_SYMBOL_GRAPH_EXTRACT: ToolConfigInfo(
            additional_tools = additional_tools,
            driver_config = _driver_config("swift-symbolgraph-extract"),
            use_param_file = True,
            worker_mode = "wrap",
        ),
    }

def _swift_linkopts_cc_info(toolchain_root, swift_platform_name, sdk_platform, sdk_dir, developer_dir, toolchain_label):
    """Returns a CcInfo with linker flags for Swift standard library linking."""
    swift_lib_dir = developer_dir + "/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/" + swift_platform_name
    platform_developer_dir = developer_dir + "/Platforms/" + sdk_platform + ".platform/Developer"
    platform_developer_framework_dir = platform_developer_dir + "/Library/Frameworks"
    platform_developer_lib_dir = platform_developer_dir + "/usr/lib"

    linkopts = [
        "-L" + swift_lib_dir,
        "-L/usr/lib/swift",
        "-L" + platform_developer_lib_dir,
        "-Wl,-objc_abi_version,2",
        "-F" + platform_developer_framework_dir,
    ]

    return CcInfo(
        linking_context = cc_common.create_linking_context(
            linker_inputs = depset([
                cc_common.create_linker_input(
                    owner = toolchain_label,
                    user_link_flags = depset(linkopts),
                ),
            ]),
        ),
    )

def _test_linking_context(sdk_platform, developer_dir, toolchain_label):
    """Returns a CcLinkingContext with linker flags for test binaries."""
    platform_developer_dir = developer_dir + "/Platforms/" + sdk_platform + ".platform/Developer"
    platform_developer_framework_dir = platform_developer_dir + "/Library/Frameworks"
    platform_developer_lib_dir = platform_developer_dir + "/usr/lib"

    linkopts = [
        "-Wl,-weak_framework,XCTest",
        "-Wl,-weak-lXCTestSwiftSupport",
        "-Wl,-rpath," + platform_developer_framework_dir,
        "-F" + platform_developer_framework_dir,
        "-L" + platform_developer_lib_dir,
    ]

    return cc_common.create_linking_context(
        linker_inputs = depset([
            cc_common.create_linker_input(
                owner = toolchain_label,
                user_link_flags = depset(linkopts),
            ),
        ]),
    )

def _swift_toolchain_impl(ctx):
    cc_toolchain = find_cpp_toolchain(ctx)
    cpu = ctx.attr.cpu

    # Parse platform and arch from cpu string (e.g. "ios_arm64" -> "ios", "arm64")
    platform_prefix, _, arch = cpu.partition("_")
    info = _PLATFORM_INFO.get(platform_prefix)
    if not info:
        fail("Unsupported cpu: " + cpu)

    is_simulator = cpu in _SIMULATOR_CPUS

    # Get xcode_config for version info
    xcode_config = ctx.attr._xcode_config[apple_common.XcodeVersionConfig]

    # Get minimum OS version
    platform_type = _PLATFORM_TYPE_MAP.get(info.platform_type)
    target_os_version = str(xcode_config.minimum_os_for_platform_type(platform_type)) if platform_type else ""

    # Build target triple (e.g. "arm64-apple-ios18.0")
    environment = "-simulator" if is_simulator else ""
    target_triple = "{arch}-apple-{os}{version}{environment}".format(
        arch = arch,
        os = info.os_name,
        version = target_os_version,
        environment = environment,
    )

    # Compute SDK path
    sdk_platform = info.sdk_platform_sim if is_simulator else info.sdk_platform_device
    xcode_version_str = str(xcode_config.xcode_version()) if xcode_config.xcode_version() else ""
    xcode_parts = xcode_version_str.split(".")
    sdk_version = ".".join(xcode_parts[:2]) if len(xcode_parts) >= 2 else xcode_version_str
    sdk_dir = _DEVELOPER_DIR + "/Platforms/" + sdk_platform + ".platform/Developer/SDKs/" + sdk_platform + sdk_version + ".sdk"

    execution_requirements = xcode_config.execution_info()

    # All compile-like actions that need target and SDK flags.
    all_compile_actions = [
        _SWIFT_ACTION_COMPILE,
        _SWIFT_ACTION_COMPILE_MODULE_INTERFACE,
        _SWIFT_ACTION_DERIVE_FILES,
        _SWIFT_ACTION_DUMP_AST,
        _SWIFT_ACTION_PRECOMPILE_C_MODULE,
        _SWIFT_ACTION_SYMBOL_GRAPH_EXTRACT,
        _SWIFT_ACTION_SYNTHESIZE_INTERFACE,
    ]

    # Actions that need -sdk (includes module interface compilation).
    sdk_actions = [
        _SWIFT_ACTION_COMPILE,
        _SWIFT_ACTION_COMPILE_MODULE_INTERFACE,
        _SWIFT_ACTION_DERIVE_FILES,
        _SWIFT_ACTION_DUMP_AST,
        _SWIFT_ACTION_PRECOMPILE_C_MODULE,
    ]

    # Platform developer paths (needed for XCTest and other developer frameworks)
    platform_developer_dir = _DEVELOPER_DIR + "/Platforms/" + sdk_platform + ".platform/Developer"
    platform_developer_framework_dir = platform_developer_dir + "/Library/Frameworks"
    platform_developer_lib_dir = platform_developer_dir + "/usr/lib"

    action_configs = [
        # Target triple
        ActionConfigInfo(
            actions = all_compile_actions,
            configurators = [add_arg("-target", target_triple)],
        ),
        # SDK path (now includes module interface compilation)
        ActionConfigInfo(
            actions = sdk_actions,
            configurators = [add_arg("-sdk", sdk_dir)],
        ),
        # Platform developer framework search path (for XCTest.framework, etc.)
        ActionConfigInfo(
            actions = [
                _SWIFT_ACTION_COMPILE,
                _SWIFT_ACTION_DERIVE_FILES,
                _SWIFT_ACTION_DUMP_AST,
            ],
            configurators = [
                add_arg("-F", platform_developer_framework_dir),
                add_arg("-I", platform_developer_lib_dir),
            ],
        ),
        # Resource directory
        ActionConfigInfo(
            actions = [
                _SWIFT_ACTION_COMPILE,
                _SWIFT_ACTION_DERIVE_FILES,
                _SWIFT_ACTION_DUMP_AST,
                _SWIFT_ACTION_PRECOMPILE_C_MODULE,
                _SWIFT_ACTION_SYMBOL_GRAPH_EXTRACT,
                _SWIFT_ACTION_SYNTHESIZE_INTERFACE,
            ],
            configurators = [
                _make_resource_directory_configurator(_DEVELOPER_DIR),
            ],
        ),
        # Resource directory for module interface compilation (always needed)
        ActionConfigInfo(
            actions = [_SWIFT_ACTION_COMPILE_MODULE_INTERFACE],
            configurators = [
                _make_resource_directory_configurator(_DEVELOPER_DIR),
            ],
        ),
        # Debug prefix map for reproducible debug info
        ActionConfigInfo(
            actions = [
                _SWIFT_ACTION_COMPILE,
                _SWIFT_ACTION_COMPILE_MODULE_INTERFACE,
                _SWIFT_ACTION_DERIVE_FILES,
                _SWIFT_ACTION_DUMP_AST,
                _SWIFT_ACTION_PRECOMPILE_C_MODULE,
            ],
            configurators = [
                add_arg(
                    "-debug-prefix-map",
                    "__BAZEL_XCODE_DEVELOPER_DIR__=/PLACEHOLDER_DEVELOPER_DIR",
                ),
            ],
        ),
    ]

    # Add all the standard rules_swift action configs.
    action_configs.extend(all_actions_action_configs())
    action_configs.extend(compile_action_configs())
    action_configs.extend(modulewrap_action_configs())
    action_configs.extend(symbol_graph_action_configs())
    action_configs.extend(compile_module_interface_action_configs())
    action_configs.extend(synthesize_interface_action_configs())

    # Requested features for Swift compilation
    requested_features = list(ctx.features)
    requested_features.extend([
        "swift.bundled_xctests",
        "swift.enable_testing",
        "swift.module_map_no_private_headers",
        "swift.enable_batch_mode",
        "swift.use_response_files",
        "swift.debug_prefix_map",
        "swift.supports_library_evolution",
        "swift.supports_private_deps",
        "swift.enable_skip_function_bodies",
        "swift.experimental.AccessLevelOnImport",
        "swift.remap_xcode_path",
    ])

    # Collect toolchain files as additional tools for sandbox access.
    additional_tools = ctx.attr._toolchain_files.files.to_list()
    tool_configs = _make_tool_configs(additional_tools)

    # Build Swift linker opts provider
    swift_platform_name = info.swift_platform_name
    if is_simulator:
        swift_platform_name = swift_platform_name.replace("os", "simulator") if swift_platform_name.endswith("os") else swift_platform_name + "simulator"

    swift_linkopts = _swift_linkopts_cc_info(
        toolchain_root = _TOOLCHAIN_ROOT,
        swift_platform_name = swift_platform_name,
        sdk_platform = sdk_platform,
        sdk_dir = sdk_dir,
        developer_dir = _DEVELOPER_DIR,
        toolchain_label = ctx.label,
    )

    # Build test linking context
    test_linking_ctx = _test_linking_context(
        sdk_platform = sdk_platform,
        developer_dir = _DEVELOPER_DIR,
        toolchain_label = ctx.label,
    )

    def _entry_point_linkopts_provider(*, entry_point_name):
        return struct(
            linkopts = ["-Wl,-alias,_{},_main".format(entry_point_name)],
        )

    swift_toolchain_info = SwiftToolchainInfo(
        action_configs = action_configs,
        cc_language = "objc",
        cc_toolchain_info = cc_toolchain,
        clang_implicit_deps_providers = struct(
            cc_infos = [],
            swift_infos = [],
        ),
        const_protocols_to_gather = None,
        cross_import_overlays = [],
        debug_outputs_provider = None,
        developer_dirs = [],
        entry_point_linkopts_provider = _entry_point_linkopts_provider,
        feature_allowlists = [],
        generated_header_module_implicit_deps_providers = struct(
            cc_infos = [],
            swift_infos = [],
        ),
        implicit_deps_providers = struct(
            cc_infos = [swift_linkopts],
            swift_infos = [],
        ),
        module_aliases = {},
        package_configurations = [],
        requested_features = requested_features,
        root_dir = _TOOLCHAIN_ROOT,
        swift_worker = ctx.attr._worker[DefaultInfo].files_to_run,
        test_configuration = struct(
            binary_name = "{name}",
            env = {},
            execution_requirements = execution_requirements,
            objc_test_discovery = True,
            test_linking_contexts = [test_linking_ctx],
        ),
        tool_configs = tool_configs,
        unsupported_features = ctx.disabled_features + [
            "swift.module_map_home_is_cwd",
        ],
    )

    return [
        platform_common.ToolchainInfo(
            swift_toolchain = swift_toolchain_info,
        ),
        swift_toolchain_info,
    ]

swift_toolchain = rule(
    attrs = {
        "cpu": attr.string(
            mandatory = True,
            doc = "The target CPU (e.g. ios_arm64, darwin_arm64).",
        ),
        "_toolchain_files": attr.label(
            default = Label(":toolchain_files"),
        ),
        "_cc_toolchain": attr.label(
            default = Label("@bazel_tools//tools/cpp:current_cc_toolchain"),
        ),
        "_worker": attr.label(
            cfg = "exec",
            allow_files = True,
            default = Label(
                "@build_bazel_rules_swift//tools/worker",
            ),
            executable = True,
        ),
        "_xcode_config": attr.label(
            default = configuration_field(
                name = "xcode_config_label",
                fragment = "apple",
            ),
        ),
    },
    doc = "Represents a Swift compiler toolchain for cross-compiling Apple targets from Linux.",
    fragments = [
        "apple",
        "swift",
    ],
    toolchains = ["@bazel_tools//tools/cpp:toolchain_type"],
    implementation = _swift_toolchain_impl,
)
