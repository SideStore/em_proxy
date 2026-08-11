# Emotional Mangling Proxy

_What Apple and I both share after this is done_

[![Build em_proxy](https://github.com/SideStore/em_proxy/actions/workflows/ci.yml/badge.svg)](https://github.com/SideStore/em_proxy/actions/workflows/ci.yml)

## What is EMP

A transparent interface level proxy for TCP packets. This project is designed to work around the loopback limitations on iOS.

## How does it work

`em_proxy` acts as a local WireGuard server listening on loopback (`127.0.0.1`) that performs packet source/destination address swapping.

It leverages Cloudflare's **boringtun** userspace WireGuard implementation to understand WireGuard encryption, decrypting incoming UDP packets from the WireGuard VPN app and rewriting IP headers so packets appear from a non-self origin address, acting as a proxy server for loopback relay.

## Prerequisites

Building `em_proxy` locally requires:

- **Rust & Cargo** (installed via [rustup](https://rustup.rs/))
- **just** task runner (`brew install just`)
- **Xcode & Command Line Tools** (for `xcodebuild` and `xcrun`)

Install the required Apple compilation targets:

```sh
rustup target add aarch64-apple-ios
rustup target add aarch64-apple-ios-sim
rustup target add aarch64-apple-darwin
```

## How to build

Generate the Apple XCFramework (`libs/EMProxy.xcframework`):

```sh
just xcframework
```

Or build individual target static libraries using cargo:

```sh
cargo build --release --target aarch64-apple-ios
```

## CI & Publishing Releases

### Continuous Integration

Pushes to branches (`develop`, `main`, `master`), pull requests, and tag pushes all trigger GitHub Actions CI to build and verify `EMProxy.xcframework.zip`.

### Publishing a Release

Only pushing a version tag matching `v*.*.*` will generate and publish a GitHub Release with `EMProxy.xcframework.zip`, ready for use in Swift Package Manager via `.binaryTarget`:

```swift
.binaryTarget(
    name: "EMProxyFFI",
    url: "https://github.com/SideStore/em_proxy/releases/download/v0.1.0/EMProxy.xcframework.zip",
    checksum: "<sha256-checksum>"
)
```

To publish a release:

- Push a version tag matching `v*.*.*` (e.g. `v0.1.0`):

```sh
git tag v0.1.0
git push origin v0.1.0
```

> **Note**: Updating or force-pushing tags is **only allowed for the latest release**. Correcting or updating tags for any releases prior to the last release is disallowed and will trigger a build error.

## Progress

![Alt](https://repobeats.axiom.co/api/embed/bb97132e96fd2c4caac60aa1441ae55b6382afec.svg "Repobeats analytics image")
