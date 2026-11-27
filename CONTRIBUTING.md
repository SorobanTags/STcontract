# Contributing

Thanks for your interest in improving this project. This document covers the
basics of setting up a development environment, running the test suite, and
submitting changes.

## Getting started

1. Fork the repository and clone your fork locally.
2. Install dependencies with your package manager of choice.
3. Create a topic branch for your change (for example `fix/revoked-cert-example`).

## Running validation

Before opening a pull request, run the relevant validation and report the exact
command and its result in the pull request description. For example:

## Contract workspace layout

Use this map to find where each kind of change belongs:

| Path | What lives here |
| --- | --- |
| `contracts/sorobanlard/src/lib.rs` | All contract logic: types, storage keys, and the `impl` block with public functions. |
| `contracts/sorobanlard/src/test.rs` | The main test suite (unit and integration-style tests). |
| `contracts/sorobanlard/src/test/proptests.rs` | Property-based tests. |
| `scripts/` | Tooling such as `check_readme_api.py` (README/API signature check) and `check_invariants.sh` (invariant checks). |
| `fuzz/` | Fuzz targets. |
| `DEPLOYMENTS.md` | Deployment configuration and records. |
| `contracts/sorobanlard/.env.example` | Example environment variables for deployment. |
| `contracts/sorobanlard/Makefile` | Build / optimize / deployment targets. |

### Change-to-file map

| Contribution | Where to make the change |
| --- | --- |
| New contract function | `contracts/sorobanlard/src/lib.rs` (inside the `impl` block, near related feature code), plus a doc comment and a `README.md` update. |
| New test | `contracts/sorobanlard/src/test.rs` (or `src/test/proptests.rs` for property-based tests). |
| New fuzz target | `fuzz/` (add the target and register it in the fuzz manifest). |
| New panic message | The panic site in `contracts/sorobanlard/src/lib.rs`, plus a row in `ERRORS.md`. |
| Deployment change | `DEPLOYMENTS.md`, `contracts/sorobanlard/.env.example`, and `contracts/sorobanlard/Makefile` as applicable. |

### Verifying your change

Run the checks relevant to what you touched, and report the exact command and
result in the pull request description:

```sh
# Contract logic or tests
cargo test

# README/API signature consistency
python3 scripts/check_readme_api.py

# Invariant checks
bash scripts/check_invariants.sh

# Fuzz targets (from the fuzz workspace)
cargo fuzz build
```

### How `lib.rs` is organized

`lib.rs` is intentionally kept as one file, but it follows a fixed top-to-bottom
layout. When adding code, place it in the matching section rather than at the end
of the file:

1. **Fee / risk types** — enums and structs describing fees, risk parameters,
   and related configuration.
2. **Data types** — the domain structs and enums used by the contract's public
   API (positions, pools, etc.).
3. **Storage keys** — the `DataKey` / `DataKey2` enums used to address persistent
   storage.
4. **The `impl` block** — one large `impl` block containing the contract's
   functions, grouped by feature area.

Keep new public functions inside the existing `impl` block and near the related
feature code, and add a doc comment (`///`) describing behavior and any panics.

### `DataKey` vs `DataKey2`

Soroban's XDR representation limits an enum to **50 variants**. Because the
contract's storage keys exceed that limit, they are split across two enums:

- `DataKey` — the original set of storage keys.
- `DataKey2` — overflow keys added once `DataKey` reached the 50-variant cap.

When adding a new storage key:

- Add it to `DataKey` if it still has room (fewer than 50 variants).
- Otherwise add it to `DataKey2`.
- Never reorder or remove existing variants — their discriminants are part of
  the on-chain storage layout.

## Building and testing

Run these from `contracts/sorobanlard/`:

```sh
npm test
```

## Submitting changes

- Keep changes scoped to the issue you are addressing.
- Update directly related documentation alongside code changes.
- Add focused regression tests for behavior changes. Documentation-only changes
  should include verifiable commands or examples.
- Do not include credentials, tokens, or other secrets in code, tests, or docs.

## Example: verifying a revoked certificate

The CLI can verify a certificate against the current revocation state. The
example below uses a placeholder certificate path and a placeholder issuer
identifier — substitute your own values. No credentials are required for this
invocation.

```sh
# Verify a certificate after it has been revoked.
# Expected result: the command exits successfully and prints `false`,
# indicating the certificate no longer verifies.
cert-cli verify --cert ./path/to/certificate.pem --issuer example-issuer
```

Expected output:

```text
false
```

### Inspecting the revocation record

To confirm why verification returned `false`, inspect the revocation record for
the certificate:

```sh
# Show the revocation record for the certificate serial number.
cert-cli revocation show --serial 00A1B2C3D4
```

The record lists the revocation timestamp and reason. If no record is returned,
the certificate has not been revoked and verification should return `true`.

> Note: the commands above reflect the current public CLI contract. If the
> interface changes, update this example in the same pull request.
