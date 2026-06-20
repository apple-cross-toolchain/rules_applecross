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

def _toolchain_path(rctx, path):
    return rctx.path(Label("@rules_applecross//toolchain:" + path))

def _python3(rctx):
    python = rctx.which("python3")
    if not python:
        fail("python3 is required to prepare the Apple cross toolchain repository")
    return str(python)

def _install_executable(rctx, source, destination):
    rctx.template(destination, _toolchain_path(rctx, source), {}, executable = True)

def _ensure_swift_compatibility_stub_archives(rctx, toolchain_bindir, toolchain_dir):
    """Create Swift compatibility stub archives expected by Apple linkers."""
    if not rctx.os.name.startswith("linux"):
        return

    clang = toolchain_bindir + "clang"
    llvm_ar = toolchain_bindir + "llvm-ar"
    if rctx.execute(["test", "-x", clang]).return_code != 0:
        return
    if rctx.execute(["test", "-x", llvm_ar]).return_code != 0:
        return

    compatibility_targets = {
        "iphoneos": "arm64-apple-ios13.0",
        "iphonesimulator": "arm64-apple-ios13.0-simulator",
        "macosx": "arm64-apple-macosx10.15",
    }
    compatibility_libs = [
        "swiftCompatibility51",
        "swiftCompatibility56",
        "swiftCompatibilityConcurrency",
        "swiftCompatibilityDynamicReplacements",
        "swiftCompatibilityPacks",
    ]

    for platform_dir, target in compatibility_targets.items():
        swift_platform_dir = toolchain_dir + "lib/swift/" + platform_dir
        if rctx.execute(["test", "-d", swift_platform_dir]).return_code != 0:
            continue

        for lib in compatibility_libs:
            archive = swift_platform_dir + "/lib" + lib + ".a"
            if rctx.execute(["test", "-e", archive]).return_code == 0:
                continue

            src = "_{}_{}.S".format(platform_dir, lib)
            obj = "_{}_{}.o".format(platform_dir, lib)
            symbol = "_swift_FORCE_LOAD_$_" + lib
            rctx.file(src, content = """\
.globl {symbol}
.p2align 2
{symbol}:
  ret
""".format(symbol = symbol))

            result = rctx.execute([clang, "-target", target, "-c", src, "-o", obj])
            if result.return_code != 0:
                fail("Failed to compile Swift compatibility stub {}: {}".format(archive, result.stderr or result.stdout))

            result = rctx.execute([llvm_ar, "rcs", archive, obj])
            if result.return_code != 0:
                fail("Failed to create Swift compatibility archive {}: {}".format(archive, result.stderr or result.stdout))

            rctx.delete(src)
            rctx.delete(obj)

def _restore_tbd_symlinks(rctx):
    result = rctx.execute([
        _python3(rctx),
        str(_toolchain_path(rctx, "repo_tools/restore_tbd_symlinks.py")),
        "Xcode.app",
    ])
    if result.return_code != 0:
        fail("Failed to restore SDK TBD symlinks: " + (result.stderr or result.stdout))

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

def _apple_cross_toolchain_impl(rctx):
    # Resolve label paths
    libtool_cc = rctx.path(Label("@rules_applecross//toolchain:libtool.cc"))
    build_tpl = rctx.path(Label("@rules_applecross//toolchain:BUILD.template.bzl"))
    wrapped_clang_src = rctx.path(Label("@rules_applecross//toolchain:wrapped_clang.cc"))

    repo_path = str(rctx.path(""))
    relative_path_prefix = "external/{}/".format(rctx.name)
    toolchain_path_prefix = relative_path_prefix
    developer_dir = "Xcode.app/Contents/Developer"
    xcode_toolchain_bindir = "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/"

    substitutions = {
        "%{swift_tools}": rctx.attr.swift_tools,
        "%{toolchain_path_prefix}": toolchain_path_prefix,
    }

    # Setup C++ toolchain helpers (BUILD is deferred until after clang version
    # detection so we can populate include dirs).
    rctx.symlink(libtool_cc, "libtool.cc")
    rctx.symlink(wrapped_clang_src, "wrapped_clang.cc")

    # Extract Apple SDKs - either from local path/directory or URL
    if rctx.attr.apple_sdk_path:
        apple_sdk_local = rctx.workspace_root.get_child(rctx.attr.apple_sdk_path)
        if rctx.execute(["test", "-d", str(apple_sdk_local)]).return_code == 0:
            # Pre-extracted directory — hardlink copy (fast, no re-extraction,
            # and tools like xcrun resolve paths correctly unlike symlinks).
            rctx.execute(["bash", "-c", "cp -al '" + str(apple_sdk_local) + "/.' ."])
        else:
            # Tarball
            rctx.extract(
                archive = apple_sdk_local,
                stripPrefix = rctx.attr.apple_sdk_strip_prefix or "",
                type = rctx.attr.apple_sdk_archive_type or "",
            )
    elif rctx.attr.apple_sdk_urls:
        _sdk_download(rctx, rctx.attr.apple_sdk_urls, rctx.attr.apple_sdk_sha256, rctx.attr.apple_sdk_strip_prefix)

    _restore_tbd_symlinks(rctx)
    _normalize_sdk_modulemaps(rctx)

    # Resolve the @llvm_prebuilt alias imported from @llvm's minimal prebuilt
    # toolchain extension.
    llvm_prebuilt_bin = str(rctx.path(Label("@llvm_prebuilt//:bin/clang")).dirname)
    llvm_prebuilt_lib = str(rctx.path(Label("@llvm_prebuilt//:bin/clang")).dirname.dirname) + "/lib"
    xcode_toolchain_dir = "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/"

    rctx.download_and_extract(
        url = ["https://github.com/apple-cross-toolchain/ci/releases/download/0.0.22/ported-tools-linux-x86_64.tar.xz"],
        sha256 = "a41beff504746258ffd62d012b4ab8f09ab38136696472252dbc623f92a09a01",
        output = "Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/",
    )

    # Copy LLVM binaries from @llvm's prebuilt repo AFTER ported-tools
    # extraction so they take precedence.
    result = rctx.execute([
        "bash",
        "-c",
        "cp -a " + llvm_prebuilt_bin + "/* " + xcode_toolchain_bindir,
    ])
    if result.return_code != 0:
        fail("Failed to copy LLVM binaries: " + result.stderr)

    # Also copy clang resource headers (lib/clang/<ver>/include/) if present
    result = rctx.execute(["test", "-d", llvm_prebuilt_lib])
    if result.return_code == 0:
        rctx.execute([
            "bash",
            "-c",
            "cp -a " + llvm_prebuilt_lib + "/* " + xcode_toolchain_dir + "lib/",
        ])

    _ensure_swift_compatibility_stub_archives(rctx, xcode_toolchain_bindir, xcode_toolchain_dir)

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

    # Populate cxx_builtin_include_directories with the absolute repo path so
    # that Bazel's include scanner matches resolved absolute include paths from
    # the compiler (clang resource dir, SDK headers, framework headers, etc.).
    substitutions["%{cxx_builtin_include_directories}"] = repo_path

    # Detect SDK directory paths for the rule-based toolchain.
    # Computes full sdk_path, sdk_fw (framework dir), and plat_fw (platform
    # developer framework dir) for each Apple SDK platform.
    _developer_dir_path = "Xcode.app/Contents/Developer"
    _sdk_names = [
        "iPhoneOS",
        "iPhoneSimulator",
        "MacOSX",
        "AppleTVOS",
        "AppleTVSimulator",
        "XROS",
        "XRSimulator",
        "WatchOS",
        "WatchSimulator",
    ]
    for _sdk_name in _sdk_names:
        _sdk_glob = _developer_dir_path + "/Platforms/" + _sdk_name + ".platform/Developer/SDKs/" + _sdk_name + "*.sdk"
        result = rctx.execute(["bash", "-c", "ls -d " + _sdk_glob + " 2>/dev/null | head -1"])
        if result.return_code == 0 and result.stdout.strip():
            _sdk_rel = result.stdout.strip()
        else:
            _sdk_rel = _developer_dir_path + "/Platforms/" + _sdk_name + ".platform/Developer/SDKs/" + _sdk_name + ".sdk"

        _lower = _sdk_name.lower()
        substitutions["%{sdk_path_" + _lower + "}"] = toolchain_path_prefix + _sdk_rel
        substitutions["%{sdk_fw_" + _lower + "}"] = toolchain_path_prefix + _sdk_rel + "/System/Library/Frameworks"
        substitutions["%{plat_fw_" + _lower + "}"] = toolchain_path_prefix + _developer_dir_path + "/Platforms/" + _sdk_name + ".platform/Developer/Library/Frameworks"
        substitutions["%{plat_lib_" + _lower + "}"] = toolchain_path_prefix + _developer_dir_path + "/Platforms/" + _sdk_name + ".platform/Developer/usr/lib"

    # Detect Xcode version and SDK version for environment variables.
    _xcode_version_plist = "Xcode.app/Contents/version.plist"
    result = rctx.execute([
        "bash",
        "-c",
        xcode_toolchain_bindir + "PlistBuddy -c 'Print CFBundleShortVersionString' " + _xcode_version_plist + " 2>/dev/null || echo '16.0'",
    ])
    substitutions["%{xcode_version}"] = result.stdout.strip() if result.return_code == 0 else "16.0"

    # Detect SDK version from SDKSettings.json (more reliable than directory name).
    _sdk_settings = _developer_dir_path + "/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/SDKSettings.json"
    result = rctx.execute([
        _python3(rctx),
        str(_toolchain_path(rctx, "repo_tools/read_sdk_settings_version.py")),
        _sdk_settings,
    ])
    _sdk_version = result.stdout.strip()
    if not _sdk_version:
        fail("Failed to detect SDK version from {}: {}".format(_sdk_settings, result.stderr.strip()))
    substitutions["%{sdk_version_override}"] = _sdk_version

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

    # Create xcstringtool stub for Linux (compiles .xcstrings to .strings).
    _xcstringtool_path = xcode_toolchain_bindir + "xcstringtool"
    result = rctx.execute(["test", "-e", _xcstringtool_path])
    if result.return_code != 0:
        _install_executable(rctx, "stubs/xcstringtool.py", _xcstringtool_path)

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
        _install_executable(rctx, "stubs/security.py", _security_path)

    if rctx.os.name.startswith("linux"):
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
    _patch_swiftinterfaces(rctx, [
        developer_dir + "/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/System/Library/Frameworks",
        developer_dir + "/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator.sdk/System/Library/Frameworks",
    ])

apple_cross_toolchain = repository_rule(
    attrs = {
        "apple_sdk_path": attr.string(
            doc = "Workspace-relative path to a local Apple SDK tarball or pre-extracted directory.",
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
        "swift_tools": attr.string(
            doc = "Label of a rules_swift swift_tools target used as the Linux-host Swift compiler payload.",
            mandatory = True,
        ),
    },
    environ = ["HOME", "PATH"],
    implementation = _apple_cross_toolchain_impl,
)
