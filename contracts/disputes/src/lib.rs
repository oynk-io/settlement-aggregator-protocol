#![no_std]
use oynk_sdk::*;
use soroban_sdk::{contract, contractimpl, symbol_short, Address, BytesN, Env};

#[contract]
pub struct DisputesContract;

#[contractimpl]
impl DisputesContract {
    pub fn init(e: Env, admin: Address, registry: Address, payments: Address, treasury: Address) {
        if e.storage().instance().has(&DisputeKey::Admin) {
            panic!("already initialized")
        }
        admin.require_auth();
        e.storage().instance().set(&DisputeKey::Admin, &admin);
        e.storage().instance().set(&DisputeKey::Registry, &registry);
        e.storage().instance().set(&DisputeKey::Payments, &payments);
        e.storage().instance().set(&DisputeKey::Treasury, &treasury);
        e.storage().instance().set(&DisputeKey::NextId, &1u64);
    }

    pub fn open(
        e: Env,
        payment_id: PaymentId,
        opened_by: Address,
        respondent: Address,
        evidence_hash: BytesN<32>,
        slash_amount: i128,
    ) -> DisputeId {
        opened_by.require_auth();
        if e.storage()
            .persistent()
            .has(&DisputeKey::ByPayment(payment_id))
        {
            panic!("exists")
        }
        let id: DisputeId = e.storage().instance().get(&DisputeKey::NextId).unwrap();
        e.storage().instance().set(&DisputeKey::NextId, &(id + 1));
        let d = Dispute {
            id,
            payment_id,
            opened_by: opened_by.clone(),
            respondent: respondent.clone(),
            evidence_hash: evidence_hash.clone(),
            status: DisputeStatus::Open,
            slash_amount,
            winner: None,
        };
        e.storage().persistent().set(&DisputeKey::Dispute(id), &d);
        e.storage()
            .persistent()
            .set(&DisputeKey::ByPayment(payment_id), &id);
        e.events()
            .publish((symbol_short!(dopen), id, payment_id), respondent);
        id
    }

    pub fn submit_evidence(e: Env, id: DisputeId, actor: Address, evidence_hash: BytesN<32>) {
        actor.require_auth();
        let mut d = Self::get(e.clone(), id);
        if actor != d.opened_by && actor != d.respondent {
            panic!("not party")
        }
        d.evidence_hash = evidence_hash.clone();
        d.status = DisputeStatus::EvidenceSubmitted;
        e.storage().persistent().set(&DisputeKey::Dispute(id), &d);
        e.events()
            .publish((symbol_short!(ev), id, actor), evidence_hash);
    }

    pub fn resolve(e: Env, id: DisputeId, winner: Address, slash: bool) {
        let admin: Address = e.storage().instance().get(&DisputeKey::Admin).unwrap();
        admin.require_auth();
        let mut d = Self::get(e.clone(), id);
        if d.status == DisputeStatus::Resolved {
            panic!("resolved")
        }
        d.status = DisputeStatus::Resolved;
        d.winner = Some(winner.clone());
        e.storage().persistent().set(&DisputeKey::Dispute(id), &d);
        e.events().publish((symbol_short!(dres), id, winner), slash);
    }

    pub fn get(e: Env, id: DisputeId) -> Dispute {
        e.storage()
            .persistent()
            .get(&DisputeKey::Dispute(id))
            .unwrap()
    }
}
