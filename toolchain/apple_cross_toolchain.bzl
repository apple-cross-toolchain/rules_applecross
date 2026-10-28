load("@bazel_tools//tools/build_defs/repo:utils.bzl", "read_netrc", "read_user_netrc", "use_netrc")
load(":xcode_versions.bzl", "XCODE_VERSIONS")

def _download_auth(rctx, urls):
    """Credentials for urls from the .netrc NETRC names, else ~/.netrc, as
    http_archive reads them: Basic with a login, Bearer with only a password,
    and a GitHub token for GitHub's release asset API."""
    netrc_file = rctx.os.environ.get("NETRC", "")
    netrc = read_netrc(rctx, netrc_file) if netrc_file else read_user_netrc(rctx)
    return use_netrc(netrc, urls, {"api.github.com": "token <password>"})

# Archive types download_and_extract takes, longest suffix first.
_ARCHIVE_TYPES = ["tar.gz", "tar.zst", "tar.xz", "tar.bz2", "tgz", "tzst", "txz", "tbz", "zip", "tar"]

def _archive_type(urls):
    """The archive type the configured URLs name, so a --downloader_config
    rewrite to a content-addressed URL (no suffix) still extracts."""
    for url in urls:
        path = url.split("?")[0].split("#")[0]
        for archive_type in _ARCHIVE_TYPES:
            if path.endswith("." + archive_type):
                return archive_type
    return ""

def _sdk_download(rctx, urls, sha256, strip_prefix):
    """Download and extract an SDK archive, with auth support for private hosts."""
    kwargs = {
        "url": urls,
        "sha256": sha256 or "",
        "stripPrefix": strip_prefix or "",
        "auth": _download_auth(rctx, urls),
    }
    archive_type = rctx.attr.apple_sdk_archive_type or _archive_type(urls)
    if archive_type:
        kwargs["type"] = archive_type
    if [u for u in urls if "api.github.com" in u]:
        # GitHub's release asset API answers with the asset only when asked.
        kwargs["headers"] = {"Accept": "application/octet-stream"}
    rctx.download_and_extract(**kwargs)

def _repo_file(rctx, label):
    resolved = rctx.path(Label(label))
    # Register the file as an input of this repository so edits to helper
    # scripts invalidate and re-run the rule.
    rctx.watch(resolved)
    return resolved

def _toolchain_path(rctx, path):
    return _repo_file(rctx, "@rules_applecross//toolchain:" + path)

def _python3(rctx):
    python = rctx.which("python3")
    if not python:
        fail("python3 is required to prepare the Apple cross toolchain repository")
    return str(python)

def _install_executable(rctx, source, destination):
    rctx.template(destination, _toolchain_path(rctx, source), {}, executable = True)

# Ubuntu packages providing shared libraries that the swift.org toolchain's
# driver binaries need at runtime but minimal executor images do not ship.
_SWIFT_HOST_DEPS_DEBS = [
    (
        "libncurses6.deb",
        "n/ncurses/libncurses6_6.4+20240113-1ubuntu2.1_{arch}.deb",
        {
            "amd64": "c4d0fd9d7c997f4b13dbfdd9b9a5e14ed0222303ac931f9c2594c6b99696b63d",
            "arm64": "a75589876c510b0cc163a92efedbba0f4744bd1584736d29ccedb7b7a620fed2",
        },
    ),
    (
        "libsqlite3.deb",
        "s/sqlite3/libsqlite3-0_3.45.1-1ubuntu2.7_{arch}.deb",
        {
            "amd64": "488511119cad001a00f7e00e597112cf743ccfbd3f7a03c82d66237e1bfd82c8",
            "arm64": "2633907e89b37ae88fb1b18736b0a7a14feb614382c6297fca91cea4182fbbad",
        },
    ),
]

_DEB_ARCH = {
    "aarch64": "arm64",
    "x86_64": "amd64",
}

_MULTIARCH = {
    "aarch64": "aarch64-linux-gnu",
    "x86_64": "x86_64-linux-gnu",
}

_PORTED_TOOLS_URL = "https://github.com/apple-cross-toolchain/rules_applecross/releases/download/ported-tools-0.0.26/"

def _linux_exec_cpu(rctx):
    """The host's CPU on Linux; x86_64 (remote executors) elsewhere."""
    if rctx.os.name.startswith("linux"):
        arch = rctx.os.arch
        if arch in ("aarch64", "arm64"):
            return "aarch64"
        if arch in ("amd64", "x86_64"):
            return "x86_64"
        fail("Unsupported Linux host CPU: " + arch)
    return "x86_64"

def _install_swift_host_deps(rctx, toolchain_dir, cpu):
    """Stage Ubuntu shared libraries needed by the Swift driver at runtime."""
    arch = _DEB_ARCH[cpu]
    for name, pool_path, sha256 in _SWIFT_HOST_DEPS_DEBS:
        path = pool_path.format(arch = arch)
        rctx.download(
            url = [
                "https://snapshot.ubuntu.com/ubuntu/20260829T000000Z/pool/main/" + path,
                "https://launchpad.net/ubuntu/+archive/primary/+files/" + path.rpartition("/")[2],
            ],
            output = "_debs/" + name,
            sha256 = sha256[arch],
        )

        # A .deb is an ar archive whose payload is data.tar.zst.
        result = rctx.execute(["ar", "x", name], working_directory = "_debs")
        if result.return_code != 0:
            fail("Failed to unpack {}: {}".format(name, result.stderr))
        rctx.extract(archive = "_debs/data.tar.zst", output = "_debs/root")

    result = rctx.execute([
        "bash",
        "-c",
        "cp -a _debs/root/usr/lib/" + _MULTIARCH[cpu] + "/*.so* " + toolchain_dir + "lib/",
    ])
    if result.return_code != 0:
        fail("Failed to stage Swift host dependencies: " + result.stderr)
    rctx.delete("_debs")

# The SDK whose version apple_support reports for each Apple OS. Device and
# simulator SDKs always ship together at the same version.
_SDK_PLATFORM_FOR_OS = {
    "ios": "iPhoneOS",
    "macos": "MacOSX",
    "tvos": "AppleTVOS",
    "visionos": "XROS",
    "watchos": "WatchOS",
}

_LINUX_XCODE_DEVELOPER_DIR = "/Applications/Xcode.app/Contents/Developer"

def _find_local_developer_dir(rctx):
    """The Developer directory of the Xcode selected on this host, as
    (directory, None), or (None, why there is no usable one)."""
    developer_dir = rctx.os.environ.get("DEVELOPER_DIR", "").strip()
    if not developer_dir and not rctx.os.name.startswith("mac"):
        # On Linux, an Xcode mounted at the usual place.
        developer_dir = _LINUX_XCODE_DEVELOPER_DIR
    if not developer_dir:
        result = rctx.execute(["xcode-select", "-p"])
        if result.return_code != 0:
            return None, (
                "local_xcode is set but no Xcode was found: `xcode-select -p` " +
                "failed with: " + (result.stderr.strip() or result.stdout.strip())
            )
        developer_dir = result.stdout.strip()

    # DEVELOPER_DIR is conventionally the Developer directory, but accept an
    # Xcode.app bundle too since that is what people usually have at hand.
    if developer_dir.endswith(".app"):
        developer_dir += "/Contents/Developer"

    if developer_dir.endswith("/CommandLineTools"):
        return None, (
            "The selected developer directory is " + developer_dir + ", which " +
            "ships no Apple SDKs other than macOS. Select a full Xcode with " +
            "`sudo xcode-select -s /Applications/Xcode.app` or DEVELOPER_DIR."
        )

    if not rctx.path(developer_dir + "/Platforms").exists:
        return None, "The selected developer directory has no Xcode at it: " + developer_dir

    return developer_dir, None

def _local_developer_dir(rctx):
    """Resolves the Developer directory of the Xcode selected on this host."""
    developer_dir, problem = _find_local_developer_dir(rctx)
    if problem:
        fail(problem)
    return developer_dir

def _readtapi(rctx, cpu):
    """LLVM's llvm-readtapi for this host, which stubs Mach-O binaries."""
    clang = _LLVM_PREBUILT_MACOS if rctx.os.name.startswith("mac") else _LLVM_PREBUILT[cpu]
    return str(rctx.path(Label(clang)).dirname) + "/llvm-readtapi"

def _pinned_xcode(rctx):
    """The XCODE_VERSIONS entry xcode_version names, or None."""
    if not rctx.attr.xcode_version:
        return None
    pinned = XCODE_VERSIONS.get(rctx.attr.xcode_version)
    if not pinned:
        fail("xcode_version {} is not one of the Xcode releases rules_applecross knows: {}".format(
            rctx.attr.xcode_version,
            ", ".join(sorted(XCODE_VERSIONS.keys())),
        ))
    return pinned

def _apple_sdk_sha256(rctx):
    """The SDK archive's SHA-256: apple_sdk_sha256, else xcode_version's."""
    pinned = _pinned_xcode(rctx)
    if not pinned or not pinned.sdk_sha256:
        return rctx.attr.apple_sdk_sha256
    if rctx.attr.apple_sdk_sha256 and rctx.attr.apple_sdk_sha256 != pinned.sdk_sha256:
        fail("apple_sdk_sha256 {} is not the SHA-256 of Xcode {}'s SDK archive, {}".format(
            rctx.attr.apple_sdk_sha256,
            rctx.attr.xcode_version,
            pinned.sdk_sha256,
        ))
    return pinned.sdk_sha256

def _stage_xcode_xip(rctx, cpu):
    """Stages the SDK tree out of an Xcode .xip from xcode_xip_urls."""
    pinned = _pinned_xcode(rctx)
    if not pinned:
        fail("xcode_xip_urls needs xcode_version, which pins the .xip's SHA-256")

    rctx.report_progress("Downloading Xcode {}".format(rctx.attr.xcode_version))
    rctx.download(
        url = rctx.attr.xcode_xip_urls,
        output = "_xip/Xcode.xip",
        sha256 = pinned.xip_sha256,
        auth = _download_auth(rctx, rctx.attr.xcode_xip_urls),
    )

    rctx.report_progress("Extracting the Apple SDKs from Xcode {}".format(rctx.attr.xcode_version))
    for command in (
        ["extract_xip.py", "_xip/Xcode.xip", "_xip/stage"],
        [
            "link_local_xcode.py",
            "--move",
            "_xip/stage/Xcode.app/Contents/Developer",
            str(rctx.path("Xcode.app")),
            _readtapi(rctx, cpu),
        ],
    ):
        result = rctx.execute(
            [_python3(rctx), str(_toolchain_path(rctx, "repo_tools/" + command[0]))] + command[1:],
            # Decompressing the whole archive and stubbing the frameworks each
            # take minutes on a slow host.
            timeout = 3600,
        )
        if result.return_code != 0:
            fail("Failed to stage the Apple SDKs from Xcode {}'s .xip:\n{}".format(
                rctx.attr.xcode_version,
                result.stderr or result.stdout,
            ))
    rctx.delete("_xip")

def _check_pinned_xcode(rctx, xcode_version, xcode_build):
    """Fails unless the staged tree is the Xcode xcode_version pins."""
    pinned = _pinned_xcode(rctx)
    if not pinned:
        return
    if xcode_version != rctx.attr.xcode_version or xcode_build != pinned.build:
        fail(("xcode_version is {} ({}), but the Apple SDKs staged for @{} are from Xcode {} ({}). " +
              "On a Mac with another Xcode, --repo_env=RULES_APPLECROSS_LOCAL_XCODE=0 " +
              "stages the pinned one instead.").format(
            rctx.attr.xcode_version,
            pinned.build,
            rctx.name,
            xcode_version,
            xcode_build,
        ))

def _link_local_xcode(rctx, developer_dir, cpu):
    """Links a cross-compilable SDK tree to a locally installed Xcode."""

    # An Xcode upgrade in place keeps the same path, so tie the repository to
    # the version of the Xcode it was linked against.
    #
    # Only that case is caught automatically. `xcode-select -s` to a different
    # Xcode leaves this file untouched, and watching the symlink it rewrites
    # does not help: Bazel records a symlink to a directory as the literal
    # "DIR" rather than a digest, so every Xcode on the machine looks alike.
    # The rule is marked `configure` for that case instead.
    rctx.watch(rctx.path(developer_dir + "/../version.plist"))

    rctx.report_progress("Linking Apple SDKs from " + developer_dir)
    result = rctx.execute(
        [
            _python3(rctx),
            str(_toolchain_path(rctx, "repo_tools/link_local_xcode.py")),
            developer_dir,
            str(rctx.path("Xcode.app")),
        ] + ([] if rctx.os.name.startswith("mac") else [_readtapi(rctx, cpu)]),
        # Generating the framework stubs dominates, and a cold filesystem makes
        # the scan that finds them much slower than the default 600s allows.
        timeout = 3600,
    )
    if result.return_code != 0:
        fail("Failed to link Apple SDKs from {}:\n{}".format(
            developer_dir,
            result.stderr or result.stdout,
        ))

def _read_xcode_versions(rctx):
    """Returns (xcode_version, xcode_build, {sdk platform: sdk version}) for the staged tree."""
    result = rctx.execute([
        _python3(rctx),
        str(_toolchain_path(rctx, "repo_tools/read_xcode_versions.py")),
        "Xcode.app",
    ])
    if result.return_code != 0:
        fail("Failed to read Xcode and SDK versions: " + (result.stderr or result.stdout))

    xcode_version = ""
    xcode_build = ""
    sdk_versions = {}
    for line in result.stdout.splitlines():
        fields = line.split(" ")
        if fields[0] == "xcode":
            xcode_version = fields[1]
        elif fields[0] == "build":
            xcode_build = fields[1]
        elif fields[0] == "sdk":
            sdk_versions[fields[1]] = fields[2]

    if not xcode_version:
        fail("Failed to read the Xcode version from the Apple SDK tree")
    return xcode_version, xcode_build, sdk_versions

def _xcode_version_and_aliases(version, build):
    """Returns the version xcode-locator would report and its aliases, e.g.
    ("27.0.0.27A266a", ["27.0", "27.0.0", "27A266a"]) for 27.0 build 27A266a.

    rules_apple reads the build number from the fourth component."""
    if not build:
        return version, repr([version])
    components = version.split(".")
    full = ".".join((components + ["0", "0"])[:3])
    return full + "." + build, repr(sorted({version: None, full: None, build: None}.keys()))

def _normalize_sdk_modulemaps(rctx):
    result = rctx.execute([
        _python3(rctx),
        str(_toolchain_path(rctx, "repo_tools/normalize_sdk_modulemaps.py")),
        "Xcode.app",
    ])
    if result.return_code != 0:
        fail("Failed to normalize SDK module maps: " + (result.stderr or result.stdout))

def _ensure_clang_resource_libs(rctx, clang_lib_dir, toolchain_bindir):
    result = rctx.execute([
        _python3(rctx),
        str(_toolchain_path(rctx, "repo_tools/ensure_clang_resource_libs.py")),
        clang_lib_dir,
        toolchain_bindir,
    ])
    if result.return_code != 0:
        fail("Failed to prepare clang resource libraries: " + (result.stderr or result.stdout))

def _patch_swiftinterfaces(rctx, framework_dirs):
    result = rctx.execute([
        _python3(rctx),
        str(_toolchain_path(rctx, "repo_tools/patch_swiftinterfaces.py")),
    ] + framework_dirs)
    if result.return_code != 0:
        fail("Failed to patch SDK Swift interfaces: " + (result.stderr or result.stdout))

# Every file this rule reads through a label. Resolving a not-yet-fetched
# label mid-implementation aborts and re-runs the whole function (Bazel
# repository-rule restarts), and any restart after the SDK download repays
# the multi-gigabyte extraction. Resolving all of them upfront to avoid this behavior
_RULE_INPUT_FILES = [
    "BUILD.template.bzl",
    "libtool.cc",
    "ported_tools.sha256",
    "wrapped_clang.cc",
    "repo_tools/ensure_clang_resource_libs.py",
    "repo_tools/normalize_sdk_modulemaps.py",
    "repo_tools/extract_xip.py",
    "repo_tools/link_local_xcode.py",
    "repo_tools/patch_swiftinterfaces.py",
    "repo_tools/read_xcode_versions.py",
    "stubs/metal_unavailable.sh",
    "stubs/intentbuilderc.py",
    "stubs/intentbuilderc.sh",
    "stubs/codesign_identity.sh",
    "stubs/noop_tool.sh",
    "stubs/security.sh",
    "stubs/strip.sh",
    "stubs/sw_vers.sh",
    "stubs/xcstringstool.py",
    "stubs/xcstringstool.sh",
    "stubs/appintentsmetadataprocessor.py",
    "stubs/appintentsmetadataprocessor.sh",
    "stubs/appintentsnltrainingprocessor.sh",
    "stubs/coremlc.py",
    "stubs/coremlc.sh",
    "stubs/coremlc_ops.json",
]

_LLVM_PREBUILT = {
    "aarch64": "@llvm_prebuilt_aarch64//:bin/clang",
    "x86_64": "@llvm_prebuilt//:bin/clang",
}

# The same LLVM release for macOS execution platforms.
_LLVM_PREBUILT_MACOS = "@llvm_prebuilt_macos//:bin/clang"

_LLVM_TOOL_ALIASES = {
    "bitcode_strip": "llvm-bitcode-strip",
    "install_name_tool": "llvm-install-name-tool",
    "lipo": "llvm-lipo",
    "ar": "llvm-ar",
    "ranlib": "llvm-ranlib",
    "otool": "llvm-otool",
    "nm": "llvm-nm",
    "objdump": "llvm-objdump",
}

def _stage_macos_llvm(rctx):
    """Stages macOS LLVM at macos_toolchain/usr on a macOS host."""
    clang = rctx.path(Label(_LLVM_PREBUILT_MACOS))
    root = clang.dirname.dirname
    result = rctx.execute(["bash", "-c", "mkdir -p macos_toolchain/usr && cp -a '{0}/bin' '{0}/lib' macos_toolchain/usr/".format(root)])
    if result.return_code != 0:
        fail("Failed to stage macOS LLVM: " + result.stderr)
    for apple_name, llvm_name in _LLVM_TOOL_ALIASES.items():
        rctx.execute(["ln", "-sf", llvm_name, "macos_toolchain/usr/bin/" + apple_name])
    # llvm-strip -S -T crashes on macOS too; split it as on Linux.
    _install_executable(rctx, "stubs/strip.sh", "macos_toolchain/usr/bin/strip")

    # Use the SDK's Darwin runtime libraries, as Linux does; LLVM's only has
    # macOS ones.
    result = rctx.execute([_python3(rctx), "-c", """
import glob, os, shutil
llvm = sorted(glob.glob("macos_toolchain/usr/lib/clang/*/"))
xcode = sorted(glob.glob("Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/clang/*/lib/darwin"))
if llvm and xcode:
    darwin = os.path.join(llvm[0], "lib", "darwin")
    shutil.rmtree(darwin, ignore_errors = True)
    os.makedirs(darwin)
    for name in sorted(os.listdir(xcode[0])):
        os.symlink(os.path.relpath(os.path.join(xcode[0], name), darwin), os.path.join(darwin, name))
"""])
    if result.return_code != 0:
        fail("Failed to stage the Darwin runtime libraries for macOS LLVM: " + result.stderr)

def _prefetch_rule_inputs(rctx, cpu):
    for f in _RULE_INPUT_FILES:
        _toolchain_path(rctx, f)
    if rctx.attr.ported_tools_manifest:
        rctx.path(rctx.attr.ported_tools_manifest)
    rctx.path(Label(_LLVM_PREBUILT[cpu]))
    if rctx.os.name.startswith("mac"):
        rctx.path(Label(_LLVM_PREBUILT_MACOS))

def _use_local_xcode(rctx):
    """Decides whether to read this host's Xcode or unpack an archive."""
    override = rctx.os.environ.get("RULES_APPLECROSS_LOCAL_XCODE", "").strip()
    if override:
        if override not in ("0", "1"):
            fail("RULES_APPLECROSS_LOCAL_XCODE must be \"0\" or \"1\", got: " + override)
        return override == "1"

    if not rctx.attr.local_xcode:
        return False

    # Only if this host has an Xcode (installed, or mounted on Linux);
    # otherwise the archive source. With no archive source, the local Xcode
    # it is, so the failure says why there is none.
    developer_dir, _ = _find_local_developer_dir(rctx)
    if developer_dir:
        return True
    return not (rctx.attr.apple_sdk_path or rctx.attr.apple_sdk_urls or rctx.attr.xcode_xip_urls)

def _materialize_apple_sdks(rctx, cpu):
    """Populates the repository's Xcode.app tree from the configured source."""
    if _use_local_xcode(rctx):
        _link_local_xcode(rctx, _local_developer_dir(rctx), cpu)
    elif rctx.attr.apple_sdk_path:
        apple_sdk_local = rctx.workspace_root.get_child(rctx.attr.apple_sdk_path)
        if rctx.execute(["test", "-d", str(apple_sdk_local)]).return_code == 0:
            # Pre-extracted directory — hardlink copy (fast, no re-extraction,
            # and tools like xcrun resolve paths correctly unlike symlinks).
            rctx.execute(["bash", "-c", "cp -al '" + str(apple_sdk_local) + "/.' ."])
        else:
            # Tarball
            kwargs = {}
            if rctx.attr.apple_sdk_archive_type:
                kwargs["type"] = rctx.attr.apple_sdk_archive_type
            rctx.extract(
                archive = apple_sdk_local,
                stripPrefix = rctx.attr.apple_sdk_strip_prefix or "",
                **kwargs
            )
    elif rctx.attr.apple_sdk_urls:
        _sdk_download(rctx, rctx.attr.apple_sdk_urls, _apple_sdk_sha256(rctx), rctx.attr.apple_sdk_strip_prefix)
    elif rctx.attr.xcode_xip_urls:
        _stage_xcode_xip(rctx, cpu)
    else:
        fail(
            "No Apple SDK source is configured for @{}. Set apple_sdk_path or ".format(rctx.name) +
            "apple_sdk_urls to a packaged SDK archive, xcode_xip_urls to an Xcode .xip, " +
            "or local_xcode = True to read the SDKs from an Xcode install.",
        )

def _ported_tool_destination(name, destination):
    if name.startswith("python3-"):
        return destination + "libexec/python3"
    return destination

def _install_ported_tools(rctx, cpu, destination):
    suffix = "-linux-{}.tar.zst".format(cpu)
    local = rctx.attr.ported_tools_archives.get(cpu)
    if local:
        path = rctx.workspace_root.get_child(local)
        if path.is_dir:
            rctx.watch_tree(path)
            archives = [p for p in path.readdir() if p.basename.endswith(suffix)]
        else:
            rctx.watch(path)
            archives = [path]
        for archive in archives:
            rctx.extract(archive = archive, output = _ported_tool_destination(archive.basename, destination))
        return

    if rctx.attr.ported_tools_manifest:
        manifest = rctx.path(rctx.attr.ported_tools_manifest)
        rctx.watch(manifest)
    else:
        manifest = _toolchain_path(rctx, "ported_tools.sha256")
    archives = {}
    for line in rctx.read(manifest).splitlines():
        fields = [field for field in line.split(" ") if field]
        if len(fields) == 2 and fields[1].endswith(suffix):
            archives[fields[1]] = fields[0]
    if not archives:
        fail(
            ("No ported Apple tools are published for linux-{0}; set " +
             "ported_tools_archives on apple_cross_toolchain.configure().").format(cpu),
        )
    url_templates = rctx.attr.ported_tools_urls or [_PORTED_TOOLS_URL + "{name}"]
    downloads = []
    for name, sha256 in archives.items():
        urls = [t.format(name = name, sha256 = sha256) for t in url_templates]
        downloads.append(rctx.download(
            url = urls,
            output = "_ported_tools/" + name,
            sha256 = sha256,
            auth = _download_auth(rctx, urls),
            block = False,
        ))
    for download in downloads:
        download.wait()
    for name in archives:
        rctx.extract(archive = "_ported_tools/" + name, output = _ported_tool_destination(name, destination))
    rctx.delete("_ported_tools")

def _apple_cross_toolchain_impl(rctx):
    cpu = _linux_exec_cpu(rctx)

    # Force all label->path restarts before any expensive work below.
    _prefetch_rule_inputs(rctx, cpu)

    # Resolve label paths
    libtool_cc = rctx.path(Label("@rules_applecross//toolchain:libtool.cc"))
    build_tpl = rctx.path(Label("@rules_applecross//toolchain:BUILD.template.bzl"))
    wrapped_clang_src = rctx.path(Label("@rules_applecross//toolchain:wrapped_clang.cc"))

    relative_path_prefix = "external/{}/".format(rctx.name)
    toolchain_path_prefix = relative_path_prefix
    developer_dir = "Xcode.app/Contents/Developer"
    xcode_toolchain_bindir = "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/"

    swift_tools = rctx.attr.swift_tools.get(cpu)
    if not swift_tools:
        fail("No swift_tools for linux-{0}: add \"{0}\" to apple_cross_toolchain.configure()'s swift_tools.".format(cpu))

    substitutions = {
        "%{linux_exec_cpu}": cpu,
        "%{swift_tools}": swift_tools,
        "%{swift_tools_macos}": rctx.attr.swift_tools_macos,
        "%{toolchain_path_prefix}": toolchain_path_prefix,
    }

    # Setup C++ toolchain helpers (BUILD is deferred until after clang version
    # detection so we can populate include dirs).
    rctx.symlink(libtool_cc, "libtool.cc")
    rctx.symlink(wrapped_clang_src, "wrapped_clang.cc")

    _materialize_apple_sdks(rctx, cpu)

    _normalize_sdk_modulemaps(rctx)

    # Resolve the @llvm_prebuilt alias imported from @llvm's minimal prebuilt
    # toolchain extension.
    llvm_prebuilt_bin = str(rctx.path(Label(_LLVM_PREBUILT[cpu])).dirname)
    llvm_prebuilt_lib = str(rctx.path(Label(_LLVM_PREBUILT[cpu])).dirname.dirname) + "/lib"
    xcode_toolchain_dir = "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/"

    _install_ported_tools(rctx, cpu, xcode_toolchain_dir)

    # Copy LLVM binaries from @llvm's prebuilt repo AFTER ported-tools
    # extraction so they take precedence.
    result = rctx.execute([
        "bash",
        "-c",
        "cp -a " + llvm_prebuilt_bin + "/* " + xcode_toolchain_bindir,
    ])
    if result.return_code != 0:
        fail("Failed to copy LLVM binaries: " + result.stderr)

    result = rctx.execute(["ln", "-f", xcode_toolchain_bindir + "llvm", xcode_toolchain_bindir + "clang"])
    if result.return_code != 0:
        fail("Failed to link clang: " + result.stderr)

    # Also copy clang resource headers (lib/clang/<ver>/include/) if present
    result = rctx.execute(["test", "-d", llvm_prebuilt_lib])
    if result.return_code == 0:
        rctx.execute([
            "bash",
            "-c",
            "cp -a " + llvm_prebuilt_lib + "/* " + xcode_toolchain_dir + "lib/",
        ])

    _install_swift_host_deps(rctx, xcode_toolchain_dir, cpu)

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
    _ensure_clang_resource_libs(rctx, xcode_toolchain_dir + "lib/clang/", xcode_toolchain_bindir)

    # Describe the Xcode that produced this tree, so the generated xcode_config
    # reports the versions the SDKs actually have rather than a pinned guess.
    _xcode_version, _xcode_build, _sdk_versions = _read_xcode_versions(rctx)
    _check_pinned_xcode(rctx, _xcode_version, _xcode_build)
    substitutions["%{xcode_version}"], substitutions["%{xcode_aliases}"] = _xcode_version_and_aliases(_xcode_version, _xcode_build)
    for _os_name, _sdk_name in _SDK_PLATFORM_FOR_OS.items():
        _sdk_version = _sdk_versions.get(_sdk_name)
        if not _sdk_version:
            fail("The Apple SDK tree is missing the {} SDK".format(_sdk_name))
        substitutions["%{" + _os_name + "_sdk_version}"] = _sdk_version

    if rctx.os.name.startswith("mac"):
        _stage_macos_llvm(rctx)

    rctx.template("BUILD", build_tpl, substitutions)

    # Create Apple-compatible symlinks for LLVM tools so that actions can
    # invoke them by their traditional Apple names. Done AFTER all extractions
    # so that symlinks don't interfere with tarball extraction.
    # Not libtool: libtool.cc wraps llvm-libtool-darwin.
    for apple_name, llvm_name in _LLVM_TOOL_ALIASES.items():
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

    _install_executable(rctx, "stubs/strip.sh", xcode_toolchain_bindir + "strip")

    # Create metal/metallib stubs for Linux (Metal compiler is macOS-only).
    for _tool_name in ["metal", "metallib"]:
        _tool_path = xcode_toolchain_bindir + _tool_name
        result = rctx.execute(["test", "-e", _tool_path])
        if result.return_code != 0:
            _install_executable(rctx, "stubs/metal_unavailable.sh", _tool_path)

    # Create intentbuilderc stub for Linux.
    _intentbuilderc_path = xcode_toolchain_bindir + "intentbuilderc"
    result = rctx.execute(["test", "-e", _intentbuilderc_path])
    if result.return_code != 0:
        _install_executable(rctx, "stubs/intentbuilderc.sh", _intentbuilderc_path)
        _install_executable(rctx, "stubs/intentbuilderc.py", _intentbuilderc_path + ".py")

    # Create xcstringstool stub for Linux (compiles .xcstrings to .strings).
    _xcstringstool_path = xcode_toolchain_bindir + "xcstringstool"
    result = rctx.execute(["test", "-e", _xcstringstool_path])
    if result.return_code != 0:
        _install_executable(rctx, "stubs/xcstringstool.sh", _xcstringstool_path)
        _install_executable(rctx, "stubs/xcstringstool.py", _xcstringstool_path + ".py")

    # App Intents metadata from the Swift compiler's constant values.
    _appintents_path = xcode_toolchain_bindir + "appintentsmetadataprocessor"
    result = rctx.execute(["test", "-e", _appintents_path])
    if result.return_code != 0:
        _install_executable(rctx, "stubs/appintentsmetadataprocessor.sh", _appintents_path)
        _install_executable(rctx, "stubs/appintentsmetadataprocessor.py", _appintents_path + ".py")
    # Core ML models: compiled and their classes generated from the model's
    # specification.
    _coremlc_path = xcode_toolchain_bindir + "coremlc"
    result = rctx.execute(["test", "-e", _coremlc_path])
    if result.return_code != 0:
        _install_executable(rctx, "stubs/coremlc.sh", _coremlc_path)
        _install_executable(rctx, "stubs/coremlc.py", _coremlc_path + ".py")
        _install_executable(rctx, "stubs/coremlc_ops.json", xcode_toolchain_bindir + "coremlc_ops.json")

    _nl_training_path = xcode_toolchain_bindir + "appintentsnltrainingprocessor"
    result = rctx.execute(["test", "-e", _nl_training_path])
    if result.return_code != 0:
        _install_executable(rctx, "stubs/appintentsnltrainingprocessor.sh", _nl_training_path)

    # Create codesign/codesign_allocate stubs for Linux cross-compilation.
    # Stub codesign unless the ported tools carry one.
    _codesign_path = xcode_toolchain_bindir + "codesign"
    _codesign_stubbed = not rctx.path(_codesign_path).exists
    if _codesign_stubbed:
        _install_executable(rctx, "stubs/noop_tool.sh", _codesign_path)

    # A signing identity for `codesign -s NAME`: the paths of its private key
    # and certificates, PEM. They are referenced, never copied into the
    # repository, so the key is no action input and reaches no cache.
    _signing_key = rctx.os.environ.get("RULES_APPLECROSS_CODESIGN_KEY", "").strip()
    _signing_certificates = rctx.os.environ.get("RULES_APPLECROSS_CODESIGN_CERTIFICATES", "").strip()
    if (_signing_key or _signing_certificates) and not rctx.os.name.startswith("mac"):
        if not (_signing_key.startswith("/") and _signing_certificates.startswith("/")):
            fail("RULES_APPLECROSS_CODESIGN_KEY and RULES_APPLECROSS_CODESIGN_CERTIFICATES must both be set, to absolute paths")
        for _f in (_signing_key, _signing_certificates):
            if not rctx.path(_f).exists:
                fail("No such file: " + _f)
        if _codesign_stubbed:
            fail("Signing with an identity needs the clean-room codesign, which these ported tools lack")
        rctx.execute(["mv", _codesign_path, _codesign_path + ".bin"])
        rctx.template(_codesign_path, _toolchain_path(rctx, "stubs/codesign_identity.sh"), {
            "%{certificates}": _signing_certificates,
            "%{key}": _signing_key,
        }, executable = True)

    _codesign_allocate_path = xcode_toolchain_bindir + "codesign_allocate"
    _install_executable(rctx, "stubs/noop_tool.sh", _codesign_allocate_path)

    # BuildMachineOSBuild to report, to match a build on that Mac.
    _build_machine = rctx.os.environ.get("RULES_APPLECROSS_BUILD_MACHINE_OS_BUILD", "").strip()
    if _build_machine and not rctx.os.name.startswith("mac"):
        rctx.execute(["mv", xcode_toolchain_bindir + "sw_vers", xcode_toolchain_bindir + "sw_vers.ported"])
        rctx.template(
            xcode_toolchain_bindir + "sw_vers",
            _toolchain_path(rctx, "stubs/sw_vers.sh"),
            {"%{build_machine_os_build}": _build_machine},
            executable = True,
        )

    # Create security stub for Linux (handles mobileprovision parsing).
    _security_path = xcode_toolchain_bindir + "security"
    result = rctx.execute(["test", "-e", _security_path])
    if result.return_code != 0:
        rctx.template(_security_path, _toolchain_path(rctx, "stubs/security.sh"), {
            "%{certificates}": _signing_certificates if not rctx.os.name.startswith("mac") else "",
        }, executable = True)

    # Create arm64 swiftinterface files for arm64e-only frameworks.
    _patch_swiftinterfaces(rctx, [
        developer_dir + "/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/System/Library/Frameworks",
        developer_dir + "/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator.sdk/System/Library/Frameworks",
    ])

apple_cross_toolchain = repository_rule(
    attrs = {
        "apple_sdk_path": attr.string(
            doc = "Workspace-relative path to a local Apple SDK tarball or pre-extracted directory.",
        ),
        "local_xcode": attr.bool(
            default = False,
            doc = """\
Read the Apple SDKs from the Xcode installed on this host instead of from
apple_sdk_path or apple_sdk_urls. The tree is assembled out of symlinks into
that install, so it costs seconds and tens of megabytes rather than a full copy.

The Xcode is the one DEVELOPER_DIR names, falling back to `xcode-select -p`
on macOS and to an Xcode mounted at /Applications/Xcode.app on Linux. A host
without one, a Mac with only the Command Line Tools included, uses the archive
source instead; that is what lets one MODULE.bazel serve a Linux CI host, a Mac
without Xcode and a macOS developer host without editing a checked-in file.

Set the repository environment variable RULES_APPLECROSS_LOCAL_XCODE to "0" to
force the archive source on a macOS host, or to "1" to make a missing Xcode a
hard error rather than a silent fallback.
""",
        ),
        "apple_sdk_urls": attr.string_list(
            doc = "URLs of a packaged SDK archive, tried in order. file:// URLs name a local copy.",
        ),
        "apple_sdk_sha256": attr.string(
            doc = "The archive's SHA-256. Defaults to xcode_version's, for the archive //tools:package_sdks makes.",
            mandatory = False,
        ),
        "apple_sdk_strip_prefix": attr.string(
            mandatory = False,
        ),
        "apple_sdk_archive_type": attr.string(
            doc = "Archive type (e.g. 'tar.zst') when it can't be inferred from the archive name.",
            mandatory = False,
        ),
        "xcode_version": attr.string(
            doc = """\
An Xcode release from toolchain/xcode_versions.bzl, such as "27.0". It pins the
SHA-256 of xcode_xip_urls' .xip and apple_sdk_urls' archive, and the staged SDKs
must come from that release whatever their source.
""",
        ),
        "xcode_xip_urls": attr.string_list(
            doc = """\
URLs of xcode_version's .xip, tried in order: a mirror, a bucket, or a file://
URL to a local copy. The SDKs are extracted from it; the rest is discarded.
Used when neither apple_sdk_path nor apple_sdk_urls is set.
""",
        ),
        "ported_tools_archives": attr.string_dict(
            doc = """\
Workspace-relative ported Apple tools keyed by Linux CPU ("x86_64",
"aarch64"), used instead of the ones this repository releases: a directory of
<tool>-linux-<cpu>.tar.zst archives, as tools/build-ported-tools.sh writes
them, or a single archive.
""",
        ),
        "ported_tools_manifest": attr.label(
            allow_single_file = True,
            doc = """\
The ported Apple tools to download, as lines of "<sha256>  <tool>-linux-<cpu>.tar.zst"
like toolchain/ported_tools.sha256, which is the default: tools built from a
newer commit than the published release, for instance.
""",
        ),
        "ported_tools_urls": attr.string_list(
            doc = """\
URL templates for the ported Apple tools, tried in order, with {name} for an
archive's file name and {sha256} for its SHA-256. Defaults to this
repository's release. A content-addressed server takes
"https://ported-tools.invalid/sha256/{sha256}" with --downloader_config.
""",
        ),
        "swift_tools": attr.string_dict(
            doc = "Labels of rules_swift swift_tools targets used as the Linux-host Swift compiler payload, keyed by Linux CPU (\"x86_64\", \"aarch64\").",
            mandatory = True,
        ),
        "swift_tools_macos": attr.string(
            doc = "Label of a rules_swift swift_tools target from swift.org's macOS Swift, used by Swift toolchains for macOS execution platforms.",
        ),
    },
    # Reads the host's Xcode, so `bazel fetch --configure --force` refetches it.
    configure = True,
    environ = [
        "DEVELOPER_DIR",
        "HOME",
        "NETRC",
        "PATH",
        "RULES_APPLECROSS_BUILD_MACHINE_OS_BUILD",
        "RULES_APPLECROSS_CODESIGN_CERTIFICATES",
        "RULES_APPLECROSS_CODESIGN_KEY",
        "RULES_APPLECROSS_LOCAL_XCODE",
    ],
    implementation = _apple_cross_toolchain_impl,
)
