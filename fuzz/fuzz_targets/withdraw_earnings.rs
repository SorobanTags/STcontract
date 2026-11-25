#![no_main]

use libfuzzer_sys::fuzz_target;
use soroban_sdk::{
    testutils::Address as _,
    token, Address, Env, String,
};
use sorobanlard::{SorobanLardContract, SorobanLardContractClient};

fuzz_target!(|data: &[u8]| {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(SorobanLardContract, ());
    let client = SorobanLardContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let instructor = Address::generate(&env);
    let student = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();
    let token_client = token::Client::new(&env, &token_id);
    let token_admin_client = token::StellarAssetClient::new(&env, &token_id);

    client.init(&admin, &token_id);
    client.add_approved_token(&admin, &token_id);

    let course_id = String::from_str(&env, "COURSE1");
    let price: i128 = 100;
    client.register_course(&instructor, &course_id, &price, &token_id);
    client.approve_course(&admin, &course_id);
    client.enroll(&student, &course_id);

    // Fund the contract so withdrawals can be paid out.
    token_admin_client.mint(&contract_id, &price);

    let credited_balance = client.get_instructor_earnings(&instructor, &token_id);
    let balance_before = token_client.balance(&instructor);

    // Derive the withdrawal amount from the first 16 bytes of the input.
    let mut amount_bytes = [0u8; 16];
    let n = core::cmp::min(data.len(), 16);
    amount_bytes[..n].copy_from_slice(&data[..n]);
    let amount = i128::from_le_bytes(amount_bytes);

    let result = client.try_withdraw_earnings(&instructor, &token_id, &amount);

    let expected_success = amount >= 1 && amount <= credited_balance;
    assert_eq!(
        result.is_ok(),
        expected_success,
        "withdraw_earnings success mismatch: amount={}, credited_balance={}, data={:?}",
        amount,
        credited_balance,
        data
    );

    if result.is_ok() {
        let balance_after = token_client.balance(&instructor);
        assert_eq!(
            balance_after - balance_before,
            amount,
            "withdraw_earnings balance delta mismatch: amount={}, before={}, after={}, data={:?}",
            amount,
            balance_before,
            balance_after,
            data
        );
    }
});