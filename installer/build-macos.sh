#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

export MACOSX_DEPLOYMENT_TARGET=11.0
cargo test --locked
cargo build --release --locked

version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -1)
arch=$(uname -m)
output="$PWD/target/installer"
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
bundle="$stage/Margem.app/Contents"
mkdir -p "$output" "$bundle/MacOS"
install -m 755 target/release/margem-desktop "$bundle/MacOS/Margem"
cat > "$bundle/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key><string>Margem</string>
    <key>CFBundleExecutable</key><string>Margem</string>
    <key>CFBundleIdentifier</key><string>br.com.margem.desktop</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>$version</string>
    <key>CFBundleVersion</key><string>$version</string>
    <key>LSMinimumSystemVersion</key><string>$MACOSX_DEPLOYMENT_TARGET</string>
    <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
EOF

plutil -lint "$bundle/Info.plist"
lipo "$bundle/MacOS/Margem" -verify_arch "$arch"
codesign --force --sign - "$stage/Margem.app"
codesign --verify --strict "$stage/Margem.app"
ln -s /Applications "$stage/Applications"
hdiutil create -ov -srcfolder "$stage" -volname Margem -format UDZO "$output/Margem-$version-macos-$arch.dmg"
hdiutil verify "$output/Margem-$version-macos-$arch.dmg"
