use std::fmt;
use std::io;
use thiserror::Error;

/// All errors produced by 8z.
#[derive(Error, Debug)]
pub enum EightZError {
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

impl EightZError {
    /// Construct an [`InvalidSignature`](EightZError::InvalidSignature) from any `Display` value.
    pub fn invalid_signature<T: fmt::Display>(msg: T) -> Self {
        EightZError::InvalidSignature(msg.to_string())
    }

    /// Construct an [`InvalidHeader`](EightZError::InvalidHeader) from any `Display` value.
    pub fn invalid_header<T: fmt::Display>(msg: T) -> Self {
        EightZError::InvalidHeader(msg.to_string())
    }

    /// Construct a [`Truncated`](EightZError::Truncated) error from any `Display` value.
    pub fn truncated<T: fmt::Display>(msg: T) -> Self {
        EightZError::Truncated(msg.to_string())
    }

    /// Construct an [`InvalidArgument`](EightZError::InvalidArgument) from any `Display` value.
    pub fn invalid_argument<T: fmt::Display>(msg: T) -> Self {
        EightZError::InvalidArgument(msg.to_string())
    }

    /// Construct a [`NotYetImplemented`](EightZError::NotYetImplemented) error.
    pub fn not_yet_implemented(label: &'static str) -> Self {
        EightZError::NotYetImplemented(label)
    }
}

/// Convenience alias used throughout 8z.
pub type EightZResult<T> = Result<T, EightZError>;

// ── Codec sub-crate error conversions ────────────────────────────────────────

#[cfg(feature = "lzma")]
impl From<lazippy::error::LazippyError> for EightZError {
    fn from(e: lazippy::error::LazippyError) -> Self {
        EightZError::Coder(Box::new(e))
    }
}
