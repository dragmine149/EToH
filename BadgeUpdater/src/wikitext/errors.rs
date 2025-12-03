//! Rich error types for the wikitext module.
//!
//! This module centralizes all error kinds used by the wikitext parser and its
//! helper APIs. Errors are designed to be informative (they carry messages and
//!, where appropriate, offsets into the source text).
//!
//! Exported items:
//! - `WtError` - main error enum with variants for parse failures, missing
//!    items, index issues and invalid arguments.
//! - `Result<T>` - convenient alias `std::result::Result<T, WtError>`.
//!
//! The error type implements `std::error::Error`, `Debug`, and `Display`.
//! Conversion helpers for common error types are provided to make it easy to
//! propagate underlying errors into `WtError`.

use std::error::Error;
use std::fmt;

/// The canonical result type used across the wikitext module.
pub type Result<T> = std::result::Result<T, WtError>;

/// Wikitext error with rich variants.
///
/// - `ParseError` - problems encountered while parsing wikitext. Includes an
///    explanatory message and optionally the byte offset where the problem
///    occurred (useful for diagnostics).
/// - `NotFound` - requested item was not present (templates/arguments/links).
/// - `IndexOutOfBounds` - asked for the Nth element but the collection was
///    smaller; contains both the requested index and the available length.
/// - `InvalidArgument` - e.g., trying to convert an `Argument` to `Template`
///    when it's a different kind.
/// - `Io` - wrapper for underlying I/O errors if relevant to future helpers.
/// - `Other` - catch-all carrying a message and optional boxed cause.
#[derive(Debug)]
pub enum WtError {
    ParseError {
        msg: String,
        /// Byte offset in the source where the parse error was detected, if known.
        offset: Option<usize>,
    },
    NotFound {
        msg: String,
    },
    IndexOutOfBounds {
        idx: usize,
        len: usize,
    },
    InvalidArgument {
        msg: String,
    },
    Io {
        msg: String,
        source: Option<Box<dyn Error + Send + Sync + 'static>>,
    },
    Other {
        msg: String,
        source: Option<Box<dyn Error + Send + Sync + 'static>>,
    },
}

impl WtError {
    /// Construct a parse error with a message.
    pub fn parse<S: Into<String>>(msg: S) -> Self {
        WtError::ParseError {
            msg: msg.into(),
            offset: None,
        }
    }

    /// Construct a parse error with a message and offset.
    pub fn parse_at<S: Into<String>>(msg: S, offset: usize) -> Self {
        WtError::ParseError {
            msg: msg.into(),
            offset: Some(offset),
        }
    }

    /// Construct a not-found error.
    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        WtError::NotFound { msg: msg.into() }
    }

    /// Construct an index-out-of-bounds error.
    pub fn index_oob(idx: usize, len: usize) -> Self {
        WtError::IndexOutOfBounds { idx, len }
    }

    /// Construct an invalid argument error.
    pub fn invalid_arg<S: Into<String>>(msg: S) -> Self {
        WtError::InvalidArgument { msg: msg.into() }
    }

    /// Wrap a std::io::Error or other error as an Io variant.
    pub fn io_err<E: Error + Send + Sync + 'static>(msg: impl Into<String>, e: E) -> Self {
        WtError::Io {
            msg: msg.into(),
            source: Some(Box::new(e)),
        }
    }

    /// Generic helper to produce Other(...) with an optional source.
    pub fn other_with_source<E: Error + Send + Sync + 'static>(
        msg: impl Into<String>,
        source: Option<E>,
    ) -> Self {
        WtError::Other {
            msg: msg.into(),
            source: source.map(|e| Box::new(e) as Box<dyn Error + Send + Sync>),
        }
    }

    /// Returns a short, user-friendly description of the error kind.
    pub fn kind(&self) -> &'static str {
        match self {
            WtError::ParseError { .. } => "ParseError",
            WtError::NotFound { .. } => "NotFound",
            WtError::IndexOutOfBounds { .. } => "IndexOutOfBounds",
            WtError::InvalidArgument { .. } => "InvalidArgument",
            WtError::Io { .. } => "Io",
            WtError::Other { .. } => "Other",
        }
    }

    /// If the error has an underlying source, return it (if any).
    pub fn source_opt(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            WtError::Io { source, .. } | WtError::Other { source, .. } => {
                source.as_ref().map(|b| b.as_ref() as &dyn Error)
            }
            _ => None,
        }
    }
}

impl fmt::Display for WtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WtError::ParseError { msg, offset } => {
                if let Some(off) = offset {
                    write!(f, "Parse error at {}: {}", off, msg)
                } else {
                    write!(f, "Parse error: {}", msg)
                }
            }
            WtError::NotFound { msg } => write!(f, "Not found: {}", msg),
            WtError::IndexOutOfBounds { idx, len } => {
                write!(f, "Index out of bounds: requested {}, length {}", idx, len)
            }
            WtError::InvalidArgument { msg } => write!(f, "Invalid argument: {}", msg),
            WtError::Io { msg, source } => {
                if let Some(s) = source {
                    write!(f, "IO error: {} (cause: {})", msg, s)
                } else {
                    write!(f, "IO error: {}", msg)
                }
            }
            WtError::Other { msg, source } => {
                if let Some(s) = source {
                    write!(f, "{} (cause: {})", msg, s)
                } else {
                    write!(f, "{}", msg)
                }
            }
        }
    }
}

impl Error for WtError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source_opt()
    }
}

/* Common conversions to make error propagation ergonomic. */

impl From<std::io::Error> for WtError {
    fn from(e: std::io::Error) -> Self {
        WtError::io_err("I/O error", e)
    }
}

impl From<std::num::ParseIntError> for WtError {
    fn from(e: std::num::ParseIntError) -> Self {
        WtError::other_with_source("parse int error", Some(e))
    }
}

impl From<std::num::ParseFloatError> for WtError {
    fn from(e: std::num::ParseFloatError) -> Self {
        WtError::other_with_source("parse float error", Some(e))
    }
}

impl From<std::string::FromUtf8Error> for WtError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        WtError::other_with_source("utf8 conversion error", Some(e))
    }
}

/* Unit tests for the error formatting and helpers. */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_parse_error_with_offset() {
        let e = WtError::parse_at("unexpected token '}}'", 123);
        let s = format!("{}", e);
        assert!(s.contains("123"));
        assert!(s.contains("unexpected token"));
    }

    #[test]
    fn display_not_found() {
        let e = WtError::not_found("template 'X' missing");
        let s = format!("{}", e);
        assert!(s.contains("template 'X' missing"));
    }

    #[test]
    fn io_conversion_has_source() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "oh no");
        let e: WtError = io_err.into();
        let s = format!("{}", e);
        assert!(s.contains("I/O error"));
        assert!(s.contains("oh no"));
    }
}
