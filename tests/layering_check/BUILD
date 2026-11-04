load("@bazel_skylib//rules:build_test.bzl", "build_test")
load("@rules_cc//cc:cc_library.bzl", "cc_library")
load(":ios_simulator.bzl", "ios_simulator_files")

licenses(["notice"])

# layering_check with the toolchain's `crosstool` module: a library using its
# direct dependency's header and the SDK's and C++ library's headers.
package(features = ["layering_check"])

cc_library(
    name = "base",
    hdrs = ["base.h"],
    tags = ["manual"],
)

cc_library(
    name = "user",
    srcs = ["user.cc"],
    tags = ["manual"],
    deps = [":base"],
)

ios_simulator_files(
    name = "user_ios",
    srcs = [":user"],
)

build_test(
    name = "layering_check_test",
    targets = [":user_ios"],
)
