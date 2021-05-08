load("@bazel_skylib//lib:paths.bzl", "paths")
load(
    "@bazel_tools//tools/cpp:lib_cc_configure.bzl",
    "resolve_labels",
)

def _compile_cc_file(rctx, developer_dir, src_name, out_name):
    rctx.report_progress("Compiling {}".format(paths.basename(src_name)))
    env = rctx.os.environ
    if developer_dir:
        bin_root = developer_dir + "/Toolchains/XcodeDefault.xctoolchain/usr/bin"
    else:
        bin_root = rctx.which("clang").dirname
    cc = "{}/clang".format(bin_root)
    result = rctx.execute([
        "env",
        "-i",
        cc,
    ] + ([
        "-B",
        bin_root,
        "-fuse-ld=lld",
    ] if developer_dir else []) + [
        "-std=c++11",
        "-lstdc++",
        "-O3",
        "-o",
        out_name,
        src_name,
    ], 30)
    if (result.return_code != 0):
        error_msg = (
            "return code {code}, stderr: {err}, stdout: {out}"
        ).format(
            code = result.return_code,
            err = result.stderr,
            out = result.stdout,
        )
        fail(out_name + " failed to generate. Please file an issue at " +
             "https://github.com/apple-cross-toolchain/rules_applecross/issues with the following:\n" +
             error_msg)

def _apple_cross_toolchain_impl(rctx):
    paths = resolve_labels(rctx, [
        "@bazel_tools//tools/objc:libtool.sh",
        "@bazel_tools//tools/objc:libtool_check_unique.cc",
        "@bazel_tools//tools/objc:make_hashed_objlist.py",
        "@build_bazel_rules_swift//third_party/bazel_protos:BUILD",
        "@build_bazel_rules_swift//third_party/bazel_protos:worker_protocol.proto",
        "@build_bazel_rules_swift//tools/common:BUILD",
        "@build_bazel_rules_swift//tools/common:file_system.cc",
        "@build_bazel_rules_swift//tools/common:file_system.h",
        "@build_bazel_rules_swift//tools/common:path_utils.cc",
        "@build_bazel_rules_swift//tools/common:path_utils.h",
        "@build_bazel_rules_swift//tools/common:process.cc",
        "@build_bazel_rules_swift//tools/common:process.h",
        "@build_bazel_rules_swift//tools/common:string_utils.cc",
        "@build_bazel_rules_swift//tools/common:string_utils.h",
        "@build_bazel_rules_swift//tools/common:temp_file.h",
        "@build_bazel_rules_swift//tools/worker:BUILD",
        "@build_bazel_rules_swift//tools/worker:compile_with_worker.cc",
        "@build_bazel_rules_swift//tools/worker:compile_with_worker.h",
        "@build_bazel_rules_swift//tools/worker:compile_without_worker.cc",
        "@build_bazel_rules_swift//tools/worker:compile_without_worker.h",
        "@build_bazel_rules_swift//tools/worker:no_worker_main.cc",
        "@build_bazel_rules_swift//tools/worker:output_file_map.cc",
        "@build_bazel_rules_swift//tools/worker:output_file_map.h",
        "@build_bazel_rules_swift//tools/worker:swift_runner.h",
        "@build_bazel_rules_swift//tools/worker:work_processor.cc",
        "@build_bazel_rules_swift//tools/worker:work_processor.h",
        "@build_bazel_rules_swift//tools/worker:worker_main.cc",
        "@rules_applecross//toolchain:BUILD.tpl",
        "@rules_applecross//toolchain:cc_toolchain_config.bzl.tpl",
        "@rules_applecross//toolchain:cc_wrapper.sh.tpl",
        "@rules_applecross//toolchain:swift_toolchain.bzl.tpl",
        "@rules_applecross//toolchain:repositories.bzl.tpl",
        "@rules_applecross//toolchain:swift_autoconfiguration.bzl.tpl",
        "@rules_applecross//toolchain:swift_runner.cc.tpl",
        "@rules_applecross//toolchain:wrapped_clang.cc.tpl",
        "@rules_applecross//toolchain:xcrunwrapper.sh.tpl",
    ])

    repo_path = str(rctx.path(""))
    relative_path_prefix = "external/{}/".format(rctx.name)
    toolchain_path_prefix = relative_path_prefix
    developer_dir = ""
    tools_path_prefix = ""
    xcode_toolchain_bindir = "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/"
    if rctx.attr.xcode_urls:
        developer_dir = "Xcode.app/Contents/Developer"
        tools_path_prefix = toolchain_path_prefix + xcode_toolchain_bindir

    substitutions = {
        "%{cc}": relative_path_prefix + "wrapped_clang",
        "%{repo_name}": rctx.name,
        "%{repo_path}": repo_path,
        "%{toolchain_path_prefix}": toolchain_path_prefix,
        "%{tools_path_prefix}": tools_path_prefix,
    }

    # Setup C++ toolchain
    rctx.template(
        "BUILD",
        paths["@rules_applecross//toolchain:BUILD.tpl"],
        substitutions,
    )
    rctx.template(
        "cc_toolchain_config.bzl",
        paths["@rules_applecross//toolchain:cc_toolchain_config.bzl.tpl"],
        substitutions,
    )
    rctx.template(
        "cc_wrapper.sh",
        paths["@rules_applecross//toolchain:cc_wrapper.sh.tpl"],
        substitutions,
    )
    rctx.template(
        "xcrunwrapper.sh",
        paths["@rules_applecross//toolchain:xcrunwrapper.sh.tpl"],
        substitutions,
    )
    rctx.template(
        "libtool",
        paths["@bazel_tools//tools/objc:libtool.sh"],
        substitutions,
    )
    rctx.symlink(
        paths["@bazel_tools//tools/objc:make_hashed_objlist.py"],
        "make_hashed_objlist.py",
    )
    libtool_check_unique_src_path = str(rctx.path(
        paths["@bazel_tools//tools/objc:libtool_check_unique.cc"],
    ))

    if rctx.attr.xcode_urls:
        rctx.download_and_extract(
            url = rctx.attr.xcode_urls,
            sha256 = rctx.attr.xcode_sha256,
            stripPrefix = rctx.attr.xcode_strip_prefix,
        )

    if rctx.attr.clang_urls:
        rctx.download_and_extract(
            url = rctx.attr.clang_urls,
            sha256 = rctx.attr.clang_sha256,
            stripPrefix = rctx.attr.clang_strip_prefix,
            output = "tmp_clang",
        )
        rctx.execute([
            "mv",
            "tmp_clang/bin/clang",
            "tmp_clang/bin/clang++",
            "tmp_clang/bin/clang-12",
            "tmp_clang/bin/ld.lld",
            "tmp_clang/bin/ld64.lld",
            "tmp_clang/bin/ld64.lld.darwinnew",
            "tmp_clang/bin/lld",
            "tmp_clang/bin/llvm-nm",
            xcode_toolchain_bindir,
        ])
        rctx.delete("tmp_clang")

    if rctx.attr.swift_urls:
        rctx.download_and_extract(
            url = rctx.attr.swift_urls,
            sha256 = rctx.attr.swift_sha256,
            stripPrefix = rctx.attr.swift_strip_prefix,
            output = "tmp_swift",
        )
        rctx.execute([
            "mv",
            "tmp_swift/usr/bin/swift",
            "tmp_swift/usr/bin/swiftc",
            xcode_toolchain_bindir,
        ])
        rctx.delete("tmp_swift")

    rctx.download_and_extract(
        url = ["https://github.com/apple-cross-toolchain/ci/releases/download/0.0.4/ported-tools-linux-x86_64.tar.xz"],
        sha256 = "c8d410d13f4f3bd076c69647999a3fb0dea7d4f095c1cde8466d9e0bba5e0deb",
        output = "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/"
    )

    swift_worker_srcs = {
        "third_party/bazel_protos/BUILD": "@build_bazel_rules_swift//third_party/bazel_protos:BUILD",
        "third_party/bazel_protos/worker_protocol.proto": "@build_bazel_rules_swift//third_party/bazel_protos:worker_protocol.proto",
        "tools/common/BUILD": "@build_bazel_rules_swift//tools/common:BUILD",
        "tools/common/file_system.cc": "@build_bazel_rules_swift//tools/common:file_system.cc",
        "tools/common/file_system.h": "@build_bazel_rules_swift//tools/common:file_system.h",
        "tools/common/path_utils.cc": "@build_bazel_rules_swift//tools/common:path_utils.cc",
        "tools/common/path_utils.h": "@build_bazel_rules_swift//tools/common:path_utils.h",
        "tools/common/process.cc": "@build_bazel_rules_swift//tools/common:process.cc",
        "tools/common/process.h": "@build_bazel_rules_swift//tools/common:process.h",
        "tools/common/string_utils.cc": "@build_bazel_rules_swift//tools/common:string_utils.cc",
        "tools/common/string_utils.h": "@build_bazel_rules_swift//tools/common:string_utils.h",
        "tools/common/temp_file.h": "@build_bazel_rules_swift//tools/common:temp_file.h",
        "tools/worker/BUILD": "@build_bazel_rules_swift//tools/worker:BUILD",
        "tools/worker/compile_with_worker.cc": "@build_bazel_rules_swift//tools/worker:compile_with_worker.cc",
        "tools/worker/compile_with_worker.h": "@build_bazel_rules_swift//tools/worker:compile_with_worker.h",
        "tools/worker/compile_without_worker.cc": "@build_bazel_rules_swift//tools/worker:compile_without_worker.cc",
        "tools/worker/compile_without_worker.h": "@build_bazel_rules_swift//tools/worker:compile_without_worker.h",
        "tools/worker/no_worker_main.cc": "@build_bazel_rules_swift//tools/worker:no_worker_main.cc",
        "tools/worker/output_file_map.cc": "@build_bazel_rules_swift//tools/worker:output_file_map.cc",
        "tools/worker/output_file_map.h": "@build_bazel_rules_swift//tools/worker:output_file_map.h",
        "tools/worker/swift_runner.h": "@build_bazel_rules_swift//tools/worker:swift_runner.h",
        "tools/worker/work_processor.cc": "@build_bazel_rules_swift//tools/worker:work_processor.cc",
        "tools/worker/work_processor.h": "@build_bazel_rules_swift//tools/worker:work_processor.h",
        "tools/worker/worker_main.cc": "@build_bazel_rules_swift//tools/worker:worker_main.cc",
    }
    for file, label in swift_worker_srcs.items():
        rctx.symlink(paths[label], file)

    _compile_cc_file(
        rctx,
        developer_dir,
        libtool_check_unique_src_path,
        "libtool_check_unique",
    )
    rctx.template(
        "wrapped_clang.cc",
        paths["@rules_applecross//toolchain:wrapped_clang.cc.tpl"],
        substitutions,
    )
    wrapped_clang_src_path = str(rctx.path("wrapped_clang.cc"))
    _compile_cc_file(
        rctx,
        developer_dir,
        str(rctx.path("wrapped_clang.cc")),
        "wrapped_clang",
    )
    rctx.delete("wrapped_clang.cc")
    rctx.symlink("wrapped_clang", "wrapped_clang_pp")

    # Setup Swift toolchain
    rctx.template(
        "swift_toolchain.bzl",
        paths["@rules_applecross//toolchain:swift_toolchain.bzl.tpl"],
        substitutions,
    )
    rctx.template(
        "repositories.bzl",
        paths["@rules_applecross//toolchain:repositories.bzl.tpl"],
        substitutions,
    )
    rctx.template(
        "swift_autoconfiguration.bzl",
        paths["@rules_applecross//toolchain:swift_autoconfiguration.bzl.tpl"],
        substitutions,
    )

    # We use a modified version of swift_runner.cc
    rctx.template(
        "tools/worker/swift_runner.cc",
        paths["@rules_applecross//toolchain:swift_runner.cc.tpl"],
        substitutions,
    )

apple_cross_toolchain = repository_rule(
    attrs = {
        "xcode_urls": attr.string_list(
        ),
        "xcode_sha256": attr.string(
            mandatory = False,
        ),
        "xcode_strip_prefix": attr.string(
            mandatory = False,
        ),
        "clang_urls": attr.string_list(
        ),
        "clang_sha256": attr.string(
            mandatory = False,
        ),
        "clang_strip_prefix": attr.string(
            mandatory = False,
        ),
        "swift_urls": attr.string_list(
        ),
        "swift_sha256": attr.string(
            mandatory = False,
        ),
        "swift_strip_prefix": attr.string(
            mandatory = False,
        ),
    },
    environ = ["PATH"],
    implementation = _apple_cross_toolchain_impl,
)
