//! Enums used by the wikitext module.
//!
//! This module defines the primary enum types referenced by other submodules:
//! - `QueryType` — strategies for matching template/argument names.
//! - `LinkType` — distinguishes internal vs external links.
//! - `ListType` — common list kinds used in MediaWiki wikitext.
//!
//! Each type implements `Debug`, `Clone`, `PartialEq`, `Eq` and `Display`. They
//! also implement `FromStr` to allow convenient parsing from textual form in
//! tests or higher-level code.

use std::fmt;
use std::str::FromStr;

/// Strategy used when searching for templates, arguments, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    /// Exact (case-insensitive) match.
    Exact,
    /// Prefix match (case-insensitive).
    StartsWith,
    /// Substring match (case-insensitive).
    Contains,
}

impl fmt::Display for QueryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryType::Exact => write!(f, "Exact"),
            QueryType::StartsWith => write!(f, "StartsWith"),
            QueryType::Contains => write!(f, "Contains"),
        }
    }
}

impl FromStr for QueryType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "exact" | "eq" | "e" => Ok(QueryType::Exact),
            "startswith" | "start" | "prefix" | "s" => Ok(QueryType::StartsWith),
            "contains" | "contain" | "substr" | "c" => Ok(QueryType::Contains),
            other => Err(format!("unknown QueryType '{}'", other)),
        }
    }
}

/// The kind of link encountered in parsed wikitext.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkType {
    /// Internal wiki link using `[[...]]`.
    Internal,
    /// External link using `[http://...]` or similar.
    External,
}

impl fmt::Display for LinkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinkType::Internal => write!(f, "Internal"),
            LinkType::External => write!(f, "External"),
        }
    }
}

impl FromStr for LinkType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "internal" | "int" | "i" => Ok(LinkType::Internal),
            "external" | "ext" | "e" => Ok(LinkType::External),
            other => Err(format!("unknown LinkType '{}'", other)),
        }
    }
}

/// The kind of list line in wikitext.
///
/// Common tokens:
/// - `*` unordered
/// - `#` ordered (numbered)
/// - `;` definition term / list
/// - `:` indented / definition description
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListType {
    /// Unordered list (bulleted) — `*`
    Unordered,
    /// Ordered (numbered) list — `#`
    Ordered,
    /// Definition-style list (`;` or `:` semantics)
    Definition,
    /// Any other marker not covered above; stores the raw marker string.
    Other(String),
}

impl fmt::Display for ListType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ListType::Unordered => write!(f, "Unordered"),
            ListType::Ordered => write!(f, "Ordered"),
            ListType::Definition => write!(f, "Definition"),
            ListType::Other(s) => write!(f, "Other({})", s),
        }
    }
}

impl FromStr for ListType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "*" | "unordered" | "bullet" => Ok(ListType::Unordered),
            "#" | "ordered" | "numbered" => Ok(ListType::Ordered),
            ";" | ":" | "definition" | "def" => Ok(ListType::Definition),
            other => Ok(ListType::Other(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn querytype_fromstr_and_display() {
        assert_eq!(QueryType::from_str("exact").unwrap(), QueryType::Exact);
        assert_eq!(
            QueryType::from_str("StartsWith").unwrap(),
            QueryType::StartsWith
        );
        assert_eq!(QueryType::from_str("c").unwrap(), QueryType::Contains);
        assert_eq!(format!("{}", QueryType::Exact), "Exact");
    }

    #[test]
    fn linktype_fromstr_and_display() {
        assert_eq!(LinkType::from_str("internal").unwrap(), LinkType::Internal);
        assert_eq!(LinkType::from_str("EXT").unwrap(), LinkType::External);
        assert_eq!(format!("{}", LinkType::External), "External");
    }

    #[test]
    fn listtype_fromstr_and_display() {
        assert_eq!(ListType::from_str("*").unwrap(), ListType::Unordered);
        assert_eq!(ListType::from_str("#").unwrap(), ListType::Ordered);
        assert_eq!(ListType::from_str(":").unwrap(), ListType::Definition);
        let other = ListType::from_str(">").unwrap();
        assert_eq!(format!("{}", other), "Other(>)");
    }
}
