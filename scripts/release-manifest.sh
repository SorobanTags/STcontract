#!/usr/bin/env bash
# Generate a release manifest for the optimized SorobanLard WASM.
#
# Usage: scripts/release-manifest.sh [path/to/sorobanlard.optimized.wasm] [out.json]
# Defaults to the `make optimize` output and prints JSON to stdout.
set -euo pipefail

WASM="${1:-contracts/sorobanlard/target/wasm32v1-none/release/sorobanlard.optimized.wasm}"
OUT="${2:-}"

case "$WASM" in
  *.optimized.wasm) ;;
  *) echo "ERROR: $WASM is not an optimized artifact (*.optimized.wasm). Run: make -C contracts/sorobanlard optimize" >&2; exit 1 ;;
esac
[ -f "$WASM" ] || { echo "ERROR: artifact not found: $WASM" >&2; exit 1; }

rev="$(git rev-parse HEAD)"
dirty="false"
git diff --quiet HEAD -- || dirty="true"
sha="$(sha256sum "$WASM" | awk '{print $1}')"
size="$(wc -c < "$WASM" | tr -d ' ')"

manifest=$(cat <<JSON
{
  "contract": "sorobanlard",
  "source_revision": "$rev",
  "source_dirty": $dirty,
  "target": "wasm32v1-none",
  "profile": "release",
  "artifact": "$(basename "$WASM")",
  "sha256": "$sha",
  "size_bytes": $size,
  "generated_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
JSON
)

if [ -n "$OUT" ]; then printf '%s\n' "$manifest" > "$OUT"; else printf '%s\n' "$manifest"; fi
