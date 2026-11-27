# Persistent Storage TTL Renewal

Constants (`contracts/sorobanlard/src/lib.rs`):

| Constant | Value | Meaning |
|---|---|---|
| `PERSISTENT_TTL_THRESHOLD` | 6_000_000 ledgers (~1 year) | Extend only when remaining TTL is below this |
| `PERSISTENT_TTL_EXTEND_TO` | 6_300_000 ledgers | New TTL after extension |
| `INSTANCE_TTL_THRESHOLD` / `INSTANCE_TTL_EXTEND_TO` | same values | Instance entry (all instance keys share one TTL) |

Persistent entries are renewed through `extend_persistent_ttl` (or an inline
`persistent().extend_ttl` with the same constants). Instance keys from
[storage-keys.md](storage-keys.md) are covered by the instance extension that
most state-changing methods perform and are not listed below.

## Policy by record family

| Record family | Renewed on write | Renewed on read | Can expire after inactivity? |
|---|---|---|---|
| `DataKey::Course` | Yes — register, approve/reject, pause/unpause, freeze, archive, expiry/start/prerequisite/max-cert/content-hash updates, enroll, completion | Yes — `get_course` | Only if neither written nor read via `get_course` for the TTL window |
| `DataKey::ArchivedCourse` | Yes — `archive_course` | No | **Yes** |
| `DataKey::CourseInstructorRef` | Yes — `register_course` | No | **Yes** |
| `DataKey::Enrollment` | Yes — enroll, `mark_completed`, `issue_certificate` | No | **Yes** |
| `DataKey2::EnrollmentByRef` | Yes — on reference generation | No | **Yes** |
| `DataKey::Certificate` | Yes — `issue_certificate` | No (`get_certificate` / `verify_certificate` do not extend) | **Yes** |
| `DataKey::CourseList`, `DataKey::InstructorRegistry` | Yes — `register_course` | No | **Yes** (only if no new course is registered) |
| `DataKey::InstructorTotalEarnings`, `DataKey::InstructorEarnings` | Yes — when earnings are credited | No | **Yes** |
| `DataKey::InstructorStats` | Yes — via `update_instructor_stats` | No | **Yes** |
| `DataKey::EnrollmentHistory` | Yes — on re-enroll / history append | No | **Yes** (see README "Lifecycle and storage growth") |
| `DataKey::RefundRequest` | Yes — request/process refund | No | **Yes** |
| `DataKey::CourseWaitlist` | Yes — join/leave/promote | No | **Yes** |
| Arbitration case lists (`CourseArbitrationCases`, `StudentArbitrationCases`, `ArbitrationCaseKey`) | Yes — append, escalate, resolve, timeout | No | **Yes** |
| Other persistent `DataKey2` records | Written with the network default TTL unless the writer extends them | No | **Yes** |

"Can expire" means the entry is archived by the network when its TTL runs out;
it is not deleted and can be restored with `stellar contract restore`, but reads
fail until it is. Long-lived records that matter to third parties (certificates,
enrollments) should be kept alive off-chain, e.g.:

```bash
stellar contract extend --id <CONTRACT_ID> --key-xdr <KEY_XDR> \
  --ledgers-to-extend 6300000 --durability persistent \
  --source "$STELLAR_ACCOUNT" --network testnet
```

## Verify

```bash
grep -n "PERSISTENT_TTL_\|fn extend_persistent_ttl" contracts/sorobanlard/src/lib.rs
grep -n "extend_persistent_ttl\|persistent().extend_ttl" contracts/sorobanlard/src/lib.rs
```

Update this table whenever a persistent key is added or a writer/reader starts
or stops extending its TTL.
