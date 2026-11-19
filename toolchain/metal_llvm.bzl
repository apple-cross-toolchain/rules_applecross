"""LLVM 15 for Metal shaders under relax_metal (stubs/metal.sh).

toolchains_llvm's `llvm` extension may be used by the root module only, so a
module depending on rules_applecross couldn't fetch it. This extension makes
the same repository with toolchains_llvm's `llvm` repository rule, which has
no such limit, under the name that extension gave it.
"""

load("@toolchains_llvm//toolchain:rules.bzl", "llvm")

def _metal_llvm_impl(module_ctx):
    llvm(
        name = "applecross_metal_llvm_llvm",
        llvm_versions = {"": "15.0.6"},
    )
    return module_ctx.extension_metadata(reproducible = True)

metal_llvm = module_extension(
    implementation = _metal_llvm_impl,
    doc = "LLVM 15's clang, opt and llvm-as, fetched only by relax_metal builds.",
)
