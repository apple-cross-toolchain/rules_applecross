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

def _sdk_download(rctx, urls, sha256, strip_prefix):
    """Download and extract an SDK archive, with auth support for private hosts."""
    kwargs = {
        "url": urls,
        "sha256": sha256 or "",
        "stripPrefix": strip_prefix or "",
    }
    if rctx.attr.apple_sdk_archive_type:
        kwargs["type"] = rctx.attr.apple_sdk_archive_type

    # Build auth headers from ~/.netrc for any URL that needs them.
    headers = {}
    for u in urls:
        if "api.github.com" in u:
            # GitHub API release assets require this Accept header.
            headers["Accept"] = "application/octet-stream"
            token = _read_netrc_token(rctx, "api.github.com")
            if token:
                headers["Authorization"] = "token " + token
            break

    if not headers:
        # Check ~/.netrc for a Bearer/token for the first URL's host.
        for u in urls:
            # Extract host from URL: "https://host/path" -> "host"
            host = u.split("://")[-1].split("/")[0]
            token = _read_netrc_token(rctx, host)
            if token:
                headers["Authorization"] = "Bearer " + token
                break

    if headers:
        kwargs["headers"] = headers

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

# Ubuntu packages providing the shared libraries beyond glibc that swift.org's
# Swift and the ported tools need at runtime, so that an executor needs no more
# than glibc and bash of its own. actiond's sandbox has exactly that.
_HOST_LIBRARY_DEBS = [
    (
        "libgcc-s1.deb",
        "g/gcc-14/libgcc-s1_14.2.0-4ubuntu2~24.04.1_{arch}.deb",
        {
            "amd64": "aa7fadbe33b78bcf99885318040601c550c208929565b179891d9a3cc2aa68cd",
            "arm64": "5b191f79ad985a9a653dbba45214619f729f9178a9bdf92f7c68db06c50d070b",
        },
    ),
    (
        "libstdc++6.deb",
        "g/gcc-14/libstdc++6_14.2.0-4ubuntu2~24.04.1_{arch}.deb",
        {
            "amd64": "a51f8de7829211db961a31f02158058ad1a95f92ac6d0a5dff6350e2821c54c0",
            "arm64": "f84a05ac45a6884109b6527e911081a38263b89e042fc94a338cf975f61a330c",
        },
    ),
    (
        "libtinfo6.deb",
        "n/ncurses/libtinfo6_6.4+20240113-1ubuntu2.1_{arch}.deb",
        {
            "amd64": "768e43c268e1e49b8dae7a50ed4778d6db88688abf72fdf779641e124c8a9a0b",
            "arm64": "bb907b0a9735c08ebc94b3c7a9f725e501cb515ea8b08611566b17135ce3722b",
        },
    ),
    (
        "libuuid1.deb",
        "u/util-linux/libuuid1_2.39.3-9ubuntu6.5_{arch}.deb",
        {
            "amd64": "2f801582b5f0e7cb0c2d3d3651193e69f4540ee3b7dcb6079a3562f0880b9edf",
            "arm64": "4456b14c7573a8f3fd3b068d08edd7459d16968bfcc0cb39e154ac8afe431c7b",
        },
    ),
    (
        "zlib1g.deb",
        "z/zlib/zlib1g_1.3.dfsg-3.1ubuntu2.1_{arch}.deb",
        {
            "amd64": "7074b6a2f6367a10d280c00a1cb02e74277709180bab4f2491a2f355ab2d6c20",
            "arm64": "cbe3d39ec32d3cc27c021ae4af11e7c67bdf9d700d573207e0941d4038056278",
        },
    ),
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
    """The CPU Linux actions run on.

    RULES_APPLECROSS_LINUX_EXEC_CPU if set; otherwise the host's CPU on Linux
    and x86_64 (remote executors) elsewhere.
    """
    cpu = rctx.os.environ.get("RULES_APPLECROSS_LINUX_EXEC_CPU", "").strip()
    if not cpu:
        cpu = "host" if rctx.os.name.startswith("linux") else "x86_64"
    if cpu == "host":
        cpu = rctx.os.arch
    if cpu in ("aarch64", "arm64"):
        return "aarch64"
    if cpu in ("amd64", "x86_64"):
        return "x86_64"
    fail("Unsupported Linux execution CPU: " + cpu)

_COREUTILS_VERSION = "0.12.0"

_COREUTILS_SHA256 = {
    "aarch64": "614a0224ea49f85020cfc30ca980599965fa8501131f9588db4fa41c125a7805",
    "x86_64": "9b555b01457279bcc3e46753f0ef324fd3d02a0535111200a4632d1297263b54",
}

_FINDUTILS_VERSION = "0.10.0"

_FINDUTILS_SHA256 = {
    "aarch64": "c2dad7fbf0cd6b108f833b0a42651f3ad796b6dfa71d303ca1ab51eed3ce3afe",
    "x86_64": "e7c10ea19e1dce2549aaf6f35e15bf42a5e8141d6100c499b6d6fc9b25ea9209",
}

_USERLAND_COMMANDS = [
    "basename",
    "chmod",
    "cp",
    "mkdir",
    "mktemp",
    "mv",
    "rm",
    "sort",
    "touch",
]

def _extract_deb(rctx, deb, cpu, output):
    name, pool_path, sha256 = deb
    arch = _DEB_ARCH[cpu]
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
    rctx.extract(archive = "_debs/data.tar.zst", output = output)

def _install_userland(rctx, cpu):
    coreutils = "coreutils-{}-{}-unknown-linux-musl".format(_COREUTILS_VERSION, cpu)
    rctx.download_and_extract(
        url = "https://github.com/uutils/coreutils/releases/download/{}/{}.tar.gz".format(_COREUTILS_VERSION, coreutils),
        output = "userland/coreutils",
        sha256 = _COREUTILS_SHA256[cpu],
        stripPrefix = coreutils,
    )
    findutils = "findutils-{}-unknown-linux-musl".format(cpu)
    rctx.download_and_extract(
        url = "https://github.com/uutils/findutils/releases/download/{}/{}.tar.xz".format(_FINDUTILS_VERSION, findutils),
        output = "userland/findutils",
        sha256 = _FINDUTILS_SHA256[cpu],
        stripPrefix = findutils,
    )
    for command in _USERLAND_COMMANDS:
        rctx.symlink("userland/coreutils/coreutils", "userland/bin/" + command)
    rctx.symlink("userland/findutils/find", "userland/bin/find")

def _install_host_libraries(rctx, toolchain_dir, cpu):
    """Stage the Ubuntu shared libraries that Linux-hosted tools need."""
    for deb in _HOST_LIBRARY_DEBS:
        _extract_deb(rctx, deb, cpu, "_debs/root")

    result = rctx.execute([
        "bash",
        "-c",
        "cp -a _debs/root/usr/lib/" + _MULTIARCH[cpu] + "/*.so* " + toolchain_dir + "lib/",
    ])
    if result.return_code != 0:
        fail("Failed to stage host libraries: " + result.stderr)
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

def _local_developer_dir(rctx):
    """Resolves the Developer directory of the Xcode selected on this host."""
    developer_dir = rctx.os.environ.get("DEVELOPER_DIR", "").strip()
    if not developer_dir and not rctx.os.name.startswith("mac"):
        # On Linux, an Xcode mounted at the usual place.
        developer_dir = _LINUX_XCODE_DEVELOPER_DIR
    if not developer_dir:
        result = rctx.execute(["xcode-select", "-p"])
        if result.return_code != 0:
            fail(
                "local_xcode is set but no Xcode was found: `xcode-select -p` " +
                "failed with: " + (result.stderr.strip() or result.stdout.strip()),
            )
        developer_dir = result.stdout.strip()

    # DEVELOPER_DIR is conventionally the Developer directory, but accept an
    # Xcode.app bundle too since that is what people usually have at hand.
    if developer_dir.endswith(".app"):
        developer_dir += "/Contents/Developer"

    if developer_dir.endswith("/CommandLineTools"):
        fail(
            "The selected developer directory is " + developer_dir + ", which " +
            "ships no Apple SDKs other than macOS. Select a full Xcode with " +
            "`sudo xcode-select -s /Applications/Xcode.app` or DEVELOPER_DIR.",
        )

    if not rctx.path(developer_dir).exists:
        fail("The selected developer directory does not exist: " + developer_dir)

    return developer_dir

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
        ] + ([] if rctx.os.name.startswith("mac") else [
            str(rctx.path(Label(_LLVM_PREBUILT[cpu])).dirname) + "/llvm-readtapi",
        ]),
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
    "repo_tools/link_local_xcode.py",
    "repo_tools/patch_swiftinterfaces.py",
    "repo_tools/read_xcode_versions.py",
    "stubs/empty_output_tool.sh",
    "stubs/intentbuilderc.py",
    "stubs/intentbuilderc.sh",
    "stubs/noop_tool.sh",
    "stubs/security.sh",
    "stubs/strip.sh",
    "stubs/sw_vers.sh",
    "stubs/xcstringstool.py",
    "stubs/xcstringstool.sh",
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
    if rctx.os.name.startswith("mac"):
        return True

    # On Linux, only if an Xcode is mounted; otherwise use the archive.
    developer_dir = rctx.os.environ.get("DEVELOPER_DIR", "").strip() or _LINUX_XCODE_DEVELOPER_DIR
    return rctx.path(developer_dir + "/Platforms").exists

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
        _sdk_download(rctx, rctx.attr.apple_sdk_urls, rctx.attr.apple_sdk_sha256, rctx.attr.apple_sdk_strip_prefix)
    else:
        fail(
            "No Apple SDK source is configured for @{}. Set apple_sdk_path or ".format(rctx.name) +
            "apple_sdk_urls to a packaged SDK archive, or set local_xcode = True " +
            "to read the SDKs from an Xcode install (macOS hosts only).",
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

    archives = {}
    for line in rctx.read(_toolchain_path(rctx, "ported_tools.sha256")).splitlines():
        fields = [field for field in line.split(" ") if field]
        if len(fields) == 2 and fields[1].endswith(suffix):
            archives[fields[1]] = fields[0]
    if not archives:
        fail(
            ("No ported Apple tools are published for linux-{0}; set " +
             "ported_tools_archives on apple_cross_toolchain.configure().").format(cpu),
        )
    downloads = [
        rctx.download(
            url = _PORTED_TOOLS_URL + name,
            output = "_ported_tools/" + name,
            sha256 = sha256,
            block = False,
        )
        for name, sha256 in archives.items()
    ]
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

    _install_host_libraries(rctx, xcode_toolchain_dir, cpu)
    _install_userland(rctx, cpu)
    if rctx.path(xcode_toolchain_bindir + "SwiftUIMacros").exists:
        for lib in ["libgcc_s.so.1", "libstdc++.so.6"]:
            rctx.symlink(xcode_toolchain_dir + "lib/" + lib, xcode_toolchain_bindir + lib)

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
            _install_executable(rctx, "stubs/empty_output_tool.sh", _tool_path)

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

    # Create codesign/codesign_allocate stubs for Linux cross-compilation.
    # Stub codesign unless the ported tools carry one.
    _codesign_path = xcode_toolchain_bindir + "codesign"
    if not rctx.path(_codesign_path).exists:
        _install_executable(rctx, "stubs/noop_tool.sh", _codesign_path)

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
        _install_executable(rctx, "stubs/security.sh", _security_path)

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

The Xcode is the one DEVELOPER_DIR names, falling back to `xcode-select -p`.
Only macOS has an Xcode to read, so every other host ignores this and uses the
archive source instead; that is what lets one MODULE.bazel serve both a Linux CI
host and a macOS developer host without editing a checked-in file.

Set the repository environment variable RULES_APPLECROSS_LOCAL_XCODE to "0" to
force the archive source on a macOS host, or to "1" to make a missing Xcode a
hard error rather than a silent fallback.
""",
        ),
        "apple_sdk_urls": attr.string_list(),
        "apple_sdk_sha256": attr.string(
            mandatory = False,
        ),
        "apple_sdk_strip_prefix": attr.string(
            mandatory = False,
        ),
        "apple_sdk_archive_type": attr.string(
            doc = "Archive type (e.g. 'tar.zst') when it can't be inferred from the archive name.",
            mandatory = False,
        ),
        "ported_tools_archives": attr.string_dict(
            doc = """\
Workspace-relative ported Apple tools keyed by Linux CPU ("x86_64",
"aarch64"), used instead of the ones this repository releases: a directory of
<tool>-linux-<cpu>.tar.zst archives, as tools/build-ported-tools.sh writes
them, or a single archive.
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
        "PATH",
        "RULES_APPLECROSS_BUILD_MACHINE_OS_BUILD",
        "RULES_APPLECROSS_LINUX_EXEC_CPU",
        "RULES_APPLECROSS_LOCAL_XCODE",
    ],
    implementation = _apple_cross_toolchain_impl,
)
