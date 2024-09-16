// swift-tools-version: 6.0
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "em_proxy",
    products: [
        .library(
            name: "em_proxy",
            targets: ["em_proxy"]),
    ],
    targets: [
        .target(
            name: "em_proxy",
            dependencies: ["libem_proxy"]),
        
        .target(
            name: "libem_proxy",
            dependencies: ["em_proxy-binary"]
        ),

        .binaryTarget(
            name: "em_proxy-binary",
            url: "https://github.com/SideStore/em_proxy/releases/download/build/em_proxy.xcframework.zip",
            checksum: "79f90075b8ff2f47540a5bccf5fb7740905cda63463f833e2505256237df3c1b"),
    
        .testTarget(
            name: "em_proxyTests",
            dependencies: ["em_proxy", "libem_proxy"]
        ),
    ],
    swiftLanguageModes: [.v5]
)
