# Fuzzing

This directory holds [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz)
targets for the SorobanLard contract. It is kept out of the root workspace, so
`cargo test` and CI do not build it.

## Running locally

Fuzzing needs a nightly toolchain and the cargo-fuzz subcommand:

```sh
cargo install cargo-fuzz
cargo +nightly fuzz run register_course
```

Run it from the repository root. Stop it with Ctrl-C. Crashing inputs are saved
under `fuzz/artifacts/register_course/` and can be replayed with
`cargo +nightly fuzz run register_course <path-to-artifact>`.

## Targets

- `register_course` feeds arbitrary bytes to `register_course` as the course ID
  and checks that the contract accepts exactly the IDs that are non-empty,
  at most 256 bytes, and printable ASCII. Any other outcome fails the run.
- `withdraw_earnings` enrolls a student, then fuzzes `withdraw_earnings` with an
  arbitrary amount derived from the input. It asserts that the call succeeds
  iff `1 <= amount <= credited_balance` and that the instructor's token balance
  increases by exactly the withdrawn amount, covering INV-06
  (`withdrawal_pays_exactly_credited_balance`). Run it with
  `cargo +nightly fuzz run withdraw_earnings`.

Fuzzing is slow, so run it periodically rather than in CI.