TARGET="em_proxy"

add_targets:
	@echo "add_targets"
	rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios


compile:
	@echo "build aarch64-apple-ios"
	@# @cargo build --release --target aarch64-apple-ios
	@cargo build --target aarch64-apple-ios
	@echo "lipo"
	@# @lipo -create \
		-output target/lib$(TARGET)-ios.a \
		target/aarch64-apple-ios/release/lib$(TARGET).a
	@lipo -create \
		-output target/lib$(TARGET)-ios.a \
		target/aarch64-apple-ios/debug/lib$(TARGET).a

	@echo "build aarch64-apple-ios-sim"
	@# @cargo build --release --target aarch64-apple-ios-sim
	@cargo build --target aarch64-apple-ios-sim

	@echo "build x86_64-apple-ios"
	@# @cargo build --release --target x86_64-apple-ios
	@cargo build --target x86_64-apple-ios

	@# @lipo -create \
		-output target/lib$(TARGET)-sim.a \
		target/aarch64-apple-ios-sim/release/lib$(TARGET).a \
		target/x86_64-apple-ios/release/lib$(TARGET).a
	@lipo -create \
		-output target/lib$(TARGET)-sim.a \
		target/aarch64-apple-ios-sim/debug/lib$(TARGET).a \
		target/x86_64-apple-ios/debug/lib$(TARGET).a

copy:
	@echo "copying binaries:"
	@if [ -d "target" ]; then \
		echo "    target/lib$(TARGET)-sim.a -> lib${TARGET}-sim.a"; \
		cp "target/lib$(TARGET)-sim.a" "lib${TARGET}-sim.a"; \
		echo "    target/lib$(TARGET)-ios.a -> lib${TARGET}-ios.a"; \
		cp "target/lib$(TARGET)-ios.a" "lib${TARGET}-ios.a"; \
	fi

build: compile copy

clean:
	@echo "clean"
	@if [ -d "include" ]; then \
		echo "cleaning include"; \
		rm -r include; \
	fi
	@if [ -d "target" ]; then \
		echo "cleaning target"; \
		rm -r target; \
	fi
	@if [ -d "$(TARGET).xcframework" ]; then \
		echo "cleaning $(TARGET).xcframework"; \
		rm -r $(TARGET).xcframework; \
	fi
	@if [ -f "$(TARGET).xcframework.zip" ]; then \
		echo "cleaning $(TARGET).xcframework.zip"; \
		rm $(TARGET).xcframework.zip; \
	fi
	@rm -f *.a 

xcframework: build
	@echo "xcframework"

	@if [ -d "include" ]; then \
		echo "cleaning include"; \
		rm -rf include; \
	fi
	@mkdir include
	@mkdir include/$(TARGET)
	@cp $(TARGET).h include/$(TARGET)
	@cp module.modulemap include/$(TARGET)

	@if [ -d "$(TARGET).xcframework" ]; then \
		echo "cleaning $(TARGET).xcframework"; \
		rm -rf $(TARGET).xcframework; \
	fi
	@xcodebuild \
			-create-xcframework \
			-library target/aarch64-apple-ios/release/lib$(TARGET).a \
			-headers include/ \
			-library target/lib$(TARGET)-sim.a \
			-headers include/ \
			-output $(TARGET).xcframework

xcframework_frameworks: build
	@echo "xcframework_frameworks"

	@if [ -d "include" ]; then \
		echo "cleaning include"; \
		rm -rf include; \
	fi
	@mkdir include
	@mkdir include/$(TARGET)
	@cp $(TARGET).h include/$(TARGET)
	@cp module.modulemap include/$(TARGET)

	@if [ -d "target/ios" ]; then \
		echo "cleaning target/ios"; \
		rm -rf target/ios; \
	fi
	@mkdir target/ios
	@mkdir target/ios/$(TARGET).framework
	@mkdir target/ios/$(TARGET).framework/Headers

	@if [ -d "target/sim" ]; then \
		echo "cleaning target/sim"; \
		rm -rf target/sim; \
	fi
	@mkdir target/sim
	@mkdir target/sim/$(TARGET).framework
	@mkdir target/sim/$(TARGET).framework/Headers

	@cp include/*.* target/ios/$(TARGET).framework/Headers
	@libtool -static \
		-o target/ios/$(TARGET).framework/$(TARGET) \
		target/lib$(TARGET)-ios.a

	@cp include/*.* target/sim/$(TARGET).framework/Headers
	@xcrun \
		-sdk iphonesimulator \
		libtool -static \
		-o target/sim/$(TARGET).framework/$(TARGET) \
		target/lib$(TARGET)-sim.a

	@if [ -d "$(TARGET).xcframework" ]; then \
		echo "cleaning $(TARGET).xcframework"; \
		rm -rf $(TARGET).xcframework; \
	fi
	@xcodebuild -create-xcframework \
			-library target/sim/$(TARGET).framework \
			-headers include/ \
			-library target/ios/$(TARGET).framework \
			-headers include/ \
			-output $(TARGET).xcframework

zip: xcframework
	@echo "zip"
	@if [ -f "$(TARGET).xcframework.zip" ]; then \
		echo "cleaning $(TARGET).xcframework.zip"; \
		rm $(TARGET).xcframework.zip; \
	fi
	zip -r $(TARGET).xcframework.zip $(TARGET).xcframework
