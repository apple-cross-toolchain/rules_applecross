platform(
    name = "docker_image_platform",
    constraint_values = [
        "@bazel_tools//platforms:x86_64",
        "@bazel_tools//platforms:linux",
        "@bazel_tools//tools/cpp:clang",
    ],
    exec_properties = {
        "OSFamily": "Linux",
        "container-image": "docker://ghcr.io/apple-cross-toolchain/xcode@sha256:3440b551d4ab26eec20a95d4c0de5f01a1c4b12b699710e8c3624fd1652de02c",
    },
)
