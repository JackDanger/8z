use std::fmt;
use std::io;
use thiserror::Error;

/// All errors produced by 7zippy.
#[derive(Error, Debug)]
pub enum SevenZippyError {
    /// Wraps an underlying IO error.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// The 7z signature header magic bytes or CRC did not match.
    #[error("invalid signature: {0}")]
    InvalidSignature(String),

    /// A metadata header field was malformed or out of range.
    #[error("invalid header: {0}")]
    InvalidHeader(String),

    /// The coder method ID is not recognised.
    #[error("unsupported method {method_id:?}{}", .name.as_deref().map(|n| format!(" ({n})")).unwrap_or_default())]
    UnsupportedMethod {
        method_id: Vec<u8>,
        name: Option<String>,
    },

    /// The coder method is recognised but the feature flag that enables it is
    /// disabled for this build.
    #[error(
        "coder '{name}' is recognised but not enabled (compile with the matching feature flag)"
    )]
    MissingCoder { name: &'static str },

    /// The input stream was cut short before the expected end.
    #[error("truncated input: {0}")]
    Truncated(String),

    /// Wraps an error returned by a codec sub-crate (e.g. `LazippyError`).
    #[error("coder error: {0}")]
    Coder(Box<dyn std::error::Error + Send + Sync>),

    /// Misuse of the public API or CLI argument.
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    /// Functionality that is planned but not yet implemented.
    #[error("not yet implemented: {0}")]
    NotYetImplemented(&'static str),
}

impl SevenZippyError {
    /// Construct an [`InvalidSignature`](SevenZippyError::InvalidSignature) from any `Display` value.
    pub fn invalid_signature<T: fmt::Display>(msg: T) -> Self {
        SevenZippyError::InvalidSignature(msg.to_string())
    }

    /// Construct an [`InvalidHeader`](SevenZippyError::InvalidHeader) from any `Display` value.
    pub fn invalid_header<T: fmt::Display>(msg: T) -> Self {
        SevenZippyError::InvalidHeader(msg.to_string())
    }

    /// Construct a [`Truncated`](SevenZippyError::Truncated) error from any `Display` value.
    pub fn truncated<T: fmt::Display>(msg: T) -> Self {
        SevenZippyError::Truncated(msg.to_string())
    }

    /// Construct an [`InvalidArgument`](SevenZippyError::InvalidArgument) from any `Display` value.
    pub fn invalid_argument<T: fmt::Display>(msg: T) -> Self {
        SevenZippyError::InvalidArgument(msg.to_string())
    }

    /// Construct a [`NotYetImplemented`](SevenZippyError::NotYetImplemented) error.
    pub fn not_yet_implemented(label: &'static str) -> Self {
        SevenZippyError::NotYetImplemented(label)
    }

    /// Construct a [`MissingCoder`](SevenZippyError::MissingCoder) error.
    pub fn missing_coder(name: &'static str) -> Self {
        SevenZippyError::MissingCoder { name }
    }

    /// Construct an [`UnsupportedMethod`](SevenZippyError::UnsupportedMethod) error.
    pub fn unsupported_method(method_id: Vec<u8>) -> Self {
        let name = match method_id.as_slice() {
            [0x03, 0x01, 0x01] => Some("LZMA".to_string()),
            [0x21] => Some("LZMA2".to_string()),
            [0x04, 0x01, 0x08] => Some("Deflate".to_string()),
            [0x04, 0x01, 0x09] => Some("Deflate64".to_string()),
            [0x04, 0x02, 0x02] => Some("BZip2".to_string()),
            [0x03, 0x04, 0x01] => Some("PPMd".to_string()),
            [0x06, 0xF1, 0x07, 0x01] => Some("AES-SHA256".to_string()),
            _ => None,
        };
        SevenZippyError::UnsupportedMethod { method_id, name }
    }
}

/// Convenience alias used throughout 7zippy.
pub type SevenZippyResult<T> = Result<T, SevenZippyError>;

// ── Codec sub-crate error conversions ────────────────────────────────────────

#[cfg(feature = "lzma")]
impl From<lazippy::error::LazippyError> for SevenZippyError {
    fn from(e: lazippy::error::LazippyError) -> Self {
        SevenZippyError::Coder(Box::new(e))
    }
}
