//! Wikitext module root
//!
//! This file is intentionally small: it declares and re-exports the submodules
//! that implement the wikitext parser according to `spec.md`.
//!
//! Implementation details (types and functions) live in submodules so callers
//! can `use wikitext::...` to access commonly-used items.

pub mod argument;
pub mod enums;
pub mod errors;
pub mod parsed_data;
pub mod wiki_text;

// Re-export commonly used types for ergonomic access.
pub use enums::QueryType;

// Re-export data types implemented inside `parsed_data` (we don't have
// separate `link`, `list`, or `template` modules; those types live in
// `parsed_data.rs`).
pub use parsed_data::{Argument, Template};

pub use wiki_text::WikiText;
