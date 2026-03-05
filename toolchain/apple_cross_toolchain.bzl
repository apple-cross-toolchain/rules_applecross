load("@bazel_skylib//lib:paths.bzl", "paths")

_GITHUB_ASSET_HEADERS = {"Accept": "application/octet-stream"}

def _read_netrc_token(rctx, host):
    """Read a password for the given host from ~/.netrc."""
    home = rctx.os.environ.get("HOME", "")
    if not home:
        return None
    netrc_path = rctx.path(home + "/.netrc")
    if not netrc_path.exists:
        return None
    content = rctx.read(netrc_path)
    found_host = False
    for line in content.split("\n"):
        parts = line.strip().split(" ")
        if not parts:
            continue
        if parts[0] == "machine" and len(parts) > 1 and parts[1] == host:
            found_host = True
        elif found_host and parts[0] == "password" and len(parts) > 1:
            return parts[1]
    return None

def _github_asset_download(rctx, urls, sha256, strip_prefix):
    """Download and extract a GitHub release asset, handling private repos."""
    api_urls = [u for u in urls if "api.github.com" in u]
    if api_urls:
        token = _read_netrc_token(rctx, "api.github.com")
        auth = {}
        headers = dict(_GITHUB_ASSET_HEADERS)
        if token:
            headers["Authorization"] = "token " + token
        rctx.download_and_extract(
            url = urls,
            sha256 = sha256 or "",
            stripPrefix = strip_prefix or "",
            type = rctx.attr.apple_sdk_archive_type or "",
            headers = headers,
        )
    else:
        rctx.download_and_extract(
            url = urls,
            sha256 = sha256 or "",
            stripPrefix = strip_prefix or "",
        )

def _compile_cc_file(rctx, src_name, out_name, toolchain_bindir = None):
    rctx.report_progress("Compiling {}".format(paths.basename(src_name)))
    cc = None
    link_flags = ["-lstdc++"]
    if toolchain_bindir:
        toolchain_clang = toolchain_bindir + "clang"
        result = rctx.execute(["test", "-x", toolchain_clang])
        if result.return_code == 0:
            cc = toolchain_clang
            link_flags = ["-fuse-ld=lld", "-lstdc++"]
    if not cc:
        cc = str(rctx.which("cc") or rctx.which("gcc") or rctx.which("clang"))
        if not cc:
            fail("No C compiler found on PATH. Need cc, gcc, or clang to compile " + out_name)
    result = rctx.execute([
        cc,
        "-std=c++11",
        "-O3",
        "-o",
        out_name,
        src_name,
    ] + link_flags, 30)
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
    # Resolve label paths
    libtool_sh = rctx.path(Label("@rules_applecross//toolchain:libtool.sh"))
    build_tpl = rctx.path(Label("@rules_applecross//toolchain:BUILD.template.bzl"))
    cc_toolchain_config_tpl = rctx.path(Label("@rules_applecross//toolchain:cc_toolchain_config.template.bzl"))
    cc_wrapper_tpl = rctx.path(Label("@rules_applecross//toolchain:cc_wrapper.template.sh"))
    swift_toolchain_tpl = rctx.path(Label("@rules_applecross//toolchain:swift_toolchain.template.bzl"))
    repositories_tpl = rctx.path(Label("@rules_applecross//toolchain:repositories.template.bzl"))
    swift_autoconfig_tpl = rctx.path(Label("@rules_applecross//toolchain:swift_autoconfiguration.template.bzl"))
    wrapped_clang_tpl = rctx.path(Label("@rules_applecross//toolchain:wrapped_clang.template.cc"))
    xcrunwrapper_tpl = rctx.path(Label("@rules_applecross//toolchain:xcrunwrapper.template.sh"))

    repo_path = str(rctx.path(""))
    relative_path_prefix = "external/{}/".format(rctx.name)
    toolchain_path_prefix = relative_path_prefix
    developer_dir = ""
    tools_path_prefix = ""
    xcode_toolchain_bindir = "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/"
    if rctx.attr.apple_sdk_urls or rctx.attr.apple_sdk_path:
        developer_dir = "Xcode.app/Contents/Developer"
        tools_path_prefix = toolchain_path_prefix + xcode_toolchain_bindir

    substitutions = {
        "%{cc}": relative_path_prefix + "wrapped_clang",
        "%{repo_name}": rctx.name,
        "%{repo_path}": repo_path,
        "%{toolchain_path_prefix}": toolchain_path_prefix,
        "%{tools_path_prefix}": tools_path_prefix,
    }

    # Setup C++ toolchain helpers (BUILD and cc_toolchain_config are deferred
    # until after clang version detection so we can populate include dirs).
    rctx.template("cc_wrapper.sh", cc_wrapper_tpl, substitutions)
    rctx.template("xcrunwrapper.sh", xcrunwrapper_tpl, substitutions)
    rctx.template("libtool", libtool_sh, substitutions)

    # Extract Apple SDKs - either from local path or URL
    if rctx.attr.apple_sdk_path:
        # Local tarball
        apple_sdk_tarball = rctx.workspace_root.get_child(rctx.attr.apple_sdk_path)
        rctx.extract(
            archive = apple_sdk_tarball,
            stripPrefix = rctx.attr.apple_sdk_strip_prefix or "",
        )
    elif rctx.attr.apple_sdk_urls:
        _github_asset_download(rctx, rctx.attr.apple_sdk_urls, rctx.attr.apple_sdk_sha256, rctx.attr.apple_sdk_strip_prefix)

    # Resolve the @llvm_prebuilt repo (same URL+SHA as @llvm's own prebuilt;
    # Bazel's download cache deduplicates the network fetch).
    llvm_prebuilt_bin = str(rctx.path(Label("@llvm_prebuilt//:bin/clang")).dirname)
    llvm_prebuilt_lib = str(rctx.path(Label("@llvm_prebuilt//:bin/clang")).dirname.dirname) + "/lib"
    xcode_toolchain_dir = "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/"

    # Extract Swift - either from local path or URL
    if rctx.attr.swift_path:
        swift_tarball = rctx.workspace_root.get_child(rctx.attr.swift_path)
        rctx.extract(
            archive = swift_tarball,
            stripPrefix = rctx.attr.swift_strip_prefix or "",
            output = "tmp_swift",
        )
    elif rctx.attr.swift_urls:
        rctx.download_and_extract(
            url = rctx.attr.swift_urls,
            sha256 = rctx.attr.swift_sha256,
            stripPrefix = rctx.attr.swift_strip_prefix,
            output = "tmp_swift",
        )

    if rctx.attr.swift_path or rctx.attr.swift_urls:
        # Copy all swift-related binaries (including swift-driver,
        # swift-frontend, etc. that symlinks like swiftc point to)
        rctx.execute([
            "bash", "-c",
            "cp -a tmp_swift/usr/bin/swift* " + xcode_toolchain_bindir,
        ])
        # Also copy Swift runtime/stdlib libraries if present
        result = rctx.execute(["test", "-d", "tmp_swift/usr/lib/swift"])
        if result.return_code == 0:
            rctx.execute([
                "cp", "-a",
                "tmp_swift/usr/lib/swift",
                xcode_toolchain_bindir + "../lib/",
            ])
        rctx.delete("tmp_swift")

        # Remove Linux-specific overlay modules that conflict with Apple SDK
        # modules. When cross-compiling for Apple platforms, the SDK provides
        # these modules (dispatch, CoreFoundation, Block, os).
        swift_lib = xcode_toolchain_bindir + "../lib/swift"
        for overlay in ["dispatch", "CoreFoundation", "Block", "os"]:
            rctx.delete(swift_lib + "/" + overlay)

    rctx.download_and_extract(
        url = ["https://github.com/apple-cross-toolchain/ci/releases/download/0.0.22/ported-tools-linux-x86_64.tar.xz"],
        sha256 = "a41beff504746258ffd62d012b4ab8f09ab38136696472252dbc623f92a09a01",
        output = "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/",
    )

    # Copy LLVM binaries from @llvm's prebuilt repo AFTER ported-tools
    # extraction so they take precedence.
    result = rctx.execute([
        "bash", "-c",
        "cp -a " + llvm_prebuilt_bin + "/* " + xcode_toolchain_bindir,
    ])
    if result.return_code != 0:
        fail("Failed to copy LLVM binaries: " + result.stderr)
    # Also copy clang resource headers (lib/clang/<ver>/include/) if present
    result = rctx.execute(["test", "-d", llvm_prebuilt_lib])
    if result.return_code == 0:
        rctx.execute([
            "bash", "-c",
            "cp -a " + llvm_prebuilt_lib + "/* " + xcode_toolchain_dir + "lib/",
        ])

    # Create lib/swift/ symlinks so Swift binaries can find their runtime
    # libraries.  Swift binaries have RUNPATH $ORIGIN/../lib/swift/linux
    # and $ORIGIN/../lib/swift/host/compiler, but the ported-tools tarball
    # places the libraries at lib/linux/ and lib/host/ respectively.
    swift_lib_dir = xcode_toolchain_dir + "lib/swift"
    result = rctx.execute(["test", "-d", swift_lib_dir])
    if result.return_code != 0:
        rctx.execute(["mkdir", "-p", swift_lib_dir])
        result = rctx.execute(["test", "-d", xcode_toolchain_dir + "lib/linux"])
        if result.return_code == 0:
            rctx.execute(["ln", "-sfn", "../linux", swift_lib_dir + "/linux"])
        result = rctx.execute(["test", "-d", xcode_toolchain_dir + "lib/host"])
        if result.return_code == 0:
            rctx.execute(["ln", "-sfn", "../host", swift_lib_dir + "/host"])

    # Ensure the clang resource directory matches the actual clang version.
    # The Xcode SDK ships clang resource headers and compiler-rt builtins
    # under lib/clang/<sdk_ver>/ but our LLVM binary expects
    # lib/clang/<llvm_ver>/. Bridge the version gap so clang finds both.
    clang_lib_dir = xcode_toolchain_dir + "lib/clang/"
    llvm_ver = ""
    result = rctx.execute([
        "bash", "-c",
        "ls -1 " + clang_lib_dir + " 2>/dev/null | head -1",
    ])
    sdk_clang_ver = result.stdout.strip()
    if sdk_clang_ver:
        result = rctx.execute([
            xcode_toolchain_bindir + "clang", "--version",
        ])
        # Extract major version from "clang version X.Y.Z"
        for line in result.stdout.split("\n"):
            if "clang version" in line:
                llvm_ver = line.split("clang version")[1].strip().split(".")[0]
                if llvm_ver != sdk_clang_ver:
                    llvm_clang_dir = clang_lib_dir + llvm_ver
                    sdk_clang_dir = clang_lib_dir + sdk_clang_ver
                    result = rctx.execute(["test", "-d", llvm_clang_dir])
                    if result.return_code != 0:
                        # LLVM version dir doesn't exist at all — symlink it
                        # to the SDK version.
                        rctx.execute([
                            "ln", "-sfn", sdk_clang_ver, llvm_clang_dir,
                        ])
                    else:
                        # LLVM version dir exists (from prebuilt) with headers
                        # but the SDK's compiler-rt builtins (lib/darwin/) are
                        # under the SDK version. Symlink missing subdirs.
                        result = rctx.execute(["test", "-d", sdk_clang_dir + "/lib"])
                        if result.return_code == 0:
                            result = rctx.execute(["test", "-e", llvm_clang_dir + "/lib"])
                            if result.return_code != 0:
                                rctx.execute([
                                    "ln", "-sfn",
                                    "../" + sdk_clang_ver + "/lib",
                                    llvm_clang_dir + "/lib",
                                ])
                break

    # Populate cxx_builtin_include_directories with the absolute repo path so
    # that Bazel's include scanner matches resolved absolute include paths from
    # the compiler (clang resource dir, SDK headers, framework headers, etc.).
    substitutions["%{cxx_builtin_include_directories}"] = repo_path
    rctx.template("BUILD", build_tpl, substitutions)
    rctx.template("cc_toolchain_config.bzl", cc_toolchain_config_tpl, substitutions)

    # Create Apple-compatible symlinks for LLVM tools so that
    # toolchain configs and xcrunwrapper can invoke them by their
    # traditional Apple names. Done AFTER all extractions so that
    # symlinks don't interfere with tarball extraction.
    _llvm_symlinks = {
        # NOTE: "libtool" is intentionally omitted — the llvm multicall binary
        # doesn't recognize "libtool" as a subcommand (only "libtool-darwin").
        # The libtool template script from libtool.sh handles this instead.
        "install_name_tool": "llvm-install-name-tool",
        "lipo": "llvm-lipo",
        "ar": "llvm-ar",
        "ranlib": "llvm-ranlib",
        "otool": "llvm-otool",
        "strip": "llvm-strip",
        "nm": "llvm-nm",
        "objdump": "llvm-objdump",
    }
    for apple_name, llvm_name in _llvm_symlinks.items():
        target = xcode_toolchain_bindir + llvm_name
        link = xcode_toolchain_bindir + apple_name
        result = rctx.execute(["test", "-e", target])
        if result.return_code == 0:
            rctx.execute(["bash", "-c", "ln -sf " + llvm_name + " " + link])
        else:
            # LLVM tool not available; fall back to system tool if it exists
            sys_tool = rctx.which(apple_name)
            if sys_tool:
                rctx.execute(["bash", "-c", "rm -f " + link + " && cp " + str(sys_tool) + " " + link])

    # Create an Apple-compatible libtool shim if llvm-libtool-darwin is not
    # available. Apple's libtool -static creates a static archive like ar rcs.
    _libtool_path = xcode_toolchain_bindir + "libtool"
    result = rctx.execute(["test", "-e", _libtool_path])
    if result.return_code != 0:
        rctx.file(
            _libtool_path,
            content = """\
#!/bin/bash
# Apple libtool shim: translates Apple libtool flags to ar.
# Usage: libtool -static -o output [-D] [-no_warning_for_no_symbols] inputs...
set -eu
OUTPUT=""
DETERMINISTIC=""
INPUTS=()
SKIP_NEXT=0
for arg in "$@"; do
  if [[ $SKIP_NEXT -eq 1 ]]; then
    OUTPUT="$arg"
    SKIP_NEXT=0
  elif [[ "$arg" == "-o" ]]; then
    SKIP_NEXT=1
  elif [[ "$arg" == "-static" || "$arg" == "-no_warning_for_no_symbols" || "$arg" == "-warning_for_no_symbols" ]]; then
    :
  elif [[ "$arg" == "-D" ]]; then
    DETERMINISTIC="D"
  else
    INPUTS+=("$arg")
  fi
done
if [[ -z "$OUTPUT" ]]; then
  echo "error: libtool: no output specified" >&2
  exit 1
fi
MYDIR="$(cd "$(dirname "$0")" && pwd)"
AR="$MYDIR/ar"
if [[ ! -x "$AR" ]]; then
  AR=ar
fi
exec "$AR" "rcs${DETERMINISTIC}" "$OUTPUT" "${INPUTS[@]}"
""",
            executable = True,
        )

    # Create metal/metallib stubs for Linux (Metal compiler is macOS-only).
    for _tool_name in ["metal", "metallib"]:
        _tool_path = xcode_toolchain_bindir + _tool_name
        result = rctx.execute(["test", "-e", _tool_path])
        if result.return_code != 0:
            rctx.file(
                _tool_path,
                content = """\
#!/bin/bash
# Stub: {tool} is not available on Linux. Creates empty output.
OUTPUT=""
for arg in "$@"; do
  if [[ "$prev" == "-o" ]]; then OUTPUT="$arg"; fi
  prev="$arg"
done
if [[ -n "$OUTPUT" ]]; then touch "$OUTPUT"; fi
""".format(tool = _tool_name),
                executable = True,
            )

    # Create intentbuilderc stub for Linux.
    _intentbuilderc_path = xcode_toolchain_bindir + "intentbuilderc"
    result = rctx.execute(["test", "-e", _intentbuilderc_path])
    if result.return_code != 0:
        rctx.file(
            _intentbuilderc_path,
            content = """\
#!/bin/bash
# Stub: intentbuilderc is not available on Linux.
OUTPUT_DIR=""
LANGUAGE=""
for arg in "$@"; do
  if [[ "$prev" == "-output" ]]; then OUTPUT_DIR="$arg"; fi
  if [[ "$prev" == "-language" ]]; then LANGUAGE="$arg"; fi
  prev="$arg"
done
if [[ -n "$OUTPUT_DIR" ]]; then
  mkdir -p "$OUTPUT_DIR"
  if [[ "$LANGUAGE" == "Swift" ]]; then
    touch "$OUTPUT_DIR/Intents.swift"
  fi
fi
""",
            executable = True,
        )

    # Create codesign stub for Linux (ad-hoc signing no-op).
    _codesign_path = xcode_toolchain_bindir + "codesign"
    result = rctx.execute(["test", "-e", _codesign_path])
    if result.return_code != 0:
        rctx.file(
            _codesign_path,
            content = """\
#!/bin/bash
# Stub: codesign is not available on Linux. No-op for cross-compilation.
exit 0
""",
            executable = True,
        )

    # Create codesign_allocate stub for Linux.
    _codesign_allocate_path = xcode_toolchain_bindir + "codesign_allocate"
    result = rctx.execute(["test", "-e", _codesign_allocate_path])
    if result.return_code != 0:
        rctx.file(
            _codesign_allocate_path,
            content = """\
#!/bin/bash
# Stub: codesign_allocate is not available on Linux. No-op for cross-compilation.
exit 0
""",
            executable = True,
        )

    # Create security stub for Linux (handles mobileprovision parsing).
    _security_path = xcode_toolchain_bindir + "security"
    result = rctx.execute(["test", "-e", _security_path])
    if result.return_code != 0:
        rctx.file(
            _security_path,
            content = """\
#!/usr/bin/env python3
\"\"\"Minimal security stub for cross-compilation on Linux.
Handles the subset of macOS 'security' commands used by rules_apple codesigningtool.\"\"\"
import subprocess, sys, os

def main():
    args = sys.argv[1:]
    if not args:
        sys.exit(0)
    # security cms -D -i <file>: extract plist from PKCS#7 signed mobileprovision
    if args[0] == "cms" and "-D" in args and "-i" in args:
        idx = args.index("-i")
        if idx + 1 < len(args):
            mp_file = args[idx + 1]
            # Try openssl to extract the signed content
            try:
                result = subprocess.run(
                    ["openssl", "smime", "-inform", "DER", "-verify", "-noverify", "-in", mp_file],
                    capture_output=True
                )
                if result.returncode == 0 and result.stdout:
                    sys.stdout.buffer.write(result.stdout)
                    sys.exit(0)
            except FileNotFoundError:
                pass
            # Fallback: return a minimal empty plist
            print('<?xml version="1.0" encoding="UTF-8"?>')
            print('<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">')
            print('<plist version="1.0"><dict>')
            print('<key>DeveloperCertificates</key><array></array>')
            print('</dict></plist>')
            sys.exit(0)
    # security find-identity -p codesigning: return a fake identity
    if args[0] == "find-identity":
        print("  1) AABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDD \\"Apple Development: Cross Compilation\\"")
        print("     1 valid identities found")
        sys.exit(0)
    sys.exit(0)

if __name__ == "__main__":
    main()
""",
            executable = True,
        )

    # Create stub Swift compatibility libraries with FORCE_LOAD symbols.
    _swift_compat_symbols = {
        "libswiftCompatibility51.a": "_swift_FORCE_LOAD_$_swiftCompatibility51",
        "libswiftCompatibility56.a": "_swift_FORCE_LOAD_$_swiftCompatibility56",
        "libswiftCompatibilityConcurrency.a": "_swift_FORCE_LOAD_$_swiftCompatibilityConcurrency",
        "libswiftCompatibilityDynamicReplacements.a": "_swift_FORCE_LOAD_$_swiftCompatibilityDynamicReplacements",
        "libswiftCompatibilityPacks.a": "_swift_FORCE_LOAD_$_swiftCompatibilityPacks",
    }
    _clang = xcode_toolchain_bindir + "clang"
    _ar = xcode_toolchain_bindir + "ar"
    _xcode_toolchain_libdir = xcode_toolchain_bindir + "../lib/swift/"
    for _platform_info in [("iphoneos", "arm64-apple-ios13.0"), ("iphonesimulator", "arm64-apple-ios13.0-simulator"), ("macosx", "arm64-apple-macos11.0")]:
        _platform_name = _platform_info[0]
        _target_triple = _platform_info[1]
        _swift_platform_lib = _xcode_toolchain_libdir + _platform_name
        result = rctx.execute(["test", "-d", _swift_platform_lib])
        if result.return_code == 0:
            _sdk_platform = "iPhoneOS" if _platform_name == "iphoneos" else ("iPhoneSimulator" if _platform_name == "iphonesimulator" else "MacOSX")
            _sdk_path = developer_dir + "/Platforms/" + _sdk_platform + ".platform/Developer/SDKs/" + _sdk_platform + ".sdk"
            for _compat_lib, _symbol in _swift_compat_symbols.items():
                _lib_path = _swift_platform_lib + "/" + _compat_lib
                result = rctx.execute(["test", "-e", _lib_path])
                if result.return_code != 0:
                    _c_symbol = _symbol.lstrip("_")
                    _stub_c = _swift_platform_lib + "/_compat_stub.c"
                    _stub_o = _swift_platform_lib + "/_compat_stub.o"
                    rctx.file(_stub_c, content = "void " + _c_symbol + "(void) {}\n")
                    rctx.execute([_clang, "-target", _target_triple, "-isysroot", _sdk_path, "-c", _stub_c, "-o", _stub_o])
                    rctx.execute([_ar, "rcs", _lib_path, _stub_o])
                    rctx.delete(_stub_c)
                    rctx.delete(_stub_o)

    # Create arm64 swiftinterface files for arm64e-only frameworks.
    # Some SDK frameworks (especially cross-import overlays like
    # _Translation_SwiftUI) ship only arm64e swiftinterfaces. The Swift
    # compiler won't load these when targeting arm64, causing "has no
    # member" errors. Create arm64 copies with the target triple replaced.
    for _sdk_info in [("iPhoneOS", "iPhoneOS"), ("iPhoneSimulator", "iPhoneSimulator")]:
        _sdk_platform_name = _sdk_info[0]
        _sdk_name = _sdk_info[1]
        _sdk_fw_dir = developer_dir + "/Platforms/" + _sdk_platform_name + ".platform/Developer/SDKs/" + _sdk_name + ".sdk/System/Library/Frameworks"
        result = rctx.execute([
            "bash", "-c",
            "find '" + _sdk_fw_dir + "' -type d -name '*.swiftmodule' 2>/dev/null",
        ])
        if result.return_code == 0:
            for _swiftmod_dir in result.stdout.strip().split("\n"):
                if not _swiftmod_dir:
                    continue
                _arm64e = _swiftmod_dir + "/arm64e-apple-ios.swiftinterface"
                _arm64 = _swiftmod_dir + "/arm64-apple-ios.swiftinterface"
                r1 = rctx.execute(["test", "-e", _arm64e])
                r2 = rctx.execute(["test", "-e", _arm64])
                if r1.return_code == 0 and r2.return_code != 0:
                    rctx.execute([
                        "bash", "-c",
                        "sed 's/arm64e-apple-ios/arm64-apple-ios/g' '" + _arm64e + "' > '" + _arm64 + "'",
                    ])

    rctx.template("wrapped_clang.cc", wrapped_clang_tpl, substitutions)
    _compile_cc_file(
        rctx,
        str(rctx.path("wrapped_clang.cc")),
        "wrapped_clang",
        toolchain_bindir = xcode_toolchain_bindir,
    )
    rctx.delete("wrapped_clang.cc")
    rctx.symlink("wrapped_clang", "wrapped_clang_pp")

    # Setup Swift toolchain
    rctx.template("swift_toolchain.bzl", swift_toolchain_tpl, substitutions)
    rctx.template("repositories.bzl", repositories_tpl, substitutions)
    rctx.template("swift_autoconfiguration.bzl", swift_autoconfig_tpl, substitutions)

_DEFAULT_SWIFT_URLS = ["https://github.com/apple-cross-toolchain/ci/releases/download/0.0.20/swift-6.2.3-RELEASE-ubuntu24.04-stripped.tar.xz"]
_DEFAULT_SWIFT_SHA256 = "b84d5a7ced3ce25a8b1f94be448f1927e159712e7e2c95b7047afeb0f5c266f5"
_DEFAULT_SWIFT_STRIP_PREFIX = "swift-6.2.3-RELEASE-ubuntu24.04"

apple_cross_toolchain = repository_rule(
    attrs = {
        "apple_sdk_path": attr.string(
            doc = "Workspace-relative path to a local Apple SDK tarball.",
        ),
        "apple_sdk_urls": attr.string_list(),
        "apple_sdk_sha256": attr.string(
            mandatory = False,
        ),
        "apple_sdk_strip_prefix": attr.string(
            mandatory = False,
        ),
        "apple_sdk_archive_type": attr.string(
            doc = "Archive type (e.g. 'tar.xz') when it can't be inferred from the URL.",
            mandatory = False,
        ),
        "swift_path": attr.string(
            doc = "Workspace-relative path to a local Swift tarball.",
        ),
        "swift_urls": attr.string_list(
            default = _DEFAULT_SWIFT_URLS,
        ),
        "swift_sha256": attr.string(
            default = _DEFAULT_SWIFT_SHA256,
        ),
        "swift_strip_prefix": attr.string(
            default = _DEFAULT_SWIFT_STRIP_PREFIX,
        ),
    },
    environ = ["HOME", "PATH"],
    implementation = _apple_cross_toolchain_impl,
)
