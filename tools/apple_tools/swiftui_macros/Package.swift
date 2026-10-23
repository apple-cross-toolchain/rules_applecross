// swift-tools-version: 6.0

// SwiftUI's macro plugin for Linux; see docs/clean-room/swiftui-macros.md.
import PackageDescription

let package = Package(
    name: "swiftui_macros",
    platforms: [.macOS(.v13)],
    dependencies: [
        .package(url: "https://github.com/swiftlang/swift-syntax.git", exact: "604.0.0"),
    ],
    targets: [
        .executableTarget(
            name: "SwiftUIMacros",
            dependencies: [
                .product(name: "SwiftCompilerPlugin", package: "swift-syntax"),
                .product(name: "SwiftSyntax", package: "swift-syntax"),
                .product(name: "SwiftSyntaxMacros", package: "swift-syntax"),
            ]
        ),
        .testTarget(
            name: "SwiftUIMacrosTests",
            dependencies: [
                "SwiftUIMacros",
                .product(name: "SwiftSyntaxMacrosTestSupport", package: "swift-syntax"),
            ]
        ),
    ]
)
