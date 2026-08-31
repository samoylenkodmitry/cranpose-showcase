#!/usr/bin/env bash
set -euo pipefail

if command -v wasm-pack >/dev/null 2>&1; then
  WASM_PACK="wasm-pack"
elif [[ -x "$HOME/.cargo/bin/wasm-pack" ]]; then
  WASM_PACK="$HOME/.cargo/bin/wasm-pack"
else
  echo "wasm-pack is required. Install it with: cargo install wasm-pack" >&2
  exit 1
fi

BUILD_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/cranpose-showcase-web.XXXXXX")"
PKG_STAGE="$BUILD_ROOT/pkg"
DIST_STAGE="$BUILD_ROOT/dist"

"$WASM_PACK" build \
  --target web \
  --release \
  --out-dir "$PKG_STAGE" \
  --no-default-features \
  --features web,renderer-wgpu

mkdir -p "$DIST_STAGE"
cp index.html "$DIST_STAGE/index.html"
cp assets/app-icon.png "$DIST_STAGE/app-icon.png"
cp -R "$PKG_STAGE" "$DIST_STAGE/pkg"

for output in pkg dist; do
  if [[ -e "$output" ]]; then
    mv "$output" "${output}.previous-$(date +%s)"
  fi
done
mv "$PKG_STAGE" pkg
mv "$DIST_STAGE" dist

echo "WASM demo written to dist/index.html"
