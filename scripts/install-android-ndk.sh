#!/usr/bin/env bash
# Installs the Android NDK and cargo-ndk on a GitHub Actions runner, whose
# Android SDK has no NDK of the version the build expects, and points
# ANDROID_NDK_HOME at it for the later steps.
set -euo pipefail

NDK_VERSION="28.2.13676358"

sdk_root="${ANDROID_HOME:-${ANDROID_SDK_ROOT:?Android SDK is unavailable}}"
sdkmanager="$(find "$sdk_root/cmdline-tools" -path '*/bin/sdkmanager' -type f -print -quit)"
test -n "$sdkmanager" || { echo "Android sdkmanager is unavailable" >&2; exit 1; }
set +o pipefail
yes | "$sdkmanager" --licenses >/dev/null
license_status=${PIPESTATUS[1]}
set -o pipefail
test "$license_status" -eq 0
"$sdkmanager" --install "ndk;$NDK_VERSION" >/dev/null
if ! command -v cargo-ndk >/dev/null; then
  cargo install cargo-ndk --locked
fi
echo "ANDROID_NDK_HOME=$sdk_root/ndk/$NDK_VERSION" >> "${GITHUB_ENV:-/dev/null}"
