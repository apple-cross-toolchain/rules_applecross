load("//testing:ios_vm_runtime.bzl", "ios_vm_firmware", "ios_vm_runtime")
load("//testing:ios_vm_test_runner.bzl", "ios_vm_test_runner")

package(default_visibility = ["//visibility:private"])

exports_files([
    "empty",
    "slot.json",
])

ios_vm_firmware(
    name = "analysis_firmware",
    bootkc = ":empty",
    device_tree = ":empty",
    device_type = "iPhone 12",
    os_version = "26.5",
    ramdisk = ":empty",
    runtime_name = "analysis_runtime",
    slot_manifest = ":slot.json",
    trust_cache = ":empty",
)

ios_vm_runtime(
    name = "analysis_runtime",
    firmware = ":analysis_firmware",
    qemu = ":empty",
)

ios_vm_test_runner(
    name = "analysis_runner",
    runtime = ":analysis_runtime",
)
