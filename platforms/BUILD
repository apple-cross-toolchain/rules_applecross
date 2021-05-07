platform(
    name = "docker_image_platform",
    constraint_values = [
        "@bazel_tools//platforms:x86_64",
        "@bazel_tools//platforms:linux",
        "@bazel_tools//tools/cpp:clang",
    ],
    exec_properties = {
        "OSFamily": "Linux",
        "container-image": "docker://ghcr.io/apple-cross-toolchain/xcode@sha256:5c17b8add19de66e0a473f6f63bbc53a7a328613881c37205d353286d4bdec97",
    },
)
