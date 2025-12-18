//! Helpers and convenience APIs for the `Argument` enum.
//!
//! This module purposefully provides utility methods and conversions for the
//! `Argument` type (which is defined in the parent module). The implementation
//! keeps everything lightweight and avoids duplicating core data-structures.
//!
//! The methods here are ergonomic helpers used by callers to inspect and extract
//! values from `Argument` without needing to pattern-match everywhere.

use std::convert::TryFrom;
use std::fmt;

use crate::wikitext::errors::WtError;
#[allow(unused_imports)]
use crate::wikitext::parsed_data::{Argument, Link, List, Table, Template, Text};

impl Argument {
    /// Returns a short textual kind for the argument.
    ///
    /// Examples: "Template", "Link", "List", "Text".
    pub fn kind(&self) -> &'static str {
        match self {
            Argument::Template(_) => "Template",
            Argument::Link(_) => "Link",
            Argument::List(_) => "List",
            Argument::Table(_) => "Table",
            Argument::Text(_) => "Text",
        }
    }

    /// If this argument is a template, return a reference to it.
    pub fn as_template(&self) -> Option<&Template> {
        match self {
            Argument::Template(t) => Some(t),
            _ => None,
        }
    }

    /// If this argument is a link, return a reference to it.
    pub fn as_link(&self) -> Option<&Link> {
        match self {
            Argument::Link(l) => Some(l),
            _ => None,
        }
    }

    /// If this argument is a list, return a reference to it.
    pub fn as_list(&self) -> Option<&List> {
        match self {
            Argument::List(l) => Some(l),
            _ => None,
        }
    }

    /// If this argument is text, return a reference to it.
    pub fn as_text(&self) -> Option<&Text> {
        match self {
            Argument::Text(t) => Some(t),
            _ => None,
        }
    }

    /// Attempts to produce a best-effort textual representation of the
    /// argument's content. This is intended for logging/debugging and to
    /// provide simple access to textual data without drilling into nested
    /// structures.
    pub fn to_text_lossy(&self) -> String {
        match self {
            Argument::Text(t) => t.raw.clone(),
            Argument::Link(l) => {
                if l.label.is_empty() {
                    l.target.clone()
                } else {
                    l.label.clone()
                }
            }
            Argument::Template(t) => {
                // prefer named arguments when available; otherwise join positional
                if let Ok(v) = t.get_named_arg("1") {
                    // if template had a named "1" use that as an example
                    v.raw
                } else if let Some(first_pos) = t.arguments.iter().find(|a| a.name.is_none()) {
                    first_pos.value.raw.clone()
                } else {
                    format!("{{{{{}}}}}", t.name)
                }
            }
            Argument::List(l) => {
                // join first few entry textual representations
                let mut parts = Vec::with_capacity(l.entries.len());
                for e in &l.entries {
                    parts.push(e.to_text_lossy());
                }
                parts.join(" | ")
            }
            Argument::Table(tb) => {
                // Prefer table title, otherwise headers, otherwise a summary.
                if let Some(ref t) = tb.title {
                    t.clone()
                } else if !tb.headers.is_empty() {
                    tb.headers.join(", ")
                } else {
                    format!("Table(rows={})", tb.rows.len())
                }
            }
        }
    }
}

/// Try to convert a top-level `Argument` into a `Template`.
///
/// Consumes the argument. On mismatch returns `WtError::InvalidArgument`.
impl TryFrom<Argument> for Template {
    type Error = WtError;

    fn try_from(value: Argument) -> Result<Self, Self::Error> {
        match value {
            Argument::Template(t) => Ok(t),
            other => Err(WtError::InvalidArgument {
                msg: format!("expected Template, found {}", other.kind()),
            }),
        }
    }
}

/// Try to convert a top-level `Argument` into a `Link`.
impl TryFrom<Argument> for Link {
    type Error = WtError;

    fn try_from(value: Argument) -> Result<Self, Self::Error> {
        match value {
            Argument::Link(l) => Ok(l),
            other => Err(WtError::InvalidArgument {
                msg: format!("expected Link, found {}", other.kind()),
            }),
        }
    }
}

/// Try to convert a top-level `Argument` into a `List`.
impl TryFrom<Argument> for List {
    type Error = WtError;

    fn try_from(value: Argument) -> Result<Self, Self::Error> {
        match value {
            Argument::List(l) => Ok(l),
            other => Err(WtError::InvalidArgument {
                msg: format!("expected List, found {}", other.kind()),
            }),
        }
    }
}

/// Try to convert a top-level `Argument` into a `Text`.
impl TryFrom<Argument> for Text {
    type Error = WtError;

    fn try_from(value: Argument) -> Result<Self, Self::Error> {
        match value {
            Argument::Text(t) => Ok(t),
            other => Err(WtError::InvalidArgument {
                msg: format!("expected Text, found {}", other.kind()),
            }),
        }
    }
}

impl fmt::Display for Argument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Argument::Template(t) => {
                write!(f, "Template({})", t.name)
            }
            Argument::Link(l) => {
                write!(f, "Link({} -> {})", l.label, l.target)
            }
            Argument::List(lst) => {
                write!(f, "List(len={})", lst.entries.len())
            }
            Argument::Table(tb) => {
                if let Some(ref title) = tb.title {
                    write!(f, "Table(\"{}\")", title)
                } else {
                    write!(f, "Table(rows={})", tb.rows.len())
                }
            }
            Argument::Text(txt) => {
                let mut s = txt.raw.clone();
                // short display: collapse newlines for readability
                s = s.replace('\n', "\\n");
                if s.len() > 80 {
                    s.truncate(77);
                    s.push_str("...");
                }
                write!(f, "Text(\"{}\")", s)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use crate::wikitext::parsed_data::{Link, List, Table, Template, TemplateArgument, Text};

    #[test]
    fn kind_and_text_lossy() {
        // Text
        let a = Argument::Text(Text::new("hello"));
        assert_eq!(a.kind(), "Text");
        assert_eq!(a.to_text_lossy(), "hello");

        // Link
        let l = Link::new_internal("Page", "Label");
        let a = Argument::Link(l);
        assert_eq!(a.kind(), "Link");
        assert_eq!(a.to_text_lossy(), "Label");
    }

    #[test]
    fn try_from_mismatch_errors() {
        let a = Argument::Text(Text::new("x"));
        let res: Result<Template, _> = Template::try_from(a);
        assert!(res.is_err());
    }
}
