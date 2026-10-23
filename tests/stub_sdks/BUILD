load("@build_bazel_rules_apple//apple:ios.bzl", "ios_application", "ios_framework")
load("@build_bazel_rules_apple//apple:macos.bzl", "macos_command_line_application")
load("@build_bazel_rules_apple//apple:tvos.bzl", "tvos_application")
load("@build_bazel_rules_apple//apple:visionos.bzl", "visionos_application")
load("@build_bazel_rules_apple//apple:watchos.bzl", "watchos_application")
load("@rules_cc//cc:cc_library.bzl", "cc_library")
load("@rules_python//python:py_test.bzl", "py_test")
load("@rules_swift//swift:swift.bzl", "swift_library")
load("@rules_swift//swift:swift_interop_hint.bzl", "swift_interop_hint")
load(":apple_transition_filegroup.bzl", "apple_transition_filegroup")

licenses(["notice"])

swift_interop_hint(
    name = "GreetingModule",
    module_name = "Greeting",
)

cc_library(
    name = "Greeting",
    srcs = ["Sources/greeting.c"],
    hdrs = ["Sources/greeting.h"],
    aspect_hints = [":GreetingModule"],
    tags = ["manual"],
)

cc_library(
    name = "CMain",
    srcs = ["Sources/main.c"],
    tags = ["manual"],
    deps = [":Greeting"],
)

swift_library(
    name = "Greeter",
    srcs = ["Sources/Greeter.swift"],
    module_name = "Greeter",
    tags = ["manual"],
    deps = [":Greeting"],
)

swift_library(
    name = "Main",
    srcs = ["Sources/main.swift"],
    module_name = "Main",
    tags = ["manual"],
    deps = [":Greeter"],
)

macos_command_line_application(
    name = "CTool",
    bundle_id = "com.example.stub-sdks.c-tool",
    infoplists = ["Resources/Tool-Info.plist"],
    minimum_os_version = "14.0",
    tags = ["manual"],
    deps = [":CMain"],
)

macos_command_line_application(
    name = "SwiftTool",
    bundle_id = "com.example.stub-sdks.swift-tool",
    infoplists = ["Resources/Tool-Info.plist"],
    minimum_os_version = "14.0",
    tags = ["manual"],
    deps = [":Main"],
)

ios_framework(
    name = "GreeterFramework",
    bundle_id = "com.example.stub-sdks.greeter",
    bundle_name = "Greeter",
    families = [
        "iphone",
        "ipad",
    ],
    infoplists = ["Resources/Info.plist"],
    minimum_os_version = "17.0",
    tags = ["manual"],
    deps = [":Greeter"],
)

ios_application(
    name = "App",
    bundle_id = "com.example.stub-sdks",
    families = [
        "iphone",
        "ipad",
    ],
    frameworks = [":GreeterFramework"],
    infoplists = ["Resources/Info.plist"],
    minimum_os_version = "17.0",
    resources = ["Resources/Greeting.txt"],
    strings = ["Resources/en.lproj/Localizable.strings"],
    tags = ["manual"],
    deps = [":Main"],
)

ios_application(
    name = "DeviceApp",
    bundle_id = "com.example.stub-sdks.device",
    families = ["iphone"],
    features = ["disable_legacy_signing"],
    frameworks = [":GreeterFramework"],
    infoplists = ["Resources/Info.plist"],
    minimum_os_version = "17.0",
    tags = ["manual"],
    deps = [":Main"],
)

tvos_application(
    name = "TVApp",
    bundle_id = "com.example.stub-sdks.tv",
    infoplists = ["Resources/Info.plist"],
    minimum_os_version = "17.0",
    tags = ["manual"],
    deps = [":Main"],
)

visionos_application(
    name = "VisionApp",
    bundle_id = "com.example.stub-sdks.vision",
    infoplists = ["Resources/Info.plist"],
    minimum_os_version = "1.0",
    tags = ["manual"],
    deps = [":Main"],
)

watchos_application(
    name = "WatchApp",
    bundle_id = "com.example.stub-sdks.watch",
    infoplists = ["Resources/Watch-Info.plist"],
    minimum_os_version = "10.0",
    tags = ["manual"],
    deps = [":Main"],
)

apple_transition_filegroup(
    name = "ios_simulator",
    ios_multi_cpus = ["sim_arm64"],
    srcs = [":App"],
)

apple_transition_filegroup(
    name = "ios_device",
    compilation_mode = "opt",
    ios_multi_cpus = ["arm64"],
    srcs = [":DeviceApp"],
)

apple_transition_filegroup(
    name = "macos_c",
    macos_cpus = ["x86_64"],
    srcs = [":CTool"],
)

apple_transition_filegroup(
    name = "macos_swift",
    macos_cpus = ["arm64"],
    srcs = [":SwiftTool"],
)

apple_transition_filegroup(
    name = "tvos_simulator",
    srcs = [":TVApp"],
    tvos_cpus = ["x86_64"],
)

apple_transition_filegroup(
    name = "visionos_simulator",
    srcs = [":VisionApp"],
    visionos_cpus = ["sim_arm64"],
)

apple_transition_filegroup(
    name = "watchos_simulator",
    srcs = [":WatchApp"],
    watchos_cpus = ["arm64"],
)

_OUTPUTS = [
    "ios_device",
    "ios_simulator",
    "macos_c",
    "macos_swift",
    "tvos_simulator",
    "visionos_simulator",
    "watchos_simulator",
]

py_test(
    name = "stub_sdks_test",
    srcs = ["stub_sdks_test.py"],
    args = ["{}=$(rootpath :{})".format(name, name) for name in _OUTPUTS],
    data = [":" + name for name in _OUTPUTS],
)
