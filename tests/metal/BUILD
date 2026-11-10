load("@build_bazel_rules_apple//apple:ios.bzl", "ios_application")
load("@rules_python//python:py_test.bzl", "py_test")
load("@rules_swift//swift:swift.bzl", "swift_library")
load(":relax_metal.bzl", "relax_metal_filegroup")

licenses(["notice"])

swift_library(
    name = "Main",
    srcs = ["Sources/main.swift"],
    tags = ["manual"],
)

# Its shaders compiled off a Mac with relax_metal (tools/apple_tools/crates/metal).
ios_application(
    name = "ShaderApp",
    bundle_id = "com.example.metal.shader-app",
    families = ["iphone"],
    infoplists = ["Resources/Info.plist"],
    minimum_os_version = "17.0",
    resources = ["Resources/Shaders.metal"],
    tags = ["manual"],
    deps = [":Main"],
)

relax_metal_filegroup(
    name = "shader_app",
    srcs = [":ShaderApp"],
)

py_test(
    name = "metal_test",
    srcs = ["metal_test.py"],
    args = ["$(rootpath :shader_app)"],
    data = [":shader_app"],
)
