#!/usr/bin/env bash
set -euo pipefail

tag="${1:-${GITHUB_REF_NAME:-}}"
if [[ ! "$tag" =~ ^v([0-9]+)\.([0-9]+)\.([0-9]+)$ ]]; then
  echo "Expected a release tag like v0.1.2, got: ${tag:-<empty>}" >&2
  exit 1
fi

version="${tag#v}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cargo_version="$(sed -n 's/^version = "\([^"]*\)"$/\1/p' "$root/Cargo.toml" | head -1)"
android_version="$(sed -n 's/^[[:space:]]*versionName = "\([^"]*\)"$/\1/p' "$root/android/app/build.gradle.kts" | head -1)"
ios_version="$(awk '/<key>CFBundleShortVersionString<\/key>/ { getline; sub(/.*<string>/, ""); sub(/<\/string>.*/, ""); print; exit }' "$root/ios/CranposeShowcase/Info.plist")"

for entry in "Cargo:$cargo_version" "Android:$android_version" "iOS:$ios_version"; do
  name="${entry%%:*}"
  actual="${entry#*:}"
  if [[ "$actual" != "$version" ]]; then
    echo "$name version mismatch: tag is $version, metadata is ${actual:-<empty>}" >&2
    exit 1
  fi
done

if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  printf 'version=%s\n' "$version" >> "$GITHUB_OUTPUT"
else
  printf 'version=%s\n' "$version"
fi
