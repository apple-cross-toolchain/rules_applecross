load("@rules_python//python:py_test.bzl", "py_test")

# coremlc compile and generate off a Mac, against Xcode 27.0's output for the
# same models: cases/X/<name>.mlpackage (or .mlmodel) compiles to expected/X/compiled/ and
# generates expected/X/swift/ and expected/X/objc/, where those exist.
py_test(
    name = "coremlc_test",
    srcs = ["coremlc_test.py"],
    data = [
        "//toolchain:stubs/coremlc.py",
        "//toolchain:stubs/coremlc_ops.json",
    ] + glob([
        "cases/**",
        "expected/**",
    ]),
    deps = ["@rules_python//python/runfiles"],
)
