xcframework: apple-build
  rm -rf lib/EMProxyFFI.xcframework
  rm -rf libs
  mkdir libs
  cp NativeBridge/target/aarch64-apple-ios-sim/release/libem_proxy.a libs/em_proxy-ios-sim.a
  cp NativeBridge/target/aarch64-apple-darwin/release/libem_proxy.a libs/em_proxy-macos.a

  mkdir -p lib
  xcodebuild -create-xcframework \
    -library NativeBridge/target/aarch64-apple-ios/release/libem_proxy.a -headers NativeBridge/include \
    -library libs/em_proxy-ios-sim.a -headers NativeBridge/include \
    -library libs/em_proxy-macos.a -headers NativeBridge/include \
    -output lib/EMProxyFFI.xcframework

[working-directory: 'NativeBridge']
apple-build:
  BINDGEN_EXTRA_CLANG_ARGS="--sysroot=$(xcrun --sdk iphoneos --show-sdk-path)" \
    IPHONEOS_DEPLOYMENT_TARGET=15.0 \
    cargo build --release --target aarch64-apple-ios

  BINDGEN_EXTRA_CLANG_ARGS="--sysroot=$(xcrun --sdk iphonesimulator --show-sdk-path)" \
    cargo build --release --target aarch64-apple-ios-sim

  cargo build --release --target aarch64-apple-darwin
