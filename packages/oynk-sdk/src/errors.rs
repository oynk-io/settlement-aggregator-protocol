use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SettlementError {
    AlreadyInitialized = 200,
    BadDeadline = 201,
    IdOverflow = 202,
    InvalidAmount = 203,
    InvalidRoute = 204,
    UnsupportedCurrency = 205,
    RequestNotFound = 206,
    InvalidRequestStatus = 207,
    BadFundingStatus = 208,
    SettlementExpired = 209,
    FiatFundingNotRequired = 210,
    QuoteNotSet = 211,
    SourceAmountMissing = 212,
    SettlerAmountMissing = 213,
    SourceSettlerNotNeeded = 214,
    DestinationSettlerNotNeeded = 215,
    FiatNotConfirmed = 216,
    SourceSettlerMissing = 219,
    DestinationSettlerMissing = 220,
    ConditionFailed = 224,
    SettlementAssetNotDeposited = 228,
    PayoutExceedsDeposit = 230,
    NotAuthorized = 232,
    RequestCancelled = 237,
    RequestDisputed = 238,
    ReadyForClaim = 239,
    NothingToDispute = 240,
    AlreadyProcessed = 245,
}
