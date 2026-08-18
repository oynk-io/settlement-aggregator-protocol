use super::{
    is_supported_currency, require_positive, require_valid_route, SettlementError, SettlementRoute,
    SettlementType, EUR, GBP, USD,
};

#[test]
fn supported_currency_codes_are_accepted() {
    assert!(is_supported_currency(USD));
    assert!(is_supported_currency(GBP));
    assert!(is_supported_currency(EUR));
    assert!(!is_supported_currency(0));
}

#[test]
fn settlement_amounts_must_be_positive() {
    assert_eq!(require_positive(1), Ok(()));
    assert_eq!(require_positive(0), Err(SettlementError::InvalidAmount));
    assert_eq!(require_positive(-1), Err(SettlementError::InvalidAmount));
}

#[test]
fn fiat_to_crypto_requires_only_an_origin_currency() {
    let valid = SettlementRoute {
        origin: Some(USD),
        destination: None,
    };
    let invalid = SettlementRoute {
        origin: Some(USD),
        destination: Some(EUR),
    };

    assert_eq!(
        require_valid_route(&SettlementType::FiatToCrypto, &valid),
        Ok(())
    );
    assert_eq!(
        require_valid_route(&SettlementType::FiatToCrypto, &invalid),
        Err(SettlementError::InvalidRoute)
    );
}

#[test]
fn crypto_to_fiat_requires_only_a_destination_currency() {
    let valid = SettlementRoute {
        origin: None,
        destination: Some(GBP),
    };
    let invalid = SettlementRoute {
        origin: Some(USD),
        destination: Some(GBP),
    };

    assert_eq!(
        require_valid_route(&SettlementType::CryptoToFiat, &valid),
        Ok(())
    );
    assert_eq!(
        require_valid_route(&SettlementType::CryptoToFiat, &invalid),
        Err(SettlementError::InvalidRoute)
    );
}

#[test]
fn fiat_to_fiat_requires_distinct_supported_currencies() {
    let valid = SettlementRoute {
        origin: Some(USD),
        destination: Some(EUR),
    };
    let same_currency = SettlementRoute {
        origin: Some(USD),
        destination: Some(USD),
    };
    let unsupported = SettlementRoute {
        origin: Some(USD),
        destination: Some(999),
    };

    assert_eq!(
        require_valid_route(&SettlementType::FiatToFiat, &valid),
        Ok(())
    );
    assert_eq!(
        require_valid_route(&SettlementType::FiatToFiat, &same_currency),
        Err(SettlementError::InvalidRoute)
    );
    assert_eq!(
        require_valid_route(&SettlementType::FiatToFiat, &unsupported),
        Err(SettlementError::UnsupportedCurrency)
    );
}
