# SorobanLard Contract — Makefile
#
# Deployment runbook: after running `make deploy-testnet`, record the resulting
# contract ID, deployed Wasm hash, and deployment date in DEPLOYMENTS.md at the
# repository root. Keep DEPLOYMENTS.md in sync with every deployment.

# Toolchain assumptions:
#   - Rust with the wasm32v1-none target installed:
#       rustup target add wasm32v1-none
#   - Stellar CLI (stellar-cli) providing the `stellar` binary, with the
#     `contract build`, `contract optimize`, and `contract deploy` subcommands
#     (recent stellar-cli releases). Verify with `stellar --version`.
#   - `cargo` / `rustup` on PATH for the test/fmt/check/clean targets.
#
# Environment variables:
#   STELLAR_ACCOUNT  (required for deploy-testnet) — the source identity used by
#                    `stellar contract deploy --source`. This is a named identity
#                    configured in stellar-cli (e.g. via `stellar keys generate`)
#                    or a secret key, not a filesystem path.
#   NETWORK          (optional) — overrides the target network for deploy-testnet.
#                    Defaults to `testnet` below.
#   CONTRACT_NAME    (optional) — overrides the contract/wasm base name.
#                    Defaults to `sorobanlard` below.
#   CONTRACT_ID      (required for verify-revoked) — the deployed contract ID to
#                    query. No credentials are stored here; the read-only
#                    invocation below needs no signing identity.

NETWORK       = testnet
CONTRACT_NAME = sorobanlard
WASM_PATH     = target/wasm32v1-none/release/$(CONTRACT_NAME).wasm
WASM_OPT      = target/wasm32v1-none/release/$(CONTRACT_NAME).optimized.wasm

.PHONY: test build optimize deploy-testnet clean fmt check check-api verify-revoked fuzz

# Run the contract's Rust test suite via `cargo test`.
test:
	cargo test

# Compile the contract to wasm using `stellar contract build`,
# producing $(WASM_PATH).
build:
	stellar contract build

# Build, then shrink the wasm with `stellar contract optimize`,
# producing $(WASM_OPT).
optimize: build
	stellar contract optimize --wasm $(WASM_PATH)
	@echo "Optimized: $(WASM_OPT)"

# Deploy to testnet. After a successful deploy, update DEPLOYMENTS.md with the
# new contract ID, Wasm hash, and deployment date (see repo root DEPLOYMENTS.md).

# Optimize, then deploy the optimized wasm to $(NETWORK) using the identity
# in $(STELLAR_ACCOUNT) via `stellar contract deploy`.
deploy-testnet: optimize
	stellar contract deploy \
		--wasm $(WASM_OPT) \
		--source $(STELLAR_ACCOUNT) \
		--network $(NETWORK)
	@echo "Deployed. Remember to record the contract ID, Wasm hash, and date in DEPLOYMENTS.md."

# Verify a certificate after it has been revoked.
#
# Usage:
#   make verify-revoked CONTRACT_ID=<contract-id> CERT_ID=<certificate-id>
#
# This is a read-only simulation (no --source / signing identity required), so
# the example carries no credentials or secrets. It invokes the public
# `verify_certificate` entry point and prints the returned boolean.
#
# Expected result after revocation: `false`.
#
# To inspect the revocation record itself, query the contract's revocation
# storage entry for the certificate (e.g. via `stellar contract invoke` on the
# revocation getter, or `stellar contract read` against the ledger entry keyed
# by the certificate ID) and confirm the certificate ID is present.
verify-revoked:
	@test -n "$(CONTRACT_ID)" || (echo "CONTRACT_ID is required" && exit 1)
	@test -n "$(CERT_ID)" || (echo "CERT_ID is required" && exit 1)
	stellar contract invoke \
		--id $(CONTRACT_ID) \
		--network $(NETWORK) \
		-- verify_certificate --certificate_id $(CERT_ID)

# Format the Rust sources with `cargo fmt`.
fmt:
	cargo fmt

# Type-check the crate with `cargo check` (no codegen).
check:
	cargo check

# Check that README contract-function signatures match lib.rs.
check-api:
	python3 $(CURDIR)/../../scripts/check_readme_api.py

# Remove build artifacts with `cargo clean`.
clean:
	cargo clean

# Build all fuzz targets to verify they compile (does not run the fuzzer).
fuzz:
	cargo build --manifest-path ../../fuzz/Cargo.toml --target x86_64-unknown-linux-gnu
