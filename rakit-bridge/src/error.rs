use std::fmt;

#[derive(Debug, Clone)]
pub enum BridgeError {
    UnsupportedFeature(String),
    ConversionError(String),
}

impl fmt::Display for BridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BridgeError::UnsupportedFeature(msg) => {
                write!(f, "Fitur tidak didukung di Brak bridge: {}", msg)
            }
            BridgeError::ConversionError(msg) => {
                write!(f, "Kesalahan konversi: {}", msg)
            }
        }
    }
}

impl std::error::Error for BridgeError {}
