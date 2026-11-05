load("@bazel_skylib//rules:build_test.bzl", "build_test")
load("@build_bazel_rules_apple//apple:apple.bzl", "apple_dynamic_framework_import")
load("@build_bazel_rules_apple//apple:ios.bzl", "ios_application", "ios_framework")
load("@build_bazel_rules_apple//apple:versioning.bzl", "apple_bundle_version")
load("@rules_cc//cc:objc_library.bzl", "objc_library")
load("@rules_python//python:py_binary.bzl", "py_binary")

licenses(["notice"])

objc_library(
    name = "GreeterLib",
    srcs = ["Sources/Greeter.m"],
    hdrs = ["Sources/Greeter.h"],
    tags = ["manual"],
)

ios_framework(
    name = "GreeterFramework",
    bundle_id = "com.example.greeter-framework",
    bundle_name = "Greeter",
    families = [
        "iphone",
        "ipad",
    ],
    hdrs = ["Sources/Greeter.h"],
    infoplists = ["Info.plist"],
    minimum_os_version = "15.0",
    deps = [":GreeterLib"],
)

# The framework above, imported as a prebuilt dynamic framework into an app:
# its processing runs the toolchain's lipo through xcrun, which a sandbox or
# remote executor has only as declared inputs.
py_binary(
    name = "extract",
    srcs = ["extract.py"],
    tags = ["manual"],
)

genrule(
    name = "ImportedGreeterFiles",
    srcs = [":GreeterFramework"],
    outs = [
        "Imported/Greeter.framework/Greeter",
        "Imported/Greeter.framework/Info.plist",
    ],
    cmd = "$(execpath :extract) $< $(RULEDIR)/Imported Greeter.framework/Greeter Greeter.framework/Info.plist",
    tags = ["manual"],
    tools = [":extract"],
)

apple_dynamic_framework_import(
    name = "ImportedGreeter",
    framework_imports = [":ImportedGreeterFiles"],
    tags = ["manual"],
)

objc_library(
    name = "ImporterSources",
    srcs = ["ImporterSources/main.m"],
    tags = ["manual"],
    deps = [":ImportedGreeter"],
)

apple_bundle_version(
    name = "GreeterImporterVersion",
    build_version = "1.0",
)

ios_application(
    name = "GreeterImporter",
    bundle_id = "com.example.greeter-importer",
    families = [
        "iphone",
        "ipad",
    ],
    infoplists = ["Importer-Info.plist"],
    minimum_os_version = "15.0",
    version = ":GreeterImporterVersion",
    deps = [":ImporterSources"],
)

build_test(
    name = "ExamplesBuildTest",
    targets = [
        ":GreeterFramework",
        ":GreeterImporter",
    ],
)
