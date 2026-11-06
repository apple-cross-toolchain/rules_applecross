load("@build_bazel_rules_apple//apple:ios.bzl", "ios_application")
load("@rules_cc//cc:objc_library.bzl", "objc_library")
load("@rules_swift//swift:swift.bzl", "swift_library")

# A Swift app built with explicit modules: its Objective-C dependency and the
# SDK's modules (UIKit, Foundation and below, from MODULE.bazel's
# swift_system_modules) are precompiled (SwiftPrecompileCModule,
# SwiftCompileModuleInterface) rather than built implicitly by the compiler.
# For the whole package, as a project turns them on for its build: the
# Objective-C library's module is only precompiled where it is enabled.
package(features = [
    "swift.emit_c_module",
    "swift.use_c_modules",
])

licenses(["notice"])

objc_library(
    name = "Greeting",
    srcs = ["Sources/Greeting.m"],
    hdrs = ["Sources/Greeting.h"],
    module_name = "Greeting",
    tags = ["manual"],
)

swift_library(
    name = "Sources",
    srcs = ["Sources/App.swift"],
    module_name = "ExplicitModules",
    tags = ["manual"],
    deps = [":Greeting"],
)

ios_application(
    name = "ExplicitModules",
    bundle_id = "com.example.explicit-modules",
    families = ["iphone"],
    infoplists = ["Info.plist"],
    minimum_os_version = "17.0",
    deps = [":Sources"],
)
