use oynk_sdk::{errors::SettlementError, RequestStatus, SettlementRequest};
use soroban_sdk::Env;

pub fn read_is_expired(e: &Env, r: &SettlementRequest) -> bool {
    e.ledger().sequence() > r.deadline_ledger
}

pub fn ensure_positive(amount: i128) -> Result<(), SettlementError> {
    if amount <= 0 {
        return Err(SettlementError::InvalidAmount);
    }

    Ok(())
}

pub fn ensure_status(
    request: &SettlementRequest,
    expected: RequestStatus,
) -> Result<(), SettlementError> {
    if request.status != expected {
        return Err(SettlementError::InvalidRequestStatus);
    }

    Ok(())
}

pub fn ensure_not_expired(e: &Env, request: &SettlementRequest) -> Result<(), SettlementError> {
    if read_is_expired(e, request) {
        return Err(SettlementError::SettlementExpired);
    }

    Ok(())
}

pub fn ensure_active_for_dispute(request: &SettlementRequest) -> Result<(), SettlementError> {
    match request.status {
        RequestStatus::Completed => Err(SettlementError::InvalidRequestStatus),

        RequestStatus::Refunded => Err(SettlementError::InvalidRequestStatus),

        RequestStatus::Cancelled => Err(SettlementError::InvalidRequestStatus),

        RequestStatus::Disputed => Err(SettlementError::InvalidRequestStatus),

        _ => Ok(()),
    }
}
