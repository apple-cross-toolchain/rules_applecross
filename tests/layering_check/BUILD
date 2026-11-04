load("@bazel_skylib//rules:build_test.bzl", "build_test")
load("@rules_cc//cc:cc_library.bzl", "cc_library")
load(":ios_simulator.bzl", "ios_simulator_files")

licenses(["notice"])

# layering_check with the toolchain's `crosstool` module: a library using its
# direct dependency's header and one of the developer directory's (clang's
# stddef.h: the stub SDKs CI builds against have no C library headers).
package(features = ["layering_check"])

cc_library(
    name = "base",
    hdrs = ["base.h"],
    tags = ["manual"],
)

cc_library(
    name = "user",
    srcs = ["user.c"],
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
