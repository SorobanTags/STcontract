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
