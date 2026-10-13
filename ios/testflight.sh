#!/bin/bash
# Signs the iPhone app that ios/build-app.sh built, with the Apple Watch app in
# it when there is one, and uploads it to TestFlight.
#
# Usage:
#   ios/testflight.sh APP BUILD_NUMBER OUTPUT.ipa
#   UPLOAD=0 ios/testflight.sh ...   # make the signed .ipa and stop there
#
# The signing material comes from environment variables, which the TestFlight
# workflow fills from the GitHub secrets of the same names. docs/testflight.md
# says how to make each one.
#
#   IOS_DISTRIBUTION_CERTIFICATE           the Apple Distribution certificate
#                                          with its private key (.p12), base64
#   IOS_DISTRIBUTION_CERTIFICATE_PASSWORD  the password of the .p12
#   IOS_APP_STORE_PROFILE                  the iPhone app's App Store
#                                          provisioning profile, base64
#   WATCH_APP_STORE_PROFILE                the watch app's App Store profile,
#                                          base64; needed when APP holds the
#                                          watch app
#   APP_STORE_CONNECT_KEY_ID               the App Store Connect API key's id
#   APP_STORE_CONNECT_ISSUER_ID            its issuer id
#   APP_STORE_CONNECT_PRIVATE_KEY          the text of its AuthKey_<id>.p8
#   IOS_BUNDLE_ID                          optional: the iPhone app's bundle id
#                                          in place of the one in Info.plist
#
# The certificate goes into a keychain of its own, which is deleted at the end.
set -euo pipefail

if [[ $# -ne 3 ]]; then
  echo "usage: $0 APP BUILD_NUMBER OUTPUT.ipa" >&2
  exit 2
fi
APP="$1"
BUILD="$2"
OUTPUT="$3"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UPLOAD="${UPLOAD:-1}"

required=(IOS_DISTRIBUTION_CERTIFICATE IOS_DISTRIBUTION_CERTIFICATE_PASSWORD IOS_APP_STORE_PROFILE)
if [[ -d "$APP/Watch/CranposeWatch.app" ]]; then
  required+=(WATCH_APP_STORE_PROFILE)
fi
if [[ "$UPLOAD" == 1 ]]; then
  required+=(APP_STORE_CONNECT_KEY_ID APP_STORE_CONNECT_ISSUER_ID APP_STORE_CONNECT_PRIVATE_KEY)
fi
missing=()
for name in "${required[@]}"; do
  [[ -n "${!name:-}" ]] || missing+=("$name")
done
if (( ${#missing[@]} )); then
  echo "Missing: ${missing[*]}. docs/testflight.md says how to make each one." >&2
  exit 1
fi

work="$(mktemp -d "${TMPDIR:-/tmp}/testflight.XXXXXX")"
keychain="$work/signing.keychain-db"
keychain_password="$(uuidgen)"
# The keychains the user had, to put back at the end. (macOS bash 3.2 has no
# mapfile.)
previous_keychains=()
while IFS= read -r line; do
  previous_keychains+=("$(printf '%s' "$line" | sed -e 's/^ *"//' -e 's/"$//')")
done < <(security list-keychains -d user)
cleanup() {
  security list-keychains -d user -s ${previous_keychains[@]+"${previous_keychains[@]}"} || true
  security delete-keychain "$keychain" 2>/dev/null || true
  rm -rf "$work"
}
trap cleanup EXIT

# A keychain for this run only, with the certificate and its private key.
security create-keychain -p "$keychain_password" "$keychain"
security set-keychain-settings -lut 3600 "$keychain"
security unlock-keychain -p "$keychain_password" "$keychain"
# Apple's intermediate certificate, so the signature carries the whole chain.
curl -fsSL https://www.apple.com/certificateauthority/AppleWWDRCAG3.cer -o "$work/wwdr.cer"
security import "$work/wwdr.cer" -k "$keychain" -T /usr/bin/codesign >/dev/null
printf '%s' "$IOS_DISTRIBUTION_CERTIFICATE" | base64 --decode > "$work/certificate.p12"
security import "$work/certificate.p12" -k "$keychain" \
  -P "$IOS_DISTRIBUTION_CERTIFICATE_PASSWORD" -T /usr/bin/codesign >/dev/null
security set-key-partition-list -S apple-tool:,apple:,codesign: -s \
  -k "$keychain_password" "$keychain" >/dev/null
security list-keychains -d user -s "$keychain" ${previous_keychains[@]+"${previous_keychains[@]}"}
identity="$(security find-identity -p codesigning "$keychain" \
  | awk '/"Apple Distribution/ { print $2; exit }')"
if [[ -z "$identity" ]]; then
  echo "IOS_DISTRIBUTION_CERTIFICATE holds no Apple Distribution certificate with its private key." >&2
  security find-identity -p codesigning "$keychain" >&2
  exit 1
fi

printf '%s' "$IOS_APP_STORE_PROFILE" | base64 --decode > "$work/app.mobileprovision"
profiles=(--app-profile "$work/app.mobileprovision")
if [[ -d "$APP/Watch/CranposeWatch.app" ]]; then
  printf '%s' "$WATCH_APP_STORE_PROFILE" | base64 --decode > "$work/watch.mobileprovision"
  profiles+=(--watch-profile "$work/watch.mobileprovision")
fi
bundle_id=()
if [[ -n "${IOS_BUNDLE_ID:-}" ]]; then
  bundle_id=(--bundle-id "$IOS_BUNDLE_ID")
fi

python3 "$SCRIPT_DIR/ipa.py" "$APP" --build "$BUILD" --identity "$identity" \
  --keychain "$keychain" "${profiles[@]}" ${bundle_id[@]+"${bundle_id[@]}"} --output "$OUTPUT"

if [[ "$UPLOAD" != 1 ]]; then
  echo "UPLOAD=0: kept $OUTPUT and uploaded nothing"
  exit 0
fi
# altool reads the key from API_PRIVATE_KEYS_DIR/AuthKey_<id>.p8.
mkdir -p "$work/private_keys"
printf '%s\n' "$APP_STORE_CONNECT_PRIVATE_KEY" > "$work/private_keys/AuthKey_${APP_STORE_CONNECT_KEY_ID}.p8"
API_PRIVATE_KEYS_DIR="$work/private_keys" xcrun altool --upload-app -f "$OUTPUT" -t ios \
  --apiKey "$APP_STORE_CONNECT_KEY_ID" --apiIssuer "$APP_STORE_CONNECT_ISSUER_ID"
echo "Uploaded $OUTPUT. App Store Connect processes it for a few minutes before TestFlight shows it."
