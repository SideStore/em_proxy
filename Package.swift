// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "em_proxy",
    platforms: [
        .iOS(.v13),
        .macOS(.v11)
    ],
    products: [
        .library(
            name: "em_proxy",
            targets: ["em_proxy"]
        )
    ],
    targets: [
        .binaryTarget(
            name: "NativeBridge",
            path: "lib/EMProxyFFI.xcframework"
        ),
        .target(
            name: "em_proxy",
            dependencies: ["NativeBridge"],
            path: "Sources"
        )
    ]
)
