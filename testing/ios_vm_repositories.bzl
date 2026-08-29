"""Repository rules backing the iOS VM module extension."""

_FIRMWARE_FILES = [
    "bootkc",
    "dtree",
    "ramdisk.dmg",
    "ramdisk.tc",
    "slot.json",
]
_MANIFEST_INPUT_KEYS = [
    "bootkc",
    "device_tree",
    "ramdisk",
    "trust_cache",
]
_QEMU_OUTPUT = "qemu-system-aarch64"
_RESERVED_REPOSITORY_FILES = [
    "BUILD",
    "BUILD.bazel",
    "MODULE.bazel",
    "REPO.bazel",
    "WORKSPACE",
    "WORKSPACE.bazel",
]

def _validate_relative_path(path, field_name):
    if not path or path.startswith("/") or path.startswith("\\"):
        fail("{} must be a nonempty relative path".format(field_name))
    if "\\" in path:
        fail("{} must use forward slashes".format(field_name))
    for component in path.split("/"):
        if not component or component == "." or component == "..":
            fail("{} must be a normalized relative path: {}".format(field_name, path))

def _require_file(path, description):
    if not path.exists or path.is_dir:
        fail("{} is missing or is not a file: {}".format(description, path))

def _validate_sha256(value, description):
    if type(value) != "string" or len(value) != 64:
        fail("{} must be a 64-character SHA-256 digest".format(description))
    for character in value.elems():
        if character not in "0123456789abcdef":
            fail("{} must be a lowercase hexadecimal SHA-256 digest".format(description))

def _validate_slot_manifest(repository_ctx, directory, has_sptm):
    manifest_path = directory.get_child("slot.json")
    manifest = json.decode(repository_ctx.read(manifest_path))
    if type(manifest) != "dict":
        fail("slot.json must contain a JSON object")
    if manifest.get("version") != 2:
        fail("slot.json version must be 2")
    if manifest.get("architecture") != "arm64":
        fail("slot.json architecture must be arm64")

    input_sha256 = manifest.get("input_sha256")
    if type(input_sha256) != "dict":
        fail("slot.json input_sha256 must be a JSON object")
    expected_keys = list(_MANIFEST_INPUT_KEYS)
    if has_sptm:
        expected_keys.extend(["sptm", "txm"])
    if sorted(input_sha256.keys()) != sorted(expected_keys):
        fail("slot.json input_sha256 keys must be {}; got {}".format(
            sorted(expected_keys),
            sorted(input_sha256.keys()),
        ))
    for key in expected_keys:
        _validate_sha256(input_sha256[key], "slot.json input_sha256[{}]".format(repr(key)))

def _inspect_firmware(repository_ctx, directory):
    for filename in _FIRMWARE_FILES:
        _require_file(
            directory.get_child(filename),
            "required firmware file",
        )

    sptm = directory.get_child("sptm")
    txm = directory.get_child("txm")
    if sptm.exists != txm.exists:
        fail("firmware must contain both sptm and txm or neither")
    has_sptm = sptm.exists
    if has_sptm:
        _require_file(sptm, "SPTM firmware file")
        _require_file(txm, "TXM firmware file")

    _validate_slot_manifest(repository_ctx, directory, has_sptm)
    return has_sptm

def _firmware_build(runtime_name, device_type, os_version, has_sptm):
    files = list(_FIRMWARE_FILES)
    optional_attributes = ""
    if has_sptm:
        files.extend(["sptm", "txm"])
        optional_attributes = """\
    sptm = ":sptm",
    txm = ":txm",
"""

    return """\
load("@rules_applecross//testing:ios_vm_runtime.bzl", "ios_vm_firmware")

package(default_visibility = ["//visibility:public"])

exports_files({files})

ios_vm_firmware(
    name = "firmware",
    bootkc = ":bootkc",
    device_tree = ":dtree",
    device_type = {device_type},
    os_version = {os_version},
    ramdisk = ":ramdisk.dmg",
    runtime_name = {runtime_name},
    slot_manifest = ":slot.json",
    trust_cache = ":ramdisk.tc",
{optional_attributes})
""".format(
        device_type = repr(device_type),
        files = repr(files),
        optional_attributes = optional_attributes,
        os_version = repr(os_version),
        runtime_name = repr(runtime_name),
    )

def _reject_reserved_files(repository_ctx):
    for filename in _RESERVED_REPOSITORY_FILES:
        if repository_ctx.path(filename).exists:
            fail("archive must not contain reserved repository file {}".format(filename))

def _download_and_extract(repository_ctx, canonical_id):
    kwargs = {
        "canonical_id": canonical_id,
        "sha256": repository_ctx.attr.sha256,
        "stripPrefix": repository_ctx.attr.strip_prefix,
        "url": repository_ctx.attr.urls,
    }
    if repository_ctx.attr.archive_type:
        kwargs["type"] = repository_ctx.attr.archive_type
    repository_ctx.download_and_extract(**kwargs)

def _ios_vm_firmware_url_repository_impl(repository_ctx):
    _download_and_extract(
        repository_ctx,
        "rules_applecross-ios-vm-firmware-v1-{}".format(repository_ctx.attr.runtime_key),
    )
    _reject_reserved_files(repository_ctx)
    has_sptm = _inspect_firmware(repository_ctx, repository_ctx.path("."))
    repository_ctx.file(
        "BUILD.bazel",
        _firmware_build(
            repository_ctx.attr.runtime_name,
            repository_ctx.attr.device_type,
            repository_ctx.attr.os_version,
            has_sptm,
        ),
    )

ios_vm_firmware_url_repository = repository_rule(
    implementation = _ios_vm_firmware_url_repository_impl,
    attrs = {
        "archive_type": attr.string(),
        "device_type": attr.string(mandatory = True),
        "os_version": attr.string(mandatory = True),
        "runtime_key": attr.string(mandatory = True),
        "runtime_name": attr.string(mandatory = True),
        "sha256": attr.string(mandatory = True),
        "strip_prefix": attr.string(),
        "urls": attr.string_list(mandatory = True),
    },
)

def _ios_vm_firmware_local_repository_impl(repository_ctx):
    _validate_relative_path(repository_ctx.attr.path, "path")
    source_directory = repository_ctx.workspace_root.get_child(repository_ctx.attr.path)
    repository_ctx.watch(source_directory)
    if not source_directory.exists or not source_directory.is_dir:
        fail("local firmware path is missing or is not a directory: {}".format(source_directory))

    for filename in _FIRMWARE_FILES + ["sptm", "txm"]:
        repository_ctx.watch(source_directory.get_child(filename))
    has_sptm = _inspect_firmware(repository_ctx, source_directory)
    files = list(_FIRMWARE_FILES)
    if has_sptm:
        files.extend(["sptm", "txm"])
    for filename in files:
        source = source_directory.get_child(filename)
        repository_ctx.symlink(source, filename)

    repository_ctx.file(
        "BUILD.bazel",
        _firmware_build(
            repository_ctx.attr.runtime_name,
            repository_ctx.attr.device_type,
            repository_ctx.attr.os_version,
            has_sptm,
        ),
    )

ios_vm_firmware_local_repository = repository_rule(
    implementation = _ios_vm_firmware_local_repository_impl,
    attrs = {
        "device_type": attr.string(mandatory = True),
        "os_version": attr.string(mandatory = True),
        "path": attr.string(mandatory = True),
        "runtime_name": attr.string(mandatory = True),
    },
    local = True,
)

def _require_executable(repository_ctx, path):
    test_tool = repository_ctx.which("test")
    if not test_tool:
        fail("the test command is required to validate the QEMU executable")
    result = repository_ctx.execute([str(test_tool), "-x", str(path)])
    if result.return_code:
        fail("QEMU binary must be executable: {}".format(path))

def _write_qemu_build(repository_ctx):
    repository_ctx.file(
        "BUILD.bazel",
        """\
package(default_visibility = ["//visibility:public"])

exports_files(["qemu-system-aarch64"])
""",
    )

def _normalize_qemu_binary(repository_ctx, binary):
    _require_file(binary, "QEMU binary")
    _require_executable(repository_ctx, binary)
    output = repository_ctx.path(_QEMU_OUTPUT)
    if str(binary) != str(output):
        if output.exists:
            fail("archive contains {} in addition to binary_path {}".format(
                _QEMU_OUTPUT,
                repository_ctx.attr.binary_path,
            ))
        repository_ctx.symlink(binary, _QEMU_OUTPUT)

def _ios_vm_qemu_url_repository_impl(repository_ctx):
    _validate_relative_path(repository_ctx.attr.binary_path, "binary_path")
    _download_and_extract(repository_ctx, "rules_applecross-ios-vm-qemu-v1")
    _reject_reserved_files(repository_ctx)
    _normalize_qemu_binary(
        repository_ctx,
        repository_ctx.path(repository_ctx.attr.binary_path),
    )
    _write_qemu_build(repository_ctx)

ios_vm_qemu_url_repository = repository_rule(
    implementation = _ios_vm_qemu_url_repository_impl,
    attrs = {
        "archive_type": attr.string(),
        "binary_path": attr.string(mandatory = True),
        "sha256": attr.string(mandatory = True),
        "strip_prefix": attr.string(),
        "urls": attr.string_list(mandatory = True),
    },
)

def _ios_vm_qemu_local_repository_impl(repository_ctx):
    _validate_relative_path(repository_ctx.attr.path, "path")
    _validate_relative_path(repository_ctx.attr.binary_path, "binary_path")
    source = repository_ctx.workspace_root.get_child(repository_ctx.attr.path)
    repository_ctx.watch(source)
    if source.exists and source.is_dir:
        source = source.get_child(repository_ctx.attr.binary_path)
        repository_ctx.watch(source)
    elif repository_ctx.attr.binary_path != _QEMU_OUTPUT:
        fail("binary_path may be changed for a local directory, not a local QEMU file")

    _require_file(source, "local QEMU binary")
    _require_executable(repository_ctx, source)
    repository_ctx.symlink(source, _QEMU_OUTPUT)
    _write_qemu_build(repository_ctx)

ios_vm_qemu_local_repository = repository_rule(
    implementation = _ios_vm_qemu_local_repository_impl,
    attrs = {
        "binary_path": attr.string(mandatory = True),
        "path": attr.string(mandatory = True),
    },
    local = True,
)

def _ios_vm_runtime_repository_impl(repository_ctx):
    if len(repository_ctx.attr.runtime_keys) != len(repository_ctx.attr.firmware_repositories):
        fail("runtime_keys and firmware_repositories must have the same length")

    runtimes = []
    for index in range(len(repository_ctx.attr.runtime_keys)):
        runtimes.append("""\
ios_vm_runtime(
    name = {key},
    firmware = "@{repository}//:firmware",
    qemu = "@{qemu_repository}//:qemu-system-aarch64",
)
""".format(
            key = repr(repository_ctx.attr.runtime_keys[index]),
            qemu_repository = repository_ctx.attr.qemu_repository,
            repository = repository_ctx.attr.firmware_repositories[index],
        ))

    repository_ctx.file(
        "BUILD.bazel",
        """\
load("@rules_applecross//testing:ios_vm_runtime.bzl", "ios_vm_runtime")

package(default_visibility = ["//visibility:public"])

{runtimes}
""".format(
            runtimes = "\n".join(runtimes),
        ),
    )

ios_vm_runtime_repository = repository_rule(
    implementation = _ios_vm_runtime_repository_impl,
    attrs = {
        "firmware_repositories": attr.string_list(mandatory = True),
        "qemu_repository": attr.string(mandatory = True),
        "runtime_keys": attr.string_list(mandatory = True),
    },
)
