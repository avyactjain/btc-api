#[derive(Debug)]
pub enum BtcApiError {
    ConfigLoadError(String),
    ReqwestError(reqwest::Error),
    SerdeJsonError(serde_json::Error),
    InvalidBroadcastResponse(String),
    UnableToVerifyTxnStatus,
    ExternalApiError(String),
    InvalidFee(String),
    BitcoinParseOutPointError(bitcoin::hex::HexToArrayError),
    BitcoinParseAddressError(bitcoin::address::ParseError),
    UrlParseError(url::ParseError),
    NoUtxosFound(String),
    InsufficientFunds(u64),
    RegexError(regex::Error),
    InvalidAddress(String),
}

impl From<reqwest::Error> for BtcApiError {
    fn from(error: reqwest::Error) -> Self {
        BtcApiError::ReqwestError(error)
    }
}

impl From<serde_json::Error> for BtcApiError {
    fn from(error: serde_json::Error) -> Self {
        BtcApiError::SerdeJsonError(error)
    }
}

impl From<String> for BtcApiError {
    fn from(error: String) -> Self {
        BtcApiError::InvalidBroadcastResponse(error)
    }
}

impl From<bitcoin::hex::HexToArrayError> for BtcApiError {
    fn from(error: bitcoin::hex::HexToArrayError) -> Self {
        BtcApiError::BitcoinParseOutPointError(error)
    }
}

impl From<bitcoin::address::ParseError> for BtcApiError {
    fn from(error: bitcoin::address::ParseError) -> Self {
        BtcApiError::BitcoinParseAddressError(error)
    }
}

impl From<url::ParseError> for BtcApiError {
    fn from(error: url::ParseError) -> Self {
        BtcApiError::UrlParseError(error)
    }
}

impl From<regex::Error> for BtcApiError {
    fn from(error: regex::Error) -> Self {
        BtcApiError::RegexError(error)
    }
}

impl std::fmt::Display for BtcApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BtcApiError::ConfigLoadError(e) => write!(f, "ConfigLoadError: {}", e),
            BtcApiError::ReqwestError(e) => write!(f, "ReqwestError: {}", e),
            BtcApiError::SerdeJsonError(e) => write!(f, "SerdeJsonError: {}", e),
            BtcApiError::InvalidBroadcastResponse(e) => write!(f, "InvalidResponse: {}", e),
            BtcApiError::UnableToVerifyTxnStatus => write!(f, "UnableToVerifyTxnStatus"),
            BtcApiError::ExternalApiError(e) => write!(f, "ExternalApiError: {}", e),
            BtcApiError::InvalidFee(e) => write!(f, "InvalidFee: {}", e),
            BtcApiError::BitcoinParseOutPointError(e) => {
                write!(f, "BitcoinParseOutPointError: {}", e)
            }
            BtcApiError::BitcoinParseAddressError(e) => {
                write!(f, "BitcoinParseAddressError: {}", e)
            }
            BtcApiError::UrlParseError(e) => write!(f, "UrlParseError: {}", e),
            BtcApiError::NoUtxosFound(address) => write!(f, "NoUtxosFound for Address:{}", address),
            BtcApiError::InsufficientFunds(amount) => {
                write!(f, "InsufficientFunds: {}", amount)
            }
            BtcApiError::RegexError(e) => write!(f, "RegexError: {}", e),
            BtcApiError::InvalidAddress(address) => write!(f, "InvalidAddress: {}", address),
        }
    }
}
