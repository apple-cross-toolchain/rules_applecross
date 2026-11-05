load("@rules_python//python:py_test.bzl", "py_test")

# momc off a Mac, against Xcode 27.2's output for the same models:
# cases/X/Model.xcdatamodeld compiles to expected/X/*.mom (each version's),
# each refused/X/Model.xcdatamodeld is refused, and generate/X/Model.xcdatamodeld
# generates generate_expected/X/*.swift.
py_test(
    name = "momc_test",
    srcs = ["momc_test.py"],
    data = [
        "//toolchain:stubs/corefoundation.py",
        "//toolchain:stubs/momc.py",
    ] + glob([
        "cases/**",
        "expected/**",
        "generate/**",
        "generate_expected/**",
        "refused/**",
    ]),
    deps = ["@rules_python//python/runfiles"],
)
