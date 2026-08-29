"""Provider and rule for a prepared iOS VM firmware image."""

_RUNTIME_NAME_FIRST = "abcdefghijklmnopqrstuvwxyz0123456789"
_RUNTIME_NAME_REST = _RUNTIME_NAME_FIRST + "_.-"

def ios_vm_runtime_key(runtime_name):
    """Validates a runtime name and returns it unchanged as its label key.

    Args:
        runtime_name: User-defined runtime name.

    Returns:
        The validated runtime name.
    """
    if not runtime_name:
        fail("runtime name must not be empty")
    if runtime_name == "all":
        fail("runtime name 'all' is reserved")
    if runtime_name[0] not in _RUNTIME_NAME_FIRST:
        fail("runtime name must start with a lowercase ASCII letter or digit: {}".format(
            repr(runtime_name),
        ))
    for character in runtime_name.elems()[1:]:
        if character not in _RUNTIME_NAME_REST:
            fail("runtime name may contain only lowercase ASCII letters, digits, '_', '.', or '-': {}".format(
                repr(runtime_name),
            ))
    return runtime_name

IosVmFirmwareInfo = provider(
    doc = "Files and identity of one prepared iOS VM firmware image.",
    fields = {
        "bootkc": "Boot kernel collection File.",
        "device_tree": "Patched device tree File.",
        "device_type": "Configured device type for the image.",
        "os_version": "iOS version used to prepare the image.",
        "ramdisk": "Prepared XCTest ramdisk File.",
        "runtime_name": "User-defined name selecting this runtime.",
        "slot_manifest": "Manifest describing the executable slot in the ramdisk.",
        "sptm": "Optional SPTM firmware File.",
        "trust_cache": "Base trust cache File.",
        "txm": "Optional TXM firmware File.",
    },
)

def _ios_vm_firmware_impl(ctx):
    ios_vm_runtime_key(ctx.attr.runtime_name)
    if bool(ctx.file.sptm) != bool(ctx.file.txm):
        fail("sptm and txm must either both be set or both be omitted")

    files = [
        ctx.file.bootkc,
        ctx.file.device_tree,
        ctx.file.ramdisk,
        ctx.file.slot_manifest,
        ctx.file.trust_cache,
    ]
    if ctx.file.sptm:
        files.extend([ctx.file.sptm, ctx.file.txm])

    return [
        IosVmFirmwareInfo(
            bootkc = ctx.file.bootkc,
            device_tree = ctx.file.device_tree,
            device_type = ctx.attr.device_type,
            os_version = ctx.attr.os_version,
            ramdisk = ctx.file.ramdisk,
            runtime_name = ctx.attr.runtime_name,
            slot_manifest = ctx.file.slot_manifest,
            sptm = ctx.file.sptm,
            trust_cache = ctx.file.trust_cache,
            txm = ctx.file.txm,
        ),
        DefaultInfo(
            files = depset(files),
            runfiles = ctx.runfiles(files = files),
        ),
    ]

ios_vm_firmware = rule(
    implementation = _ios_vm_firmware_impl,
    attrs = {
        "bootkc": attr.label(allow_single_file = True, mandatory = True),
        "device_tree": attr.label(allow_single_file = True, mandatory = True),
        "device_type": attr.string(mandatory = True),
        "os_version": attr.string(mandatory = True),
        "ramdisk": attr.label(allow_single_file = True, mandatory = True),
        "runtime_name": attr.string(mandatory = True),
        "slot_manifest": attr.label(allow_single_file = True, mandatory = True),
        "sptm": attr.label(allow_single_file = True),
        "trust_cache": attr.label(allow_single_file = True, mandatory = True),
        "txm": attr.label(allow_single_file = True),
    },
    doc = "Groups the files for one prepared iOS VM firmware image.",
    provides = [IosVmFirmwareInfo],
)

IosVmRuntimeInfo = provider(
    doc = "A prepared iOS VM firmware image paired with its QEMU executable.",
    fields = {
        "firmware": "IosVmFirmwareInfo for the selected runtime.",
        "qemu": "QEMU executable File.",
    },
)

def _ios_vm_runtime_impl(ctx):
    firmware = ctx.attr.firmware[IosVmFirmwareInfo]
    runfiles = ctx.runfiles(files = [ctx.file.qemu])
    runfiles = runfiles.merge(ctx.attr.firmware[DefaultInfo].default_runfiles)
    runfiles = runfiles.merge(ctx.attr.firmware[DefaultInfo].data_runfiles)
    runfiles = runfiles.merge(ctx.attr.qemu[DefaultInfo].default_runfiles)
    runfiles = runfiles.merge(ctx.attr.qemu[DefaultInfo].data_runfiles)

    return [
        IosVmRuntimeInfo(
            firmware = firmware,
            qemu = ctx.file.qemu,
        ),
        DefaultInfo(
            files = depset(
                direct = [ctx.file.qemu],
                transitive = [ctx.attr.firmware[DefaultInfo].files],
            ),
            runfiles = runfiles,
        ),
    ]

ios_vm_runtime = rule(
    implementation = _ios_vm_runtime_impl,
    attrs = {
        "firmware": attr.label(
            mandatory = True,
            providers = [IosVmFirmwareInfo],
        ),
        "qemu": attr.label(
            allow_single_file = True,
            cfg = "exec",
            mandatory = True,
        ),
    },
    doc = "Pairs prepared firmware with the QEMU executable used to boot it.",
    provides = [IosVmRuntimeInfo],
)
