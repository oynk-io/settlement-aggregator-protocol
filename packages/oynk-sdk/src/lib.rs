#![no_std]

pub mod errors;
pub mod events;

pub use errors::SettlementError;

use soroban_sdk::{contracttype, Address, BytesN};

pub type CurrencyCode = u32;

pub const USD: CurrencyCode = 840;
pub const GBP: CurrencyCode = 826;
pub const EUR: CurrencyCode = 978;
pub const CAD: CurrencyCode = 124;
pub const KWD: CurrencyCode = 414;
pub const NGN: CurrencyCode = 566;
pub const KES: CurrencyCode = 404;
pub const GHS: CurrencyCode = 936;
pub const ZAR: CurrencyCode = 710;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfirmationType {
    SourceSettlement,
    DestinationSettlement,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SettlerType {
    Source,
    Destination,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettlementRoute {
    pub origin: Option<u32>,
    pub destination: Option<u32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeResolution {
    ReleaseToClaimant,
    RefundDepositor,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SettlementType {
    FiatToCrypto,
    CryptoToFiat,
    FiatToFiat,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FundingStatus {
    PendingQuote,
    QuoteSet,
    FiatConfirmed,
    Ready,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RequestStatus {
    Created,
    SourceAccepted,
    SettlementFunded,
    SourceSettlementConfirmed,
    SettlementInProgress,
    ReadyForClaim,
    Disputed,
    Completed,
    Cancelled,
    Refunded,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettlerAssignment {
    pub settler: Address,
    pub fiat_amount: i128,
    pub settlement_asset_amount: i128,
    pub confirmed: bool,
    pub proof_hash: Option<BytesN<32>>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestSettlers {
    pub source: Option<SettlerAssignment>,
    pub destination: Option<SettlerAssignment>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettlementRequest {
    pub id: u64,
    pub creator: Address,

    pub sender_ref: BytesN<32>,
    pub recipient_ref: BytesN<32>,

    pub route: SettlementRoute,
    pub settlement_type: SettlementType,

    pub source_amount: Option<i128>,
    pub destination_amount: i128,
    pub settler_amount: Option<i128>,
    pub funding_status: FundingStatus,
    pub status: RequestStatus,

    pub created_ledger: u32,
    pub deadline_ledger: u32,

    pub settlers: RequestSettlers,

    pub quote_evidence_hash: Option<BytesN<32>>,
    pub fiat_evidence_hash: Option<BytesN<32>>,
    pub settlement_evidence_hash: Option<BytesN<32>>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Manager,
    SettlementAsset,
    NextId,
    Request(u64),
}

pub fn require_positive(amount: i128) -> Result<(), SettlementError> {
    if amount <= 0 {
        return Err(SettlementError::InvalidAmount);
    }

    Ok(())
}

pub fn is_supported_currency(code: CurrencyCode) -> bool {
    matches!(code, USD | GBP | EUR | CAD | KWD | NGN | KES | GHS | ZAR)
}

pub fn require_supported_currency(code: CurrencyCode) -> Result<(), SettlementError> {
    if !is_supported_currency(code) {
        return Err(SettlementError::UnsupportedCurrency);
    }

    Ok(())
}

pub fn require_valid_route(
    settlement_type: &SettlementType,
    route: &SettlementRoute,
) -> Result<(), SettlementError> {
    match settlement_type {
        SettlementType::FiatToCrypto => {
            let origin = route.origin.ok_or(SettlementError::InvalidRoute)?;

            require_supported_currency(origin)?;

            if route.destination.is_some() {
                return Err(SettlementError::InvalidRoute);
            }
        }

        SettlementType::CryptoToFiat => {
            if route.origin.is_some() {
                return Err(SettlementError::InvalidRoute);
            }

            let destination = route.destination.ok_or(SettlementError::InvalidRoute)?;

            require_supported_currency(destination)?;
        }

        SettlementType::FiatToFiat => {
            let origin = route.origin.ok_or(SettlementError::InvalidRoute)?;

            let destination = route.destination.ok_or(SettlementError::InvalidRoute)?;

            require_supported_currency(origin)?;
            require_supported_currency(destination)?;

            if origin == destination {
                return Err(SettlementError::InvalidRoute);
            }
        }
    }

    Ok(())
}
