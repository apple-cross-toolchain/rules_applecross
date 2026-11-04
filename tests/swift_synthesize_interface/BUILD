load("@bazel_skylib//rules:build_test.bzl", "build_test")
load(":synthesized_interfaces.bzl", "synthesized_interfaces")

licenses(["notice"])

# swift-synthesize-interface (swift.org's, the toolchain's Swift) on a Swift
# module over a C one, for the iOS simulator.
synthesized_interfaces(
    name = "Greeter",
    tags = ["manual"],
    deps = ["//tests/stub_sdks:Greeter"],
)

build_test(
    name = "synthesize_interface_test",
    targets = [":Greeter"],
)
