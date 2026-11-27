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
# Run the full test suite
cargo test

# Lint (must be clean)
cargo clippy --all-targets -- -D warnings

# Check that README contract-function signatures match lib.rs
make check-api

# Build the wasm artifact
make build

# Build an optimized wasm artifact
make optimize
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

### Where new keys go

1. **Composite** keys must be standalone `#[contracttype]` structs stored
   directly (for example `ArbitrationCaseKey`), not tuple variants.
2. **Singleton** keys go in `DataKey2` while it has headroom.
3. Once `DataKey2` approaches the 50-variant limit, add a new overflow enum
   (`DataKey3`, then `DataKey4`, ...) following the same pattern: a fresh
   `#[contracttype]` enum with its own documented variant count, and route new
   singleton keys there. Never reopen a full enum.

Keep the documented variant counts, the enum comments in `lib.rs`, and the
`test_data_key_variant_budgets_are_exact_and_documented` test in sync whenever
you add a key.

### Guardrails

- A compile-time assertion fails the build if either key enum exceeds the
  50-variant XDR limit.
- A CI check warns when either enum approaches the limit so the next overflow
  enum can be planned before the build breaks.

- Every panic message/condition must have a row in `ERRORS.md` describing its
  intended end-user meaning and the API response it maps to.
- Keep `code` values stable; only add new codes, never rename existing ones.
- Unknown panic messages must map to a generic `500` and be logged
  server-side, never leaked to end users.

## Panic messages

Panic message strings in `lib.rs` follow a short style guide so the eventual
error-enum migration stays mechanical:

- **Lowercase** the first word (e.g. `course not found`, not `Course not found`).
- **No trailing period** or other end punctuation.
- Use consistent **`X cannot Y`** phrasing for precondition failures
  (e.g. `instructor cannot grade frozen course`).
- Use one term per concept — prefer `unauthorized` for caller/permission
  failures rather than mixing role-specific or state-specific wording.
- Dynamic values are fine, but keep the surrounding text in the same style
  (e.g. `token decimal precision must be {} (found {})`).

A CI grep check enforces the basics (no trailing period, no leading capital) on
panic strings; run it locally with the command documented in the workflow.

## Pull requests

- Branch from `main` and keep each PR focused.
- Run `cargo fmt --check` and `cargo test` before opening a PR.
- Add or update tests for any behaviour change, especially new events, guards,
  and storage keys.
- Update `README.md` when a public contract function or emitted event changes.
- Update `ERRORS.md` whenever a new panic condition or error message is added,
  and keep the API error mapping in sync.

## Continuous integration

Every push and pull request runs the CI workflow defined in
[`.github/workflows/ci.yml`](.github/workflows/ci.yml), which executes
`cargo test --workspace` and caches the cargo registry and `target`
directories to keep runs fast.

Maintainers should configure branch protection on `main` so the CI check is
**required for merge**. This is applied in the repository settings (Settings →
Branches → branch protection rules) and is not part of the code in this repo.

CI runs `python3 scripts/check_readme_api.py`, which fails if a documented
`name(...)` call has a different number of arguments than the matching
`pub fn` in `lib.rs`. It only warns about public functions the README does
not mention yet.

Run `make check-api` from `contracts/sorobanlard/` to run the same check locally.

## Dependency advisories

CI runs `cargo audit` against `Cargo.lock` on every push and pull request.

- A vulnerability with a fix available must be resolved by upgrading the
  dependency. Do not ignore it.
- An advisory with **no available fix** may be accepted by adding its ID to
  `ignore` in [`.cargo/audit.toml`](.cargo/audit.toml). Each entry must have a
  comment giving the advisory's reason for acceptance, the affected crate,
  and the date it should next be reviewed.
- Ignored advisories are reviewed when the dependency is upgraded and at least
  once per release. Remove the entry as soon as a fix is available.
