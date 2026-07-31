platform(
    name = "linux_x86_64",
    constraint_values = [
        "@llvm//constraints/libc:musl",
        "@platforms//cpu:x86_64",
        "@platforms//os:linux",
    ],
    exec_properties = {
        "OSFamily": "Linux",
        "container-image": "docker://ubuntu:24.04",
    },
)
