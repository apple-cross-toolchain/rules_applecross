load("@rules_python//python:py_test.bzl", "py_test")

# tools/xcode_equivalence.py on two of Xcode 27.0's own runs of actool over
# the same catalog (rules_apple's test/testdata/resources/assets.xcassets,
# which has a dark appearance), and of coremlc over the same models (a
# categorical mapping, a custom model with parameters and a k-nearest
# neighbors classifier with user-defined metadata), of docc over the same
# module (a page of its DocC archive), and of ibtool over the same Mac
# storyboard (data/Window.storyboard: its window controller's nib and the
# storyboard's Info.plist): different bytes, equivalent files.
py_test(
    name = "xcode_equivalence_test",
    srcs = ["xcode_equivalence_test.py"],
    data = glob(["data/**"]),
    deps = [
        "//tools:xcode_equivalence",
        "@rules_python//python/runfiles",
    ],
)
