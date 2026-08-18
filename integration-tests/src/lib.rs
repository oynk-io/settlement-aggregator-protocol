#![cfg(test)]

use soroban_sdk::{
    testutils::Address as _, token, vec, Address, BytesN, Env, IntoVal, Map, Symbol, Val,
};

const WASM: &[u8] =
    include_bytes!("../../target/wasm32v1-none/release/oynk_settlement_protocol_contract.wasm");
const USD: u32 = 840;
const EUR: u32 = 978;

struct Fixture {
    env: Env,
    contract_id: Address,
    token_id: Address,
}

fn fixture() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token_id = token_contract.address();
    let contract_id = env.register(WASM, (admin, manager.clone(), token_id.clone()));

    Fixture {
        env,
        contract_id,
        token_id,
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
            source_settler.into_val(&fixture.env),
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
