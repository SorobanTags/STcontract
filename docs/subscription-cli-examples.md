# Subscription Enrollment & Renewal CLI Examples (#110)

This guide shows end-to-end `stellar contract invoke` commands for time-bounded
(subscription-style) course enrollments and renewals using the current public
contract interface in `contracts/sorobanlard/src/lib.rs`.

## Prerequisites

Export the target deployment variables before running the commands below:

```bash
export NETWORK="testnet"
export CONTRACT_ID="<SOROBANLARD_CONTRACT_ID>"
export COURSE_ID="COURSE-TAILORING-001"
export INSTRUCTOR="<INSTRUCTOR_IDENTITY_OR_ADDRESS>"
export STUDENT="<STUDENT_IDENTITY_OR_ADDRESS>"
```

## 1. Configure an Enrollment Expiry Window (Instructor or Admin)

Use `set_enrollment_expiry(caller, course_id, expiry_ledgers)` to attach a
subscription duration in ledgers (~5 s/ledger; `518400` ledgers ≈ 30 days):

```bash
stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source "$INSTRUCTOR" \
  --network "$NETWORK" \
  -- set_enrollment_expiry \
  --caller "$INSTRUCTOR" \
  --course_id "$COURSE_ID" \
  --expiry_ledgers 518400
```

## 2. Enroll in the Course (Initial Subscription Period)

Call `enroll(student, course_id)`. The contract transfers `course.price` in the
course's approved token, routes the platform fee to `treasury`, credits the
instructor share to `InstructorEarnings`, and records the enrollment ledger and
expiry window:

```bash
stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source "$STUDENT" \
  --network "$NETWORK" \
  -- enroll \
  --student "$STUDENT" \
  --course_id "$COURSE_ID"
```

## 3. Check Enrollment & Expiry Status

Query `get_enrollment(caller, student, course_id)` (caller must be the student,
the course instructor, or an admin):

```bash
stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source "$STUDENT" \
  --network "$NETWORK" \
  -- get_enrollment \
  --caller "$STUDENT" \
  --student "$STUDENT" \
  --course_id "$COURSE_ID"
```

Verify active enrollment status with `is_enrolled(student, course_id)`:

```bash
stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source "$STUDENT" \
  --network "$NETWORK" \
  -- is_enrolled \
  --student "$STUDENT" \
  --course_id "$COURSE_ID"
```

## 4. Renew an Expired Subscription (`re_enroll`)

Once the current enrollment period has elapsed, call `re_enroll(student, course_id)`
to start a new enrollment period with the current fee split and record the prior
period in `get_enrollment_history(caller, student, course_id)`:

```bash
stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source "$STUDENT" \
  --network "$NETWORK" \
  -- re_enroll \
  --student "$STUDENT" \
  --course_id "$COURSE_ID"
```

Inspect renewal history:

```bash
stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source "$STUDENT" \
  --network "$NETWORK" \
  -- get_enrollment_history \
  --caller "$STUDENT" \
  --student "$STUDENT" \
  --course_id "$COURSE_ID"
```
