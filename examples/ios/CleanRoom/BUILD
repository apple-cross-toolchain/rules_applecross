load("@build_bazel_rules_apple//apple:ios.bzl", "ios_application")
load("@rules_swift//swift:swift.bzl", "swift_library")

package(default_visibility = ["//visibility:public"])

# Compared byte for byte between Linux and a Mac by tools/compare_ipa.sh.
swift_library(
    name = "CleanRoomLib",
    srcs = glob(["Sources/*.swift"]),
    linkopts = ["-Wl,-framework,SwiftUI"],
    module_name = "CleanRoom",
    tags = ["manual"],
)

ios_application(
    name = "CleanRoom",
    app_icons = glob(["Resources/Assets.xcassets/AppIcon.appiconset/**"]),
    bundle_id = "com.example.CleanRoom",
    families = [
        "iphone",
        "ipad",
    ],
    infoplists = ["Sources/Info.plist"],
    launch_storyboard = "Resources/LaunchScreen.storyboard",
    minimum_os_version = "18.0",
    # Synthetic profile from a throwaway certificate; ad hoc signing only embeds it.
    provisioning_profile = "CleanRoom.mobileprovision",
    deps = [":CleanRoomLib"],
)
