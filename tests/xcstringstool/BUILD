load("@rules_python//python:py_test.bzl", "py_test")

# xcstringstool compile off a Mac, against Xcode 27.0's output for the same
# catalogs: cases/X.xcstrings compiles to expected/X/ (an `.empty` file marks
# a directory Xcode left empty), or fails where expected/X.fails is.
py_test(
    name = "xcstringstool_test",
    srcs = ["xcstringstool_test.py"],
    data = [
        "//toolchain:stubs/xcstringstool.py",
    ] + glob([
        "cases/**",
        "expected/**",
        "refused/**",
    ]),
    deps = ["@rules_python//python/runfiles"],
)
