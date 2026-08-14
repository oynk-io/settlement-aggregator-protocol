#![no_std]
use oynk_sdk::*;
use soroban_sdk::{contract, contractimpl, symbol_short, token, Address, Env};

#[contract]
pub struct TreasuryContract;

#[contractimpl]
impl TreasuryContract {
    pub fn init(e: Env, admin: Address, payments: Address, disputes: Address, fee_bps: u32) {
        if e.storage().instance().has(&TreasuryKey::Admin) {
            panic!("already initialized")
        }
        admin.require_auth();
        if fee_bps > 1000 {
            panic!("fee too high")
        }
        e.storage().instance().set(&TreasuryKey::Admin, &admin);
        e.storage()
            .instance()
            .set(&TreasuryKey::Payments, &payments);
        e.storage()
            .instance()
            .set(&TreasuryKey::Disputes, &disputes);
        e.storage().instance().set(&TreasuryKey::FeeBps, &fee_bps);
    }

    pub fn deposit(e: Env, token_id: Address, from: Address, amount: i128) {
        require_positive(amount);
        from.require_auth();
        token::Client::new(&e, &token_id).transfer(&from, &e.current_contract_address(), &amount);
        let key = TreasuryKey::Balance(token_id.clone());
        let bal: i128 = e.storage().persistent().get(&key).unwrap_or(0);
        e.storage().persistent().set(&key, &(bal + amount));
        e.events()
            .publish((symbol_short!(deposit), token_id), amount);
    }

    pub fn lock_for_payment(
        e: Env,
        caller: Address,
        payment_id: PaymentId,
        token_id: Address,
        amount: i128,
    ) {
        Self::require_payments(e.clone(), caller);
        require_positive(amount);
        let key = TreasuryKey::Balance(token_id.clone());
        let bal: i128 = e.storage().persistent().get(&key).unwrap_or(0);
        if bal < amount {
            panic!("insufficient treasury balance")
        }
        e.storage().persistent().set(&key, &(bal - amount));
        e.storage().persistent().set(
            &TreasuryKey::Locked(payment_id),
            &(token_id.clone(), amount),
        );
        e.events()
            .publish((symbol_short!(lock), payment_id, token_id), amount);
    }

    pub fn release(e: Env, caller: Address, payment_id: PaymentId, to: Address) {
        Self::require_payments_or_disputes(e.clone(), caller);
        let (token_id, amount): (Address, Amount) = e
            .storage()
            .persistent()
            .get(&TreasuryKey::Locked(payment_id))
            .unwrap();
        token::Client::new(&e, &token_id).transfer(&e.current_contract_address(), &to, &amount);
        e.storage()
            .persistent()
            .remove(&TreasuryKey::Locked(payment_id));
        e.events()
            .publish((symbol_short!(release), payment_id, to), amount);
    }

    pub fn withdraw_admin(e: Env, token_id: Address, to: Address, amount: i128) {
        let admin: Address = e.storage().instance().get(&TreasuryKey::Admin).unwrap();
        admin.require_auth();
        require_positive(amount);
        token::Client::new(&e, &token_id).transfer(&e.current_contract_address(), &to, &amount);
        e.events()
            .publish((symbol_short!(wd), token_id, to), amount);
    }

    fn require_payments(e: Env, caller: Address) {
        let payments: Address = e.storage().instance().get(&TreasuryKey::Payments).unwrap();
        if caller != payments {
            panic!("not payments")
        }
    }
    fn require_payments_or_disputes(e: Env, caller: Address) {
        let payments: Address = e.storage().instance().get(&TreasuryKey::Payments).unwrap();
        let disputes: Address = e.storage().instance().get(&TreasuryKey::Disputes).unwrap();
        if caller != payments && caller != disputes {
            panic!("not authorized")
        }
    }
}
