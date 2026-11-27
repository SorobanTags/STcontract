# Storage-Key Change Checklist

Use this checklist for any PR that adds, renames, moves, or changes the stored
shape of a key or value in `contracts/sorobanlard/src/lib.rs`.

## 1. Choose the enum

- [ ] `DataKey` is capped at 50 `#[contracttype]` variants and is full — **do
      not add variants to it**.
- [ ] Add new keys to `DataKey2`. Do not introduce new direct struct keys;
      `ArbitrationCaseKey` is a compatibility exception only.
- [ ] Never rename or reorder existing variants: the variant name is part of the
      on-chain key, so renaming orphans existing data.

## 2. Choose the tier

- [ ] Singleton config / auth flag / global counter → `instance()`.
- [ ] Per-course / per-student / per-instructor record or growing list →
      `persistent()`.
- [ ] Read and write the key through the **same** tier everywhere.
- [ ] Add a row to [storage-keys.md](storage-keys.md).
- [ ] For persistent keys, decide the TTL policy (extend on write? on read?) and
      add a row to [storage-ttl.md](storage-ttl.md).

## 3. If a stored struct shape changes

Applies to any `#[contracttype]` value already on-chain (`Course`,
`Enrollment`, `Certificate`, …) — adding, removing or renaming any field,
including `Option<T>` fields, breaks decoding of old records.

- [ ] Keep the old shape as a legacy type (e.g. `CourseV1`); add new fields as
      `Option<T>`.
- [ ] Bump `STORAGE_SCHEMA_VERSION`.
- [ ] Add a `run_migration_step` arm that reads each affected record as the
      legacy type and rewrites it in the new shape.
- [ ] Document in the release notes that both admins must call
      `migrate(admin1, admin2)` once after `upgrade_contract`; confirm with
      `get_storage_schema_version()`.

## 4. Tests

- [ ] Extend the "UPGRADE STORAGE COMPATIBILITY & MIGRATION" tests in
      `contracts/sorobanlard/src/test.rs`: write records in the old shape,
      simulate the upgrade and migration, and read them back.
- [ ] Run:

```bash
cd contracts/sorobanlard
cargo test simulated_upgrade
cargo test migrat
cargo test
```

See also: README "Upgrades & Storage Migrations" and
[upgrade-runbook.md](upgrade-runbook.md).
