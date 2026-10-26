"""Package the Apple SDKs of an Xcode .xip as the archive xcode_versions.bzl pins.

    bazel run //tools:package_sdks -- ~/Downloads/Xcode_27.xip apple-sdks-27.0.tar.gz

The archive holds the tree xcode_xip_urls stages: the content extract_xip.py
and link_local_xcode.py select, with stubs from this repository's LLVM. Its
entries are sorted, with fixed times, owners and modes, and compressed by the
pinned Python's zlib with no timestamp, so the same .xip always gives the same
archive and the SHA-256 in toolchain/xcode_versions.bzl names it wherever it is
hosted. gzip rather than xz: xz takes most of an hour over the tree, gzip a
minute.

**[Please ensure you have read and understood the Xcode license terms before
hosting the archive.](https://www.apple.com/legal/sla/docs/xcode.pdf)**
"""

import gzip
import hashlib
import os
import shutil
import sys
import tarfile
import tempfile

from python.runfiles import runfiles

_RUNFILES = runfiles.Create()
sys.path.insert(0, os.path.dirname(_RUNFILES.Rlocation("rules_applecross/toolchain/repo_tools/extract_xip.py")))

import extract_xip  # noqa: E402
import link_local_xcode  # noqa: E402


def _normalized(info):
    info.mtime = 0
    info.uid = info.gid = 0
    info.uname = info.gname = ""
    if info.isdir() or info.issym():
        info.mode = 0o755 if info.isdir() else 0o777
    else:
        info.mode = 0o755 if info.mode & 0o111 else 0o644
    return info


def _add_tree(tar, root, name):
    """Add root as name, directories before their sorted contents."""
    tar.add(root, arcname=name, recursive=False, filter=_normalized)
    if os.path.isdir(root) and not os.path.islink(root):
        for child in sorted(os.listdir(root)):
            _add_tree(tar, os.path.join(root, child), name + "/" + child)


def main():
    if len(sys.argv) != 3:
        print("usage: bazel run //tools:package_sdks -- <Xcode.xip> <output.tar.gz>", file=sys.stderr)
        return 2
    # bazel run starts in the runfiles tree; relative paths are the caller's.
    cwd = os.environ.get("BUILD_WORKING_DIRECTORY", os.getcwd())
    xip = os.path.join(cwd, os.path.expanduser(sys.argv[1]))
    output = os.path.join(cwd, os.path.expanduser(sys.argv[2]))
    readtapi = _RUNFILES.Rlocation(os.environ["LLVM_READTAPI"])

    work = tempfile.mkdtemp(prefix=".package_sdks-", dir=os.path.dirname(output))
    try:
        print("Extracting {}".format(xip), file=sys.stderr)
        extract_xip.extract(xip, os.path.join(work, "stage"))
        xcode_app = os.path.join(work, "Xcode.app")
        link_local_xcode.Linker(
            os.path.join(work, "stage", "Xcode.app", "Contents", "Developer"),
            xcode_app,
            readtapi,
            move=True,
        ).run()

        print("Writing {}".format(output), file=sys.stderr)
        partial = os.path.join(work, "archive")
        with open(partial, "wb") as raw:
            with gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0, compresslevel=6) as gz:
                with tarfile.open(fileobj=gz, mode="w", format=tarfile.GNU_FORMAT) as tar:
                    _add_tree(tar, xcode_app, "Xcode.app")
        os.replace(partial, output)
    finally:
        shutil.rmtree(work)

    digest = hashlib.sha256()
    with open(output, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            digest.update(block)
    print("{}  {}".format(digest.hexdigest(), output))
    return 0


if __name__ == "__main__":
    sys.exit(main())
