#!/bin/bash
# Builds Showcase Cranpose and assembles a `.app` bundle.
#
# The iOS backend is winit-based (`winit-uikit`), so winit starts
# `UIApplicationMain` itself: the app is a pure-Rust binary
# (`cranpose-showcase-ios`) with no Objective-C entry point and no Xcode project.
#
# Usage:
#   ./build-app.sh [target]
#     target: aarch64-apple-ios-sim (default) | aarch64-apple-ios (device)
#   PROFILE=release ./build-app.sh        # optimized build
#   WITH_WATCH=1 ./build-app.sh ...       # also build the Apple Watch app and
#                                         # put it in the bundle's Watch folder
#
# Prints the path to the assembled `.app` bundle on stdout.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE="$(cd "$SCRIPT_DIR/.." && pwd)"
TARGET="${1:-aarch64-apple-ios-sim}"
PROFILE="${PROFILE:-debug}"
APP_NAME="CranposeShowcase"
# Keep in step with MinimumOSVersion in Info.plist.
export IPHONEOS_DEPLOYMENT_TARGET=16.0

case "$PROFILE" in
  release) PROFILE_FLAG="--release" ;;
  debug)   PROFILE_FLAG="" ;;
  *) echo "PROFILE must be 'debug' or 'release', got '$PROFILE'" >&2; exit 1 ;;
esac
case "$TARGET" in
  aarch64-apple-ios) PLATFORM=iphoneos; WATCH_TARGET=device ;;
  aarch64-apple-ios-sim) PLATFORM=iphonesimulator; WATCH_TARGET=simulator ;;
  *) echo "target must be aarch64-apple-ios-sim or aarch64-apple-ios, got '$TARGET'" >&2; exit 1 ;;
esac

# shellcheck disable=SC2086
cargo build --manifest-path "$WORKSPACE/Cargo.toml" \
  --bin cranpose-showcase-ios \
  --target "$TARGET" --no-default-features --features ios $PROFILE_FLAG >&2

TARGET_DIR="${CARGO_TARGET_DIR:-$WORKSPACE/target}"
OUT="$TARGET_DIR/$TARGET/$PROFILE"
BIN="$OUT/cranpose-showcase-ios"
APP="$OUT/$APP_NAME.app"

rm -rf "$APP"
mkdir -p "$APP"
cp "$BIN" "$APP/$APP_NAME"
cp "$SCRIPT_DIR/CranposeShowcase/Info.plist" "$APP/Info.plist"
cp "$SCRIPT_DIR/CranposeShowcase/AppIcon.png" "$APP/AppIcon.png"

# The icon as the App Store takes it: one 1024 pixel image in an asset
# catalog, from which the system makes every size.
CATALOG="$OUT/Assets.xcassets"
rm -rf "$CATALOG"
mkdir -p "$CATALOG/AppIcon.appiconset"
cp "$WORKSPACE/assets/app-icon-1024.png" "$CATALOG/AppIcon.appiconset/icon.png"
echo '{"info":{"author":"xcode","version":1}}' > "$CATALOG/Contents.json"
echo '{"images":[{"filename":"icon.png","idiom":"universal","platform":"ios","size":"1024x1024"}],"info":{"author":"xcode","version":1}}' \
  > "$CATALOG/AppIcon.appiconset/Contents.json"
xcrun actool "$CATALOG" --compile "$APP" --platform "$PLATFORM" \
  --minimum-deployment-target "$IPHONEOS_DEPLOYMENT_TARGET" \
  --target-device iphone --target-device ipad --app-icon AppIcon \
  --output-partial-info-plist "$OUT/icon-info.plist" >/dev/null
/usr/libexec/PlistBuddy -c "Merge $OUT/icon-info.plist" "$APP/Info.plist" >&2

if [[ "${WITH_WATCH:-0}" == 1 ]]; then
  python3 "$WORKSPACE/watchos/build.py" --target "$WATCH_TARGET" >&2
  WATCH="$APP/Watch/CranposeWatch.app"
  mkdir -p "$APP/Watch"
  cp -R "$WORKSPACE/target/watchos/$WATCH_TARGET/CranposeWatch.app" "$WATCH"
  # A watch app carries the version and build numbers of the iPhone app.
  for key in CFBundleShortVersionString CFBundleVersion; do
    value="$(/usr/libexec/PlistBuddy -c "Print :$key" "$APP/Info.plist")"
    /usr/libexec/PlistBuddy -c "Set :$key $value" "$WATCH/Info.plist"
  done
  codesign --force --sign "${CODESIGN_IDENTITY:--}" "$WATCH" >&2
fi

# Ad-hoc sign so the bundle runs on device/simulator without a developer team.
# Pass CODESIGN_IDENTITY for a real Developer ID / distribution identity.
codesign --force --sign "${CODESIGN_IDENTITY:--}" "$APP" >&2

echo "$APP"
