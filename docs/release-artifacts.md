# Release Artifacts: Optimized WASM Verification & Manifest

Deploy **only** the optimized WASM. `make build` and `make optimize` produce two
different files with different hashes:

| Make target | Output (relative to `contracts/sorobanlard/`) | Deploy? |
|---|---|---|
| `make build` | `target/wasm32v1-none/release/sorobanlard.wasm` | No — unoptimized build output |
| `make optimize` | `target/wasm32v1-none/release/sorobanlard.optimized.wasm` | **Yes** — used by `make deploy-testnet` |

CI (`wasm-build` job in `.github/workflows/ci.yml`) builds the same optimized
artifact and fails if it is missing or larger than 256 KiB.

## Verify the artifact before deployment

```bash
make -C contracts/sorobanlard optimize
WASM=contracts/sorobanlard/target/wasm32v1-none/release/sorobanlard.optimized.wasm
sha256sum "$WASM"           # local checksum
stat -c%s "$WASM"           # must be <= 262144 bytes
```

After deploying (or installing) the code, the on-chain Wasm hash is the SHA-256
of the uploaded bytes, so it must equal the local checksum above:

```bash
stellar contract install --wasm "$WASM" --source "$STELLAR_ACCOUNT" --network testnet
# prints the Wasm hash; compare with: sha256sum "$WASM" | cut -d' ' -f1
```

If the hashes differ, you are deploying a different file (most often the
unoptimized `sorobanlard.wasm`) — stop and rebuild.

## Release manifest

`scripts/release-manifest.sh` records the source revision, target, artifact
name, size and SHA-256 of the exact optimized WASM. It refuses any file not
named `*.optimized.wasm`.

```bash
scripts/release-manifest.sh                                  # print JSON
scripts/release-manifest.sh "$WASM" release-manifest.json    # write to file
```

Example output:

```json
{
  "contract": "sorobanlard",
  "source_revision": "8a28a3a…",
  "source_dirty": false,
  "target": "wasm32v1-none",
  "profile": "release",
  "artifact": "sorobanlard.optimized.wasm",
  "sha256": "<64 hex chars>",
  "size_bytes": 123456,
  "generated_at": "2026-11-23T00:00:00Z"
}
```

`source_dirty: true` means the working tree had uncommitted changes; do not
deploy such a build. Record the manifest's `sha256` and `source_revision` in
`DEPLOYMENTS.md` alongside the contract ID, and attach the manifest to the
release.
