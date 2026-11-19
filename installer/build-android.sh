#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

: "${ANDROID_HOME:?Informe a pasta do Android SDK}"
ndk="${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk/27.2.12479018}"
case "$(uname -s)" in
    Darwin) host=darwin-x86_64 ;;
    Linux) host=linux-x86_64 ;;
    *) echo "Esta compilação usa Bash no Linux ou macOS." >&2; exit 1 ;;
esac
tools="$ndk/toolchains/llvm/prebuilt/$host/bin"
export ANDROID_NDK_HOME="$ndk"

for target in aarch64-linux-android x86_64-linux-android; do
    case "$target" in
        aarch64-*) abi=arm64-v8a ;;
        x86_64-*) abi=x86_64 ;;
    esac
    key="${target//-/_}"
    upper=$(printf '%s' "$key" | tr '[:lower:]' '[:upper:]')
    export "CARGO_TARGET_${upper}_LINKER=$tools/${target}23-clang"
    export "CARGO_TARGET_${upper}_RUSTFLAGS=-C link-arg=-Wl,-z,max-page-size=16384"
    export "CC_$key=$tools/${target}23-clang"
    export "CXX_$key=$tools/${target}23-clang++"
    export "AR_$key=$tools/llvm-ar"
    cargo build --lib --release --locked --target "$target"
    mkdir -p "target/android/jniLibs/$abi"
    install -m 755 "target/$target/release/libmargem_desktop.so" \
        "target/android/jniLibs/$abi/libmargem_desktop.so"
done

gradle --project-dir android --no-daemon assembleDebug
mkdir -p target/installer
cp android/build/outputs/apk/debug/Margem-debug.apk target/installer/Margem-android.apk
