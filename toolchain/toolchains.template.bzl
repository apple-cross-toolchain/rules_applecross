def xcode_register_toolchains():
    native.register_toolchains(
        "@%{repo_name}//:cc-toolchain-linux",
        "@%{repo_name}//:cc-toolchain-darwin",
    )
