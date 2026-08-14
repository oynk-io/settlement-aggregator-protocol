use oynk_sdk::{DataKey, SettlementRequest};
use soroban_sdk::{Address, Env};

pub fn write_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin);
}

pub fn read_admin(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Admin).unwrap()
}
pub fn has_admin(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::Admin)
}

pub fn write_manager(e: &Env, manager: &Address) {
    e.storage().instance().set(&DataKey::Manager, manager);
}

pub fn read_manager(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Manager).unwrap()
}
pub fn write_settlement_asset(e: &Env, asset: &Address) {
    e.storage().instance().set(&DataKey::SettlementAsset, asset);
}

pub fn read_settlement_asset(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&DataKey::SettlementAsset)
        .unwrap()
}
pub fn write_next_id(e: &Env, id: &u64) {
    e.storage().instance().set(&DataKey::NextId, id);
}

pub fn read_next_id(e: &Env) -> u64 {
    e.storage().instance().get(&DataKey::NextId).unwrap()
}

pub fn create_request(e: &Env, id: u64, request: &SettlementRequest) {
    e.storage().persistent().set(&DataKey::Request(id), request);
}

pub fn read_request(e: &Env, id: u64) -> Option<SettlementRequest> {
    e.storage().persistent().get(&DataKey::Request(id))
}
