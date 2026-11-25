# Deployment preflight checklist

Run from the repository root before deploying or smoke testing:

```bash
./scripts/preflight-env.sh deploy   # before make deploy-testnet
./scripts/preflight-env.sh smoke    # before scripts/smoke-test-testnet.sh
./scripts/preflight-env.sh          # both
```

It prints `OK:` or `MISSING:` followed by each variable **name** and never prints
values. Exit code `0` means all required variables are set, `1` means at least
one is missing.

| Variable | Used by | Required | Secret? | Purpose |
|---|---|---|---|---|
| `STELLAR_ACCOUNT` | `make deploy-testnet` | yes | No — a Stellar CLI key *alias*, not a key | Identity that deploys the Wasm |
| `CONTRACT_ID` | smoke test | yes | No — public contract address (`C...`) | Contract under test |
| `ADMIN_ACCOUNT` | smoke test | yes | No — key alias | Admin identity passed to `init` |
| `INSTRUCTOR_ACCOUNT` | smoke test | yes | No — key alias | Registers the smoke-test course |
| `STUDENT_ACCOUNT` | smoke test | yes | No — key alias | Enrolls in the smoke-test course |
| `NETWORK` | smoke test | no (default `testnet`) | No | Stellar CLI network name |
| `TOKEN_ADDRESS` | smoke test | no (default testnet USDC) | No | Approved SAC token for the course |
| `SMOKE_RUN_ID` | smoke test | no (auto-generated) | No | Suffix for smoke-test course/certificate IDs |

No variable holds a secret: signing keys stay in the Stellar CLI keystore and are
referenced by alias. Never put an `S...` secret key in any of these variables.

Regression test for the check: `./scripts/tests/test-preflight-env.sh`.
