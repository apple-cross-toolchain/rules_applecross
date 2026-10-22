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

_PORTED_TOOLS = {
    "aarch64": (
        "https://github.com/apple-cross-toolchain/ci/releases/download/0.0.23/ported-tools-linux-aarch64.tar.xz",
        "0e9de81f2ba7104996e7e65a456f4b69057c254ff352e10b515a4bcff28423f3",
    ),
    "x86_64": (
        "https://github.com/apple-cross-toolchain/ci/releases/download/0.0.23/ported-tools-linux-x86_64.tar.xz",
        "a1665d58c21ff9401c194e1925ab8a9963c35558ab1b3949b1e686025b65dfc1",
    ),
}

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

def _local_developer_dir(rctx):
    """Resolves the Developer directory of the Xcode selected on this host."""
    developer_dir = rctx.os.environ.get("DEVELOPER_DIR", "").strip()
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

def _link_local_xcode(rctx, developer_dir):
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
        ],
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

def _normalize_sdk_tbds(rctx):
    result = rctx.execute([
        _python3(rctx),
        str(_toolchain_path(rctx, "repo_tools/normalize_sdk_tbds.py")),
        "Xcode.app",
    ], timeout = 1800)
    if result.return_code != 0:
        fail("Failed to normalize SDK text stubs: " + (result.stderr or result.stdout))

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
    "wrapped_clang.cc",
    "repo_tools/ensure_clang_resource_libs.py",
    "repo_tools/normalize_sdk_modulemaps.py",
    "repo_tools/link_local_xcode.py",
    "repo_tools/normalize_sdk_tbds.py",
    "repo_tools/patch_swiftinterfaces.py",
    "repo_tools/read_xcode_versions.py",
    "stubs/empty_output_tool.sh",
    "stubs/intentbuilderc.py",
    "stubs/intentbuilderc.sh",
    "stubs/noop_tool.sh",
    "stubs/security.sh",
    "stubs/strip.sh",
    "stubs/xcstringstool.py",
    "stubs/xcstringstool.sh",
    "stubs/zip.py",
    "stubs/zip.sh",
]

_LLVM_PREBUILT = {
    "aarch64": "@llvm_prebuilt_aarch64//:bin/clang",
    "x86_64": "@llvm_prebuilt//:bin/clang",
}

def _prefetch_rule_inputs(rctx, cpu):
    for f in _RULE_INPUT_FILES:
        _toolchain_path(rctx, f)
    rctx.path(Label(_LLVM_PREBUILT[cpu]))

def _use_local_xcode(rctx):
    """Decides whether to read this host's Xcode or unpack an archive."""
    override = rctx.os.environ.get("RULES_APPLECROSS_LOCAL_XCODE", "").strip()
    if override:
        if override not in ("0", "1"):
            fail("RULES_APPLECROSS_LOCAL_XCODE must be \"0\" or \"1\", got: " + override)
        if override == "1" and not rctx.os.name.startswith("mac"):
            fail(
                "RULES_APPLECROSS_LOCAL_XCODE=1 needs a macOS host, but this one " +
                "reports os.name = \"" + rctx.os.name + "\". Unset it to read " +
                "apple_sdk_path or apple_sdk_urls instead.",
            )
        return override == "1"

    # Only macOS has an Xcode to read, so fall through to the archive rather
    # than failing. That is what lets one MODULE.bazel serve both host kinds.
    return rctx.attr.local_xcode and rctx.os.name.startswith("mac")

def _materialize_apple_sdks(rctx):
    """Populates the repository's Xcode.app tree from the configured source."""
    if _use_local_xcode(rctx):
        _link_local_xcode(rctx, _local_developer_dir(rctx))
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

def _install_ported_tools(rctx, cpu, destination):
    archive = rctx.attr.ported_tools_archives.get(cpu)
    if archive:
        rctx.extract(
            archive = rctx.workspace_root.get_child(archive),
            output = destination,
        )
        return
    if cpu not in _PORTED_TOOLS:
        fail(
            ("No ported Apple tools are published for linux-{0}; set " +
             "ported_tools_archives on apple_cross_toolchain.configure().").format(cpu),
        )
    url, sha256 = _PORTED_TOOLS[cpu]
    rctx.download_and_extract(url = [url], sha256 = sha256, output = destination)

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

    _materialize_apple_sdks(rctx)

    _normalize_sdk_modulemaps(rctx)
    _normalize_sdk_tbds(rctx)

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
    substitutions["%{xcode_version}"], substitutions["%{xcode_aliases}"] = _xcode_version_and_aliases(_xcode_version, _xcode_build)
    for _os_name, _sdk_name in _SDK_PLATFORM_FOR_OS.items():
        _sdk_version = _sdk_versions.get(_sdk_name)
        if not _sdk_version:
            fail("The Apple SDK tree is missing the {} SDK".format(_sdk_name))
        substitutions["%{" + _os_name + "_sdk_version}"] = _sdk_version

    rctx.template("BUILD", build_tpl, substitutions)

    # Create Apple-compatible symlinks for LLVM tools so that actions can
    # invoke them by their traditional Apple names. Done AFTER all extractions
    # so that symlinks don't interfere with tarball extraction.
    _llvm_symlinks = {
        # NOTE: "libtool" is intentionally omitted — the llvm multicall binary
        # doesn't recognize "libtool" as a subcommand (only "libtool-darwin").
        # The libtool wrapper (libtool.cc) handles this instead.
        "install_name_tool": "llvm-install-name-tool",
        "lipo": "llvm-lipo",
        "ar": "llvm-ar",
        "ranlib": "llvm-ranlib",
        "otool": "llvm-otool",
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
    # Installed as a shell trampoline so it can run under the calling tool's
    # hermetic Python on executor images without a system python3.
    _xcstringstool_path = xcode_toolchain_bindir + "xcstringstool"
    result = rctx.execute(["test", "-e", _xcstringstool_path])
    if result.return_code != 0:
        _install_executable(rctx, "stubs/xcstringstool.sh", _xcstringstool_path)
        _install_executable(rctx, "stubs/xcstringstool.py", _xcstringstool_path + ".py")

    # Provide `zip` for executor images that lack Info-ZIP (e.g. the
    # swift:*-noble container); rules_apple's process-and-sign script
    # archives bundles with it.
    _zip_path = xcode_toolchain_bindir + "zip"
    result = rctx.execute(["test", "-e", _zip_path])
    if result.return_code != 0:
        _install_executable(rctx, "stubs/zip.sh", _zip_path)
        _install_executable(rctx, "stubs/zip.py", _zip_path + ".py")

    # Create codesign/codesign_allocate stubs for Linux cross-compilation.
    # Always overwrite — the SDK may ship real binaries (e.g. codesign_allocate
    # from LLVM) that fail on Linux with "unable to find any toolchains".
    _codesign_path = xcode_toolchain_bindir + "codesign"
    _install_executable(rctx, "stubs/noop_tool.sh", _codesign_path)

    _codesign_allocate_path = xcode_toolchain_bindir + "codesign_allocate"
    _install_executable(rctx, "stubs/noop_tool.sh", _codesign_allocate_path)

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
Workspace-relative ported Apple tools archives keyed by Linux CPU ("x86_64",
"aarch64"), used instead of the ones apple-cross-toolchain/ci publishes.
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
        "RULES_APPLECROSS_LOCAL_XCODE",
    ],
    implementation = _apple_cross_toolchain_impl,
)
