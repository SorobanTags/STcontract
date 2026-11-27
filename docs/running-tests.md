# Running contract tests

Run every command from the repository root. The contract package is `sorobanlard`.

```bash
# All tests in the contract package
cargo test -p sorobanlard

# One test by name (substring match on the test path)
cargo test -p sorobanlard test_init_success

# Exactly one test, without also matching tests that share the prefix
cargo test -p sorobanlard test::test_init_success -- --exact

# Show println!/debug output from the test
cargo test -p sorobanlard test_init_success -- --nocapture
```

Expected output for the single-test command includes:

```
running 1 test
test test::test_init_success ... ok
```

Options after `--` go to the test harness, not to Cargo. To list test names, run
`cargo test -p sorobanlard -- --list`.
