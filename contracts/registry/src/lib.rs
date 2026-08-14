#![no_std]
use oynk_sdk::*;
use soroban_sdk::{contract, contractimpl, symbol_short, token, Address, BytesN, Env, Vec};

#[contract]
pub struct RegistryContract;

#[contractimpl]
impl RegistryContract {
    pub fn init(
        e: Env,
        admin: Address,
        treasury: Address,
        stake_token: Address,
        min_stake: i128,
    ) {
        if e.storage().instance().has(&RegistryKey::Admin) {
            panic!("already initialized")
        }
        admin.require_auth();
        e.storage().instance().set(&RegistryKey::Admin, &admin);
        e.storage()
            .instance()
            .set(&RegistryKey::Treasury, &treasury);
        e.storage()
            .instance()
            .set(&RegistryKey::StakeToken, &stake_token);
        e.storage()
            .instance()
            .set(&RegistryKey::MinStake, &min_stake);
    }

    pub fn register(
        e: Env,
        settler: Address,
        metadata_hash: BytesN<32>,
        capabilities: Vec<Capability>,
    ) {
        settler.require_auth();
        if e.storage()
            .persistent()
            .has(&RegistryKey::Settler(settler.clone()))
        {
            panic!("settler exists")
        }
        let profile = SettlerProfile {
            owner: settler.clone(),
            active: true,
            stake: 0,
            reputation: 500,
            completed: 0,
            failed: 0,
            capabilities,
            metadata_hash,
        };
        e.storage()
            .persistent()
            .set(&RegistryKey::Settler(settler.clone()), &profile);
        e.events()
            .publish((symbol_short!(reg), settler), symbol_short!(ok));
    }

    pub fn update_capabilities(e: Env, settler: Address, capabilities: Vec<Capability>) {
        settler.require_auth();
        let mut p = Self::get_settler(e.clone(), settler.clone());
        p.capabilities = capabilities;
        e.storage()
            .persistent()
            .set(&RegistryKey::Settler(settler.clone()), &p);
        e.events()
            .publish((symbol_short!(cap), settler), p.reputation);
    }

    pub fn deposit_stake(e: Env, settler: Address, amount: i128) {
        require_positive(amount);
        settler.require_auth();
        let token_id: Address = e
            .storage()
            .instance()
            .get(&RegistryKey::StakeToken)
            .unwrap();
        let treasury: Address = e.storage().instance().get(&RegistryKey::Treasury).unwrap();
        token::Client::new(&e, &token_id).transfer(&settler, &treasury, &amount);
        let mut p = Self::get_settler(e.clone(), settler.clone());
        p.stake += amount;
        e.storage()
            .persistent()
            .set(&RegistryKey::Settler(settler.clone()), &p);
        e.events().publish((symbol_short!(stake), settler), amount);
    }

    pub fn slash(e: Env, settler: Address, amount: i128, reason_hash: BytesN<32>) {
        let admin: Address = e.storage().instance().get(&RegistryKey::Admin).unwrap();
        admin.require_auth();
        require_positive(amount);
        let mut p = Self::get_settler(e.clone(), settler.clone());
        let slash = if amount > p.stake { p.stake } else { amount };
        p.stake -= slash;
        p.failed += 1;
        if p.reputation >= 50 {
            p.reputation -= 50
        } else {
            p.reputation = 0
        }
        e.storage()
            .persistent()
            .set(&RegistryKey::Settler(settler.clone()), &p);
        e.events()
            .publish((symbol_short!(slash), settler, reason_hash), slash);
    }

    pub fn record_success(e: Env, settler: Address) {
        let admin: Address = e.storage().instance().get(&RegistryKey::Admin).unwrap();
        admin.require_auth();
        let mut p = Self::get_settler(e.clone(), settler.clone());
        p.completed += 1;
        if p.reputation < 1000 {
            p.reputation += 5
        }
        e.storage()
            .persistent()
            .set(&RegistryKey::Settler(settler), &p);
    }

    pub fn set_active(e: Env, settler: Address, active: bool) {
        settler.require_auth();
        let mut p = Self::get_settler(e.clone(), settler.clone());
        p.active = active;
        e.storage()
            .persistent()
            .set(&RegistryKey::Settler(settler), &p);
    }

    pub fn is_eligible(e: Env, settler: Address) -> bool {
        let p = Self::get_settler(e.clone(), settler);
        let min_stake: i128 = e.storage().instance().get(&RegistryKey::MinStake).unwrap();
        p.active && p.stake >= min_stake
    }

    pub fn get_settler(e: Env, settler: Address) -> SettlerProfile {
        e.storage()
            .persistent()
            .get(&RegistryKey::Settler(settler))
            .unwrap()
    }
}
