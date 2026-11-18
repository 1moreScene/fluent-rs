#!/usr/bin/env bash
set -euo pipefail

cd "$SRCROOT/../.."
default_cargo_home=~/.cargo
export PATH="${CARGO_HOME:-$default_cargo_home}/bin:$PATH"

cargo_profile=dev
output_profile=debug
if [[ "$CONFIGURATION" == Release ]]; then
    cargo_profile=release
    output_profile=release
fi

cargo_output="$TARGET_TEMP_DIR/cargo"
executables=()
for arch in $ARCHS; do
    case "$PLATFORM_NAME:$arch" in
        iphoneos:arm64) rust_target=aarch64-apple-ios ;;
        iphonesimulator:arm64) rust_target=aarch64-apple-ios-sim ;;
        iphonesimulator:x86_64) rust_target=x86_64-apple-ios ;;
        *) echo "Unsupported iOS build target: $PLATFORM_NAME:$arch" >&2; exit 1 ;;
    esac

    cargo build --locked -p fluent-gallery --bin fluent-gallery --profile "$cargo_profile" \
        --target "$rust_target" --target-dir "$cargo_output"
    executables+=("$cargo_output/$rust_target/$output_profile/fluent-gallery")
done

output="$TARGET_BUILD_DIR/$EXECUTABLE_PATH"
mkdir -p "$(dirname "$output")"
if [[ ${#executables[@]} -eq 1 ]]; then
    cp "${executables[0]}" "$output"
else
    xcrun lipo -create "${executables[@]}" -output "$output"
fi
