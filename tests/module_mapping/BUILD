load("@rules_swift//swift:swift.bzl", "swift_library")
load("@rules_swift//swift:swift_module_mapping.bzl", "swift_module_mapping")

# A module compiled under another name (as Isomer compiles its SwiftUI as
# IsomerCompiledSwiftUI): with --@rules_swift//swift:module_mapping=:mapping
# the toolchain gives Greeter's compile `-module-alias Greeter=AliasedGreeter`
# and its importer's too.
swift_module_mapping(
    name = "mapping",
    aliases = {"Greeter": "AliasedGreeter"},
)

swift_library(
    name = "Greeter",
    srcs = ["Greeter.swift"],
    module_name = "Greeter",
    tags = ["manual"],
)

swift_library(
    name = "Greeting",
    srcs = ["Greeting.swift"],
    module_name = "Greeting",
    tags = ["manual"],
    deps = [":Greeter"],
)
