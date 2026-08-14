use soroban_sdk::{contractevent, Address, BytesN};

use crate::{DisputeResolution, SettlerAssignment, SettlerType};

#[contractevent(topics = ["Update", "AdminUpdate"])]
pub struct AdminUpdateEvent {
    pub admin: Address,
    pub new_admin: Address,
}
#[contractevent(topics = ["Update", "ManagerUpdate"])]
pub struct ManagerUpdateEvent {
    pub admin: Address,
    pub manager: Address,
}

#[contractevent(topics = ["Creation", "SettlementRequest"])]
pub struct SettlementCreationEvent {
    pub id: u64,
    pub sender_ref: BytesN<32>,
    pub recipient_ref: BytesN<32>,
    pub destination_amount: i128,
}

#[contractevent(topics = ["Quote", "SettlementQuote"])]
pub struct SettlementQuoteEvent {
    pub id: u64,
    pub source_amount: Option<i128>,
    pub settler_amount: i128,
    pub quote_hash: BytesN<32>,
}

#[contractevent(topics = ["Fiat", "SettlementFiat"])]
pub struct SettlementFiatEvent {
    pub id: u64,
    pub source_amount: i128,
    pub fiat_evidence_hash: BytesN<32>,
}
#[contractevent(topics = ["Assignment", "SettlerAssignment"])]
pub struct SettlerAssignmentEvent {
    pub id: u64,
    pub settler_type: SettlerType,
    pub assignment: SettlerAssignment,
}
#[contractevent(topics = ["Deposit", "SettlementAsset"])]
pub struct SettlementDepositEvent {
    pub id: u64,
    pub asset: Address,
    pub amount: i128,
    pub depositor: Address,
}
#[contractevent(topics = ["Settler", "SettlementAcceptance"])]
pub struct SettlementAcceptanceEvent {
    pub id: u64,
    pub settler_type: SettlerType,
    pub settler: Address,
}
#[contractevent(topics = ["Destination", "SettlementConfirmation"])]
pub struct SettlementConfirmationEvent {
    pub id: u64,
    pub proof_hash: BytesN<32>,
    pub settler: Address,
}
#[contractevent(topics = ["Claim", "SettlementClaimAsset"])]
pub struct SettlementClaimAssetEvent {
    pub id: u64,
    pub asset: Address,
    pub amount: i128,
    pub claimant: Address,
}
#[contractevent(topics = ["Refund", "SettlementRefundAsset"])]
pub struct SettlementRefundAssetEvent {
    pub id: u64,
    pub asset: Address,
    pub amount: i128,
    pub recipient: Address,
}
#[contractevent(topics = ["Dispute", "SettlementDispute"])]
pub struct SettlementDisputeEvent {
    pub id: u64,
    pub caller: Address,
    pub dispute_evidence_hash: BytesN<32>,
}
#[contractevent(topics = ["Dispute", "SettlementDispute"])]
pub struct SettlementDisputeResolutionEvent {
    pub id: u64,
    pub resolution: DisputeResolution,
    pub recipient: Address,
    pub amount: i128,
}
#[contractevent(topics = ["Cancel", "SettlementCancellation"])]
pub struct SettlementCancellationEvent {
    pub id: u64,
    pub caller: Address,
}
