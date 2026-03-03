"""Module extension that resolves @llvm module metadata and creates a
prebuilt LLVM repository.  The apple_cross_toolchain repository rule
can then reference binaries via Label("@llvm_prebuilt//:bin/clang")
instead of re-downloading the same tarball.

The actual network download is served from Bazel's repository cache
since the URL and SHA256 are identical to @llvm's own http_archive."""

def _parse_llvm_module_bazel(rctx):
    """Parse LLVM version and SHA256 from @llvm's MODULE.bazel."""
    content = rctx.read(rctx.path(Label("@llvm//:MODULE.bazel")))

    llvm_version = None
    prebuilt_suffix = None
    linux_amd64_sha = None
    in_minimal_sha_block = False
    for line in content.split("\n"):
        stripped = line.strip()
        if stripped.startswith("LLVM_VERSION") and "=" in stripped and not stripped.startswith("#"):
            llvm_version = stripped.split("\"")[1]
        elif stripped.startswith("PREBUILT_LLVM_SUFFIX") and "=" in stripped:
            prebuilt_suffix = stripped.split("\"")[1]
        elif "LLVM_TOOLCHAIN_MINIMAL_SHA256" in stripped:
            in_minimal_sha_block = True
        elif in_minimal_sha_block and stripped == "}":
            in_minimal_sha_block = False
        elif in_minimal_sha_block and "linux-amd64" in stripped and not linux_amd64_sha:
            parts = stripped.split("\"")
            if len(parts) >= 4 and "linux-amd64" in parts[1]:
                linux_amd64_sha = parts[3]

    if not llvm_version:
        fail("Could not parse LLVM_VERSION from @llvm MODULE.bazel")
    if not prebuilt_suffix:
        prebuilt_suffix = ""
    if not linux_amd64_sha:
        fail("Could not parse linux-amd64 SHA256 from @llvm MODULE.bazel")

    return llvm_version, prebuilt_suffix, linux_amd64_sha

def _llvm_prebuilt_impl(rctx):
    llvm_version, prebuilt_suffix, sha256 = _parse_llvm_module_bazel(rctx)

    url = (
        "https://github.com/cerisier/toolchains_llvm_bootstrapped/releases/download/" +
        "llvm-%s%s/llvm-toolchain-minimal-%s-linux-amd64-musl.tar.zst" % (
            llvm_version, prebuilt_suffix, llvm_version,
        )
    )

    rctx.report_progress("Downloading LLVM %s prebuilt" % llvm_version)
    rctx.download_and_extract(
        url = [url],
        sha256 = sha256,
        type = "tar.zst",
    )

    rctx.file("BUILD.bazel", 'exports_files(glob(["bin/*", "lib/**"]))\n')

_llvm_prebuilt = repository_rule(
    implementation = _llvm_prebuilt_impl,
)

def _llvm_prebuilt_ext_impl(mctx):
    _llvm_prebuilt(name = "llvm_prebuilt")
    return mctx.extension_metadata(
        root_module_direct_deps = "all",
        root_module_direct_dev_deps = [],
    )

llvm_prebuilt = module_extension(
    implementation = _llvm_prebuilt_ext_impl,
)
