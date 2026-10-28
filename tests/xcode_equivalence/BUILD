load("@rules_python//python:py_test.bzl", "py_test")

# tools/xcode_equivalence.py on two of Xcode 27.0's own runs of actool over
# the same catalog (rules_apple's test/testdata/resources/assets.xcassets,
# which has a dark appearance): different bytes, equivalent files.
py_test(
    name = "xcode_equivalence_test",
    srcs = ["xcode_equivalence_test.py"],
    data = glob(["data/**"]),
    deps = [
        "//tools:xcode_equivalence",
        "@rules_python//python/runfiles",
    ],
)
