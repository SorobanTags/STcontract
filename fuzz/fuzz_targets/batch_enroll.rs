#![no_main]

use libfuzzer_sys::fuzz_target;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, BytesN, Env, String, Vec,
};
use sorobanlard::{SorobanLardContract, SorobanLardContractClient};
use std::format;

// Mirrors SorobanLardContract::MAX_BATCH_SIZE, which is private to the crate.
const MAX_BATCH_SIZE: u32 = 50;

fuzz_target!(|data: &[u8]| {
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
    let instructor = Address::generate(&env);
    let student = Address::generate(&env);
    let client = SorobanLardContractClient::new(&env, &contract_id);
    client.init(
        &admin,
        &sec_admin,
        &treasury,
        &20u32,
        &100u32,
        &1000u32,
        &17_280u32,
    );
    client.add_approved_token(&admin, &token_id);

    // Generate one more course than the batch limit so oversized, unique
    // batches can be represented. Any longer input is oversized regardless
    // of its remaining IDs, so the first over-limit items are sufficient.
    let course_count = MAX_BATCH_SIZE + 1;
    let mut available_courses = Vec::new(&env);
    for index in 0..course_count {
        let course_id_str = format!("FUZZ-BATCH-{index:02}");
        let course_id = String::from_str(&env, &course_id_str);
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
        env.ledger().with_mut(|ledger| {
            ledger.sequence_number += 1;
        });
        client.approve_course(&admin, &course_id);
        available_courses.push_back(course_id);
    }

    let mut course_ids = Vec::new(&env);
    for index in data.iter().take(course_count as usize) {
        course_ids.push_back(
            available_courses
                .get((*index as u32) % course_count)
                .unwrap(),
        );
    }

    let mut has_duplicates = false;
    for i in 0..course_ids.len() {
        let id = course_ids.get(i).unwrap();
        for j in 0..i {
            if course_ids.get(j).unwrap() == id {
                has_duplicates = true;
                break;
            }
        }
        if has_duplicates {
            break;
        }
    }

    let should_succeed =
        !course_ids.is_empty() && course_ids.len() <= MAX_BATCH_SIZE && !has_duplicates;
    let result = client.try_batch_enroll(&student, &course_ids);

    assert_eq!(
        result.is_ok(),
        should_succeed,
        "input: {data:?}, course_ids: {course_ids:?}"
    );

    if result.is_err() {
        for i in 0..course_ids.len() {
            assert!(
                !client.is_enrolled(&student, &course_ids.get(i).unwrap()),
                "rejected batch partially enrolled course {:?}",
                course_ids.get(i).unwrap()
            );
        }
    }
});
