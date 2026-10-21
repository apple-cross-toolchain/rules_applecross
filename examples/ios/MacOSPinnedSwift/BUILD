load("@build_bazel_rules_apple//apple:ios.bzl", "ios_application")
load("@rules_swift//swift:swift.bzl", "swift_library")

# Compiled on whichever platform comes first (Linux under --config=remote_compile).
swift_library(
    name = "Counter",
    srcs = ["Lib/Counter.swift"],
    module_name = "Counter",
    tags = ["manual"],
)

# Pinned to a Mac, yet still imports Counter and expands SwiftUI's @State macro,
# whose plugin exists only for macOS.
swift_library(
    name = "App",
    srcs = [
        "App/App.swift",
        "App/ContentView.swift",
    ],
    exec_compatible_with = ["@platforms//os:macos"],
    linkopts = ["-Wl,-framework,SwiftUI"],
    module_name = "MacOSPinnedSwift",
    tags = ["manual"],
    deps = [":Counter"],
)

ios_application(
    name = "MacOSPinnedSwift",
    bundle_id = "com.example.MacOSPinnedSwift",
    families = ["iphone"],
    infoplists = ["App/Info.plist"],
    minimum_os_version = "17.0",
    tags = ["manual"],
    deps = [":App"],
)
