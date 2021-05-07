load("@build_bazel_rules_apple//apple:ios.bzl", "ios_application")
load("@build_bazel_rules_apple//apple:ios.bzl", "ios_ui_test")
load("@build_bazel_rules_apple//apple:ios.bzl", "ios_unit_test")
load("@build_bazel_rules_apple//apple:versioning.bzl", "apple_bundle_version")
load("@build_bazel_rules_swift//swift:swift.bzl", "swift_library")

package(default_visibility = ["//visibility:public"])

swift_library(
    name = "HelloWorldSwiftUI",
    srcs = glob(["Sources/*.swift"]),
    data = glob(
        [
            "Sources/Assets.xcassets/**",
        ],
        exclude = ["Sources/Assets.xcassets/AppIcon.appiconset/**"],
    ),
    linkopts = [
        "-Wl,-framework,SwiftUI",
    ],
    module_name = "HelloWorldSwiftUI",
)

apple_bundle_version(
    name = "Version",
    build_version = "1.0",
)

ios_application(
    name = "HelloWorld",
    app_icons = glob(["Sources/Assets.xcassets/AppIcon.appiconset/**"]),
    bundle_id = "com.example.HelloWorldSwiftUI",
    families = [
        "iphone",
        "ipad",
    ],
    infoplists = ["Sources/Info.plist"],
    minimum_os_version = "14.0",
    version = ":Version",
    deps = [":HelloWorldSwiftUI"],
)

swift_library(
    name = "TestsLib",
    srcs = glob(["Tests/*.swift"]),
    deps = [":HelloWorldSwiftUI"],
)

ios_unit_test(
    name = "Tests",
    minimum_os_version = "14.0",
    deps = [":TestsLib"],
)

swift_library(
    name = "UITestsLib",
    srcs = glob(["UITests/*.swift"]),
    deps = [":HelloWorldSwiftUI"],
)

ios_ui_test(
    name = "UITests",
    minimum_os_version = "14.0",
    deps = [":UITestsLib"],
    test_host = ":HelloWorld",
)
