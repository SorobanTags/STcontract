#![no_main]

use libfuzzer_sys::fuzz_target;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};
use sorobanlard::{SorobanLardContract, SorobanLardContractClient};

// Mirrors SorobanLardContract::MAX_COURSE_ID_LEN, which is private to the crate.
const MAX_COURSE_ID_LEN: usize = 256;

// Course IDs must be 1..=256 bytes of printable ASCII (0x20..=0x7E).
fn is_valid_course_id(bytes: &[u8]) -> bool {
    !bytes.is_empty()
        && bytes.len() <= MAX_COURSE_ID_LEN
        && bytes.iter().all(|b| (b' '..=b'~').contains(b))
}

// The documented rule: an evidence_hash of Some("") is rejected; None requires
// student auth (mocked here); any Some(non-empty) is accepted.
fn expected_evidence_outcome(evidence: &[u8]) -> bool {
    !evidence.is_empty()
}

fuzz_target!(|data: &[u8]| {
    // Split the input into two halves: first byte selects the split point so
    // the fuzzer can explore the full cross-product of (evidence_hash, course_id).
    if data.is_empty() {
        return;
    }
    let split = (data[0] as usize).min(data.len() - 1);
    let (evidence_bytes, course_id_bytes) = data[1..].split_at(split.min(data.len() - 1));

    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(SorobanLardContract, ());
    let token_admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    let admin = Address::generate(&env);
    let sec_admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let client = SorobanLardContractClient::new(&env, &contract_id);
    client.init(&admin, &sec_admin, &treasury, &20u32, &50u32, &1000u32, &17_280u32);
    client.add_approved_token(&admin, &token_id);

    // Only proceed when the course_id is valid so we can reach mark_completed's
    // own validation surface without being blocked by register_course's check.
    if !is_valid_course_id(course_id_bytes) {
        return;
    }

    let course_id = String::from_bytes(&env, course_id_bytes);
    let instructor = Address::generate(&env);
    client.register_course(
        &instructor,
        &course_id,
        &0i128,
        &token_id,
        &0u32,
        &None,
        &BytesN::from_array(&env, &[1u8; 32]),
        &Vec::new(&env),
    );
    client.approve_course(&admin, &course_id);

    let student = Address::generate(&env);
    client.enroll(&student, &course_id);

    let evidence_hash: Option<String> = if evidence_bytes.is_empty() {
        // None branch — student co-auth is mocked, so this should succeed.
        None
    } else {
        Some(String::from_bytes(&env, evidence_bytes))
    };

    let result = client.try_mark_completed(&admin, &student, &course_id, &evidence_hash);

    match &evidence_hash {
        None => {
            // None always succeeds (student auth is mocked).
            assert!(result.is_ok(), "None evidence_hash should succeed; got {result:?}");
        }
        Some(hash) => {
            let empty = hash.len() == 0;
            if empty {
                // Empty string must be rejected.
                assert!(
                    result.is_err(),
                    "empty evidence_hash must be rejected; got {result:?}"
                );
            } else {
                // Non-empty string must be accepted.
                assert!(
                    result.is_ok(),
                    "non-empty evidence_hash must be accepted; got {result:?}"
                );
            }
        }
    }
});
