"""Bzlmod extension that assembles lazy iOS VM runtime repositories."""

load("//testing:ios_vm_repositories.bzl", "ios_vm_firmware_local_repository", "ios_vm_firmware_url_repository", "ios_vm_qemu_local_repository", "ios_vm_qemu_url_repository", "ios_vm_runtime_repository")
load("//testing:ios_vm_runtime.bzl", "ios_vm_runtime_key")

_HEX_DIGITS = "0123456789abcdef"
_QEMU_BINARY = "qemu-system-aarch64"

def _validate_relative_path(path, field_name):
    if not path or path.startswith("/") or path.startswith("\\"):
        fail("{} must be a nonempty path relative to the root module".format(field_name))
    if "\\" in path:
        fail("{} must use forward slashes".format(field_name))
    for component in path.split("/"):
        if not component or component == "." or component == "..":
            fail("{} must be a normalized relative path: {}".format(field_name, path))

def _validate_sha256(sha256, description):
    if len(sha256) != 64:
        fail("{} sha256 must contain 64 hexadecimal characters".format(description))
    for character in sha256.lower().elems():
        if character not in _HEX_DIGITS:
            fail("{} sha256 must contain only hexadecimal characters".format(description))

def _validate_source(tag, description):
    has_urls = bool(tag.urls)
    has_path = bool(tag.path)
    if has_urls == has_path:
        fail("{} must set exactly one of urls or path".format(description))

    if has_urls:
        if not tag.sha256:
            fail("{} must set sha256 when urls are used".format(description))
        _validate_sha256(tag.sha256, description)
        seen_urls = {}
        for url in tag.urls:
            if not url:
                fail("{} urls must not contain an empty URL".format(description))
            if url in seen_urls:
                fail("{} contains duplicate URL {}".format(description, url))
            seen_urls[url] = True
        return "url"

    _validate_relative_path(tag.path, "{}.path".format(description))
    if tag.sha256:
        fail("{} must not set sha256 when path is used".format(description))
    if tag.archive_type:
        fail("{} must not set archive_type when path is used".format(description))
    if tag.strip_prefix:
        fail("{} must not set strip_prefix when path is used".format(description))
    return "local"

def _ios_vm_impl(module_ctx):
    firmware_tags = []
    qemu_tags = []
    for module in module_ctx.modules:
        if not module.is_root and (module.tags.firmware or module.tags.qemu):
            fail("ios_vm tags may only be declared by the root module")
        if module.is_root:
            firmware_tags.extend(module.tags.firmware)
            qemu_tags.extend(module.tags.qemu)

    if not firmware_tags:
        fail("the root module must declare at least one ios_vm.firmware tag")
    if len(qemu_tags) != 1:
        fail("the root module must declare exactly one ios_vm.qemu tag")

    firmware_configurations = {}
    names = {}
    for firmware in firmware_tags:
        description = "ios_vm.firmware(name = {}, device_type = {}, os_version = {})".format(
            repr(firmware.name),
            repr(firmware.device_type),
            repr(firmware.os_version),
        )
        source_kind = _validate_source(firmware, description)
        if firmware.name in names:
            fail("duplicate firmware runtime name: {}".format(description))
        names[firmware.name] = description

        key = ios_vm_runtime_key(firmware.name)
        firmware_configurations[key] = struct(
            key = key,
            source_kind = source_kind,
            tag = firmware,
        )

    runtime_keys = []
    firmware_repositories = []
    for key in sorted(firmware_configurations.keys()):
        configuration = firmware_configurations[key]
        firmware = configuration.tag
        repository_name = "rules_applecross_ios_vm_firmware_{}".format(configuration.key)
        runtime_keys.append(configuration.key)
        firmware_repositories.append(repository_name)
        if configuration.source_kind == "url":
            ios_vm_firmware_url_repository(
                name = repository_name,
                archive_type = firmware.archive_type,
                device_type = firmware.device_type,
                os_version = firmware.os_version,
                runtime_key = configuration.key,
                runtime_name = firmware.name,
                sha256 = firmware.sha256.lower(),
                strip_prefix = firmware.strip_prefix,
                urls = firmware.urls,
            )
        else:
            ios_vm_firmware_local_repository(
                name = repository_name,
                device_type = firmware.device_type,
                os_version = firmware.os_version,
                path = firmware.path,
                runtime_name = firmware.name,
            )

    qemu = qemu_tags[0]
    qemu_source_kind = _validate_source(qemu, "ios_vm.qemu")
    _validate_relative_path(qemu.binary_path, "ios_vm.qemu.binary_path")
    qemu_repository = "rules_applecross_ios_vm_qemu"
    if qemu_source_kind == "url":
        ios_vm_qemu_url_repository(
            name = qemu_repository,
            archive_type = qemu.archive_type,
            binary_path = qemu.binary_path,
            sha256 = qemu.sha256.lower(),
            strip_prefix = qemu.strip_prefix,
            urls = qemu.urls,
        )
    else:
        ios_vm_qemu_local_repository(
            name = qemu_repository,
            binary_path = qemu.binary_path,
            path = qemu.path,
        )

    ios_vm_runtime_repository(
        name = "ios_vm_runtime",
        firmware_repositories = firmware_repositories,
        qemu_repository = qemu_repository,
        runtime_keys = runtime_keys,
    )
    if not module_ctx.root_module_has_non_dev_dependency:
        return module_ctx.extension_metadata(
            root_module_direct_deps = [],
            root_module_direct_dev_deps = ["ios_vm_runtime"],
        )
    return module_ctx.extension_metadata(
        root_module_direct_deps = ["ios_vm_runtime"],
        root_module_direct_dev_deps = [],
    )

_firmware_tag = tag_class(
    attrs = {
        "archive_type": attr.string(
            doc = "Archive type when it cannot be inferred from a firmware URL.",
        ),
        "device_type": attr.string(mandatory = True),
        "name": attr.string(
            mandatory = True,
            doc = "Stable name used to select this runtime from @ios_vm_runtime.",
        ),
        "os_version": attr.string(mandatory = True),
        "path": attr.string(
            doc = "Local firmware directory relative to the root module.",
        ),
        "sha256": attr.string(
            doc = "Required SHA-256 digest for a downloaded firmware archive.",
        ),
        "strip_prefix": attr.string(),
        "urls": attr.string_list(
            doc = "Mirror URLs for one firmware archive.",
        ),
    },
)

_qemu_tag = tag_class(
    attrs = {
        "archive_type": attr.string(
            doc = "Archive type when it cannot be inferred from a QEMU URL.",
        ),
        "binary_path": attr.string(
            default = _QEMU_BINARY,
            doc = "QEMU binary path within an archive or local directory.",
        ),
        "path": attr.string(
            doc = "Local QEMU binary or directory relative to the root module.",
        ),
        "sha256": attr.string(
            doc = "Required SHA-256 digest for a downloaded QEMU archive.",
        ),
        "strip_prefix": attr.string(),
        "urls": attr.string_list(
            doc = "Mirror URLs for one QEMU archive.",
        ),
    },
)

ios_vm = module_extension(
    implementation = _ios_vm_impl,
    tag_classes = {
        "firmware": _firmware_tag,
        "qemu": _qemu_tag,
    },
)
