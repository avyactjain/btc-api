#[derive(Debug)]
pub enum BtcApiError {
    ConfigLoadError(String),
    ReqwestError(reqwest::Error),
    SerdeJsonError(serde_json::Error),
    InvalidResponse(String),
    UnableToVerifyTxnStatus,
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
        BtcApiError::InvalidResponse(error)
    }
}

impl std::fmt::Display for BtcApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BtcApiError::ConfigLoadError(e) => write!(f, "ConfigLoadError: {}", e),
            BtcApiError::ReqwestError(e) => write!(f, "ReqwestError: {}", e),
            BtcApiError::SerdeJsonError(e) => write!(f, "SerdeJsonError: {}", e),
            BtcApiError::InvalidResponse(e) => write!(f, "InvalidResponse: {}", e),
            BtcApiError::UnableToVerifyTxnStatus => write!(f, "UnableToVerifyTxnStatus"),
        }
    }
}
