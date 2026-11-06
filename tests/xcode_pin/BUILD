load("@rules_python//python:py_test.bzl", "py_test")
load(":xcode_pin_test.bzl", "xcode_pin_test_suite")

licenses(["notice"])

xcode_pin_test_suite(name = "xcode_pin_test")

py_test(
    name = "read_xcode_versions_test",
    srcs = ["read_xcode_versions_test.py"],
    args = ["$(rootpath //toolchain:repo_tools/read_xcode_versions.py)"],
    data = ["//toolchain:repo_tools/read_xcode_versions.py"],
)
