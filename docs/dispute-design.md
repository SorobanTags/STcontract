# Dispute System Design Notes

This document captures the rationale behind the arbitration/dispute data model
(`ArbitrationCase`, `DisputeStatus`, and `DataKey`). It replaces the stray
root-level file `SorobanLard dispute struct and datakey`, which was an accidental
copy of `Cargo.toml` rather than actual design notes.

## `ArbitrationCase`

An `ArbitrationCase` is the on-chain record of a single dispute. It groups
together everything needed to resolve a disagreement between the parties of an
escrow:

- the escrow it belongs to (so the case can be tied back to the funds in play),
- the parties involved (buyer/seller or payer/payee),
- the current `DisputeStatus`,
- any evidence or resolution metadata required by the arbiter.

Keeping the case as a single struct means a dispute can be loaded, inspected,
and updated atomically, and that the storage layout stays predictable across
contract upgrades.

## `DisputeStatus`

`DisputeStatus` is the state machine for a case. It is intentionally small and
explicit so that transitions are easy to audit:

- an **open** state when a dispute is first raised,
- an **in-review** state while an arbiter is evaluating evidence,
- a **resolved** state once a decision has been recorded.

Modeling status as an enum (rather than free-form flags) prevents invalid
combinations and makes the allowed transitions obvious in code review.

## `DataKey`

`DataKey` is the storage key enum used to address contract state in Soroban's
persistent/temporary storage. Each variant maps to a distinct logical record
(e.g. a case by id, an index of open cases, or configuration values). Using a
typed enum instead of raw byte keys:

- avoids key collisions between unrelated records,
- keeps serialization stable and explicit,
- makes it straightforward to enumerate and reason about what the contract
  stores.

## Certificate revocation challenges

Revoking a certificate is not always final. When a certificate is revoked in
bulk (for example, revoking every certificate issued for a course), the
contract records a **pending revocation challenge** for each affected
certificate. The holder can then contest the revocation before it is finalized.

### Challenge status values

A revocation challenge is modeled with a small, explicit status enum:

- **open** — the revocation has been recorded but the challenge window is still
  active; the holder may contest it.
- **completed** — the challenge window has closed (or the challenge was
  resolved) and the revocation stands.

As with `DisputeStatus`, using an enum instead of free-form flags keeps the
allowed transitions obvious and prevents invalid combinations.

### Deadline

Each challenge carries a `deadline` field: the ledger timestamp after which an
open challenge can no longer be contested and transitions to `completed`.
Integrators should treat the deadline as advisory data and compare it against
the current ledger time rather than assuming the contract has already flipped
the status.

### Read-only query: `get_revocation_challenge`

`get_revocation_challenge` exposes the challenge status and deadline as
read-only data. It does not mutate contract state, so it is safe to call from
off-chain indexers, UIs, and monitoring jobs. The returned fields are:

- the certificate (or its identifier) the challenge applies to,
- the current challenge `status` (`open` or `completed`),
- the `deadline` timestamp for contesting an open challenge.

### Authorization

Reading a challenge is restricted to parties with a legitimate interest:

- the **student holder** of the certificate,
- the **instructor** who issued the certificate,
- an **admin** of the contract.

Unauthorized callers are rejected, so challenge state is not disclosed to
arbitrary third parties.

### Bulk revocation and observation

Bulk revocation creates one pending (open) challenge per revoked certificate,
so a course-wide revocation leaves every affected holder with a contestable
challenge. Because `get_revocation_challenge` is read-only, integrators can
poll it to observe challenge state — open versus completed, and how much time
remains before the deadline — without ever mutating the contract.

## Why this lives in `docs/`

Design notes belong under `docs/` with a `.md` extension so GitHub renders them
as documentation. The previous root-level file had no extension and contained
workspace configuration, which was confusing to anyone looking for dispute
design rationale.
