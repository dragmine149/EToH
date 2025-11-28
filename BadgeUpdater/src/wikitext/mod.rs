/*
Top-level wikitext parser module.

This module re-exports parser and parts types. Serialization is provided by
deriving serde traits on the concrete types in `parts.rs` and `parser.rs`.
The conversion layer has been removed: the runtime types are expected to
implement (derive) `Serialize` and `Deserialize` directly.

Credits:
- GPT 5-mini (Github copilot w/ zed integration)
*/

pub mod parser;
pub mod parts;

pub use parts::Template;

/// Implement Display for runtime Template by delegating to `to_plain`.
impl std::fmt::Display for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_plain())
    }
}
