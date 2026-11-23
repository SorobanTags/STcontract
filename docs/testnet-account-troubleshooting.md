# Testnet funding and account setup troubleshooting

These steps only use **public** data. Never paste a secret key (`S...`), a seed
phrase, or the output of `stellar keys show` into an issue, PR, chat, or log.
Nobody on this project will ask for one.

## 1. Create and inspect an identity

```bash
stellar keys generate --global my-account --network testnet
stellar keys ls                      # lists alias names only
stellar keys address my-account      # prints the PUBLIC address (G...) — safe to share
```

## 2. Fund it on testnet

```bash
stellar keys fund my-account --network testnet
```

## 3. Confirm funding

```bash
curl -s "https://horizon-testnet.stellar.org/accounts/$(stellar keys address my-account)" \
  | grep -o '"balance": *"[^"]*"' | head -1
```

A balance (e.g. `"balance": "10000.0000000"`) means the account exists. A
`404 Not Found` / `Resource Missing` response means it is not funded yet.

## Common problems and safe recovery

| Symptom | Cause | Fix |
|---|---|---|
| `error: ... identity not found` / `No identity named my-account` | Alias was created on another machine or without `--global` | `stellar keys ls`; regenerate with `stellar keys generate --global <name> --network testnet` |
| Horizon returns 404 for the address | Account never funded | `stellar keys fund <name> --network testnet`, then recheck step 3 |
| `fund` fails or rate-limits | Friendbot is busy | Wait a minute and retry; do not ask others to fund using your secret |
| Transaction fails with `txNoAccount` / `account not found` | `--source` alias points to an unfunded account | Fund the alias shown by `stellar keys address <name>` |
| Wrong network errors | CLI network not configured | `stellar network ls`; use `--network testnet` |
| Testnet was reset | SDF periodically wipes testnet | Re-fund every alias and redeploy the contract |

If you think a secret key was exposed, stop using that identity: generate a new
alias, fund it, and update `STELLAR_ACCOUNT` / `*_ACCOUNT` variables to the new
name. Testnet keys have no real value, but treat them the same way you would on
mainnet.
