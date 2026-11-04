package(default_visibility = [
    "//examples:__subpackages__",
])

licenses(["notice"])

exports_files([
    "Doubler.mlpackage",
    "Intents.intentdefinition",
    "Launch.storyboard",
    "Localizable.xcstrings",
])

filegroup(
    name = "MacAppIcon.xcassets",
    srcs = glob(["MacAppIcon.xcassets/**"]),
)

filegroup(
    name = "PhoneAppIcon.xcassets",
    srcs = glob(["PhoneAppIcon.xcassets/**"]),
)

# A versioned Core Data model (off a Mac, compiled with --config=relax_momc).
filegroup(
    name = "Shelf.xcdatamodeld",
    srcs = glob(["Shelf.xcdatamodeld/**"]),
)

filegroup(
    name = "TVBrandAssets.xcassets",
    srcs = glob(["TVBrandAssets.xcassets/**"]),
)

filegroup(
    name = "VisionAppIcon.xcassets",
    srcs = glob(["VisionAppIcon.xcassets/**"]),
)

filegroup(
    name = "WatchAppIcon.xcassets",
    srcs = glob(["WatchAppIcon.xcassets/**"]),
)
