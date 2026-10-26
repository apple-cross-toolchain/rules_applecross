load("@rules_python//python:py_test.bzl", "py_test")

py_test(
    name = "extract_xip_test",
    srcs = ["extract_xip_test.py"],
    data = [
        "//toolchain:repo_tools/extract_xip.py",
        "//toolchain:repo_tools/link_local_xcode.py",
    ],
    deps = ["@rules_python//python/runfiles"],
)
