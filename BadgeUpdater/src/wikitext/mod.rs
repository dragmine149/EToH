//! Wikitext module root
//!
//! This file is intentionally small: it declares and re-exports the submodules
//! that implement the wikitext parser according to `spec.md`.
//!
//! Implementation details (types and functions) live in submodules so callers
//! can `use wikitext::...` to access commonly-used items.
//!
//! The module layout has been split into smaller files; re-exports below
//! provide a stable public surface.

#![allow(
    dead_code,
    unused_imports,
    reason = "This is a mini-module, it might be used in the future."
)]

pub mod argument;
pub mod enums;
pub mod errors;
pub mod parsed_data;
pub mod wiki_text;

/// Helper submodule grouping for parsing/types that were moved into a
/// `types/` directory. Declaring this inline module block allows the
/// compiler to find `types/templates.rs`, `types/links.rs` and `types/table.rs`.
pub mod types {
    pub mod links;
    pub mod table;
    pub mod templates;
}

// Re-export commonly used types for ergonomic access.
pub use enums::QueryType;

// Re-export data types implemented inside submodules.
// Templates, links, and table-related types were moved into `types/*`.
pub use types::links::Link;
pub use types::table::{Cell, Row, Table, TableCell, build_table_grid};
pub use types::templates::{Template, TemplateArgument};

// Keep Argument re-exported from parsed_data as the canonical top-level enum
// carrying all parsed elements (Template, Link, List, Table, Text).
pub use parsed_data::Argument;

// Expose wiki_text entrypoint
pub use wiki_text::WikiText;
