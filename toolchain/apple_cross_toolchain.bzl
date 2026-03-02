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
            type = rctx.attr.xcode_archive_type or "",
            headers = headers,
        )
    else:
        rctx.download_and_extract(
            url = urls,
            sha256 = sha256 or "",
            stripPrefix = strip_prefix or "",
        )

def _compile_cc_file(rctx, developer_dir, src_name, out_name):
    rctx.report_progress("Compiling {}".format(paths.basename(src_name)))
    env = rctx.os.environ
    if developer_dir:
        bin_root = str(rctx.path(developer_dir + "/Toolchains/XcodeDefault.xctoolchain/usr/bin"))
    else:
        bin_root = rctx.which("clang").dirname
    cc = "{}/clang".format(bin_root)
    result = rctx.execute([
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
    # Resolve label paths
    libtool_sh = rctx.path(Label("@rules_applecross//toolchain:libtool.sh"))
    build_tpl = rctx.path(Label("@rules_applecross//toolchain:BUILD.tpl"))
    cc_toolchain_config_tpl = rctx.path(Label("@rules_applecross//toolchain:cc_toolchain_config.bzl.tpl"))
    cc_wrapper_tpl = rctx.path(Label("@rules_applecross//toolchain:cc_wrapper.sh.tpl"))
    swift_toolchain_tpl = rctx.path(Label("@rules_applecross//toolchain:swift_toolchain.bzl.tpl"))
    repositories_tpl = rctx.path(Label("@rules_applecross//toolchain:repositories.bzl.tpl"))
    swift_autoconfig_tpl = rctx.path(Label("@rules_applecross//toolchain:swift_autoconfiguration.bzl.tpl"))
    wrapped_clang_tpl = rctx.path(Label("@rules_applecross//toolchain:wrapped_clang.cc.tpl"))
    xcrunwrapper_tpl = rctx.path(Label("@rules_applecross//toolchain:xcrunwrapper.sh.tpl"))

    repo_path = str(rctx.path(""))
    relative_path_prefix = "external/{}/".format(rctx.name)
    toolchain_path_prefix = relative_path_prefix
    developer_dir = ""
    tools_path_prefix = ""
    xcode_toolchain_bindir = "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/"
    if rctx.attr.xcode_urls or rctx.attr.xcode_path:
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
    rctx.template("BUILD", build_tpl, substitutions)
    rctx.template("cc_toolchain_config.bzl", cc_toolchain_config_tpl, substitutions)
    rctx.template("cc_wrapper.sh", cc_wrapper_tpl, substitutions)
    rctx.template("xcrunwrapper.sh", xcrunwrapper_tpl, substitutions)
    rctx.template("libtool", libtool_sh, substitutions)

    # Extract Xcode SDKs - either from local path or URL
    if rctx.attr.xcode_path:
        # Local tarball
        xcode_tarball = rctx.workspace_root.get_child(rctx.attr.xcode_path)
        rctx.extract(
            archive = xcode_tarball,
            stripPrefix = rctx.attr.xcode_strip_prefix or "",
        )
    elif rctx.attr.xcode_urls:
        _github_asset_download(rctx, rctx.attr.xcode_urls, rctx.attr.xcode_sha256, rctx.attr.xcode_strip_prefix)

    # The tarball may have a nested structure:
    # Xcode.app/Contents/Developer/Applications/Xcode_26.2.app/Contents/Developer/
    # We need to flatten it so that Xcode.app/Contents/Developer/ has the actual SDKs.
    nested_dev_dir = "Xcode.app/Contents/Developer/Applications"
    result = rctx.execute(["test", "-d", nested_dev_dir])
    if result.return_code == 0:
        result = rctx.execute(["ls", nested_dev_dir])
        if result.return_code == 0:
            nested_apps = result.stdout.strip().split("\n")
            for app in nested_apps:
                if app.endswith(".app"):
                    nested_developer = nested_dev_dir + "/" + app + "/Contents/Developer"
                    result = rctx.execute(["ls", nested_developer])
                    if result.return_code == 0:
                        for item in result.stdout.strip().split("\n"):
                            if item:
                                rctx.execute([
                                    "cp", "-a",
                                    nested_developer + "/" + item,
                                    "Xcode.app/Contents/Developer/",
                                ])
                    break
            rctx.execute(["rm", "-rf", nested_dev_dir])

    # Remove self-referencing symlinks that cause infinite glob loops
    # (e.g. Ruby.framework/Headers/ruby/ruby -> .)
    rctx.execute([
        "bash", "-c",
        "find Xcode.app -type l -exec sh -c 'test \"$(readlink \"$1\")\" = \".\" && rm \"$1\"' _ {} \\;",
    ])

    # Extract Clang - either from local path or URL
    if rctx.attr.clang_path:
        clang_tarball = rctx.workspace_root.get_child(rctx.attr.clang_path)
        rctx.extract(
            archive = clang_tarball,
            stripPrefix = rctx.attr.clang_strip_prefix or "",
            output = "tmp_clang",
        )
    elif rctx.attr.clang_urls:
        rctx.download_and_extract(
            url = rctx.attr.clang_urls,
            sha256 = rctx.attr.clang_sha256,
            stripPrefix = rctx.attr.clang_strip_prefix,
            output = "tmp_clang",
        )

    if rctx.attr.clang_path or rctx.attr.clang_urls:
        # Move all clang/LLVM binaries to the toolchain bin directory.
        # Uses cp -a to preserve symlinks, then deletes the temp dir.
        xcode_toolchain_dir = "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/"
        rctx.execute([
            "bash", "-c",
            "cp -a tmp_clang/bin/* " + xcode_toolchain_bindir,
        ])
        # Also copy clang resource headers (lib/clang/<ver>/include/) if present
        result = rctx.execute(["test", "-d", "tmp_clang/lib"])
        if result.return_code == 0:
            rctx.execute([
                "bash", "-c",
                "cp -a tmp_clang/lib/* " + xcode_toolchain_dir + "lib/",
            ])
        rctx.delete("tmp_clang")

        # Ensure the clang resource directory matches the actual clang version.
        # The Xcode SDK ships clang resource headers under lib/clang/<sdk_ver>/
        # but our LLVM binary expects lib/clang/<llvm_ver>/. Create a symlink
        # so the LLVM clang can find the headers from the Xcode SDK.
        clang_lib_dir = xcode_toolchain_dir + "lib/clang/"
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
                        rctx.execute([
                            "ln", "-sfn", sdk_clang_ver, clang_lib_dir + llvm_ver,
                        ])
                    break

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
        url = ["https://github.com/apple-cross-toolchain/ci/releases/download/0.0.20/ported-tools-linux-x86_64.tar.xz"],
        output = "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/",
    )

    # Create Apple-compatible symlinks for LLVM tools so that
    # toolchain configs and xcrunwrapper can invoke them by their
    # traditional Apple names. Done AFTER all extractions so that
    # symlinks don't interfere with tarball extraction.
    if rctx.attr.clang_path or rctx.attr.clang_urls:
        _llvm_symlinks = {
            "libtool": "llvm-libtool-darwin",
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

    rctx.template("wrapped_clang.cc", wrapped_clang_tpl, substitutions)
    _compile_cc_file(
        rctx,
        developer_dir,
        str(rctx.path("wrapped_clang.cc")),
        "wrapped_clang",
    )
    rctx.delete("wrapped_clang.cc")
    rctx.symlink("wrapped_clang", "wrapped_clang_pp")

    # Setup Swift toolchain
    rctx.template("swift_toolchain.bzl", swift_toolchain_tpl, substitutions)
    rctx.template("repositories.bzl", repositories_tpl, substitutions)
    rctx.template("swift_autoconfiguration.bzl", swift_autoconfig_tpl, substitutions)

apple_cross_toolchain = repository_rule(
    attrs = {
        "xcode_path": attr.string(
            doc = "Workspace-relative path to a local Xcode SDK tarball.",
        ),
        "xcode_urls": attr.string_list(),
        "xcode_sha256": attr.string(
            mandatory = False,
        ),
        "xcode_strip_prefix": attr.string(
            mandatory = False,
        ),
        "xcode_archive_type": attr.string(
            doc = "Archive type (e.g. 'tar.xz') when it can't be inferred from the URL.",
            mandatory = False,
        ),
        "clang_path": attr.string(
            doc = "Workspace-relative path to a local Clang tarball.",
        ),
        "clang_urls": attr.string_list(),
        "clang_sha256": attr.string(
            mandatory = False,
        ),
        "clang_strip_prefix": attr.string(
            mandatory = False,
        ),
        "swift_path": attr.string(
            doc = "Workspace-relative path to a local Swift tarball.",
        ),
        "swift_urls": attr.string_list(),
        "swift_sha256": attr.string(
            mandatory = False,
        ),
        "swift_strip_prefix": attr.string(
            mandatory = False,
        ),
    },
    environ = ["HOME", "PATH"],
    implementation = _apple_cross_toolchain_impl,
)
