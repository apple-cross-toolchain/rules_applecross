load("@rules_python//python:py_test.bzl", "py_test")

# appintentsmetadataprocessor off a Mac, against Xcode 27.0's output for the
# same constant values: cases/X.json (a .swiftconstvalues of module MyApp)
# gives expected/X.json once its keys are sorted, as rules_apple sorts them,
# or fails where expected/X.fails has Xcode's message.
py_test(
    name = "appintentsmetadataprocessor_test",
    srcs = ["appintentsmetadataprocessor_test.py"],
    data = [
        "//toolchain:stubs/appintentsmetadataprocessor.py",
    ] + glob([
        "cases/**",
        "expected/**",
    ]),
    deps = ["@rules_python//python/runfiles"],
)
