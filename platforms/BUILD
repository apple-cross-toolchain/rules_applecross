package(default_visibility = ["//visibility:public"])

# The developer's own Mac, used by --config=remote_compile for everything that
# wants real Xcode tools. Listed after linux_x86_64 so compiles still resolve
# remotely; only rules requiring //toolchain:apple_bundling_toolchain_type, which
# has no Linux instance in that config, land here.
platform(
    name = "macos_arm64_local",
    constraint_values = [
        "@platforms//cpu:arm64",
        "@platforms//os:macos",
    ],
    exec_properties = {
        # Only exec_properties reach the executor; the constraint_values above
        # are analysis-only and decide toolchain resolution, nothing more. So a
        # platform meaning "this machine" has no properties to route on, and
        # with a remote executor configured its actions would be sent off with
        # an empty platform and land in whichever pool the service defaults to.
        # This is what says they stay. A macOS executor pool is the other case:
        # a sibling platform carrying that pool's properties instead, which
        # apple_bundling_on_macos matches just as well, since it constrains
        # nothing but the operating system.
        "no-remote-exec": "1",
    },
)

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
