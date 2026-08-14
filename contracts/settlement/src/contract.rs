use oynk_sdk::{
    errors::SettlementError, events, require_valid_route, ConfirmationType, DataKey,
    DisputeResolution, FundingStatus, RequestSettlers, RequestStatus, SettlementRequest,
    SettlementRoute, SettlementType, SettlerAssignment, SettlerType,
};

use soroban_sdk::{contract, contractimpl, token, Address, BytesN, Env};

use crate::{
    states::{
        create_request, has_admin, read_admin, read_manager, read_next_id, read_request,
        read_settlement_asset, write_admin, write_manager, write_next_id, write_settlement_asset,
    },
    traits::OynkSettlementProtocolTrait,
    utils::{ensure_active_for_dispute, ensure_not_expired, ensure_positive, ensure_status},
};

#[contract]
pub struct OynkSettlementProtocol;

#[contractimpl]
impl OynkSettlementProtocolTrait for OynkSettlementProtocol {
    fn __constructor(
        e: Env,
        admin: Address,
        manager: Address,
        settlement_asset: Address,
    ) -> Result<(), SettlementError> {
        if has_admin(&e) {
            return Err(SettlementError::AlreadyInitialized);
        }
        write_admin(&e, &admin);
        write_manager(&e, &manager);
        write_settlement_asset(&e, &settlement_asset);
        write_next_id(&e, &1u64);

        Ok(())
    }

    fn upgrade(e: Env, wasm_hash: BytesN<32>) {
        let admin = read_admin(&e);
        admin.require_auth();
        e.deployer().update_current_contract_wasm(wasm_hash);
    }

    fn update_admin(e: Env, new_admin: Address) -> Result<(), SettlementError> {
        let admin = read_admin(&e);
        admin.require_auth();

        write_admin(&e, &new_admin);

        events::AdminUpdateEvent { admin, new_admin }.publish(&e);

        Ok(())
    }

    fn update_manager(e: Env, new_manager: Address) -> Result<(), SettlementError> {
        let admin = read_admin(&e);
        admin.require_auth();

        write_manager(&e, &new_manager);

        events::ManagerUpdateEvent {
            admin,
            manager: new_manager,
        }
        .publish(&e);

        Ok(())
    }

    fn confirm_settlement(
        e: Env,
        id: u64,
        confirmation_type: ConfirmationType,
        evidence_hash: BytesN<32>,
    ) -> Result<(), SettlementError> {
        let manager = read_manager(&e);
        manager.require_auth();

        let mut request = read_request(&e, id).ok_or(SettlementError::RequestNotFound)?;

        ensure_not_expired(&e, &request)?;

        match confirmation_type {
            ConfirmationType::SourceSettlement => {
                if request.settlement_type == SettlementType::CryptoToFiat {
                    return Err(SettlementError::FiatFundingNotRequired);
                }

                ensure_status(&request, RequestStatus::SettlementFunded)?;

                if request.funding_status != FundingStatus::Ready {
                    return Err(SettlementError::SettlementAssetNotDeposited);
                }

                let source_amount = request
                    .source_amount
                    .ok_or(SettlementError::SourceAmountMissing)?;

                let source = request
                    .settlers
                    .source
                    .clone()
                    .ok_or(SettlementError::SourceSettlerMissing)?;

                if !source.confirmed {
                    return Err(SettlementError::SettlementAssetNotDeposited);
                }

                if request.fiat_evidence_hash.is_some() {
                    return Err(SettlementError::AlreadyProcessed);
                }

                request.fiat_evidence_hash = Some(evidence_hash.clone());

                request.status = match request.settlement_type {
                    SettlementType::FiatToCrypto => RequestStatus::ReadyForClaim,
                    SettlementType::FiatToFiat => RequestStatus::SourceSettlementConfirmed,
                    SettlementType::CryptoToFiat => {
                        return Err(SettlementError::FiatFundingNotRequired);
                    }
                };

                e.storage()
                    .persistent()
                    .set(&DataKey::Request(id), &request);

                events::SettlementFiatEvent {
                    id,
                    source_amount,
                    fiat_evidence_hash: evidence_hash,
                }
                .publish(&e);
            }

            ConfirmationType::DestinationSettlement => {
                if request.settlement_type == SettlementType::FiatToCrypto {
                    return Err(SettlementError::DestinationSettlerNotNeeded);
                }

                ensure_status(&request, RequestStatus::SettlementInProgress)?;

                if request.funding_status != FundingStatus::Ready {
                    return Err(SettlementError::SettlementAssetNotDeposited);
                }

                let mut assignment = request
                    .settlers
                    .destination
                    .clone()
                    .ok_or(SettlementError::DestinationSettlerMissing)?;

                if assignment.confirmed {
                    return Err(SettlementError::AlreadyProcessed);
                }

                let settler = assignment.settler.clone();

                assignment.confirmed = true;
                assignment.proof_hash = Some(evidence_hash.clone());

                let mut settlers = request.settlers.clone();
                settlers.destination = Some(assignment);

                request.settlers = settlers;
                request.status = RequestStatus::ReadyForClaim;
                request.settlement_evidence_hash = Some(evidence_hash.clone());

                e.storage()
                    .persistent()
                    .set(&DataKey::Request(id), &request);

                events::SettlementConfirmationEvent {
                    id,
                    proof_hash: evidence_hash,
                    settler,
                }
                .publish(&e);
            }
        }

        Ok(())
    }

    fn create_settlement_request(
        e: Env,
        creator: Address,
        sender_ref: BytesN<32>,
        recipient_ref: BytesN<32>,
        route: SettlementRoute,
        settlement_type: SettlementType,
        destination_amount: i128,
        deadline_ledger: u32,
    ) -> Result<u64, SettlementError> {
        creator.require_auth();

        ensure_positive(destination_amount)?;

        if deadline_ledger <= e.ledger().sequence() {
            return Err(SettlementError::BadDeadline);
        }

        /*
         * Keep calling your SDK route validator here if it continues to panic.
         * For a fully Result-based implementation, convert that validator to:
         *
         * fn validate_route(...) -> Result<(), SettlementError>
         */
        require_valid_route(&settlement_type, &route)?;

        let id = read_next_id(&e);

        if id == u64::MAX {
            return Err(SettlementError::IdOverflow);
        }

        write_next_id(&e, &(id + 1));

        let request = SettlementRequest {
            id,
            creator: creator.clone(),

            sender_ref: sender_ref.clone(),
            recipient_ref: recipient_ref.clone(),

            route,
            settlement_type,

            source_amount: None,
            destination_amount,
            settler_amount: None,

            funding_status: FundingStatus::PendingQuote,
            status: RequestStatus::Created,

            created_ledger: e.ledger().sequence(),
            deadline_ledger,

            settlers: RequestSettlers {
                source: None,
                destination: None,
            },

            quote_evidence_hash: None,
            fiat_evidence_hash: None,
            settlement_evidence_hash: None,
        };

        create_request(&e, id, &request);

        events::SettlementCreationEvent {
            id,
            sender_ref,
            recipient_ref,
            destination_amount,
        }
        .publish(&e);

        Ok(id)
    }

    fn set_settlement_quote(
        e: Env,
        id: u64,
        source_amount: Option<i128>,
        settler_amount: i128,
        quote_hash: BytesN<32>,
    ) -> Result<(), SettlementError> {
        let manager = read_manager(&e);
        manager.require_auth();

        ensure_positive(settler_amount)?;

        let mut request = read_request(&e, id).ok_or(SettlementError::RequestNotFound)?;

        ensure_status(&request, RequestStatus::Created)?;
        ensure_not_expired(&e, &request)?;

        if request.funding_status != FundingStatus::PendingQuote {
            return Err(SettlementError::BadFundingStatus);
        }

        match request.settlement_type {
            SettlementType::CryptoToFiat => {
                if source_amount.is_some() {
                    return Err(SettlementError::InvalidAmount);
                }
            }

            SettlementType::FiatToCrypto | SettlementType::FiatToFiat => {
                let amount = source_amount.ok_or(SettlementError::SourceAmountMissing)?;

                ensure_positive(amount)?;
            }
        }

        request.source_amount = source_amount;
        request.settler_amount = Some(settler_amount);
        request.quote_evidence_hash = Some(quote_hash.clone());
        request.funding_status = FundingStatus::QuoteSet;

        e.storage()
            .persistent()
            .set(&DataKey::Request(id), &request);

        events::SettlementQuoteEvent {
            id,
            source_amount,
            settler_amount,
            quote_hash,
        }
        .publish(&e);

        Ok(())
    }

    fn accept_settlement(
        e: Env,
        id: u64,
        settler_type: SettlerType,
        settler: Address,
    ) -> Result<(), SettlementError> {
        settler.require_auth();

        let mut request = read_request(&e, id).ok_or(SettlementError::RequestNotFound)?;

        ensure_not_expired(&e, &request)?;

        let mut settlers = request.settlers.clone();

        match settler_type.clone() {
            SettlerType::Source => {
                if request.settlement_type == SettlementType::CryptoToFiat {
                    return Err(SettlementError::SourceSettlerNotNeeded);
                }

                ensure_status(&request, RequestStatus::Created)?;

                if request.funding_status != FundingStatus::QuoteSet {
                    return Err(SettlementError::QuoteNotSet);
                }

                if settlers.source.is_some() {
                    return Err(SettlementError::AlreadyProcessed);
                }

                let source_amount = request
                    .source_amount
                    .ok_or(SettlementError::SourceAmountMissing)?;

                let settler_amount = request
                    .settler_amount
                    .ok_or(SettlementError::SettlerAmountMissing)?;

                ensure_positive(source_amount)?;
                ensure_positive(settler_amount)?;

                settlers.source = Some(SettlerAssignment {
                    settler: settler.clone(),
                    fiat_amount: source_amount,
                    settlement_asset_amount: settler_amount,
                    confirmed: false,
                    proof_hash: None,
                });

                request.status = RequestStatus::SourceAccepted;
            }

            SettlerType::Destination => {
                if request.settlement_type == SettlementType::FiatToCrypto {
                    return Err(SettlementError::DestinationSettlerNotNeeded);
                }

                match request.settlement_type {
                    SettlementType::CryptoToFiat => {
                        ensure_status(&request, RequestStatus::SettlementFunded)?;
                    }

                    SettlementType::FiatToFiat => {
                        ensure_status(&request, RequestStatus::SourceSettlementConfirmed)?;
                    }

                    SettlementType::FiatToCrypto => {
                        return Err(SettlementError::DestinationSettlerNotNeeded);
                    }
                }

                if request.funding_status != FundingStatus::Ready {
                    return Err(SettlementError::SettlementAssetNotDeposited);
                }

                if settlers.destination.is_some() {
                    return Err(SettlementError::AlreadyProcessed);
                }

                let settler_amount = request
                    .settler_amount
                    .ok_or(SettlementError::SettlerAmountMissing)?;

                ensure_positive(request.destination_amount)?;
                ensure_positive(settler_amount)?;

                settlers.destination = Some(SettlerAssignment {
                    settler: settler.clone(),
                    fiat_amount: request.destination_amount,
                    settlement_asset_amount: settler_amount,
                    confirmed: false,
                    proof_hash: None,
                });

                request.status = RequestStatus::SettlementInProgress;
            }
        }

        request.settlers = settlers;

        e.storage()
            .persistent()
            .set(&DataKey::Request(id), &request);

        events::SettlementAcceptanceEvent {
            id,
            settler_type,
            settler,
        }
        .publish(&e);

        Ok(())
    }

    fn deposit_settlement_asset(
        e: Env,
        id: u64,
        depositor: Address,
    ) -> Result<(), SettlementError> {
        depositor.require_auth();

        let mut request = read_request(&e, id).ok_or(SettlementError::RequestNotFound)?;

        ensure_not_expired(&e, &request)?;

        let settler_amount = request
            .settler_amount
            .ok_or(SettlementError::SettlerAmountMissing)?;

        ensure_positive(settler_amount)?;

        match request.settlement_type.clone() {
            SettlementType::CryptoToFiat => {
                ensure_status(&request, RequestStatus::Created)?;

                if request.creator != depositor {
                    return Err(SettlementError::NotAuthorized);
                }

                if request.funding_status != FundingStatus::QuoteSet {
                    return Err(SettlementError::QuoteNotSet);
                }
            }

            SettlementType::FiatToCrypto | SettlementType::FiatToFiat => {
                ensure_status(&request, RequestStatus::SourceAccepted)?;

                if request.funding_status != FundingStatus::QuoteSet {
                    return Err(SettlementError::QuoteNotSet);
                }

                let mut source = request
                    .settlers
                    .source
                    .clone()
                    .ok_or(SettlementError::SourceSettlerMissing)?;

                if source.settler != depositor {
                    return Err(SettlementError::NotAuthorized);
                }

                if source.confirmed {
                    return Err(SettlementError::AlreadyProcessed);
                }

                if source.settlement_asset_amount != settler_amount {
                    return Err(SettlementError::ConditionFailed);
                }

                source.confirmed = true;

                let mut settlers = request.settlers.clone();
                settlers.source = Some(source);
                request.settlers = settlers;
            }
        }

        let asset = read_settlement_asset(&e);
        let contract_address = e.current_contract_address();
        let token_client = token::Client::new(&e, &asset);

        token_client.transfer(&depositor, &contract_address, &settler_amount);

        request.funding_status = FundingStatus::Ready;
        request.status = RequestStatus::SettlementFunded;

        e.storage()
            .persistent()
            .set(&DataKey::Request(id), &request);

        events::SettlementDepositEvent {
            id,
            asset,
            amount: settler_amount,
            depositor,
        }
        .publish(&e);

        Ok(())
    }

    fn claim_settlement_asset(e: Env, id: u64, claimant: Address) -> Result<(), SettlementError> {
        claimant.require_auth();

        let mut request = read_request(&e, id).ok_or(SettlementError::RequestNotFound)?;

        ensure_status(&request, RequestStatus::ReadyForClaim)?;

        let deposited_amount = request
            .settler_amount
            .ok_or(SettlementError::SettlerAmountMissing)?;

        ensure_positive(deposited_amount)?;

        let claim_amount = match request.settlement_type.clone() {
            SettlementType::FiatToCrypto => {
                if request.creator != claimant {
                    return Err(SettlementError::NotAuthorized);
                }

                if request.fiat_evidence_hash.is_none() {
                    return Err(SettlementError::ConditionFailed);
                }

                deposited_amount
            }

            SettlementType::CryptoToFiat | SettlementType::FiatToFiat => {
                let assignment = request
                    .settlers
                    .destination
                    .clone()
                    .ok_or(SettlementError::DestinationSettlerMissing)?;

                if assignment.settler != claimant {
                    return Err(SettlementError::NotAuthorized);
                }

                if !assignment.confirmed {
                    return Err(SettlementError::ConditionFailed);
                }

                ensure_positive(assignment.settlement_asset_amount)?;

                if assignment.settlement_asset_amount > deposited_amount {
                    return Err(SettlementError::PayoutExceedsDeposit);
                }

                assignment.settlement_asset_amount
            }
        };

        let asset = read_settlement_asset(&e);
        let contract_address = e.current_contract_address();
        let token_client = token::Client::new(&e, &asset);

        token_client.transfer(&contract_address, &claimant, &claim_amount);

        request.status = RequestStatus::Completed;

        e.storage()
            .persistent()
            .set(&DataKey::Request(id), &request);

        events::SettlementClaimAssetEvent {
            id,
            asset,
            amount: claim_amount,
            claimant,
        }
        .publish(&e);

        Ok(())
    }

    fn refund_settler(e: Env, id: u64) -> Result<(), SettlementError> {
        let manager = read_manager(&e);
        manager.require_auth();

        let mut request = read_request(&e, id).ok_or(SettlementError::RequestNotFound)?;

        match request.status {
            RequestStatus::Completed => {
                return Err(SettlementError::InvalidRequestStatus);
            }

            RequestStatus::Refunded => {
                return Err(SettlementError::InvalidRequestStatus);
            }

            RequestStatus::Cancelled => {
                return Err(SettlementError::RequestCancelled);
            }

            RequestStatus::Disputed => {
                return Err(SettlementError::RequestDisputed);
            }

            RequestStatus::ReadyForClaim => {
                return Err(SettlementError::ReadyForClaim);
            }

            _ => {}
        }

        if request.funding_status != FundingStatus::Ready {
            return Err(SettlementError::SettlementAssetNotDeposited);
        }

        let refund_amount = request
            .settler_amount
            .ok_or(SettlementError::SettlerAmountMissing)?;

        ensure_positive(refund_amount)?;

        let recipient = match request.settlement_type.clone() {
            SettlementType::CryptoToFiat => request.creator.clone(),

            SettlementType::FiatToCrypto | SettlementType::FiatToFiat => {
                let source = request
                    .settlers
                    .source
                    .clone()
                    .ok_or(SettlementError::SourceSettlerMissing)?;

                if !source.confirmed {
                    return Err(SettlementError::ConditionFailed);
                }

                source.settler
            }
        };

        let asset = read_settlement_asset(&e);
        let contract_address = e.current_contract_address();
        let token_client = token::Client::new(&e, &asset);

        token_client.transfer(&contract_address, &recipient, &refund_amount);

        request.status = RequestStatus::Refunded;

        e.storage()
            .persistent()
            .set(&DataKey::Request(id), &request);

        events::SettlementRefundAssetEvent {
            id,
            asset,
            amount: refund_amount,
            recipient,
        }
        .publish(&e);

        Ok(())
    }

    fn dispute(
        e: Env,
        caller: Address,
        id: u64,
        dispute_evidence_hash: BytesN<32>,
    ) -> Result<(), SettlementError> {
        caller.require_auth();

        let mut request = read_request(&e, id).ok_or(SettlementError::RequestNotFound)?;
        let manager = read_manager(&e);

        let is_creator = caller == request.creator;

        let is_source_settler = request
            .settlers
            .source
            .as_ref()
            .map(|assignment| assignment.settler == caller)
            .unwrap_or(false);

        let is_destination_settler = request
            .settlers
            .destination
            .as_ref()
            .map(|assignment| assignment.settler == caller)
            .unwrap_or(false);

        let is_manager = caller == manager;

        if !is_creator && !is_source_settler && !is_destination_settler && !is_manager {
            return Err(SettlementError::NotAuthorized);
        }

        ensure_active_for_dispute(&request)?;

        if request.funding_status != FundingStatus::Ready {
            return Err(SettlementError::NothingToDispute);
        }

        request.status = RequestStatus::Disputed;
        request.settlement_evidence_hash = Some(dispute_evidence_hash.clone());

        e.storage()
            .persistent()
            .set(&DataKey::Request(id), &request);

        events::SettlementDisputeEvent {
            id,
            caller,
            dispute_evidence_hash,
        }
        .publish(&e);

        Ok(())
    }

    fn resolve(e: Env, id: u64, resolution: DisputeResolution) -> Result<(), SettlementError> {
        let manager = read_manager(&e);
        manager.require_auth();

        let mut request = read_request(&e, id).ok_or(SettlementError::RequestNotFound)?;

        ensure_status(&request, RequestStatus::Disputed)?;

        if request.funding_status != FundingStatus::Ready {
            return Err(SettlementError::SettlementAssetNotDeposited);
        }

        let deposited_amount = request
            .settler_amount
            .ok_or(SettlementError::SettlerAmountMissing)?;

        ensure_positive(deposited_amount)?;

        let (recipient, transfer_amount) = match resolution.clone() {
            DisputeResolution::ReleaseToClaimant => match request.settlement_type.clone() {
                SettlementType::FiatToCrypto => (request.creator.clone(), deposited_amount),

                SettlementType::CryptoToFiat | SettlementType::FiatToFiat => {
                    let destination = request
                        .settlers
                        .destination
                        .clone()
                        .ok_or(SettlementError::DestinationSettlerMissing)?;

                    ensure_positive(destination.settlement_asset_amount)?;

                    if destination.settlement_asset_amount > deposited_amount {
                        return Err(SettlementError::PayoutExceedsDeposit);
                    }

                    (destination.settler, destination.settlement_asset_amount)
                }
            },

            DisputeResolution::RefundDepositor => match request.settlement_type.clone() {
                SettlementType::CryptoToFiat => (request.creator.clone(), deposited_amount),

                SettlementType::FiatToCrypto | SettlementType::FiatToFiat => {
                    let source = request
                        .settlers
                        .source
                        .clone()
                        .ok_or(SettlementError::SourceSettlerMissing)?;

                    if !source.confirmed {
                        return Err(SettlementError::ConditionFailed);
                    }

                    (source.settler, deposited_amount)
                }
            },
        };

        let asset = read_settlement_asset(&e);
        let contract_address = e.current_contract_address();
        let token_client = token::Client::new(&e, &asset);

        token_client.transfer(&contract_address, &recipient, &transfer_amount);

        request.status = match resolution.clone() {
            DisputeResolution::ReleaseToClaimant => RequestStatus::Completed,

            DisputeResolution::RefundDepositor => RequestStatus::Refunded,
        };

        e.storage()
            .persistent()
            .set(&DataKey::Request(id), &request);

        events::SettlementDisputeResolutionEvent {
            id,
            resolution,
            recipient,
            amount: transfer_amount,
        }
        .publish(&e);

        Ok(())
    }

    fn cancel(e: Env, id: u64, caller: Address) -> Result<(), SettlementError> {
        caller.require_auth();

        let mut request = read_request(&e, id).ok_or(SettlementError::RequestNotFound)?;
        let manager = read_manager(&e);

        if caller != request.creator && caller != manager {
            return Err(SettlementError::NotAuthorized);
        }

        match request.status {
            RequestStatus::Completed => {
                return Err(SettlementError::InvalidRequestStatus);
            }

            RequestStatus::Refunded => {
                return Err(SettlementError::InvalidRequestStatus);
            }

            RequestStatus::Cancelled => {
                return Err(SettlementError::InvalidRequestStatus);
            }

            RequestStatus::Disputed => {
                return Err(SettlementError::RequestDisputed);
            }

            RequestStatus::ReadyForClaim => {
                return Err(SettlementError::ReadyForClaim);
            }

            _ => {}
        }

        if request.funding_status == FundingStatus::Ready {
            return Err(SettlementError::AlreadyProcessed);
        }

        request.status = RequestStatus::Cancelled;

        e.storage()
            .persistent()
            .set(&DataKey::Request(id), &request);

        events::SettlementCancellationEvent { id, caller }.publish(&e);

        Ok(())
    }

    fn get_request(e: Env, id: u64) -> Option<SettlementRequest> {
        read_request(&e, id)
    }

    fn get_settlement_asset(e: Env) -> Address {
        read_settlement_asset(&e)
    }

    fn get_admin(e: Env) -> Address {
        read_admin(&e)
    }

    fn get_manager(e: Env) -> Address {
        read_manager(&e)
    }
}
