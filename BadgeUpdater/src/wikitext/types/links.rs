/*!
Link node and parsing helpers for MediaWiki-style links.

This module implements:
- `Link` data type with constructors and `to_wikitext`.
- `parse_internal_link_at(input, start)` for `[[...]]` style links (supports nesting).
- `parse_external_link_at(input, start)` for `[http... label]` style links.

The parsers are conservative and operate on UTF-8 character boundaries.
*/

use crate::wikitext::enums::LinkType;

/// Link node representing either an internal `[[target|label]]` or an external
/// `[http://... label]` link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub link_type: LinkType,
    pub label: String,
    pub target: String,
}

impl Link {
    /// Construct an internal link.
    pub fn new_internal<S: Into<String>>(target: S, label: S) -> Self {
        Self {
            link_type: LinkType::Internal,
            target: target.into(),
            label: label.into(),
        }
    }

    /// Construct an external link.
    pub fn new_external<S: Into<String>>(target: S, label: S) -> Self {
        Self {
            link_type: LinkType::External,
            target: target.into(),
            label: label.into(),
        }
    }

    /// Reconstruct the link as wikitext.
    pub fn to_wikitext(&self) -> String {
        match self.link_type {
            LinkType::Internal => {
                if self.label.is_empty() || self.label == self.target {
                    format!("[[{}]]", self.target)
                } else {
                    format!("[[{}|{}]]", self.target, self.label)
                }
            }
            LinkType::External => {
                if self.label.is_empty() || self.label == self.target {
                    format!("[{}]", self.target)
                } else {
                    // MediaWiki external link uses a space between target and label
                    format!("[{} {}]", self.target, self.label)
                }
            }
        }
    }
}

/// Parse an internal link `[[...]]` starting at `start` in `input`.
///
/// Returns Some((consumed_bytes, Link)) on success, or None if parse failed.
///
/// This supports nested internal links by counting nested `[[` / `]]` pairs.
pub fn parse_internal_link_at(input: &str, start: usize) -> Option<(usize, Link)> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    if start + 1 >= len || bytes[start] != b'[' || bytes[start + 1] != b'[' {
        return None;
    }

    let mut idx = start + 2;
    let mut depth: usize = 1;
    let mut content = String::new();

    while idx < len {
        // safe check for "[["
        if idx + 1 < len && bytes[idx] == b'[' && bytes[idx + 1] == b'[' {
            depth += 1;
            content.push_str("[[");
            idx += 2;
            continue;
        }
        // safe check for "]]"
        if idx + 1 < len && bytes[idx] == b']' && bytes[idx + 1] == b']' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                idx += 2; // consume closing "]]"
                break;
            } else {
                content.push_str("]]");
                idx += 2;
                continue;
            }
        }
        // otherwise append next char
        let ch = input[idx..].chars().next().unwrap();
        content.push(ch);
        idx += ch.len_utf8();
    }

    if content.is_empty() {
        return None;
    }

    // split at first top-level '|' (we don't support nested '|' detection here,
    // but for links the first '|' is the separator for target|label)
    let mut splits = content.splitn(2, '|');
    let target = splits.next().unwrap().trim().to_string();
    let label = splits
        .next()
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| target.clone());

    Some((idx - start, Link::new_internal(target, label)))
}

/// Parse an external link `[http... label]` starting at `start` in `input`.
///
/// Returns Some((consumed_bytes, Link)) on success, or None if parse failed.
///
/// This treats the first space as the separator between URL and label; label
/// may be omitted.
pub fn parse_external_link_at(input: &str, start: usize) -> Option<(usize, Link)> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    if start >= len || bytes[start] != b'[' {
        return None;
    }

    let mut idx = start + 1;
    let mut content = String::new();

    while idx < len {
        let ch = input[idx..].chars().next().unwrap();
        if ch == ']' {
            idx += ch.len_utf8(); // consume ']'
            break;
        } else {
            content.push(ch);
            idx += ch.len_utf8();
        }
    }

    if content.is_empty() {
        return None;
    }

    // split into target and optional label; split on first whitespace
    let mut parts = content.splitn(2, char::is_whitespace);
    let target = parts.next().unwrap().trim().to_string();
    if target.is_empty() {
        return None;
    }
    let label = parts
        .next()
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| target.clone());

    Some((idx - start, Link::new_external(target, label)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal_simple() {
        let s = "[[Page|Label]]";
        let res = parse_internal_link_at(s, 0).expect("should parse");
        assert_eq!(res.1.link_type, LinkType::Internal);
        assert_eq!(res.1.target, "Page");
        assert_eq!(res.1.label, "Label");
        assert_eq!(res.0, s.len());
    }

    #[test]
    fn internal_no_label() {
        let s = "[[Page Name]]";
        let res = parse_internal_link_at(s, 0).expect("should parse");
        assert_eq!(res.1.target, "Page Name");
        assert_eq!(res.1.label, "Page Name");
    }

    #[test]
    fn internal_nested() {
        let s = "[[A [[B]] C|Label]]";
        // This is somewhat ill-formed but the parser should be able to handle nested [[...]]
        let res = parse_internal_link_at(s, 0).expect("should parse nested");
        assert_eq!(res.1.link_type, LinkType::Internal);
        // target contains nested text preserved
        assert!(res.1.target.contains("B"));
    }

    #[test]
    fn external_simple() {
        let s = "[http://example.com Label]";
        let res = parse_external_link_at(s, 0).expect("should parse external");
        assert_eq!(res.1.link_type, LinkType::External);
        assert_eq!(res.1.target, "http://example.com");
        assert_eq!(res.1.label, "Label");
    }

    #[test]
    fn external_no_label() {
        let s = "[http://example.com]";
        let res = parse_external_link_at(s, 0).expect("should parse external no label");
        assert_eq!(res.1.target, "http://example.com");
        assert_eq!(res.1.label, "http://example.com");
    }

    #[test]
    fn to_wikitext_roundtrip() {
        let li = Link::new_internal("Page", "Label");
        assert_eq!(li.to_wikitext(), "[[Page|Label]]");
        let le = Link::new_external("http://x", "X");
        assert_eq!(le.to_wikitext(), "[http://x X]");
    }
}
