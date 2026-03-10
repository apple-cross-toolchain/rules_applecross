"""A C++ toolchain configuration rule to cross-compile apps for Apple platform
from Linux."""

load(
    "@bazel_tools//tools/cpp:cc_toolchain_config_lib.bzl",
    "action_config",
    "artifact_name_pattern",
    "env_entry",
    "env_set",
    "feature",
    "feature_set",
    "flag_group",
    "flag_set",
    "make_variable",
    "tool",
    "tool_path",
    "variable_with_value",
    "with_feature_set",
)
load("@rules_cc//cc:action_names.bzl", "ACTION_NAMES", "ACTION_NAME_GROUPS")
load("@rules_cc//cc/common:cc_common.bzl", "cc_common")

# In Bazel 9, objcpp_executable and objc_archive were removed from ACTION_NAMES.
# Define them as string constants for backward compatibility in action configs.
_OBJCPP_EXECUTABLE = "objcpp-executable"
_OBJC_ARCHIVE = "objc-archive"

_DYNAMIC_LINK_ACTIONS = ACTION_NAME_GROUPS.cc_link_executable_actions + ACTION_NAME_GROUPS.dynamic_library_link_actions
_STATIC_LINK_ACTIONS = [
    ACTION_NAMES.objc_fully_link,
    ACTION_NAMES.cpp_link_static_library,
]
_CPP_DYNAMIC_LINK_ACTIONS = [
    ACTION_NAMES.cpp_link_executable,
    ACTION_NAMES.cpp_link_dynamic_library,
]

# %{toolchain_path_prefix} is replaced by the template engine with the repo path.
_DEVELOPER_DIR = "%{toolchain_path_prefix}Xcode.app/Contents/Developer"

def _arch(cpu):
    _, _, arch = cpu.partition("_")
    if arch.startswith("sim_"):
        arch = arch[len("sim_"):]
    return arch

def _target_apple_platform(cpu):
    platform, _, cpu = cpu.partition("_")
    if platform == "darwin":
        platform = "macos"
    return platform

def _target_system_name(cpu):
    platform, _, arch = cpu.partition("_")
    if platform == "darwin":
        platform = "macosx"
    if arch.startswith("sim_"):
        arch = arch[len("sim_"):]
    return "{}-apple-{}".format(arch, platform)

def _target_libc(cpu):
    platform, _, cpu = cpu.partition("_")
    if platform == "darwin":
        platform = "macosx"
    return platform

def _apple_sdk_platform(cpu):
    """Returns the Apple SDK platform name (e.g., iPhoneOS, MacOSX) for the CPU."""
    platform, _, cpu_arch = cpu.partition("_")
    simulator_cpus = [
        "ios_i386",
        "ios_x86_64",
        "tvos_x86_64",
        "watchos_i386",
        "watchos_x86_64",
    ]
    is_sim = cpu_arch.startswith("sim_") or cpu in simulator_cpus
    if platform == "darwin":
        return "MacOSX"
    elif platform == "ios":
        return "iPhoneSimulator" if is_sim else "iPhoneOS"
    elif platform == "tvos":
        return "AppleTVSimulator" if is_sim else "AppleTVOS"
    elif platform == "watchos":
        return "WatchSimulator" if is_sim else "WatchOS"
    return ""

_PLATFORM_TYPE_MAP = {
    "ios": apple_common.platform_type.ios,
    "macos": apple_common.platform_type.macos,
    "tvos": apple_common.platform_type.tvos,
    "watchos": apple_common.platform_type.watchos,
}

def _impl(ctx):
    target_cpu = ctx.attr.cpu

    arch = _arch(target_cpu)
    compiler = "compiler"
    host_system_name = "local"
    platform_name = _target_apple_platform(target_cpu)
    target_libc = _target_libc(target_cpu)
    target_system_name = _target_system_name(target_cpu)
    toolchain_identifier = target_cpu
    _, _, target_arch = target_cpu.partition("_")
    is_simulator = target_arch.startswith("sim_") or target_cpu in [
        "ios_i386",
        "ios_x86_64",
        "tvos_x86_64",
        "watchos_i386",
        "watchos_x86_64",
    ]

    # Append minimum OS version to target triple (e.g. arm64-apple-ios16.0)
    xcode_config = ctx.attr._xcode_config[apple_common.XcodeVersionConfig]
    xcode_execution_requirements = xcode_config.execution_info().keys()
    platform_type = _PLATFORM_TYPE_MAP.get(platform_name)
    if platform_type:
        min_os = str(xcode_config.minimum_os_for_platform_type(platform_type))
        if min_os:
            environment_suffix = "-simulator" if is_simulator else ""
            target_system_name = target_system_name + min_os + environment_suffix
    is_tvos = platform_name == "tvos"

    if (ctx.attr.cpu == "darwin_x86_64"):
        abi_version = "darwin_x86_64"
    else:
        abi_version = "local"

    if (ctx.attr.cpu == "darwin_x86_64"):
        abi_libc_version = "darwin_x86_64"
    else:
        abi_libc_version = "local"

    cc_target_os = "apple"

    # Compute Apple environment values for env_entries.
    xcode_version_str = str(xcode_config.xcode_version()) if xcode_config.xcode_version() else ""
    sdk_platform_str = _apple_sdk_platform(target_cpu)
    xcode_parts = xcode_version_str.split(".")
    sdk_version_str = ".".join(xcode_parts[:2]) if len(xcode_parts) >= 2 else xcode_version_str

    # Compute Apple SDK paths
    sdk_dir_str = _DEVELOPER_DIR + "/Platforms/" + sdk_platform_str + ".platform/Developer/SDKs/" + sdk_platform_str + sdk_version_str + ".sdk"
    sdk_framework_dir_str = sdk_dir_str + "/System/Library/Frameworks"
    platform_developer_framework_dir_str = _DEVELOPER_DIR + "/Platforms/" + sdk_platform_str + ".platform/Developer/Library/Frameworks"

    builtin_sysroot = None

    all_compile_actions = [
        ACTION_NAMES.c_compile,
        ACTION_NAMES.cpp_compile,
        ACTION_NAMES.linkstamp_compile,
        ACTION_NAMES.assemble,
        ACTION_NAMES.preprocess_assemble,
        ACTION_NAMES.cpp_header_parsing,
        ACTION_NAMES.cpp_module_compile,
        ACTION_NAMES.cpp_module_codegen,
        ACTION_NAMES.clif_match,
        ACTION_NAMES.lto_backend,
    ]

    all_cpp_compile_actions = [
        ACTION_NAMES.cpp_compile,
        ACTION_NAMES.linkstamp_compile,
        ACTION_NAMES.cpp_header_parsing,
        ACTION_NAMES.cpp_module_compile,
        ACTION_NAMES.cpp_module_codegen,
        ACTION_NAMES.clif_match,
    ]

    preprocessor_compile_actions = [
        ACTION_NAMES.c_compile,
        ACTION_NAMES.cpp_compile,
        ACTION_NAMES.linkstamp_compile,
        ACTION_NAMES.preprocess_assemble,
        ACTION_NAMES.cpp_header_parsing,
        ACTION_NAMES.cpp_module_compile,
        ACTION_NAMES.clif_match,
    ]

    codegen_compile_actions = [
        ACTION_NAMES.c_compile,
        ACTION_NAMES.cpp_compile,
        ACTION_NAMES.linkstamp_compile,
        ACTION_NAMES.assemble,
        ACTION_NAMES.preprocess_assemble,
        ACTION_NAMES.cpp_module_codegen,
        ACTION_NAMES.lto_backend,
    ]

    all_link_actions = [
        ACTION_NAMES.cpp_link_executable,
        ACTION_NAMES.cpp_link_dynamic_library,
        ACTION_NAMES.cpp_link_nodeps_dynamic_library,
    ]

    strip_action = action_config(
        action_name = ACTION_NAMES.strip,
        flag_sets = [
            flag_set(
                flag_groups = [
                    flag_group(flags = ["-S", "-o", "%{output_file}"]),
                    flag_group(
                        flags = ["%{stripopts}"],
                        iterate_over = "stripopts",
                    ),
                    flag_group(flags = ["%{input_file}"]),
                ],
            ),
        ],
        tools = [tool(path = "strip")],
    )

    header_parsing_env_feature = feature(
        name = "header_parsing_env",
        env_sets = [
            env_set(
                actions = [ACTION_NAMES.cpp_header_parsing],
                env_entries = [
                    env_entry(
                        key = "HEADER_PARSING_OUTPUT",
                        value = "%{output_file}",
                    ),
                ],
            ),
        ],
    )

    cpp_header_parsing_action = action_config(
        action_name = ACTION_NAMES.cpp_header_parsing,
        implies = [
            "preprocessor_defines",
            "include_system_dirs",
            "objc_arc",
            "no_objc_arc",
            "apple_env",
            "user_compile_flags",
            "sysroot",
            "unfiltered_compile_flags",
            "compiler_output_flags",
            "header_parsing_env",
        ] + (["unfiltered_cxx_flags"] if is_tvos else []),
        flag_sets = [
            flag_set(
                flag_groups = [
                    flag_group(
                        flags = [
                            # Note: This treats all headers as C++ headers, which may lead to
                            # parsing failures for C headers that are not valid C++.
                            # For such headers, use features = ["-parse_headers"] to selectively
                            # disable parsing.
                            "-xc++-header",
                            "-fsyntax-only",
                        ],
                    ),
                ],
            ),
        ],
        tools = [
            tool(
                path = "wrapped_clang",
                execution_requirements = xcode_execution_requirements,
            ),
        ],
    )

    objc_compile_action_implies = [
        "compiler_input_flags",
        "compiler_output_flags",
        "objc_actions",
        "apply_default_compiler_flags",
        "apply_default_warnings",
        "framework_paths",
        "preprocessor_defines",
        "include_system_dirs",
        "objc_arc",
        "no_objc_arc",
        "apple_env",
        "user_compile_flags",
        "sysroot",
        "unfiltered_compile_flags",
    ]
    if is_simulator:
        objc_compile_action_implies.append("apply_simulator_compiler_flags")

    objc_compile_action = action_config(
        action_name = ACTION_NAMES.objc_compile,
        flag_sets = [
            flag_set(
                flag_groups = [flag_group(flags = [
                    "-arch",
                    arch,
                    "-target",
                    target_system_name,
                ])],
            ),
        ],
        implies = objc_compile_action_implies,
        tools = [
            tool(
                path = "wrapped_clang",
                execution_requirements = xcode_execution_requirements,
            ),
        ],
    )

    objc_link_flag_feature = feature(
        name = "objc_link_flag",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = [ACTION_NAMES.objc_executable],
                flag_groups = [flag_group(flags = ["-ObjC"])],
                with_features = [with_feature_set(not_features = ["kernel_extension"])],
            ),
        ],
    )

    objcpp_executable_action = action_config(
        action_name = _OBJCPP_EXECUTABLE,
        flag_sets = [
            flag_set(
                flag_groups = [
                    flag_group(flags = ["-stdlib=libc++", "-std=gnu++17"]),
                    flag_group(flags = [
                        "-arch",
                        arch,
                        "-target",
                        target_system_name,
                    ]),
                    flag_group(
                        flags = [
                            "-Xlinker",
                            "-objc_abi_version",
                            "-Xlinker",
                            "2",
                            "-fobjc-link-runtime",
                            "-ObjC",
                        ],
                    ),
                    flag_group(
                        flags = ["-framework", "%{framework_names}"],
                        iterate_over = "framework_names",
                    ),
                    flag_group(
                        flags = ["-weak_framework", "%{weak_framework_names}"],
                        iterate_over = "weak_framework_names",
                    ),
                    flag_group(
                        flags = ["-l%{library_names}"],
                        iterate_over = "library_names",
                    ),
                    flag_group(flags = ["-filelist", "%{filelist}"]),
                    flag_group(flags = ["-o", "%{linked_binary}"]),
                    flag_group(
                        flags = ["-force_load", "%{force_load_exec_paths}"],
                        iterate_over = "force_load_exec_paths",
                    ),
                    flag_group(
                        flags = ["%{dep_linkopts}"],
                        iterate_over = "dep_linkopts",
                    ),
                    flag_group(
                        flags = ["-Wl,%{attr_linkopts}"],
                        iterate_over = "attr_linkopts",
                    ),
                ],
            ),
        ],
        implies = [
            "include_system_dirs",
            "framework_paths",
            "strip_debug_symbols",
            "apple_env",
            "apply_implicit_frameworks",
        ],
        tools = [
            tool(
                path = "wrapped_clang_pp",
                execution_requirements = xcode_execution_requirements,
            ),
        ],
    )

    cpp_link_dynamic_library_action = action_config(
        action_name = ACTION_NAMES.cpp_link_dynamic_library,
        implies = [
            "contains_objc_source",
            "has_configured_linker_path",
            "shared_flag",
            "linkstamps",
            "output_execpath_flags",
            "runtime_root_flags",
            "input_param_flags",
            "strip_debug_symbols",
            "linker_param_file",
            "apple_env",
            "sysroot",
        ] + (["cpp_linker_flags"] if is_tvos else []),
        tools = [
            tool(
                path = "cc_wrapper.sh",
                execution_requirements = xcode_execution_requirements,
            ),
        ],
    )

    cpp_link_static_library_action = action_config(
        action_name = ACTION_NAMES.cpp_link_static_library,
        implies = [
            "input_param_flags",
            "linker_param_file",
            "apple_env",
        ],
        flag_sets = [
            flag_set(
                flag_groups = [
                    flag_group(
                        flags = [
                            "-D",
                            "-no_warning_for_no_symbols",
                            "-static",
                            "-o",
                            "%{output_execpath}",
                        ],
                        expand_if_available = "output_execpath",
                    ),
                ],
            ),
        ],
        tools = [
            tool(
                path = "libtool",
                execution_requirements = xcode_execution_requirements,
            ),
        ],
    )

    c_compile_action = action_config(
        action_name = ACTION_NAMES.c_compile,
        implies = [
            "preprocessor_defines",
            "include_system_dirs",
            "objc_arc",
            "no_objc_arc",
            "apple_env",
            "user_compile_flags",
            "sysroot",
            "unfiltered_compile_flags",
            "compiler_input_flags",
            "compiler_output_flags",
        ] + (["unfiltered_cxx_flags"] if is_tvos else []),
        tools = [
            tool(
                path = "wrapped_clang",
                execution_requirements = xcode_execution_requirements,
            ),
        ],
    )

    cpp_compile_action = action_config(
        action_name = ACTION_NAMES.cpp_compile,
        implies = [
            "preprocessor_defines",
            "include_system_dirs",
            "objc_arc",
            "no_objc_arc",
            "apple_env",
            "user_compile_flags",
            "sysroot",
            "unfiltered_compile_flags",
            "compiler_input_flags",
            "compiler_output_flags",
        ] + (["unfiltered_cxx_flags"] if is_tvos else []),
        tools = [
            tool(
                path = "wrapped_clang_pp",
                execution_requirements = xcode_execution_requirements,
            ),
        ],
    )

    objcpp_compile_action_implies = [
        "compiler_input_flags",
        "compiler_output_flags",
        "apply_default_compiler_flags",
        "apply_default_warnings",
        "framework_paths",
        "preprocessor_defines",
        "include_system_dirs",
        "objc_arc",
        "no_objc_arc",
        "apple_env",
        "user_compile_flags",
        "sysroot",
        "unfiltered_compile_flags",
    ]
    if is_simulator:
        objcpp_compile_action_implies.append("apply_simulator_compiler_flags")

    objcpp_compile_action = action_config(
        action_name = ACTION_NAMES.objcpp_compile,
        flag_sets = [
            flag_set(
                flag_groups = [
                    flag_group(
                        flags = [
                            "-arch",
                            arch,
                            "-stdlib=libc++",
                            "-std=gnu++17",
                            "-target",
                            target_system_name,
                        ],
                    ),
                ],
            ),
        ],
        implies = objcpp_compile_action_implies,
        tools = [
            tool(
                path = "wrapped_clang_pp",
                execution_requirements = xcode_execution_requirements,
            ),
        ],
    )

    assemble_action = action_config(
        action_name = ACTION_NAMES.assemble,
        implies = [
            "objc_arc",
            "no_objc_arc",
            "include_system_dirs",
            "apple_env",
            "user_compile_flags",
            "sysroot",
            "unfiltered_compile_flags",
            "compiler_input_flags",
            "compiler_output_flags",
        ] + (["unfiltered_cxx_flags"] if is_tvos else []),
        tools = [
            tool(
                path = "wrapped_clang",
                execution_requirements = xcode_execution_requirements,
            ),
        ],
    )

    preprocess_assemble_action = action_config(
        action_name = ACTION_NAMES.preprocess_assemble,
        implies = [
            "preprocessor_defines",
            "include_system_dirs",
            "objc_arc",
            "no_objc_arc",
            "apple_env",
            "user_compile_flags",
            "sysroot",
            "unfiltered_compile_flags",
            "compiler_input_flags",
            "compiler_output_flags",
        ] + (["unfiltered_cxx_flags"] if is_tvos else []),
        tools = [
            tool(
                path = "wrapped_clang",
                execution_requirements = xcode_execution_requirements,
            ),
        ],
    )

    objc_archive_action = action_config(
        action_name = _OBJC_ARCHIVE,
        flag_sets = [
            flag_set(
                flag_groups = [
                    flag_group(
                        flags = [
                            "-D",
                            "-no_warning_for_no_symbols",
                            "-static",
                            "-filelist",
                            "%{obj_list_path}",
                            "-arch_only",
                            arch,
                            "-syslibroot",
                            sdk_dir_str,
                            "-o",
                            "%{archive_path}",
                        ],
                    ),
                ],
            ),
        ],
        implies = ["apple_env"],
        tools = [
            tool(
                path = "libtool",
                execution_requirements = xcode_execution_requirements,
            ),
        ],
    )

    objc_executable_action = action_config(
        action_name = ACTION_NAMES.objc_executable,
        flag_sets = [
            flag_set(
                flag_groups = [
                    flag_group(
                        flags = [
                            "-Xlinker",
                            "-objc_abi_version",
                            "-Xlinker",
                            "2",
                        ],
                    ),
                ],
                with_features = [with_feature_set(not_features = ["kernel_extension"])],
            ),
            flag_set(
                flag_groups = [
                    flag_group(flags = ["-arch", arch]),
                    flag_group(
                        flags = ["-framework", "%{framework_names}"],
                        iterate_over = "framework_names",
                    ),
                    flag_group(
                        flags = ["-weak_framework", "%{weak_framework_names}"],
                        iterate_over = "weak_framework_names",
                    ),
                    flag_group(
                        flags = ["-l%{library_names}"],
                        iterate_over = "library_names",
                    ),
                    flag_group(flags = ["-filelist", "%{filelist}"]),
                    flag_group(flags = ["-o", "%{linked_binary}"]),
                    flag_group(
                        flags = ["-force_load", "%{force_load_exec_paths}"],
                        iterate_over = "force_load_exec_paths",
                    ),
                    flag_group(
                        flags = ["%{dep_linkopts}"],
                        iterate_over = "dep_linkopts",
                    ),
                    flag_group(
                        flags = ["-Wl,%{attr_linkopts}"],
                        iterate_over = "attr_linkopts",
                    ),
                ],
            ),
        ],
        implies = [
            "include_system_dirs",
            "framework_paths",
            "strip_debug_symbols",
            "apple_env",
            "apply_implicit_frameworks",
        ],
        tools = [
            tool(
                path = "wrapped_clang",
                execution_requirements = xcode_execution_requirements,
            ),
        ],
    )

    cpp_link_executable_action = action_config(
        action_name = ACTION_NAMES.cpp_link_executable,
        implies = [
            "contains_objc_source",
            "linkstamps",
            "output_execpath_flags",
            "runtime_root_flags",
            "input_param_flags",
            "force_pic_flags",
            "strip_debug_symbols",
            "linker_param_file",
            "apple_env",
            "sysroot",
        ] + (["cpp_linker_flags"] if is_tvos else []),
        tools = [
            tool(
                path = "cc_wrapper.sh",
                execution_requirements = xcode_execution_requirements,
            ),
        ],
    )

    linkstamp_compile_action = action_config(
        action_name = ACTION_NAMES.linkstamp_compile,
        implies = [
            "preprocessor_defines",
            "include_system_dirs",
            "objc_arc",
            "no_objc_arc",
            "apple_env",
            "user_compile_flags",
            "sysroot",
            "unfiltered_compile_flags",
            "compiler_input_flags",
            "compiler_output_flags",
        ],
        tools = [
            tool(
                path = "wrapped_clang",
                execution_requirements = xcode_execution_requirements,
            ),
        ],
    )

    cpp_module_compile_action = action_config(
        action_name = ACTION_NAMES.cpp_module_compile,
        implies = [
            "preprocessor_defines",
            "include_system_dirs",
            "objc_arc",
            "no_objc_arc",
            "apple_env",
            "user_compile_flags",
            "sysroot",
            "unfiltered_compile_flags",
            "compiler_input_flags",
            "compiler_output_flags",
        ] + (["unfiltered_cxx_flags"] if is_tvos else []),
        tools = [
            tool(
                path = "wrapped_clang",
                execution_requirements = xcode_execution_requirements,
            ),
        ],
    )

    cpp_link_nodeps_dynamic_library_action = action_config(
        action_name = ACTION_NAMES.cpp_link_nodeps_dynamic_library,
        implies = [
            "contains_objc_source",
            "has_configured_linker_path",
            "shared_flag",
            "linkstamps",
            "output_execpath_flags",
            "runtime_root_flags",
            "input_param_flags",
            "strip_debug_symbols",
            "linker_param_file",
            "apple_env",
            "sysroot",
        ] + (["cpp_linker_flags"] if is_tvos else []),
        tools = [
            tool(
                path = "cc_wrapper.sh",
                execution_requirements = xcode_execution_requirements,
            ),
        ],
    )

    objc_fully_link_action = action_config(
        action_name = ACTION_NAMES.objc_fully_link,
        flag_sets = [
            flag_set(
                flag_groups = [
                    flag_group(
                        flags = [
                            "-D",
                            "-no_warning_for_no_symbols",
                            "-static",
                            "-arch_only",
                            arch,
                            "-syslibroot",
                            sdk_dir_str,
                            "-o",
                            "%{fully_linked_archive_path}",
                        ],
                    ),
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
        implies = ["apple_env"],
        tools = [
            tool(
                path = "libtool",
                execution_requirements = xcode_execution_requirements,
            ),
        ],
    )

    action_configs = [
        strip_action,
        c_compile_action,
        cpp_compile_action,
        linkstamp_compile_action,
        cpp_module_compile_action,
        cpp_header_parsing_action,
        objc_compile_action,
        objcpp_compile_action,
        assemble_action,
        preprocess_assemble_action,
        objc_archive_action,
        objc_executable_action,
        objcpp_executable_action,
        cpp_link_executable_action,
        cpp_link_dynamic_library_action,
        cpp_link_nodeps_dynamic_library_action,
        cpp_link_static_library_action,
        objc_fully_link_action,
    ]

    if platform_name == "macos":
        apply_default_compiler_flags_feature = feature(
            name = "apply_default_compiler_flags",
            flag_sets = [
                flag_set(
                    actions = [ACTION_NAMES.objc_compile, ACTION_NAMES.objcpp_compile],
                    flag_groups = [flag_group(flags = ["-DOS_MACOSX", "-fno-autolink"])],
                ),
            ],
        )
    elif is_tvos:
        apply_default_compiler_flags_feature = feature(
            name = "apply_default_compiler_flags",
            flag_sets = [
                flag_set(
                    actions = [ACTION_NAMES.objc_compile, ACTION_NAMES.objcpp_compile],
                    flag_groups = [flag_group(flags = ["-DOS_TVOS", "-fno-autolink"])],
                ),
            ],
        )
    else:
        apply_default_compiler_flags_feature = feature(
            name = "apply_default_compiler_flags",
            flag_sets = [
                flag_set(
                    actions = [ACTION_NAMES.objc_compile, ACTION_NAMES.objcpp_compile],
                    flag_groups = [flag_group(flags = ["-DOS_IOS", "-fno-autolink"])],
                ),
            ],
        )

    runtime_root_flags_feature = feature(
        name = "runtime_root_flags",
        flag_sets = [
            flag_set(
                actions = _CPP_DYNAMIC_LINK_ACTIONS,
                flag_groups = [
                    flag_group(
                        flags = [
                            "-Xlinker",
                            "-rpath",
                            "-Xlinker",
                            "@loader_path/%{runtime_library_search_directories}",
                        ],
                        iterate_over = "runtime_library_search_directories",
                        expand_if_available = "runtime_library_search_directories",
                    ),
                ],
            ),
        ],
    )

    use_objc_modules_feature = feature(
        name = "use_objc_modules",
        flag_sets = [
            flag_set(
                actions = [ACTION_NAMES.objc_compile, ACTION_NAMES.objcpp_compile],
                flag_groups = [
                    flag_group(
                        flags = [
                            "-fmodule-name=%{module_name}",
                            "-iquote",
                            "%{module_maps_dir}",
                            "-fmodules-cache-path=%{modules_cache_path}",
                        ],
                    ),
                ],
            ),
        ],
    )

    objc_arc_feature = feature(
        name = "objc_arc",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.assemble,
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [
                    flag_group(
                        flags = ["-fobjc-arc"],
                        expand_if_available = "objc_arc",
                    ),
                ],
            ),
        ],
    )

    if is_tvos:
        unfiltered_cxx_flags_feature = feature(
            name = "unfiltered_cxx_flags",
            flag_sets = [
                flag_set(
                    actions = [
                        ACTION_NAMES.c_compile,
                        ACTION_NAMES.cpp_compile,
                        ACTION_NAMES.cpp_module_compile,
                        ACTION_NAMES.cpp_header_parsing,
                        ACTION_NAMES.assemble,
                        ACTION_NAMES.preprocess_assemble,
                    ],
                    flag_groups = [
                        flag_group(flags = ["-no-canonical-prefixes", "-pthread"]),
                    ],
                ),
            ],
        )
    else:
        unfiltered_cxx_flags_feature = feature(name = "unfiltered_cxx_flags")

    compiler_input_flags_feature = feature(
        name = "compiler_input_flags",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.assemble,
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.linkstamp_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.cpp_module_codegen,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [
                    flag_group(
                        flags = ["-c", "%{source_file}"],
                        expand_if_available = "source_file",
                    ),
                ],
            ),
        ],
    )

    external_include_paths_feature = feature(
        name = "external_include_paths",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.linkstamp_compile,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                    ACTION_NAMES.clif_match,
                ],
                flag_groups = [
                    flag_group(
                        flags = ["-isystem", "%{external_include_paths}"],
                        iterate_over = "external_include_paths",
                        expand_if_available = "external_include_paths",
                    ),
                ],
            ),
        ],
    )

    strip_debug_symbols_feature = feature(
        name = "strip_debug_symbols",
        flag_sets = [
            flag_set(
                actions = _DYNAMIC_LINK_ACTIONS +
                          [_OBJCPP_EXECUTABLE],
                flag_groups = [
                    flag_group(
                        flags = ["STRIP_DEBUG_SYMBOLS"],
                        expand_if_available = "strip_debug_symbols",
                    ),
                ],
            ),
        ],
    )

    shared_flag_feature = feature(
        name = "shared_flag",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.cpp_link_dynamic_library,
                ],
                flag_groups = [flag_group(flags = ["-shared"])],
            ),
        ],
    )

    if is_simulator:
        apply_simulator_compiler_flags_feature = feature(
            name = "apply_simulator_compiler_flags",
            flag_sets = [
                flag_set(
                    actions = [ACTION_NAMES.objc_compile, ACTION_NAMES.objcpp_compile],
                    flag_groups = [
                        flag_group(
                            flags = [
                                "-fexceptions",
                                "-fasm-blocks",
                                "-fobjc-abi-version=2",
                                "-fobjc-legacy-dispatch",
                            ],
                        ),
                    ],
                ),
            ],
        )
    else:
        apply_simulator_compiler_flags_feature = feature(name = "apply_simulator_compiler_flags")

    user_link_flags_feature = feature(
        name = "user_link_flags",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = _DYNAMIC_LINK_ACTIONS +
                          [_OBJCPP_EXECUTABLE],
                flag_groups = [
                    flag_group(
                        flags = ["%{user_link_flags}"],
                        iterate_over = "user_link_flags",
                        expand_if_available = "user_link_flags",
                    ),
                ],
            ),
        ],
    )

    if platform_name == "macos":
        contains_objc_source_feature = feature(
            name = "contains_objc_source",
            flag_sets = [
                flag_set(
                    actions = [
                        ACTION_NAMES.cpp_link_dynamic_library,
                        ACTION_NAMES.cpp_link_nodeps_dynamic_library,
                        ACTION_NAMES.cpp_link_executable,
                    ],
                    flag_groups = [flag_group(flags = ["-fobjc-link-runtime"])],
                ),
            ],
        )
    else:
        contains_objc_source_feature = feature(
            name = "contains_objc_source",
            flag_sets = [
                flag_set(
                    actions = [
                        ACTION_NAMES.cpp_link_dynamic_library,
                        ACTION_NAMES.cpp_link_nodeps_dynamic_library,
                        ACTION_NAMES.cpp_link_executable,
                    ],
                    flag_groups = [flag_group(flags = ["-fobjc-link-runtime"])],
                ),
                flag_set(
                    actions = [
                        ACTION_NAMES.cpp_link_dynamic_library,
                        ACTION_NAMES.cpp_link_nodeps_dynamic_library,
                        ACTION_NAMES.cpp_link_executable,
                    ],
                    flag_groups = [flag_group(flags = ["-framework", "UIKit"])],
                ),
            ],
        )

    includes_feature = feature(
        name = "includes",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.linkstamp_compile,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [
                    flag_group(
                        flags = ["-include", "%{includes}"],
                        iterate_over = "includes",
                        expand_if_available = "includes",
                    ),
                ],
            ),
        ],
    )

    gcc_coverage_map_format_feature = feature(
        name = "gcc_coverage_map_format",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [
                    flag_group(
                        flags = ["-fprofile-arcs", "-ftest-coverage", "-g"],
                    ),
                ],
            ),
            flag_set(
                actions = [
                    ACTION_NAMES.cpp_link_dynamic_library,
                    ACTION_NAMES.cpp_link_executable,
                ],
                flag_groups = [flag_group(flags = ["--coverage"])],
            ),
        ],
        requires = [feature_set(features = ["coverage"])],
    )

    # When tools_path_prefix is set, tell clang to use ld64.lld from the
    # toolchain bin directory when linking.
    _linker_search_flags = [
        "-fuse-ld=lld",
        "--ld-path=%{tools_path_prefix}ld64.lld",
        "-Wl,-force_load_swift_libs",
    ] if "%{tools_path_prefix}" else []

    if platform_name == "macos":
        default_link_flags_feature = feature(
            name = "default_link_flags",
            enabled = True,
            flag_sets = [
                flag_set(
                    actions = _DYNAMIC_LINK_ACTIONS +
                              [_OBJCPP_EXECUTABLE],
                    flag_groups = [
                        flag_group(
                            flags = [
                                "-no-canonical-prefixes",
                                "-target",
                                target_system_name,
                            ] + _linker_search_flags,
                        ),
                    ],
                ),
                flag_set(
                    actions = _DYNAMIC_LINK_ACTIONS +
                              [_OBJCPP_EXECUTABLE],
                    flag_groups = [flag_group(flags = ["-fobjc-link-runtime"])],
                ),
                flag_set(
                    actions = _DYNAMIC_LINK_ACTIONS +
                              [_OBJCPP_EXECUTABLE],
                    flag_groups = [flag_group(flags = ["-dead_strip"])],
                    with_features = [with_feature_set(features = ["opt"])],
                ),
                flag_set(
                    actions = [
                        ACTION_NAMES.cpp_link_dynamic_library,
                        ACTION_NAMES.cpp_link_nodeps_dynamic_library,
                    ],
                    flag_groups = [flag_group(flags = ["-undefined", "dynamic_lookup"])],
                ),
                flag_set(
                    actions = [
                        ACTION_NAMES.cpp_link_executable,
                        ACTION_NAMES.objc_executable,
                        _OBJCPP_EXECUTABLE,
                    ],
                    flag_groups = [flag_group(flags = ["-undefined", "dynamic_lookup"])],
                    with_features = [with_feature_set(features = ["dynamic_linking_mode"])],
                ),
            ],
        )
    else:
        default_link_flags_feature = feature(
            name = "default_link_flags",
            enabled = True,
            flag_sets = [
                flag_set(
                    actions = _DYNAMIC_LINK_ACTIONS +
                              [_OBJCPP_EXECUTABLE],
                    flag_groups = [
                        flag_group(
                            flags = [
                                "-no-canonical-prefixes",
                                "-target",
                                target_system_name,
                            ] + _linker_search_flags,
                        ),
                    ],
                ),
                flag_set(
                    actions = _DYNAMIC_LINK_ACTIONS +
                              [_OBJCPP_EXECUTABLE],
                    flag_groups = [flag_group(flags = ["-fobjc-link-runtime"])],
                ),
                flag_set(
                    actions = _DYNAMIC_LINK_ACTIONS +
                              [_OBJCPP_EXECUTABLE],
                    flag_groups = [flag_group(flags = ["-dead_strip"])],
                    with_features = [with_feature_set(features = ["opt"])],
                ),
                flag_set(
                    actions = [
                        ACTION_NAMES.cpp_link_dynamic_library,
                        ACTION_NAMES.cpp_link_nodeps_dynamic_library,
                        # iOS frameworks use objc_executable to create dylibs.
                        # ld64.lld needs -undefined dynamic_lookup to allow weak
                        # undefined Swift FORCE_LOAD symbols (naming convention
                        # mismatch between Swift 6.x compiler and compat libs).
                        ACTION_NAMES.objc_executable,
                        _OBJCPP_EXECUTABLE,
                    ],
                    flag_groups = [flag_group(flags = ["-undefined", "dynamic_lookup"])],
                ),
            ],
        )

    # We need to pass -object_path_lto in order to get debug symbols when building with LTO.
    lto_object_path_feature = feature(
        name = "lto_object_path",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = _DYNAMIC_LINK_ACTIONS +
                          [_OBJCPP_EXECUTABLE],
                flag_groups = [
                    flag_group(
                        flags = [
                            "-Xlinker",
                            "-object_path_lto",
                            "-Xlinker",
                            "%{output_execpath}.lto.o",
                        ],
                        expand_if_available = "output_execpath",
                    ),
                ],
            ),
        ],
    )

    no_deduplicate_feature = feature(
        name = "no_deduplicate",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = _DYNAMIC_LINK_ACTIONS +
                          [_OBJCPP_EXECUTABLE],
                flag_groups = [
                    flag_group(
                        flags = [
                            "-Xlinker",
                            "-no_deduplicate",
                        ],
                    ),
                ],
                with_features = [
                    with_feature_set(not_features = ["opt"]),
                ],
            ),
        ],
    )

    output_execpath_flags_feature = feature(
        name = "output_execpath_flags",
        flag_sets = [
            flag_set(
                actions = _DYNAMIC_LINK_ACTIONS +
                          [_OBJCPP_EXECUTABLE],
                flag_groups = [
                    flag_group(
                        flags = ["-o", "%{output_execpath}", "LINKED_BINARY=%{output_execpath}"],
                        expand_if_available = "output_execpath",
                    ),
                ],
            ),
        ],
    )

    no_enable_modules_feature = feature(
        name = "no_enable_modules",
        flag_sets = [
            flag_set(
                actions = [ACTION_NAMES.objc_compile, ACTION_NAMES.objcpp_compile],
                flag_groups = [flag_group(flags = ["-fmodule-maps"])],
            ),
        ],
        requires = [feature_set(features = ["use_objc_modules"])],
    )

    pic_feature = feature(
        name = "pic",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.linkstamp_compile,
                    ACTION_NAMES.preprocess_assemble,
                ],
                flag_groups = [
                    flag_group(flags = ["-fPIC"], expand_if_available = "pic"),
                ],
            ),
        ],
    )

    framework_paths_feature = feature(
        name = "framework_paths",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [
                    flag_group(
                        flags = ["-F%{framework_include_paths}"],
                        iterate_over = "framework_include_paths",
                    ),
                ],
            ),
            flag_set(
                actions = [
                    ACTION_NAMES.objc_executable,
                    _OBJCPP_EXECUTABLE,
                ],
                flag_groups = [
                    flag_group(
                        flags = ["-F%{framework_paths}"],
                        iterate_over = "framework_paths",
                    ),
                ],
            ),
        ],
    )

    compiler_output_flags_feature = feature(
        name = "compiler_output_flags",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.assemble,
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.linkstamp_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [
                    flag_group(
                        flags = ["-S"],
                        expand_if_available = "output_assembly_file",
                    ),
                    flag_group(
                        flags = ["-E"],
                        expand_if_available = "output_preprocess_file",
                    ),
                    flag_group(
                        flags = ["-o", "%{output_file}"],
                        expand_if_available = "output_file",
                    ),
                ],
            ),
        ],
    )

    pch_feature = feature(
        name = "pch",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                ],
                flag_groups = [
                    flag_group(
                        flags = [
                            "-include",
                            "%{pch_file}",
                        ],
                        expand_if_available = "pch_file",
                    ),
                ],
            ),
        ],
    )

    include_system_dirs_feature = feature(
        name = "include_system_dirs",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                    ACTION_NAMES.objc_executable,
                    _OBJCPP_EXECUTABLE,
                    ACTION_NAMES.assemble,
                    ACTION_NAMES.preprocess_assemble,
                ],
                flag_groups = [
                    flag_group(
                        flags = [
                            "-isysroot",
                            sdk_dir_str,
                            "-F" + sdk_framework_dir_str,
                            "-F" + platform_developer_framework_dir_str,
                        ],
                    ),
                ],
            ),
        ],
    )

    input_param_flags_feature = feature(
        name = "input_param_flags",
        flag_sets = [
            flag_set(
                actions = all_link_actions +
                          [ACTION_NAMES.cpp_link_static_library],
                flag_groups = [
                    flag_group(
                        flags = ["-L%{library_search_directories}"],
                        iterate_over = "library_search_directories",
                        expand_if_available = "library_search_directories",
                    ),
                ],
            ),
            flag_set(
                actions = all_link_actions +
                          [ACTION_NAMES.cpp_link_static_library],
                flag_groups = [
                    flag_group(
                        flags = ["%{libopts}"],
                        iterate_over = "libopts",
                        expand_if_available = "libopts",
                    ),
                ],
            ),
            flag_set(
                actions = all_link_actions +
                          [ACTION_NAMES.cpp_link_static_library],
                flag_groups = [
                    flag_group(
                        flags = ["-Wl,-force_load,%{whole_archive_linker_params}"],
                        iterate_over = "whole_archive_linker_params",
                        expand_if_available = "whole_archive_linker_params",
                    ),
                ],
            ),
            flag_set(
                actions = all_link_actions +
                          [ACTION_NAMES.cpp_link_static_library],
                flag_groups = [
                    flag_group(
                        flags = ["%{linker_input_params}"],
                        iterate_over = "linker_input_params",
                        expand_if_available = "linker_input_params",
                    ),
                ],
            ),
            flag_set(
                actions = all_link_actions +
                          [ACTION_NAMES.cpp_link_static_library],
                flag_groups = [
                    flag_group(
                        iterate_over = "libraries_to_link",
                        flag_groups = [
                            flag_group(
                                iterate_over = "libraries_to_link.object_files",
                                flag_groups = [
                                    flag_group(
                                        flags = ["%{libraries_to_link.object_files}"],
                                        expand_if_false = "libraries_to_link.is_whole_archive",
                                    ),
                                    flag_group(
                                        flags = ["-Wl,-force_load,%{libraries_to_link.object_files}"],
                                        expand_if_true = "libraries_to_link.is_whole_archive",
                                    ),
                                ],
                                expand_if_equal = variable_with_value(
                                    name = "libraries_to_link.type",
                                    value = "object_file_group",
                                ),
                            ),
                            flag_group(
                                flag_groups = [
                                    flag_group(
                                        flags = ["%{libraries_to_link.name}"],
                                        expand_if_false = "libraries_to_link.is_whole_archive",
                                    ),
                                    flag_group(
                                        flags = ["-Wl,-force_load,%{libraries_to_link.name}"],
                                        expand_if_true = "libraries_to_link.is_whole_archive",
                                    ),
                                ],
                                expand_if_equal = variable_with_value(
                                    name = "libraries_to_link.type",
                                    value = "object_file",
                                ),
                            ),
                            flag_group(
                                flag_groups = [
                                    flag_group(
                                        flags = ["%{libraries_to_link.name}"],
                                        expand_if_false = "libraries_to_link.is_whole_archive",
                                    ),
                                    flag_group(
                                        flags = ["-Wl,-force_load,%{libraries_to_link.name}"],
                                        expand_if_true = "libraries_to_link.is_whole_archive",
                                    ),
                                ],
                                expand_if_equal = variable_with_value(
                                    name = "libraries_to_link.type",
                                    value = "interface_library",
                                ),
                            ),
                            flag_group(
                                flag_groups = [
                                    flag_group(
                                        flags = ["%{libraries_to_link.name}"],
                                        expand_if_false = "libraries_to_link.is_whole_archive",
                                    ),
                                    flag_group(
                                        flags = ["-Wl,-force_load,%{libraries_to_link.name}"],
                                        expand_if_true = "libraries_to_link.is_whole_archive",
                                    ),
                                ],
                                expand_if_equal = variable_with_value(
                                    name = "libraries_to_link.type",
                                    value = "static_library",
                                ),
                            ),
                            flag_group(
                                flag_groups = [
                                    flag_group(
                                        flags = ["-l%{libraries_to_link.name}"],
                                        expand_if_false = "libraries_to_link.is_whole_archive",
                                    ),
                                    flag_group(
                                        flags = ["-Wl,-force_load,-l%{libraries_to_link.name}"],
                                        expand_if_true = "libraries_to_link.is_whole_archive",
                                    ),
                                ],
                                expand_if_equal = variable_with_value(
                                    name = "libraries_to_link.type",
                                    value = "dynamic_library",
                                ),
                            ),
                            flag_group(
                                flag_groups = [
                                    flag_group(
                                        flags = ["-l:%{libraries_to_link.name}"],
                                        expand_if_false = "libraries_to_link.is_whole_archive",
                                    ),
                                    flag_group(
                                        flags = ["-Wl,-force_load,-l:%{libraries_to_link.name}"],
                                        expand_if_true = "libraries_to_link.is_whole_archive",
                                    ),
                                ],
                                expand_if_equal = variable_with_value(
                                    name = "libraries_to_link.type",
                                    value = "versioned_dynamic_library",
                                ),
                            ),
                        ],
                        expand_if_available = "libraries_to_link",
                    ),
                ],
            ),
        ],
    )

    lipo_feature = feature(
        name = "lipo",
        flag_sets = [
            flag_set(
                actions = [ACTION_NAMES.c_compile, ACTION_NAMES.cpp_compile],
                flag_groups = [flag_group(flags = ["-fripa"])],
            ),
        ],
        requires = [
            feature_set(features = ["autofdo"]),
            feature_set(features = ["fdo_optimize"]),
            feature_set(features = ["fdo_instrument"]),
        ],
    )

    apple_env_feature = feature(
        name = "apple_env",
        env_sets = [
            env_set(
                actions = [
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.assemble,
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                    _OBJC_ARCHIVE,
                    ACTION_NAMES.objc_fully_link,
                    ACTION_NAMES.cpp_link_executable,
                    ACTION_NAMES.cpp_link_dynamic_library,
                    ACTION_NAMES.cpp_link_nodeps_dynamic_library,
                    ACTION_NAMES.cpp_link_static_library,
                    ACTION_NAMES.objc_executable,
                    _OBJCPP_EXECUTABLE,
                    ACTION_NAMES.linkstamp_compile,
                ],
                env_entries = [
                    env_entry(
                        key = "DEVELOPER_DIR",
                        value = _DEVELOPER_DIR,
                    ),
                    env_entry(
                        key = "XCODE_VERSION_OVERRIDE",
                        value = xcode_version_str,
                    ),
                    env_entry(
                        key = "APPLE_SDK_VERSION_OVERRIDE",
                        value = sdk_version_str,
                    ),
                    env_entry(
                        key = "APPLE_SDK_PLATFORM",
                        value = sdk_platform_str,
                    ),
                    env_entry(
                        key = "ZERO_AR_DATE",
                        value = "1",
                    ),
                ] + [env_entry(key = key, value = value) for key, value in ctx.attr.extra_env.items()],
            ),
        ],
    )

    if platform_name == "macos":
        apply_implicit_frameworks_feature = feature(
            name = "apply_implicit_frameworks",
            enabled = True,
            flag_sets = [
                flag_set(
                    actions = [
                        ACTION_NAMES.objc_executable,
                        _OBJCPP_EXECUTABLE,
                    ],
                    flag_groups = [flag_group(flags = ["-framework", "Foundation"])],
                    with_features = [with_feature_set(not_features = ["kernel_extension"])],
                ),
            ],
        )
    else:
        apply_implicit_frameworks_feature = feature(
            name = "apply_implicit_frameworks",
            enabled = True,
            flag_sets = [
                flag_set(
                    actions = [
                        ACTION_NAMES.objc_executable,
                        _OBJCPP_EXECUTABLE,
                    ],
                    flag_groups = [
                        flag_group(
                            flags = ["-framework", "Foundation", "-framework", "UIKit"],
                        ),
                    ],
                ),
            ],
        )

    llvm_coverage_map_format_feature = feature(
        name = "llvm_coverage_map_format",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [
                    flag_group(
                        flags = ["-fprofile-instr-generate", "-fcoverage-mapping", "-g"],
                    ),
                ],
            ),
            flag_set(
                actions = _DYNAMIC_LINK_ACTIONS +
                          [_OBJCPP_EXECUTABLE],
                flag_groups = [flag_group(flags = ["-fprofile-instr-generate"])],
            ),
        ],
        requires = [feature_set(features = ["coverage"])],
    )

    coverage_prefix_map_feature = feature(
        name = "coverage_prefix_map",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [
                    flag_group(
                        flags = ["-fcoverage-prefix-map=__BAZEL_EXECUTION_ROOT__=."],
                    ),
                ],
            ),
        ],
        requires = [feature_set(features = ["coverage"])],
    )

    coverage_prefix_map_absolute_sources_non_hermetic_private_feature = feature(
        name = "_coverage_prefix_map_absolute_sources_non_hermetic",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [
                    flag_group(
                        flags = ["-fcoverage-prefix-map=__BAZEL_EXECUTION_ROOT__=__BAZEL_EXECUTION_ROOT_CANONICAL__"],
                    ),
                ],
            ),
        ],
        env_sets = [
            env_set(
                actions = [
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                env_entries = [
                    env_entry(
                        key = "COVERAGE_PREFIX_MAP_USE_ABSOLUTE_CWD_PATH",
                        value = "1",
                    ),
                ],
            ),
        ],
        requires = [feature_set(features = ["coverage"])],
    )

    force_pic_flags_feature = feature(
        name = "force_pic_flags",
        flag_sets = [
            flag_set(
                actions = [ACTION_NAMES.cpp_link_executable],
                flag_groups = [
                    flag_group(
                        flags = ["-Wl,-pie"],
                        expand_if_available = "force_pic",
                    ),
                ],
            ),
        ],
    )

    sysroot_feature = feature(
        name = "sysroot",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.assemble,
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.cpp_link_executable,
                    ACTION_NAMES.cpp_link_dynamic_library,
                    ACTION_NAMES.cpp_link_nodeps_dynamic_library,
                    ACTION_NAMES.linkstamp_compile,
                    ACTION_NAMES.clif_match,
                ],
                flag_groups = [
                    flag_group(
                        flags = ["--sysroot=%{sysroot}"],
                        expand_if_available = "sysroot",
                    ),
                ],
            ),
        ],
    )

    autofdo_feature = feature(
        name = "autofdo",
        flag_sets = [
            flag_set(
                actions = [ACTION_NAMES.c_compile, ACTION_NAMES.cpp_compile],
                flag_groups = [
                    flag_group(
                        flags = [
                            "-fauto-profile=%{fdo_profile_path}",
                            "-fprofile-correction",
                        ],
                        expand_if_available = "fdo_profile_path",
                    ),
                ],
            ),
        ],
        provides = ["profile"],
    )

    link_libcpp_feature = feature(
        name = "link_libc++",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = _DYNAMIC_LINK_ACTIONS +
                          [_OBJCPP_EXECUTABLE],
                flag_groups = [flag_group(flags = ["-lc++"])],
                with_features = [with_feature_set(not_features = ["kernel_extension"])],
            ),
        ],
    )

    objc_actions_feature = feature(
        name = "objc_actions",
        implies = [
            "objc-compile",
            "objc++-compile",
            "objc-fully-link",
            "objc-archive",
            ACTION_NAMES.objc_executable,
            _OBJCPP_EXECUTABLE,
            "assemble",
            "preprocess-assemble",
            "c-compile",
            "c++-compile",
            "c++-link-static-library",
            "c++-link-dynamic-library",
            "c++-link-nodeps-dynamic-library",
            "c++-link-executable",
        ],
    )

    unfiltered_compile_flags_feature = feature(
        name = "unfiltered_compile_flags",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.assemble,
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.cpp_module_codegen,
                    ACTION_NAMES.linkstamp_compile,
                ],
                flag_groups = [
                    flag_group(
                        flags = [
                            "-no-canonical-prefixes",
                            "-Wno-builtin-macro-redefined",
                            "-D__DATE__=\"redacted\"",
                            "-D__TIMESTAMP__=\"redacted\"",
                            "-D__TIME__=\"redacted\"",
                            "-target",
                            target_system_name,
                        ],
                    ),
                ],
            ),
        ],
    )

    linker_param_file_feature = feature(
        name = "linker_param_file",
        flag_sets = [
            flag_set(
                actions = _DYNAMIC_LINK_ACTIONS + _STATIC_LINK_ACTIONS +
                          [
                              _OBJC_ARCHIVE,
                              _OBJCPP_EXECUTABLE,
                          ],
                flag_groups = [
                    flag_group(
                        flags = ["@%{linker_param_file}"],
                        expand_if_available = "linker_param_file",
                    ),
                ],
            ),
        ],
    )

    relative_ast_path_feature = feature(
        name = "relative_ast_path",
        env_sets = [
            env_set(
                actions = all_link_actions + [
                    ACTION_NAMES.objc_executable,
                    _OBJCPP_EXECUTABLE,
                ],
                env_entries = [
                    env_entry(
                        key = "RELATIVE_AST_PATH",
                        value = "true",
                    ),
                ],
            ),
        ],
    )

    archiver_flags_feature = feature(
        name = "archiver_flags",
        flag_sets = [
            flag_set(
                actions = [ACTION_NAMES.cpp_link_static_library],
                flag_groups = [
                    flag_group(
                        flags = [
                            "-D",
                            "-no_warning_for_no_symbols",
                            "-static",
                            "-o",
                            "%{output_execpath}",
                        ],
                        expand_if_available = "output_execpath",
                    ),
                ],
            ),
        ],
    )

    fdo_optimize_feature = feature(
        name = "fdo_optimize",
        flag_sets = [
            flag_set(
                actions = [ACTION_NAMES.c_compile, ACTION_NAMES.cpp_compile],
                flag_groups = [
                    flag_group(
                        flags = [
                            "-fprofile-use=%{fdo_profile_path}",
                            "-Wno-profile-instr-unprofiled",
                            "-Wno-profile-instr-out-of-date",
                            "-fprofile-correction",
                        ],
                        expand_if_available = "fdo_profile_path",
                    ),
                ],
            ),
        ],
        provides = ["profile"],
    )

    no_objc_arc_feature = feature(
        name = "no_objc_arc",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.assemble,
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [
                    flag_group(
                        flags = ["-fno-objc-arc"],
                        expand_if_available = "no_objc_arc",
                    ),
                ],
            ),
        ],
    )

    if is_tvos:
        cpp_linker_flags_feature = feature(
            name = "cpp_linker_flags",
            flag_sets = [
                flag_set(
                    actions = [
                        ACTION_NAMES.cpp_link_executable,
                        ACTION_NAMES.cpp_link_dynamic_library,
                    ],
                    flag_groups = [
                        flag_group(
                            flags = ["-lc++", "-target", target_system_name],
                        ),
                    ],
                ),
            ],
        )
    else:
        cpp_linker_flags_feature = feature(name = "cpp_linker_flags")

    debug_prefix_map_pwd_is_dot_feature = feature(
        name = "debug_prefix_map_pwd_is_dot",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.assemble,
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.linkstamp_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [flag_group(flags = [
                    "-fdebug-prefix-map=__BAZEL_EXECUTION_ROOT__=.",
                ])],
            ),
        ],
    )

    remap_xcode_path_feature = feature(
        name = "remap_xcode_path",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.assemble,
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.linkstamp_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [flag_group(flags = [
                    "-fdebug-prefix-map=__BAZEL_XCODE_DEVELOPER_DIR__=DEVELOPER_DIR",
                ])],
            ),
        ],
    )

    linkstamps_feature = feature(
        name = "linkstamps",
        flag_sets = [
            flag_set(
                actions = _CPP_DYNAMIC_LINK_ACTIONS,
                flag_groups = [
                    flag_group(
                        flags = ["%{linkstamp_paths}"],
                        iterate_over = "linkstamp_paths",
                        expand_if_available = "linkstamp_paths",
                    ),
                ],
            ),
        ],
    )

    include_paths_feature = feature(
        name = "include_paths",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.linkstamp_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [
                    flag_group(
                        flags = ["-iquote", "%{quote_include_paths}"],
                        iterate_over = "quote_include_paths",
                    ),
                    flag_group(
                        flags = ["-I%{include_paths}"],
                        iterate_over = "include_paths",
                    ),
                    flag_group(
                        flags = ["-isystem", "%{system_include_paths}"],
                        iterate_over = "system_include_paths",
                    ),
                ],
            ),
        ],
    )

    default_compile_flags_feature = feature(
        name = "default_compile_flags",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.assemble,
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.linkstamp_compile,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [
                    flag_group(
                        flags = [
                            "-D_FORTIFY_SOURCE=1",
                        ],
                    ),
                ],
                with_features = [with_feature_set(not_features = ["asan"])],
            ),
            flag_set(
                actions = [
                    ACTION_NAMES.assemble,
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.linkstamp_compile,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [
                    flag_group(
                        flags = [
                            "-fstack-protector",
                            "-fcolor-diagnostics",
                            "-Wall",
                            "-Wthread-safety",
                            "-Wself-assign",
                            "-fno-omit-frame-pointer",
                        ],
                    ),
                ],
            ),
            flag_set(
                actions = [
                    ACTION_NAMES.assemble,
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.linkstamp_compile,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [flag_group(flags = ["-O0", "-DDEBUG"])],
                with_features = [with_feature_set(features = ["fastbuild"])],
            ),
            flag_set(
                actions = [
                    ACTION_NAMES.assemble,
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.linkstamp_compile,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [
                    flag_group(
                        flags = [
                            "-g0",
                            "-O2",
                            "-DNDEBUG",
                        ],
                    ),
                ],
                with_features = [with_feature_set(features = ["opt"])],
            ),
            flag_set(
                actions = [
                    ACTION_NAMES.assemble,
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.linkstamp_compile,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [flag_group(flags = ["-g"])],
                with_features = [with_feature_set(features = ["dbg"])],
            ),
        ],
    )

    ns_block_assertions_feature = feature(
        name = "ns_block_assertions",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.assemble,
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.linkstamp_compile,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [flag_group(flags = ["-DNS_BLOCK_ASSERTIONS=1"])],
                with_features = [with_feature_set(features = ["opt"])],
            ),
        ],
    )

    dead_strip_feature = feature(
        name = "dead_strip",
        flag_sets = [
            flag_set(
                actions = _DYNAMIC_LINK_ACTIONS +
                          [_OBJCPP_EXECUTABLE],
                flag_groups = [flag_group(flags = ["-dead_strip"])],
            ),
        ],
    )

    oso_prefix_feature = feature(
        name = "oso_prefix_is_pwd",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = _DYNAMIC_LINK_ACTIONS +
                          [_OBJCPP_EXECUTABLE],
                flag_groups = [flag_group(flags = ["-Wl,-oso_prefix,__BAZEL_EXECUTION_ROOT__/"])],
            ),
        ],
    )

    generate_dsym_file_feature = feature(
        name = "generate_dsym_file",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                    ACTION_NAMES.objc_executable,
                    _OBJCPP_EXECUTABLE,
                ],
                flag_groups = [flag_group(flags = ["-g"])],
            ),
            flag_set(
                actions = _DYNAMIC_LINK_ACTIONS,
                flag_groups = [
                    flag_group(
                        flags = [
                            "DSYM_HINT_DSYM_PATH=%{dsym_path}",
                        ],
                        expand_if_available = "dsym_path",
                    ),
                ],
            ),
            flag_set(
                actions = [ACTION_NAMES.objc_executable, _OBJCPP_EXECUTABLE],
                flag_groups = [
                    flag_group(
                        flags = [
                            "LINKED_BINARY=%{linked_binary}",
                            "DSYM_PATH=%{dsym_path}",
                        ],
                    ),
                ],
            ),
        ],
    )

    # Kernel extensions for Apple Silicon are arm64e.
    if (ctx.attr.cpu == "darwin_x86_64" or
        ctx.attr.cpu == "darwin_arm64e"):
        kernel_extension_feature = feature(
            name = "kernel_extension",
            flag_sets = [
                flag_set(
                    actions = [ACTION_NAMES.objc_executable, _OBJCPP_EXECUTABLE],
                    flag_groups = [
                        flag_group(
                            flags = [
                                "-nostdlib",
                                "-lkmod",
                                "-lkmodc++",
                                "-lcc_kext",
                                "-Xlinker",
                                "-kext",
                            ],
                        ),
                    ],
                ),
            ],
        )
    else:
        kernel_extension_feature = feature(name = "kernel_extension")

    apply_default_warnings_feature = feature(
        name = "apply_default_warnings",
        flag_sets = [
            flag_set(
                actions = [ACTION_NAMES.objc_compile, ACTION_NAMES.objcpp_compile],
                flag_groups = [
                    flag_group(
                        flags = [
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
                    ),
                ],
            ),
        ],
    )

    dependency_file_feature = feature(
        name = "dependency_file",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.assemble,
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                    ACTION_NAMES.cpp_header_parsing,
                ],
                flag_groups = [
                    flag_group(
                        flags = ["-MD", "-MF", "%{dependency_file}"],
                        expand_if_available = "dependency_file",
                    ),
                ],
            ),
        ],
    )

    serialized_diagnostics_file_feature = feature(
        name = "serialized_diagnostics_file",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.assemble,
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                    ACTION_NAMES.cpp_header_parsing,
                ],
                flag_groups = [
                    flag_group(
                        flags = ["--serialize-diagnostics", "%{serialized_diagnostics_file}"],
                        expand_if_available = "serialized_diagnostics_file",
                    ),
                ],
            ),
        ],
    )

    preprocessor_defines_feature = feature(
        name = "preprocessor_defines",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.linkstamp_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [
                    flag_group(
                        flags = ["-D%{preprocessor_defines}"],
                        iterate_over = "preprocessor_defines",
                    ),
                ],
            ),
        ],
    )

    fdo_instrument_feature = feature(
        name = "fdo_instrument",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_link_dynamic_library,
                    ACTION_NAMES.cpp_link_nodeps_dynamic_library,
                    ACTION_NAMES.cpp_link_executable,
                ],
                flag_groups = [
                    flag_group(
                        flags = [
                            "-fprofile-generate=%{fdo_instrument_path}",
                            "-fno-data-sections",
                        ],
                        expand_if_available = "fdo_instrument_path",
                    ),
                ],
            ),
        ],
        provides = ["profile"],
    )

    if platform_name == "macos":
        link_cocoa_feature = feature(
            name = "link_cocoa",
            flag_sets = [
                flag_set(
                    actions = [ACTION_NAMES.objc_executable, _OBJCPP_EXECUTABLE],
                    flag_groups = [flag_group(flags = ["-framework", "Cocoa"])],
                ),
            ],
        )
    else:
        link_cocoa_feature = feature(name = "link_cocoa")

    user_compile_flags_feature = feature(
        name = "user_compile_flags",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.assemble,
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.cpp_module_codegen,
                    ACTION_NAMES.linkstamp_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [
                    flag_group(
                        flags = ["%{user_compile_flags}"],
                        iterate_over = "user_compile_flags",
                        expand_if_available = "user_compile_flags",
                    ),
                ],
            ),
        ],
    )

    headerpad_feature = feature(
        name = "headerpad",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = _DYNAMIC_LINK_ACTIONS +
                          [_OBJCPP_EXECUTABLE],
                flag_groups = [flag_group(flags = ["-headerpad_max_install_names"])],
            ),
        ],
    )

    generate_linkmap_feature = feature(
        name = "generate_linkmap",
        flag_sets = [
            flag_set(
                actions = _DYNAMIC_LINK_ACTIONS +
                          [_OBJCPP_EXECUTABLE],
                flag_groups = [
                    flag_group(
                        flags = [
                            "-Xlinker",
                            "-map",
                            "-Xlinker",
                            "%{linkmap_exec_path}",
                        ],
                        expand_if_available = "linkmap_exec_path",
                    ),
                    flag_group(
                        flags = [
                            "-Xlinker",
                            "-map",
                            "-Xlinker",
                            "%{output_execpath}.map",
                        ],
                        expand_if_available = "output_execpath",
                        expand_if_not_available = "linkmap_exec_path",
                    ),
                ],
            ),
        ],
    )

    set_install_name = feature(
        name = "set_install_name",
        enabled = getattr(ctx.fragments.cpp, "do_not_use_macos_set_install_name", True),
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.cpp_link_dynamic_library,
                ],
                flag_groups = [
                    flag_group(
                        flags = [
                            "-Xlinker",
                            "-install_name",
                            "-Xlinker",
                            "@rpath/%{runtime_solib_name}",
                        ],
                        expand_if_available = "runtime_solib_name",
                    ),
                ],
            ),
        ],
    )

    asan_feature = feature(
        name = "asan",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [
                    flag_group(flags = ["-fsanitize=address"]),
                ],
                with_features = [
                    with_feature_set(features = ["asan"]),
                ],
            ),
            flag_set(
                actions = _DYNAMIC_LINK_ACTIONS +
                          [_OBJCPP_EXECUTABLE],
                flag_groups = [
                    flag_group(flags = ["-fsanitize=address"]),
                ],
                with_features = [
                    with_feature_set(features = ["asan"]),
                ],
            ),
        ],
    )

    tsan_feature = feature(
        name = "tsan",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [
                    flag_group(flags = ["-fsanitize=thread"]),
                ],
                with_features = [
                    with_feature_set(features = ["tsan"]),
                ],
            ),
            flag_set(
                actions = _DYNAMIC_LINK_ACTIONS +
                          [_OBJCPP_EXECUTABLE],
                flag_groups = [
                    flag_group(flags = ["-fsanitize=thread"]),
                ],
                with_features = [
                    with_feature_set(features = ["tsan"]),
                ],
            ),
        ],
    )

    ubsan_feature = feature(
        name = "ubsan",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [
                    flag_group(flags = ["-fsanitize=undefined"]),
                ],
                with_features = [
                    with_feature_set(features = ["ubsan"]),
                ],
            ),
            flag_set(
                actions = _DYNAMIC_LINK_ACTIONS +
                          [_OBJCPP_EXECUTABLE],
                flag_groups = [
                    flag_group(flags = ["-fsanitize=undefined"]),
                ],
                with_features = [
                    with_feature_set(features = ["ubsan"]),
                ],
            ),
        ],
    )

    default_sanitizer_flags_feature = feature(
        name = "default_sanitizer_flags",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [
                    flag_group(
                        flags = [
                            "-gline-tables-only",
                            "-fno-omit-frame-pointer",
                            "-fno-sanitize-recover=all",
                        ],
                    ),
                ],
                with_features = [
                    with_feature_set(features = ["asan"]),
                    with_feature_set(features = ["tsan"]),
                    with_feature_set(features = ["ubsan"]),
                ],
            ),
        ],
    )

    suppress_warnings_feature = feature(
        name = "suppress_warnings",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [flag_group(flags = ["-w"])],
            ),
        ],
    )

    treat_warnings_as_errors_feature = feature(
        name = "treat_warnings_as_errors",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.objc_compile,
                    ACTION_NAMES.objcpp_compile,
                ],
                flag_groups = [flag_group(flags = ["-Werror"])],
            ),
            flag_set(
                actions = _DYNAMIC_LINK_ACTIONS +
                          [_OBJCPP_EXECUTABLE],
                flag_groups = [flag_group(flags = ["-Wl,-fatal_warnings"])],
            ),
        ],
    )

    # As of Xcode 15, linker warnings are emitted if duplicate `-l` options are
    # present. Until such linkopts can be deduped by bazel itself, we disable
    # these warnings.
    no_warn_duplicate_libraries_feature = feature(
        name = "no_warn_duplicate_libraries",
        enabled = "no_warn_duplicate_libraries" in ctx.features,
        flag_sets = [
            flag_set(
                actions = _DYNAMIC_LINK_ACTIONS +
                          [_OBJCPP_EXECUTABLE],
                flag_groups = [
                    flag_group(
                        flags = [
                            "-Wl,-no_warn_duplicate_libraries",
                        ],
                    ),
                ],
            ),
        ],
    )

    dynamic_linking_mode_feature = feature(name = "dynamic_linking_mode")

    features = [
        # Marker features
        feature(name = "archive_param_file", enabled = True),
        feature(name = "compiler_param_file"),
        feature(name = "compile_all_modules"),
        feature(name = "coverage"),
        feature(name = "dbg"),
        feature(name = "exclude_private_headers_in_module_maps"),
        feature(name = "fastbuild"),
        feature(name = "has_configured_linker_path"),
        feature(name = "module_maps", enabled = True),
        feature(name = "no_legacy_features"),
        feature(name = "only_doth_headers_in_module_maps"),
        feature(name = "opt"),
        feature(name = "parse_headers"),
        feature(name = "no_dotd_file"),
        feature(name = "supports_pic", enabled = True),

        # Features with more configuration
        link_libcpp_feature,
        default_compile_flags_feature,
        ns_block_assertions_feature,
        debug_prefix_map_pwd_is_dot_feature,
        remap_xcode_path_feature,
        generate_dsym_file_feature,
        generate_linkmap_feature,
        oso_prefix_feature,
        contains_objc_source_feature,
        objc_actions_feature,
        strip_debug_symbols_feature,
        shared_flag_feature,
        kernel_extension_feature,
        linkstamps_feature,
        output_execpath_flags_feature,
        archiver_flags_feature,
        runtime_root_flags_feature,
        input_param_flags_feature,
        objc_link_flag_feature,
        force_pic_flags_feature,
        pch_feature,
        use_objc_modules_feature,
        no_enable_modules_feature,
        apply_default_warnings_feature,
        includes_feature,
        include_paths_feature,
        sysroot_feature,
        dependency_file_feature,
        serialized_diagnostics_file_feature,
        pic_feature,
        preprocessor_defines_feature,
        framework_paths_feature,
        fdo_instrument_feature,
        fdo_optimize_feature,
        autofdo_feature,
        lipo_feature,
        lto_object_path_feature,
        llvm_coverage_map_format_feature,
        gcc_coverage_map_format_feature,
        coverage_prefix_map_feature,
        coverage_prefix_map_absolute_sources_non_hermetic_private_feature,
        apply_default_compiler_flags_feature,
        include_system_dirs_feature,
        headerpad_feature,
        objc_arc_feature,
        no_objc_arc_feature,
        apple_env_feature,
        relative_ast_path_feature,
        user_link_flags_feature,
        default_link_flags_feature,
        no_deduplicate_feature,
        dead_strip_feature,
        cpp_linker_flags_feature,
        apply_implicit_frameworks_feature,
        link_cocoa_feature,
        apply_simulator_compiler_flags_feature,
        unfiltered_cxx_flags_feature,
        user_compile_flags_feature,
        unfiltered_compile_flags_feature,
        linker_param_file_feature,
        compiler_input_flags_feature,
        compiler_output_flags_feature,
        set_install_name,
        asan_feature,
        tsan_feature,
        ubsan_feature,
        default_sanitizer_flags_feature,
        suppress_warnings_feature,
        treat_warnings_as_errors_feature,
        no_warn_duplicate_libraries_feature,
        external_include_paths_feature,
        header_parsing_env_feature,
    ]

    if platform_name == "macos":
        features.append(dynamic_linking_mode_feature)

    # macOS artifact name patterns differ from the defaults only for dynamic
    # libraries.
    artifact_name_patterns = [
        artifact_name_pattern(
            category_name = "dynamic_library",
            prefix = "lib",
            extension = ".dylib",
        ),
    ]

    make_variables = [
        make_variable(
            name = "STACK_FRAME_UNLIMITED",
            value = "-Wframe-larger-than=100000000 -Wno-vla",
        ),
    ]

    tool_paths = {
        "ar": "libtool",
        "cpp": "/usr/bin/cpp",
        "dwp": "/usr/bin/dwp",
        "gcc": "cc_wrapper.sh",
        "gcov": "/usr/bin/gcov",
        "ld": "%{tools_path_prefix}ld",
        "nm": "/usr/bin/nm",
        "objdump": "/usr/bin/objdump",
        "strip": "/usr/bin/strip",
    }

    tool_paths.update(ctx.attr.tool_paths_overrides)

    out = ctx.actions.declare_file(ctx.label.name)
    ctx.actions.write(out, "Fake executable")
    return [
        cc_common.create_cc_toolchain_config_info(
            ctx = ctx,
            features = features,
            action_configs = action_configs,
            artifact_name_patterns = artifact_name_patterns,
            cxx_builtin_include_directories = ctx.attr.cxx_builtin_include_directories,
            toolchain_identifier = toolchain_identifier,
            host_system_name = host_system_name,
            target_system_name = target_system_name,
            target_cpu = target_cpu,
            target_libc = target_libc,
            compiler = compiler,
            abi_version = abi_version,
            abi_libc_version = abi_libc_version,
            tool_paths = [tool_path(name = name, path = path) for (name, path) in tool_paths.items()],
            make_variables = make_variables,
            builtin_sysroot = builtin_sysroot,
            cc_target_os = cc_target_os,
        ),
        DefaultInfo(
            executable = out,
        ),
    ]

cc_toolchain_config = rule(
    attrs = {
        "compiler": attr.string(),
        "cpu": attr.string(mandatory = True),
        "cxx_builtin_include_directories": attr.string_list(),
        "extra_env": attr.string_dict(),
        "tool_paths_overrides": attr.string_dict(),
        "_xcode_config": attr.label(default = configuration_field(
            fragment = "apple",
            name = "xcode_config_label",
        )),
    },
    executable = True,
    fragments = ["apple", "cpp"],
    implementation = _impl,
)
