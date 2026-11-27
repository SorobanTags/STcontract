// ============================================================
// COURSE STATUS LIFECYCLE TESTS (#51, #52, #53, #54)
// ============================================================
//
// - #51: table-driven matrix of every `CourseStatus` transition documented in
//   STATE_MACHINES.md §1. Rejected transitions must leave the stored record and
//   the emitted events unchanged.
// - #52: a `Rejected` course cannot be approved or enrolled in, and the failed
//   attempts move no tokens and change no counters or status.
// - #53: `get_course` returns the archived record right up to and at the
//   `ARCHIVE_COOLDOWN_LEDGERS` boundary, without recreating an active record.
// - #54: `get_course` behaviour once the archived entry's persistent TTL has
//   lapsed without the key being read in between.

use super::*;
use soroban_sdk::testutils::storage::Persistent as _;

#[derive(Clone, Copy, Debug, PartialEq)]
enum St {
    Pending,
    Active,
    Paused,
    Rejected,
    Archived,
}

#[derive(Clone, Copy, Debug)]
enum Op {
    Approve,
    Reject,
    Pause,
    Unpause,
    Archive,
}

/// Fields of `Course` that every transition can touch. `Course` does not
/// derive `PartialEq`, so compare a snapshot instead.
type Snapshot = (CourseStatus, u32, u32, i128, Option<u32>, u32, bool, bool);

fn snapshot(env: &Env, contract_id: &Address, course_id: &String) -> Snapshot {
    let client = SorobanLardContractClient::new(env, contract_id);
    let c = client.get_course(course_id).unwrap();
    let (active_key, archived_key) = env.as_contract(contract_id, || {
        (
            env.storage()
                .persistent()
                .has(&DataKey::Course(course_id.clone())),
            env.storage()
                .persistent()
                .has(&DataKey::ArchivedCourse(course_id.clone())),
        )
    });
    (
        c.status,
        c.total_enrollments,
        c.active_enrollments,
        c.total_earned,
        c.archived_at_ledger,
        c.last_updated_ledger,
        active_key,
        archived_key,
    )
}

fn contract_event_count(env: &Env, contract_id: &Address) -> usize {
    env.events()
        .all()
        .iter()
        .filter(|(c, _, _)| c == contract_id)
        .count()
}

/// Register a course and drive it into `from`.
fn course_in(
    env: &Env,
    client: &SorobanLardContractClient,
    token_id: &Address,
    admin: &Address,
    sec_admin: &Address,
    instructor: &Address,
    id: &str,
    from: St,
) -> String {
    let course_id = String::from_str(env, id);
    client.register_course(
        instructor,
        &course_id,
        &100_000_000,
        token_id,
        &0u32,
        &None,
        &BytesN::from_array(env, &[1u8; 32]),
        &soroban_sdk::Vec::new(env),
    );
    env.ledger().with_mut(|l| l.sequence_number += 1);
    match from {
        St::Pending => {}
        St::Rejected => {
            client.reject_course(admin, &course_id, &String::from_str(env, "no"));
        }
        St::Active | St::Paused | St::Archived => {
            client.approve_course(admin, &course_id);
            if from != St::Active {
                client.pause_course(admin, &course_id);
            }
            if from == St::Archived {
                client.archive_course(admin, sec_admin, &course_id, &None);
            }
        }
    }
    env.ledger().with_mut(|l| l.sequence_number += 1);
    course_id
}

fn apply(
    env: &Env,
    client: &SorobanLardContractClient,
    admin: &Address,
    sec_admin: &Address,
    course_id: &String,
    op: Op,
) -> bool {
    match op {
        Op::Approve => client.try_approve_course(admin, course_id).is_ok(),
        Op::Reject => client
            .try_reject_course(admin, course_id, &String::from_str(env, "no"))
            .is_ok(),
        Op::Pause => client.try_pause_course(admin, course_id).is_ok(),
        Op::Unpause => client.try_unpause_course(admin, course_id).is_ok(),
        Op::Archive => client
            .try_archive_course(admin, sec_admin, course_id, &None)
            .is_ok(),
    }
}

/// Documented outcome of `op` on a course in `from` (STATE_MACHINES.md §1,
/// rows C2–C4, C7, C8). `None` means the call must fail.
fn expected(from: St, op: Op) -> Option<CourseStatus> {
    match (from, op) {
        (St::Pending, Op::Approve) => Some(CourseStatus::Active),
        (St::Pending, Op::Reject) => Some(CourseStatus::Rejected),
        (St::Active, Op::Pause) => Some(CourseStatus::Paused),
        (St::Paused, Op::Unpause) => Some(CourseStatus::Active),
        (St::Paused, Op::Archive) => Some(CourseStatus::Archived),
        _ => None,
    }
}

#[test]
fn test_course_status_transition_matrix() {
    let froms = [
        St::Pending,
        St::Active,
        St::Paused,
        St::Rejected,
        St::Archived,
    ];
    let ops = [Op::Approve, Op::Reject, Op::Pause, Op::Unpause, Op::Archive];

    for from in froms {
        for op in ops {
            let (env, contract_id, token_id, admin, sec_admin, _treasury, instructor) = setup();
            let client = SorobanLardContractClient::new(&env, &contract_id);
            let course_id = course_in(
                &env,
                &client,
                &token_id,
                &admin,
                &sec_admin,
                &instructor,
                "COURSE-MATRIX",
                from,
            );

            let before = snapshot(&env, &contract_id, &course_id);
            let events_before = contract_event_count(&env, &contract_id);
            let ok = apply(&env, &client, &admin, &sec_admin, &course_id, op);

            match expected(from, op) {
                Some(to) => {
                    assert!(ok, "{:?} --{:?}--> should succeed", from, op);
                    let after = client.get_course(&course_id).unwrap();
                    assert_eq!(after.status, to, "{:?} --{:?}-->", from, op);
                }
                None => {
                    assert!(!ok, "{:?} --{:?}--> should be rejected", from, op);
                    assert_eq!(
                        snapshot(&env, &contract_id, &course_id),
                        before,
                        "{:?} --{:?}--> must not change the record",
                        from,
                        op
                    );
                    // A failed invocation is rolled back, so it contributes no
                    // contract events; the event log is unchanged.
                    let events_after = contract_event_count(&env, &contract_id);
                    assert!(
                        events_after == 0 || events_after == events_before,
                        "{:?} --{:?}--> must not emit events",
                        from,
                        op
                    );
                }
            }
        }
    }
}

#[test]
fn test_rejected_course_cannot_be_approved_or_enrolled() {
    let (env, contract_id, token_id, admin, sec_admin, treasury, instructor) = setup();
    let client = SorobanLardContractClient::new(&env, &contract_id);
    let token = token::Client::new(&env, &token_id);
    let student = Address::generate(&env);
    token::StellarAssetClient::new(&env, &token_id).mint(&student, &1_000_000_000);

    let course_id = course_in(
        &env,
        &client,
        &token_id,
        &admin,
        &sec_admin,
        &instructor,
        "COURSE-REJECTED",
        St::Rejected,
    );

    let before = snapshot(&env, &contract_id, &course_id);
    let balances = |who: &[&Address]| -> std::vec::Vec<i128> {
        who.iter().map(|a| token.balance(a)).collect()
    };
    let parties = [&student, &instructor, &treasury, &contract_id];
    let balances_before = balances(&parties);

    assert!(client.try_approve_course(&admin, &course_id).is_err());
    assert!(client.try_approve_course(&sec_admin, &course_id).is_err());
    assert!(client.try_enroll(&student, &course_id).is_err());
    assert!(client
        .try_batch_enroll(&student, &soroban_sdk::vec![&env, course_id.clone()])
        .is_err());

    assert_eq!(snapshot(&env, &contract_id, &course_id), before);
    assert_eq!(before.0, CourseStatus::Rejected);
    assert_eq!(before.1, 0, "total_enrollments must stay 0");
    assert_eq!(before.2, 0, "active_enrollments must stay 0");
    assert_eq!(before.3, 0, "total_earned must stay 0");
    assert_eq!(balances(&parties), balances_before, "no tokens may move");
    assert!(client
        .get_enrollment(&student, &student, &course_id)
        .is_none());
}

fn archive(
    env: &Env,
    client: &SorobanLardContractClient,
    token_id: &Address,
    admin: &Address,
    sec_admin: &Address,
    instructor: &Address,
    id: &str,
) -> (String, u32) {
    let course_id = course_in(
        env,
        client,
        token_id,
        admin,
        sec_admin,
        instructor,
        id,
        St::Archived,
    );
    let archived_at = client
        .get_course(&course_id)
        .unwrap()
        .archived_at_ledger
        .unwrap();
    (course_id, archived_at)
}

fn assert_archived_only(
    env: &Env,
    contract_id: &Address,
    course_id: &String,
    instructor: &Address,
    archived_at: u32,
) {
    let client = SorobanLardContractClient::new(env, contract_id);
    let course = client.get_course(course_id).unwrap();
    assert_eq!(course.id, *course_id);
    assert_eq!(course.instructor, *instructor);
    assert_eq!(course.status, CourseStatus::Archived);
    assert_eq!(course.archived_at_ledger, Some(archived_at));
    let s = snapshot(env, contract_id, course_id);
    assert!(!s.6, "no active record may be recreated");
    assert!(s.7, "archived record must remain");
}

#[test]
fn test_archived_course_lookup_across_cooldown_boundary() {
    let (env, contract_id, token_id, admin, sec_admin, _treasury, instructor) = setup();
    let client = SorobanLardContractClient::new(&env, &contract_id);
    let (course_id, archived_at) = archive(
        &env,
        &client,
        &token_id,
        &admin,
        &sec_admin,
        &instructor,
        "COURSE-COOLDOWN",
    );
    let cooldown = SorobanLardContract::ARCHIVE_COOLDOWN_LEDGERS;
    assert_eq!(cooldown, 172_800);

    for ledger in [archived_at + cooldown - 1, archived_at + cooldown] {
        env.ledger().with_mut(|l| l.sequence_number = ledger);
        assert_archived_only(&env, &contract_id, &course_id, &instructor, archived_at);
    }
}

#[test]
fn test_archived_course_lookup_after_persistent_ttl_expires() {
    let (env, contract_id, token_id, admin, sec_admin, _treasury, instructor) = setup();
    let client = SorobanLardContractClient::new(&env, &contract_id);
    let (course_id, archived_at) = archive(
        &env,
        &client,
        &token_id,
        &admin,
        &sec_admin,
        &instructor,
        "COURSE-EXPIRED",
    );
    let key = DataKey::ArchivedCourse(course_id.clone());
    let live_until = env.as_contract(&contract_id, || env.storage().persistent().get_ttl(&key))
        + env.ledger().sequence();
    assert!(live_until >= archived_at + SorobanLardContract::PERSISTENT_TTL_THRESHOLD);

    // Keep the contract instance (and code) alive without touching the
    // archived key, then step past the archived entry's TTL.
    env.ledger()
        .with_mut(|l| l.sequence_number = live_until - 1_000_000);
    env.as_contract(&contract_id, || {
        env.storage().instance().extend_ttl(
            SorobanLardContract::PERSISTENT_TTL_THRESHOLD,
            SorobanLardContract::PERSISTENT_TTL_EXTEND_TO,
        )
    });
    env.ledger()
        .with_mut(|l| l.sequence_number = live_until + 1);

    // Documented behaviour: once the persistent entry has expired it is
    // archived by the network and cannot be read until it is restored, so
    // `get_course` fails rather than returning `None` or recreating an active
    // record. Off-chain callers must restore the entry (RestoreFootprintOp)
    // before reading it.
    // The test host escalates the archived-entry access to a panic
    // ("Accessed contract data key ... that has been archived").
    let lookup = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.get_course(&course_id)
    }));
    assert!(
        lookup.is_err(),
        "get_course must not return a value once the archived entry has expired"
    );
}
