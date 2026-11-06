load("@rules_python//python:py_test.bzl", "py_test")

licenses(["notice"])

py_test(
    name = "clang_apple_flags_test",
    srcs = ["clang_apple_flags_test.py"],
    args = ["$(rootpath @apple_cross_toolchain//:wrapped_clang)"],
    data = ["@apple_cross_toolchain//:wrapped_clang"],
)
