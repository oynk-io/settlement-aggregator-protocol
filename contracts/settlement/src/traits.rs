use oynk_sdk::{
    errors::SettlementError, ConfirmationType, DisputeResolution, SettlementRequest,
    SettlementRoute, SettlementType, SettlerType,
};
use soroban_sdk::{Address, BytesN, Env};

pub trait OynkSettlementProtocolTrait {
    fn __constructor(
        e: Env,
        admin: Address,
        manager: Address,
        settlement_asset: Address,
    ) -> Result<(), SettlementError>;

    fn upgrade(e: Env, wasm_hash: BytesN<32>);

    fn update_admin(e: Env, new_admin: Address) -> Result<(), SettlementError>;

    fn update_manager(e: Env, new_manager: Address) -> Result<(), SettlementError>;

    fn confirm_settlement(
        e: Env,
        id: u64,
        confirmation_type: ConfirmationType,
        evidence_hash: BytesN<32>,
    ) -> Result<(), SettlementError>;

    fn create_settlement_request(
        e: Env,
        creator: Address,
        sender_ref: BytesN<32>,
        recipient_ref: BytesN<32>,
        route: SettlementRoute,
        settlement_type: SettlementType,
        destination_amount: i128,
        deadline_ledger: u32,
    ) -> Result<u64, SettlementError>;

    fn set_settlement_quote(
        e: Env,
        id: u64,
        source_amount: Option<i128>,
        settler_amount: i128,
        quote_hash: BytesN<32>,
    ) -> Result<(), SettlementError>;

    fn accept_settlement(
        e: Env,
        id: u64,
        settler_type: SettlerType,
        settler: Address,
    ) -> Result<(), SettlementError>;

    fn deposit_settlement_asset(e: Env, id: u64, depositor: Address)
        -> Result<(), SettlementError>;

    fn claim_settlement_asset(e: Env, id: u64, claimant: Address) -> Result<(), SettlementError>;

    fn refund_settler(e: Env, id: u64) -> Result<(), SettlementError>;

    fn dispute(
        e: Env,
        caller: Address,
        id: u64,
        dispute_evidence_hash: BytesN<32>,
    ) -> Result<(), SettlementError>;

    fn resolve(e: Env, id: u64, resolution: DisputeResolution) -> Result<(), SettlementError>;

    fn cancel(e: Env, id: u64, caller: Address) -> Result<(), SettlementError>;

    fn get_request(e: Env, id: u64) -> Option<SettlementRequest>;

    fn get_settlement_asset(e: Env) -> Address;

    fn get_admin(e: Env) -> Address;

    fn get_manager(e: Env) -> Address;
}
