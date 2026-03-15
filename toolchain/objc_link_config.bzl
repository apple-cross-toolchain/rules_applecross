"""Wrapper that augments a rule-based cc_toolchain_config with Apple-specific bits.

Two things can't be expressed in the rule-based cc_toolchain API:

1. Target triple with dynamic minimum OS version — cc_args takes literal
   strings, but the min OS should come from apple_common.XcodeVersionConfig
   at analysis time. We inject a feature with -target <triple> flags.

2. Legacy objc_fully_link action_config — rules_apple's
   register_fully_link_action() passes library paths via variables_extension
   (fully_linked_archive_path, objc_library_exec_paths) rather than
   linking_contexts. We inject a legacy action_config that expands those
   ObjC-specific variables directly.
"""

load("@bazel_tools//tools/build_defs/cc:action_names.bzl", "ACTION_NAMES")
load(
    "@bazel_tools//tools/cpp:cc_toolchain_config_lib.bzl",
    "action_config",
    "feature",
    "flag_group",
    "flag_set",
    "tool",
)
load("@rules_cc//cc/common:cc_common.bzl", _cc_common = "cc_common")
load(
    "@rules_cc//cc/toolchains:cc_toolchain_config_info.bzl",
    "CcToolchainConfigInfo",
)

_COMPILE_AND_LINK_ACTIONS = [
    ACTION_NAMES.c_compile,
    ACTION_NAMES.cpp_compile,
    ACTION_NAMES.objc_compile,
    ACTION_NAMES.objcpp_compile,
    ACTION_NAMES.assemble,
    ACTION_NAMES.preprocess_assemble,
    ACTION_NAMES.cpp_link_executable,
    ACTION_NAMES.cpp_link_dynamic_library,
    ACTION_NAMES.cpp_link_nodeps_dynamic_library,
    ACTION_NAMES.objc_executable,
]

# Map cpu name -> (clang_arch, os_name, is_simulator)
_CPU_MAP = {
    "darwin_arm64": ("arm64", "macosx", False),
    "darwin_arm64e": ("arm64e", "macosx", False),
    "darwin_x86_64": ("x86_64", "macosx", False),
    "ios_arm64": ("arm64", "ios", False),
    "ios_arm64e": ("arm64e", "ios", False),
    "ios_sim_arm64": ("arm64", "ios", True),
    "ios_x86_64": ("x86_64", "ios", True),
    "tvos_arm64": ("arm64", "tvos", False),
    "tvos_sim_arm64": ("arm64", "tvos", True),
    "tvos_x86_64": ("x86_64", "tvos", True),
    "visionos_arm64": ("arm64", "xros", False),
    "visionos_sim_arm64": ("arm64", "xros", True),
    "watchos_arm64": ("arm64", "watchos", True),
    "watchos_device_arm64": ("arm64", "watchos", False),
    "watchos_device_arm64e": ("arm64e", "watchos", False),
    "watchos_arm64_32": ("arm64_32", "watchos", False),
    "watchos_armv7k": ("armv7k", "watchos", False),
    "watchos_x86_64": ("x86_64", "watchos", True),
}

_PLATFORM_TYPE_MAP = {
    "ios": apple_common.platform_type.ios,
    "macosx": apple_common.platform_type.macos,
    "tvos": apple_common.platform_type.tvos,
    "watchos": apple_common.platform_type.watchos,
    "xros": apple_common.platform_type.visionos,
}

def _target_triple(cpu, xcode_config):
    """Returns the -target triple string for a given cpu."""
    if cpu not in _CPU_MAP:
        fail("Unknown cpu: {}".format(cpu))
    arch, os_name, is_sim = _CPU_MAP[cpu]
    platform_type = _PLATFORM_TYPE_MAP[os_name]
    min_os = xcode_config.minimum_os_for_platform_type(platform_type)
    triple = "{}-apple-{}{}".format(arch, os_name, min_os)
    if is_sim:
        triple += "-simulator"
    return triple

def _apple_cc_toolchain_config_impl(ctx):
    inner = ctx.attr.inner_config[CcToolchainConfigInfo]
    xcode_config = ctx.attr._xcode_config[apple_common.XcodeVersionConfig]
    triple = _target_triple(ctx.attr.cpu, xcode_config)

    # Feature that injects -target <triple> into compile and link actions.
    target_flags_feature = feature(
        name = "apple_target_flags",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = _COMPILE_AND_LINK_ACTIONS,
                flag_groups = [
                    flag_group(flags = ["-target", triple]),
                ],
            ),
        ],
    )

    # Legacy objc_fully_link action_config (see module docstring).
    objc_fully_link_config = action_config(
        action_name = ACTION_NAMES.objc_fully_link,
        flag_sets = [
            flag_set(
                flag_groups = [
                    flag_group(flags = ["-static"]),
                    flag_group(flags = ["-o", "%{fully_linked_archive_path}"]),
                    flag_group(
                        flags = ["%{objc_library_exec_paths}"],
                        iterate_over = "objc_library_exec_paths",
                    ),
                    flag_group(
                        flags = ["%{cc_library_exec_paths}"],
                        iterate_over = "cc_library_exec_paths",
                    ),
                    flag_group(
                        flags = ["%{imported_library_exec_paths}"],
                        iterate_over = "imported_library_exec_paths",
                    ),
                ],
            ),
        ],
        tools = [tool(path = ctx.attr.libtool_path)],
    )

    filtered_configs = [
        ac
        for ac in inner._action_configs_DO_NOT_USE
        if ac.action_name != ACTION_NAMES.objc_fully_link
    ]
    new_config = _cc_common.create_cc_toolchain_config_info(
        ctx = ctx,
        action_configs = filtered_configs + [objc_fully_link_config],
        features = inner._features_DO_NOT_USE + [target_flags_feature],
        artifact_name_patterns = inner._artifact_name_patterns_DO_NOT_USE,
        make_variables = inner.make_variables,
        cxx_builtin_include_directories = list(inner.cxx_builtin_include_directories),
        toolchain_identifier = inner.toolchain_id,
        compiler = inner.compiler,
        target_cpu = inner.target_cpu,
        target_system_name = inner.target_system_name or "",
        target_libc = inner.target_libc or "",
        abi_version = inner.abi_version or "",
        abi_libc_version = inner.abi_libc_version or "",
    )

    return [
        new_config,
        ctx.attr.inner_config[DefaultInfo],
    ]

apple_cc_toolchain_config = rule(
    implementation = _apple_cc_toolchain_config_impl,
    attrs = {
        "inner_config": attr.label(
            mandatory = True,
            providers = [CcToolchainConfigInfo],
        ),
        "cpu": attr.string(mandatory = True),
        "libtool_path": attr.string(mandatory = True),
        "_xcode_config": attr.label(default = configuration_field(
            fragment = "apple",
            name = "xcode_config_label",
        )),
    },
    provides = [CcToolchainConfigInfo],
    fragments = ["apple", "cpp"],
)
