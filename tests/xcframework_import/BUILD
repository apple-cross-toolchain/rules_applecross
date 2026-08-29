load("@build_bazel_rules_apple//apple:apple.bzl", "apple_static_xcframework_import")

licenses(["notice"])

# Guards a regression, and is only ever analyzed. The XCFramework import rules
# take a mandatory Swift toolchain, and no Swift toolchain can execute on macOS
# here, so requiring the product toolchain type on them makes any consumer fail
# analysis. They are compile preprocessing and belong on the executors with the
# rest of compilation; the contents of the framework do not matter for that.
#
#   bazel build --nobuild --config=remote_compile //tests/xcframework_import:Probe
apple_static_xcframework_import(
    name = "Probe",
    tags = ["manual"],
    xcframework_imports = glob(["Fake.xcframework/**"]),
)
