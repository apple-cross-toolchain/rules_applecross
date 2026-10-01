"""iOS unit-test runner backed by a darwin-vm QEMU image."""

load(
    "@build_bazel_rules_apple//apple:providers.bzl",
    "AppleDeviceTestRunnerInfo",
    "apple_provider",
)
load(
    "//testing:ios_vm_runtime.bzl",
    "IosVmRuntimeInfo",
)

def _ios_vm_test_runner_impl(ctx):
    runtime = ctx.attr.runtime[IosVmRuntimeInfo]
    firmware = runtime.firmware
    if bool(firmware.sptm) != bool(firmware.txm):
        fail("sptm and txm must either both be set or both be omitted")

    output = ctx.actions.declare_file(ctx.label.name + ".sh")
    substitutions = {
        "%(boot_args)s": ctx.attr.boot_args,
        "%(boot_attempts)s": str(ctx.attr.boot_attempts),
        "%(boot_timeout)s": str(ctx.attr.boot_timeout),
        "%(bootkc_path)s": firmware.bootkc.short_path,
        "%(device_tree_path)s": firmware.device_tree.short_path,
        "%(qemu_path)s": runtime.qemu.short_path,
        "%(ramdisk_path)s": firmware.ramdisk.short_path,
        "%(runner_path)s": ctx.file._runner.short_path,
        "%(slot_manifest_path)s": firmware.slot_manifest.short_path,
        "%(sptm_path)s": firmware.sptm.short_path if firmware.sptm else "",
        "%(test_timeout)s": str(ctx.attr.test_timeout),
        "%(trust_cache_path)s": firmware.trust_cache.short_path,
        "%(txm_path)s": firmware.txm.short_path if firmware.txm else "",
    }
    ctx.actions.expand_template(
        is_executable = True,
        output = output,
        substitutions = substitutions,
        template = ctx.file._template,
    )

    files = [
        firmware.bootkc,
        firmware.device_tree,
        runtime.qemu,
        firmware.ramdisk,
        ctx.file._runner,
        firmware.slot_manifest,
        firmware.trust_cache,
    ]
    if firmware.sptm:
        files.extend([firmware.sptm, firmware.txm])

    runfiles = ctx.runfiles(files = files)
    runfiles = runfiles.merge(ctx.attr.runtime[DefaultInfo].default_runfiles)
    runfiles = runfiles.merge(ctx.attr.runtime[DefaultInfo].data_runfiles)

    return [
        apple_provider.make_apple_test_runner_info(
            execution_environment = ctx.attr.execution_environment,
            execution_requirements = ctx.attr.execution_requirements,
            test_environment = ctx.attr.test_environment,
            test_runner_template = output,
        ),
        AppleDeviceTestRunnerInfo(
            device_type = firmware.device_type,
            os_version = firmware.os_version,
        ),
        DefaultInfo(
            files = depset([output]),
            runfiles = runfiles,
        ),
    ]

ios_vm_test_runner = rule(
    implementation = _ios_vm_test_runner_impl,
    attrs = {
        "boot_args": attr.string(
            default = "rd=md0 serial=19 -v -noprogress wdt=-1 wlan-olyhal-abort",
            doc = "XNU boot arguments used by the VM.",
        ),
        "boot_attempts": attr.int(
            default = 1,
            doc = "Maximum attempts for recognized nondeterministic SPTM/TXM panics.",
        ),
        "boot_timeout": attr.int(
            default = 60,
            doc = "Seconds to wait for the guest root shell on each attempt.",
        ),
        "runtime": attr.label(
            mandatory = True,
            providers = [IosVmRuntimeInfo],
            doc = "Prepared firmware and QEMU runtime used by the VM.",
        ),
        "execution_environment": attr.string_dict(
            doc = "Environment variables for the Bazel test process.",
        ),
        "execution_requirements": attr.string_dict(
            default = {"no-remote": ""},
            doc = "Execution requirements for the Bazel test action.",
        ),
        "test_environment": attr.string_dict(
            doc = "Environment variables propagated into XCTest.",
        ),
        "test_timeout": attr.int(
            default = 300,
            doc = "Seconds to wait for XCTest after the guest has booted.",
        ),
        "_runner": attr.label(
            allow_single_file = True,
            default = Label("//testing:ios_vm_runner.py"),
        ),
        "_template": attr.label(
            allow_single_file = True,
            default = Label("//testing:ios_vm_test_runner.template.sh"),
        ),
    },
    doc = """
Provides AppleTestRunnerInfo for unhosted device arm64 ios_unit_test targets.

The runner boots a prepared darwin-vm ramdisk under QEMU on Linux, patches the
test bundle executable into a fixed APFS slot, and invokes XCTest in the guest.
Hosted unit tests, UI tests, coverage, bundle resources, and nested test
frameworks are intentionally rejected by the initial implementation. Non-SPTM
firmware is the supported configuration; SPTM/TXM inputs are experimental.
""",
)
