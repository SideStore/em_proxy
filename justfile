xcframework: apple-build
  rm -rf   libs
  mkdir -p libs

  xcodebuild -create-xcframework \
    -library target/aarch64-apple-ios/release/libem_proxy.a -headers include \
    -library target/aarch64-apple-ios-sim/release/libem_proxy.a -headers include \
    -library target/aarch64-apple-darwin/release/libem_proxy.a -headers include \
    -output  libs/EMProxy.xcframework

apple-build:
  BINDGEN_EXTRA_CLANG_ARGS="--sysroot=$(xcrun --sdk iphoneos --show-sdk-path)" \
    IPHONEOS_DEPLOYMENT_TARGET=15.0 \
    cargo build --release --target aarch64-apple-ios

  BINDGEN_EXTRA_CLANG_ARGS="--sysroot=$(xcrun --sdk iphonesimulator --show-sdk-path)" \
    cargo build --release --target aarch64-apple-ios-sim

  cargo build --release --target aarch64-apple-darwin
