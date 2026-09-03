#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token, vec, Address, BytesN, Env, Error, IntoVal, Map, Symbol, Val,
};

const WASM: &[u8] =
    include_bytes!("../../target/wasm32v1-none/release/oynk_settlement_protocol_contract.wasm");
const USD: u32 = 840;
const EUR: u32 = 978;

struct Fixture {
    env: Env,
    contract_id: Address,
    token_id: Address,
    admin: Address,
    manager: Address,
}

fn fixture() -> Fixture {
    fixture_with_auth(true)
}

fn fixture_with_auth(mock_auth: bool) -> Fixture {
    let env = Env::default();
    if mock_auth {
        env.mock_all_auths();
    }

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token_id = token_contract.address();
    let contract_id = env.register(WASM, (admin.clone(), manager.clone(), token_id.clone()));

    Fixture {
        env,
        contract_id,
        token_id,
        admin,
        manager,
    }
}

fn hash(env: &Env, byte: u8) -> BytesN<32> {
    BytesN::from_array(env, &[byte; 32])
}

fn route(env: &Env, origin: Option<u32>, destination: Option<u32>) -> Val {
    let mut route = Map::<Symbol, Val>::new(env);
    route.set(Symbol::new(env, "origin"), origin.into_val(env));
    route.set(Symbol::new(env, "destination"), destination.into_val(env));
    route.into_val(env)
}

fn unit_variant(env: &Env, name: &str) -> Val {
    let symbol: Val = Symbol::new(env, name).into_val(env);
    vec![env, symbol].into_val(env)
}

fn invoke_void(fixture: &Fixture, function: &str, args: soroban_sdk::Vec<Val>) {
    fixture.env.invoke_contract::<Val>(
        &fixture.contract_id,
        &Symbol::new(&fixture.env, function),
        args,
    );
}

fn assert_contract_error(
    fixture: &Fixture,
    function: &str,
    args: soroban_sdk::Vec<Val>,
    code: u32,
) {
    let result = fixture.env.try_invoke_contract::<Val, Error>(
        &fixture.contract_id,
        &Symbol::new(&fixture.env, function),
        args,
    );
    if let Err(Ok(error)) = result {
        assert_eq!(error, Error::from_contract_error(code));
    } else {
        panic!("expected contract error {code}");
    }
}

fn create_request(
    fixture: &Fixture,
    creator: &Address,
    settlement_type: &str,
    origin: Option<u32>,
    destination: Option<u32>,
    amount: i128,
    deadline: u32,
    seed: u8,
) -> u64 {
    fixture.env.invoke_contract(
        &fixture.contract_id,
        &Symbol::new(&fixture.env, "create_settlement_request"),
        vec![
            &fixture.env,
            creator.clone().into_val(&fixture.env),
            hash(&fixture.env, seed).into_val(&fixture.env),
            hash(&fixture.env, seed.wrapping_add(1)).into_val(&fixture.env),
            route(&fixture.env, origin, destination),
            unit_variant(&fixture.env, settlement_type),
            amount.into_val(&fixture.env),
            deadline.into_val(&fixture.env),
        ],
    )
}

fn set_quote(fixture: &Fixture, id: u64, source_amount: Option<i128>, amount: i128) {
    invoke_void(
        fixture,
        "set_settlement_quote",
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            source_amount.into_val(&fixture.env),
            amount.into_val(&fixture.env),
            hash(&fixture.env, 20).into_val(&fixture.env),
        ],
    );
}

fn accept(fixture: &Fixture, id: u64, kind: &str, settler: &Address) {
    invoke_void(
        fixture,
        "accept_settlement",
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            unit_variant(&fixture.env, kind),
            settler.clone().into_val(&fixture.env),
        ],
    );
}

fn deposit(fixture: &Fixture, id: u64, depositor: &Address) {
    invoke_void(
        fixture,
        "deposit_settlement_asset",
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            depositor.clone().into_val(&fixture.env),
        ],
    );
}

fn confirm(fixture: &Fixture, id: u64, kind: &str, seed: u8) {
    invoke_void(
        fixture,
        "confirm_settlement",
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            unit_variant(&fixture.env, kind),
            hash(&fixture.env, seed).into_val(&fixture.env),
        ],
    );
}

fn funded_fiat_to_crypto(
    fixture: &Fixture,
    creator: &Address,
    source_settler: &Address,
    amount: i128,
    seed: u8,
) -> u64 {
    let token_admin = token::StellarAssetClient::new(&fixture.env, &fixture.token_id);
    token_admin.mint(source_settler, &amount);
    let id = create_request(
        fixture,
        creator,
        "FiatToCrypto",
        Some(USD),
        None,
        amount,
        100,
        seed,
    );
    set_quote(fixture, id, Some(amount), amount);
    accept(fixture, id, "Source", source_settler);
    deposit(fixture, id, source_settler);
    id
}

#[test]
fn fiat_to_crypto_completes_with_exact_escrow_transfer() {
    let fixture = fixture();
    let token = token::Client::new(&fixture.env, &fixture.token_id);
    let token_admin = token::StellarAssetClient::new(&fixture.env, &fixture.token_id);
    let creator = Address::generate(&fixture.env);
    let source_settler = Address::generate(&fixture.env);
    let amount = 50_000_000i128;

    token_admin.mint(&source_settler, &amount);

    let id: u64 = fixture.env.invoke_contract(
        &fixture.contract_id,
        &Symbol::new(&fixture.env, "create_settlement_request"),
        vec![
            &fixture.env,
            creator.clone().into_val(&fixture.env),
            hash(&fixture.env, 1).into_val(&fixture.env),
            hash(&fixture.env, 2).into_val(&fixture.env),
            route(&fixture.env, Some(USD), None),
            unit_variant(&fixture.env, "FiatToCrypto"),
            amount.into_val(&fixture.env),
            100u32.into_val(&fixture.env),
        ],
    );
    fixture.env.invoke_contract::<Val>(
        &fixture.contract_id,
        &Symbol::new(&fixture.env, "set_settlement_quote"),
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            Some(amount).into_val(&fixture.env),
            amount.into_val(&fixture.env),
            hash(&fixture.env, 3).into_val(&fixture.env),
        ],
    );
    fixture.env.invoke_contract::<Val>(
        &fixture.contract_id,
        &Symbol::new(&fixture.env, "accept_settlement"),
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            unit_variant(&fixture.env, "Source"),
            source_settler.clone().into_val(&fixture.env),
        ],
    );
    fixture.env.invoke_contract::<Val>(
        &fixture.contract_id,
        &Symbol::new(&fixture.env, "deposit_settlement_asset"),
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            source_settler.clone().into_val(&fixture.env),
        ],
    );
    fixture.env.invoke_contract::<Val>(
        &fixture.contract_id,
        &Symbol::new(&fixture.env, "confirm_settlement"),
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            unit_variant(&fixture.env, "SourceSettlement"),
            hash(&fixture.env, 4).into_val(&fixture.env),
        ],
    );
    fixture.env.invoke_contract::<Val>(
        &fixture.contract_id,
        &Symbol::new(&fixture.env, "claim_settlement_asset"),
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            creator.clone().into_val(&fixture.env),
        ],
    );

    assert_eq!(token.balance(&source_settler), 0);
    assert_eq!(token.balance(&creator), amount);
}

#[test]
fn fiat_to_fiat_requires_both_settlement_legs_before_claim() {
    let fixture = fixture();
    let token = token::Client::new(&fixture.env, &fixture.token_id);
    let token_admin = token::StellarAssetClient::new(&fixture.env, &fixture.token_id);
    let creator = Address::generate(&fixture.env);
    let source_settler = Address::generate(&fixture.env);
    let destination_settler = Address::generate(&fixture.env);
    let amount = 150_000_000i128;

    token_admin.mint(&source_settler, &amount);

    let id: u64 = fixture.env.invoke_contract(
        &fixture.contract_id,
        &Symbol::new(&fixture.env, "create_settlement_request"),
        vec![
            &fixture.env,
            creator.into_val(&fixture.env),
            hash(&fixture.env, 5).into_val(&fixture.env),
            hash(&fixture.env, 6).into_val(&fixture.env),
            route(&fixture.env, Some(USD), Some(EUR)),
            unit_variant(&fixture.env, "FiatToFiat"),
            amount.into_val(&fixture.env),
            100u32.into_val(&fixture.env),
        ],
    );
    fixture.env.invoke_contract::<Val>(
        &fixture.contract_id,
        &Symbol::new(&fixture.env, "set_settlement_quote"),
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            Some(amount).into_val(&fixture.env),
            amount.into_val(&fixture.env),
            hash(&fixture.env, 7).into_val(&fixture.env),
        ],
    );
    fixture.env.invoke_contract::<Val>(
        &fixture.contract_id,
        &Symbol::new(&fixture.env, "accept_settlement"),
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            unit_variant(&fixture.env, "Source"),
            source_settler.clone().into_val(&fixture.env),
        ],
    );
    fixture.env.invoke_contract::<Val>(
        &fixture.contract_id,
        &Symbol::new(&fixture.env, "deposit_settlement_asset"),
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            source_settler.clone().into_val(&fixture.env),
        ],
    );
    fixture.env.invoke_contract::<Val>(
        &fixture.contract_id,
        &Symbol::new(&fixture.env, "confirm_settlement"),
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            unit_variant(&fixture.env, "SourceSettlement"),
            hash(&fixture.env, 8).into_val(&fixture.env),
        ],
    );
    fixture.env.invoke_contract::<Val>(
        &fixture.contract_id,
        &Symbol::new(&fixture.env, "accept_settlement"),
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            unit_variant(&fixture.env, "Destination"),
            destination_settler.clone().into_val(&fixture.env),
        ],
    );
    fixture.env.invoke_contract::<Val>(
        &fixture.contract_id,
        &Symbol::new(&fixture.env, "confirm_settlement"),
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            unit_variant(&fixture.env, "DestinationSettlement"),
            hash(&fixture.env, 9).into_val(&fixture.env),
        ],
    );
    fixture.env.invoke_contract::<Val>(
        &fixture.contract_id,
        &Symbol::new(&fixture.env, "claim_settlement_asset"),
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            destination_settler.clone().into_val(&fixture.env),
        ],
    );

    assert_eq!(token.balance(&destination_settler), amount);
}

#[test]
fn state_changing_calls_require_the_expected_authority() {
    let unauthenticated = fixture_with_auth(false);
    let creator = Address::generate(&unauthenticated.env);
    let create_result = unauthenticated.env.try_invoke_contract::<u64, Error>(
        &unauthenticated.contract_id,
        &Symbol::new(&unauthenticated.env, "create_settlement_request"),
        vec![
            &unauthenticated.env,
            creator.clone().into_val(&unauthenticated.env),
            hash(&unauthenticated.env, 30).into_val(&unauthenticated.env),
            hash(&unauthenticated.env, 31).into_val(&unauthenticated.env),
            route(&unauthenticated.env, Some(USD), None),
            unit_variant(&unauthenticated.env, "FiatToCrypto"),
            1i128.into_val(&unauthenticated.env),
            100u32.into_val(&unauthenticated.env),
        ],
    );
    assert!(create_result.is_err());

    let fixture = fixture();
    let creator = Address::generate(&fixture.env);
    let id = create_request(
        &fixture,
        &creator,
        "FiatToCrypto",
        Some(USD),
        None,
        1,
        100,
        32,
    );
    assert_eq!(id, 1);
    assert_eq!(fixture.env.auths()[0].0, creator);

    set_quote(&fixture, id, Some(1), 1);
    assert_eq!(fixture.env.auths()[0].0, fixture.manager);

    let new_manager = Address::generate(&fixture.env);
    invoke_void(
        &fixture,
        "update_manager",
        vec![&fixture.env, new_manager.into_val(&fixture.env)],
    );
    assert_eq!(fixture.env.auths()[0].0, fixture.admin);
}

#[test]
fn quotes_acceptance_deposits_and_claims_are_replay_safe() {
    let fixture = fixture();
    let creator = Address::generate(&fixture.env);
    let source_settler = Address::generate(&fixture.env);
    let amount = 25_000_000i128;
    let token = token::Client::new(&fixture.env, &fixture.token_id);
    let token_admin = token::StellarAssetClient::new(&fixture.env, &fixture.token_id);
    token_admin.mint(&source_settler, &amount);

    let id = create_request(
        &fixture,
        &creator,
        "FiatToCrypto",
        Some(USD),
        None,
        amount,
        100,
        40,
    );
    set_quote(&fixture, id, Some(amount), amount);
    assert_contract_error(
        &fixture,
        "set_settlement_quote",
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            Some(amount).into_val(&fixture.env),
            amount.into_val(&fixture.env),
            hash(&fixture.env, 41).into_val(&fixture.env),
        ],
        208,
    );

    accept(&fixture, id, "Source", &source_settler);
    assert_eq!(fixture.env.auths()[0].0, source_settler);
    assert_contract_error(
        &fixture,
        "accept_settlement",
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            unit_variant(&fixture.env, "Source"),
            source_settler.clone().into_val(&fixture.env),
        ],
        207,
    );

    deposit(&fixture, id, &source_settler);
    assert_eq!(fixture.env.auths()[0].0, source_settler);
    assert_contract_error(
        &fixture,
        "deposit_settlement_asset",
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            source_settler.clone().into_val(&fixture.env),
        ],
        207,
    );

    confirm(&fixture, id, "SourceSettlement", 42);
    assert_eq!(fixture.env.auths()[0].0, fixture.manager);
    invoke_void(
        &fixture,
        "claim_settlement_asset",
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            creator.clone().into_val(&fixture.env),
        ],
    );
    assert_eq!(fixture.env.auths()[0].0, creator);
    assert_contract_error(
        &fixture,
        "claim_settlement_asset",
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            creator.clone().into_val(&fixture.env),
        ],
        207,
    );
    assert_eq!(token.balance(&creator), amount);
    assert_eq!(token.balance(&fixture.contract_id), 0);
}

#[test]
fn expired_requests_reject_late_state_transitions() {
    let fixture = fixture();
    let creator = Address::generate(&fixture.env);
    let id = create_request(
        &fixture,
        &creator,
        "FiatToCrypto",
        Some(USD),
        None,
        100,
        5,
        50,
    );
    fixture.env.ledger().set_sequence_number(6);

    assert_contract_error(
        &fixture,
        "set_settlement_quote",
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            Some(100i128).into_val(&fixture.env),
            100i128.into_val(&fixture.env),
            hash(&fixture.env, 51).into_val(&fixture.env),
        ],
        209,
    );
}

#[test]
fn cancellation_requires_a_participant_and_is_terminal() {
    let fixture = fixture();
    let creator = Address::generate(&fixture.env);
    let stranger = Address::generate(&fixture.env);
    let id = create_request(
        &fixture,
        &creator,
        "FiatToCrypto",
        Some(USD),
        None,
        100,
        100,
        60,
    );

    assert_contract_error(
        &fixture,
        "cancel",
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            stranger.into_val(&fixture.env),
        ],
        232,
    );
    invoke_void(
        &fixture,
        "cancel",
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            creator.clone().into_val(&fixture.env),
        ],
    );
    assert_contract_error(
        &fixture,
        "cancel",
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            creator.into_val(&fixture.env),
        ],
        207,
    );
}

#[test]
fn refund_returns_the_exact_deposit_and_cannot_repeat() {
    let fixture = fixture();
    let creator = Address::generate(&fixture.env);
    let source_settler = Address::generate(&fixture.env);
    let amount = 70_000_000i128;
    let token = token::Client::new(&fixture.env, &fixture.token_id);
    let id = funded_fiat_to_crypto(&fixture, &creator, &source_settler, amount, 70);

    invoke_void(
        &fixture,
        "refund_settler",
        vec![&fixture.env, id.into_val(&fixture.env)],
    );
    assert_eq!(fixture.env.auths()[0].0, fixture.manager);
    assert_eq!(token.balance(&source_settler), amount);
    assert_eq!(token.balance(&fixture.contract_id), 0);
    assert_contract_error(
        &fixture,
        "refund_settler",
        vec![&fixture.env, id.into_val(&fixture.env)],
        207,
    );
}

#[test]
fn disputes_reject_outsiders_and_refund_the_depositor() {
    let fixture = fixture();
    let creator = Address::generate(&fixture.env);
    let source_settler = Address::generate(&fixture.env);
    let outsider = Address::generate(&fixture.env);
    let amount = 80_000_000i128;
    let token = token::Client::new(&fixture.env, &fixture.token_id);
    let id = funded_fiat_to_crypto(&fixture, &creator, &source_settler, amount, 80);

    assert_contract_error(
        &fixture,
        "dispute",
        vec![
            &fixture.env,
            outsider.into_val(&fixture.env),
            id.into_val(&fixture.env),
            hash(&fixture.env, 81).into_val(&fixture.env),
        ],
        232,
    );
    invoke_void(
        &fixture,
        "dispute",
        vec![
            &fixture.env,
            creator.clone().into_val(&fixture.env),
            id.into_val(&fixture.env),
            hash(&fixture.env, 82).into_val(&fixture.env),
        ],
    );
    assert_eq!(fixture.env.auths()[0].0, creator);
    assert_contract_error(
        &fixture,
        "dispute",
        vec![
            &fixture.env,
            creator.into_val(&fixture.env),
            id.into_val(&fixture.env),
            hash(&fixture.env, 83).into_val(&fixture.env),
        ],
        207,
    );
    invoke_void(
        &fixture,
        "resolve",
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            unit_variant(&fixture.env, "RefundDepositor"),
        ],
    );
    assert_eq!(fixture.env.auths()[0].0, fixture.manager);
    assert_eq!(token.balance(&source_settler), amount);
    assert_eq!(token.balance(&fixture.contract_id), 0);
}

#[test]
fn dispute_release_pays_the_authorized_claimant_once() {
    let fixture = fixture();
    let creator = Address::generate(&fixture.env);
    let source_settler = Address::generate(&fixture.env);
    let amount = 90_000_000i128;
    let token = token::Client::new(&fixture.env, &fixture.token_id);
    let id = funded_fiat_to_crypto(&fixture, &creator, &source_settler, amount, 90);

    invoke_void(
        &fixture,
        "dispute",
        vec![
            &fixture.env,
            source_settler.clone().into_val(&fixture.env),
            id.into_val(&fixture.env),
            hash(&fixture.env, 91).into_val(&fixture.env),
        ],
    );
    assert_eq!(fixture.env.auths()[0].0, source_settler);
    invoke_void(
        &fixture,
        "resolve",
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            unit_variant(&fixture.env, "ReleaseToClaimant"),
        ],
    );
    assert_eq!(fixture.env.auths()[0].0, fixture.manager);
    assert_eq!(token.balance(&creator), amount);
    assert_eq!(token.balance(&fixture.contract_id), 0);
    assert_contract_error(
        &fixture,
        "resolve",
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            unit_variant(&fixture.env, "ReleaseToClaimant"),
        ],
        207,
    );
}

#[test]
fn premature_and_wrong_party_actions_fail_without_moving_value() {
    let fixture = fixture();
    let creator = Address::generate(&fixture.env);
    let source_settler = Address::generate(&fixture.env);
    let stranger = Address::generate(&fixture.env);
    let amount = 40_000_000i128;
    let token = token::Client::new(&fixture.env, &fixture.token_id);
    let token_admin = token::StellarAssetClient::new(&fixture.env, &fixture.token_id);
    token_admin.mint(&source_settler, &amount);
    token_admin.mint(&stranger, &amount);
    let id = create_request(
        &fixture,
        &creator,
        "FiatToCrypto",
        Some(USD),
        None,
        amount,
        100,
        100,
    );

    assert_contract_error(
        &fixture,
        "claim_settlement_asset",
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            creator.clone().into_val(&fixture.env),
        ],
        207,
    );
    assert_contract_error(
        &fixture,
        "refund_settler",
        vec![&fixture.env, id.into_val(&fixture.env)],
        228,
    );
    set_quote(&fixture, id, Some(amount), amount);
    accept(&fixture, id, "Source", &source_settler);
    assert_contract_error(
        &fixture,
        "deposit_settlement_asset",
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            stranger.clone().into_val(&fixture.env),
        ],
        232,
    );
    assert_eq!(token.balance(&stranger), amount);
    assert_eq!(token.balance(&fixture.contract_id), 0);

    deposit(&fixture, id, &source_settler);
    assert_contract_error(
        &fixture,
        "cancel",
        vec![
            &fixture.env,
            id.into_val(&fixture.env),
            creator.into_val(&fixture.env),
        ],
        245,
    );
    assert_eq!(token.balance(&fixture.contract_id), amount);
}

#[test]
fn upgrade_requires_admin_authorization_and_accepts_an_installed_hash() {
    let unauthenticated = fixture_with_auth(false);
    let wasm_hash = unauthenticated.env.deployer().upload_contract_wasm(WASM);
    let result = unauthenticated.env.try_invoke_contract::<Val, Error>(
        &unauthenticated.contract_id,
        &Symbol::new(&unauthenticated.env, "upgrade"),
        vec![
            &unauthenticated.env,
            wasm_hash.into_val(&unauthenticated.env),
        ],
    );
    assert!(result.is_err());

    let fixture = fixture();
    let wasm_hash = fixture.env.deployer().upload_contract_wasm(WASM);
    invoke_void(
        &fixture,
        "upgrade",
        vec![&fixture.env, wasm_hash.into_val(&fixture.env)],
    );
    assert_eq!(fixture.env.auths()[0].0, fixture.admin);
}

#[test]
fn amount_conservation_holds_across_representative_positive_values() {
    for (index, amount) in [1i128, 10, 10_000_000, 123_456_789, 9_000_000_000]
        .into_iter()
        .enumerate()
    {
        let fixture = fixture();
        let creator = Address::generate(&fixture.env);
        let source_settler = Address::generate(&fixture.env);
        let token = token::Client::new(&fixture.env, &fixture.token_id);
        let id = funded_fiat_to_crypto(
            &fixture,
            &creator,
            &source_settler,
            amount,
            120u8.wrapping_add(index as u8),
        );
        confirm(&fixture, id, "SourceSettlement", 130);
        invoke_void(
            &fixture,
            "claim_settlement_asset",
            vec![
                &fixture.env,
                id.into_val(&fixture.env),
                creator.clone().into_val(&fixture.env),
            ],
        );

        assert_eq!(token.balance(&creator), amount);
        assert_eq!(token.balance(&source_settler), 0);
        assert_eq!(token.balance(&fixture.contract_id), 0);
    }
}
