.DEFAULT_GOAL := build

TARGET := em_proxy

ARCH_IOS_NATIVE := aarch64-apple-ios
ARCH_IOS_SIM    := aarch64-apple-ios-sim
ARCH_IOS_SIMX86 := x86_64-apple-ios

ALL_ARCH := ARCH_IOS_NATIVE ARCH_IOS_SIM ARCH_IOS_SIMX86

add_rust_targets:
	@echo "Adding rustup targets"
	rustup target add $(ALL_ARCH)

DEBUG_FLAGS := 
RELEASE_FLAGS := --release

IOS_NATIVE_DEBUG := target/$(ARCH_IOS_NATIVE)/debug/lib$(TARGET).a
IOS_NATIVE_RELEASE := target/$(ARCH_IOS_NATIVE)/release/lib$(TARGET).a
IOS_NATIVE_LIPO := target/lib$(TARGET)-ios.a
NATIVE_LIPO_DEBUG := target/debug/lib$(TARGET)-ios.a
NATIVE_LIPO_RELEASE := target/release/lib$(TARGET)-ios.a

IOS_SIM_DEBUG := target/$(ARCH_IOS_SIM)/debug/lib$(TARGET).a
IOS_SIMX86_DEBUG := target/$(ARCH_IOS_SIMX86)/debug/lib$(TARGET).a
IOS_SIM_RELEASE := target/$(ARCH_IOS_SIM)/release/lib$(TARGET).a
IOS_SIMX86_RELEASE := target/$(ARCH_IOS_SIMX86)/release/lib$(TARGET).a
SIM_LIPO_DEBUG := target/debug/lib$(TARGET)-sim.a
SIM_LIPO_RELEASE := target/release/lib$(TARGET)-sim.a

define compile
$(1):
	@echo Building $(TARGET) for $(3)
	cargo build $(2) --target $(3)
endef

define lipo
$(1): $(2)
	@echo Performing lipo on $(2)
	lipo -create -output $(1) $(2)
endef

$(eval $(call compile,$(IOS_NATIVE_DEBUG),$(DEBUG_FLAGS),$(ARCH_IOS_NATIVE)))
$(eval $(call compile,$(IOS_NATIVE_RELEASE),$(RELEASE_FLAGS),$(ARCH_IOS_NATIVE)))
$(eval $(call compile,$(IOS_SIM_DEBUG),$(DEBUG_FLAGS),$(ARCH_IOS_SIM)))
$(eval $(call compile,$(IOS_SIMX86_DEBUG),$(DEBUG_FLAGS),$(ARCH_IOS_SIMX86)))
$(eval $(call compile,$(IOS_SIM_RELEASE),$(RELEASE_FLAGS),$(ARCH_IOS_SIM)))
$(eval $(call compile,$(IOS_SIMX86_RELEASE),$(RELEASE_FLAGS),$(ARCH_IOS_SIMX86)))
$(eval $(call lipo,$(NATIVE_LIPO_DEBUG),$(IOS_NATIVE_DEBUG)))
$(eval $(call lipo,$(NATIVE_LIPO_RELEASE),$(IOS_NATIVE_RELEASE)))
$(eval $(call lipo,$(SIM_LIPO_DEBUG),$(IOS_SIM_DEBUG) $(IOS_SIMX86_DEBUG)))
$(eval $(call lipo,$(SIM_LIPO_RELEASE),$(IOS_SIM_RELEASE) $(IOS_SIMX86_RELEASE)))

copy-debug: $(NATIVE_LIPO_DEBUG) $(SIM_LIPO_DEBUG)
	$(info Copying $^ -> ./)
	@cp $^ ./

copy-release: $(NATIVE_LIPO_RELEASE) $(SIM_LIPO_RELEASE)
	$(info Copying $^ -> ./)
	@cp $^ ./

build: copy-debug # You could use release if you want
build-release: copy-release


define remove-r
@if [ -d $(1) ]; then $(info Cleaning $(1)) rm -r $(1); fi
endef

define remove
@if [ -f $(1) ]; then $(info Cleaning $(1)) rm $(1); fi
endef

$(TARGET)-release.xcframework: build
	$(info Building $@)
	$(call remove-r,include)
	@mkdir -p include/$(TARGET)
	@cp $(TARGET).h module.modulemap include/$(TARGET)/
	$(call remove-r,$@)
	@xcodebuild -create-xcframework \
		-library $(IOS_NATIVE_RELEASE) \
		-headers include/ \
		-library $(SIM_LIPO_RELEASE) \
		-headers include/ \
		-output $@

$(TARGET).xcframework: build
	$(info Building $@)
	$(call remove-r,include)
	@mkdir -p include/$(TARGET)
	@cp $(TARGET).h module.modulemap include/$(TARGET)/

	$(call remove-r,target/ios)
	@mkdir -p target/ios/$(TARGET).framework/Headers
	$(call remove-r,target/sim)
	@mkdir -p target/sim/$(TARGET).framework/Headers

	@cp include/*.* target/ios/$(TARGET).framework/Headers
	@libtool -static \
		-o target/ios/$(TARGET).framework/$(TARGET) \
		$(NATIVE_LIPO_RELEASE)

	@cp include/*.* target/sim/$(TARGET).framework/Headers
	@xcrun -sdk iphonesimulator libtool -static \
		-o target/sim/$(TARGET).framework/$(TARGET) \
		$(SIM_LIPO_RELEASE)

	$(call remove-r,$(TARGET).xcframework)
	@xcodebuild -create-xcframework \
		-library target/sim/$(TARGET).framework \
		-headers include/ \
		-library target/ios/$(TARGET).framework \
		-headers include/ \
		-output $(TARGET).xcframework

zip: $(TARGET).xcframework
	$(call remove,$(TARGET).xcframework.zip)
	zip -r $(TARGET).xcframework.zip $(TARGET).xcframework


clean:
	$(call remove-r,include)
	$(call remove-r,target)
	$(call remove-r,$(TARGET).xcframework)
	$(call remove,$(TARGET).xcframework.zip)
	@rm -f ./*.a

.PHONY := copy-debug copy-release build build-release clean zip
