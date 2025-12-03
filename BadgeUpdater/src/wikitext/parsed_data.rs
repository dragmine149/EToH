//! Core parsed-data types and wikitext parsing helpers.
//!
//! This module implements the primary data structures required by the spec:
//! - `Text`
//! - `Link`
//! - `Template` (+ `TemplateArgument`)
//! - `List`
//! - `Argument` (top-level variant carrying the above)
//! - `ParsedData` (owner of parsed elements)
//!
//! It also provides a reasonably small, resilient parser implemented as
//! utility functions. Parsing is conservative: it only extracts top-level
//! templates, links and list blocks and keeps the rest as `Text` nodes.
//!
//! The API is designed so `ParsedData` and its contained elements are fully
//! owned and can be cloned by callers as needed.

use crate::wikitext::enums::{LinkType, ListType, QueryType};
use crate::wikitext::errors::WtError;

/// Helper: check whether the byte slice starting at `pos` begins with
/// ASCII "http" or "https" (case-insensitive). This performs byte-wise
/// checks and therefore avoids slicing the UTF-8 string at arbitrary
/// byte offsets which may fall inside a multibyte character.
fn starts_with_http(bytes: &[u8], pos: usize) -> bool {
    let len = bytes.len();
    // check "http" (4 bytes)
    if pos + 4 <= len {
        let slice = &bytes[pos..pos + 4];
        if slice
            .iter()
            .map(|b| b.to_ascii_lowercase())
            .eq(b"http".iter().cloned())
        {
            return true;
        }
    }
    // check "https" (5 bytes)
    if pos + 5 <= len {
        let slice = &bytes[pos..pos + 5];
        if slice
            .iter()
            .map(|b| b.to_ascii_lowercase())
            .eq(b"https".iter().cloned())
        {
            return true;
        }
    }
    false
}

/// Raw text node that wasn't parsed into other structures.
#[derive(Debug, Clone)]
pub struct Text {
    pub raw: String,
}

impl Text {
    pub fn new<S: Into<String>>(s: S) -> Self {
        Self { raw: s.into() }
    }
}

/// Link node.
#[derive(Debug, Clone)]
pub struct Link {
    pub link_type: LinkType,
    pub label: String,
    pub target: String,
}

impl Link {
    pub fn new_internal<S: Into<String>>(target: S, label: S) -> Self {
        Self {
            link_type: LinkType::Internal,
            label: label.into(),
            target: target.into(),
        }
    }
    pub fn new_external<S: Into<String>>(target: S, label: S) -> Self {
        Self {
            link_type: LinkType::External,
            label: label.into(),
            target: target.into(),
        }
    }
}

/// Template argument value - represented as `ParsedData` so it may contain
/// nested templates/links/lists/etc.
#[derive(Debug, Clone)]
pub struct TemplateArgument {
    pub name: Option<String>,
    pub value: ParsedData,
}

/// Template node.
#[derive(Debug, Clone)]
pub struct Template {
    pub name: String,
    pub arguments: Vec<TemplateArgument>,
}

impl Template {
    /// Return all arguments (owned clone).
    pub fn arguments(&self) -> Vec<TemplateArgument> {
        self.arguments.clone()
    }

    /// Get the first named argument matching `name` (case-insensitive).
    pub fn get_named_arg(&self, name: &str) -> Result<ParsedData, WtError> {
        for arg in &self.arguments {
            if let Some(ref n) = arg.name
                && n.eq_ignore_ascii_case(name) {
                    return Ok(arg.value.clone());
                }
        }
        Err(WtError::not_found(format!(
            "Named argument '{}' not found in template '{}'",
            name, self.name
        )))
    }

    /// Get all named args matching `query` according to `QueryType`.
    pub fn get_named_args_query(&self, query: &str, qtype: QueryType) -> Vec<ParsedData> {
        let query_lc = query.to_lowercase();
        let mut out = Vec::new();
        for arg in &self.arguments {
            if let Some(ref n) = arg.name {
                let n_lc = n.to_lowercase();
                let matched = match qtype {
                    QueryType::Exact => n_lc == query_lc,
                    QueryType::StartsWith => n_lc.starts_with(&query_lc),
                    QueryType::Contains => n_lc.contains(&query_lc),
                };
                if matched {
                    out.push(arg.value.clone());
                }
            }
        }
        out
    }

    /// Get positional argument by index (0-based).
    pub fn get_positional_arg(&self, pos: usize) -> Result<ParsedData, WtError> {
        let pos_args: Vec<&TemplateArgument> =
            self.arguments.iter().filter(|a| a.name.is_none()).collect();
        if pos < pos_args.len() {
            Ok(pos_args[pos].value.clone())
        } else {
            Err(WtError::index_oob(pos, pos_args.len()))
        }
    }
}

/// A list node containing entries which are top-level arguments (text/templates/etc).
#[derive(Debug, Clone)]
pub struct List {
    pub list_type: ListType,
    pub entries: Vec<Argument>,
}

/// Top-level argument - variant for every kind of parsed component.
#[derive(Debug, Clone)]
pub enum Argument {
    Template(Template),
    Link(Link),
    List(List),
    Text(Text),
}

impl Argument {
    pub fn into_template(self) -> Option<Template> {
        match self {
            Argument::Template(t) => Some(t),
            _ => None,
        }
    }
    pub fn into_link(self) -> Option<Link> {
        match self {
            Argument::Link(l) => Some(l),
            _ => None,
        }
    }
}

/// The result of parsing a fragment or whole page. Contains owned elements and
/// the original raw string.
#[derive(Debug, Clone)]
pub struct ParsedData {
    pub raw: String,
    pub elements: Vec<Argument>,
}

impl ParsedData {
    /// Create a new ParsedData with given raw text and no elements.
    pub fn new<S: Into<String>>(raw: S) -> Self {
        Self {
            raw: raw.into(),
            elements: Vec::new(),
        }
    }

    /// Get first template by exact name (case-insensitive).
    pub fn get_template(&self, name: &str) -> Result<Template, WtError> {
        let mut matches = self.get_template_query(name, QueryType::Exact).into_iter();
        if let Some(t) = matches.next() {
            Ok(t)
        } else {
            Err(WtError::not_found(format!("Template '{}' not found", name)))
        }
    }

    /// Get all templates that match `query` according to `qtype`.
    pub fn get_template_query(&self, query: &str, qtype: QueryType) -> Vec<Template> {
        let q_lc = query.to_lowercase();
        let mut out = Vec::new();
        for elem in &self.elements {
            if let Argument::Template(t) = elem {
                let name_lc = t.name.to_lowercase();
                let matched = match qtype {
                    QueryType::Exact => name_lc == q_lc,
                    QueryType::StartsWith => name_lc.starts_with(&q_lc),
                    QueryType::Contains => name_lc.contains(&q_lc),
                };
                if matched {
                    out.push(t.clone());
                }
            }
        }
        out
    }

    /// Return top-level links. If `lt` is None, returns all link types.
    pub fn get_links(&self, lt: Option<LinkType>) -> Vec<Link> {
        let mut out = Vec::new();
        for elem in &self.elements {
            if let Argument::Link(l) = elem {
                if let Some(ref want) = lt {
                    if &l.link_type == want {
                        out.push(l.clone());
                    }
                } else {
                    out.push(l.clone());
                }
            }
        }
        out
    }

    /// Return nth top-level element (0-based). If out of bounds returns an error.
    pub fn get(&self, nth: usize) -> Result<Argument, WtError> {
        if nth < self.elements.len() {
            Ok(self.elements[nth].clone())
        } else {
            Err(WtError::index_oob(nth, self.elements.len()))
        }
    }
}

/// Parse a wikitext fragment into `ParsedData`.
///
/// The parser extracts top-level:
/// - templates ({{...}}) with nesting support
/// - internal links ([[...]])
/// - external links ([http... label])
/// - simple list blocks (lines starting with *, #, ;, :)
///
/// All other content is returned as `Text` nodes. The function is conservative
/// and aims to be robust rather than fully feature-complete.
pub fn parse_wikitext_fragment(input: &str) -> Result<ParsedData, WtError> {
    let mut pd = ParsedData::new(input.to_string());
    let mut idx = 0usize;
    let bytes = input.as_bytes();
    let len = bytes.len();

    // accumulate contiguous plain text
    let mut current_text = String::new();

    while idx < len {
        // detect template start "{{"
        if idx + 1 < len && bytes[idx] == b'{' && bytes[idx + 1] == b'{' {
            // flush current_text
            if !current_text.is_empty() {
                pd.elements
                    .push(Argument::Text(Text::new(current_text.clone())));
                current_text.clear();
            }
            if let Some((consumed, tpl)) = parse_template_at(input, idx) {
                pd.elements.push(Argument::Template(tpl));
                idx += consumed;
                continue;
            } else {
                // treat as literal
                current_text.push_str("{{");
                idx += 2;
                continue;
            }
        }

        // internal link "[["
        if idx + 1 < len && bytes[idx] == b'[' && bytes[idx + 1] == b'[' {
            if !current_text.is_empty() {
                pd.elements
                    .push(Argument::Text(Text::new(current_text.clone())));
                current_text.clear();
            }
            if let Some((consumed, link)) = parse_internal_link_at(input, idx) {
                pd.elements.push(Argument::Link(link));
                idx += consumed;
                continue;
            } else {
                current_text.push_str("[[");
                idx += 2;
                continue;
            }
        }

        // external link "[http" or "[https"
        if bytes[idx] == b'[' {
            // Use the helper to check for "http"/"https" safely at byte level.
            if starts_with_http(bytes, idx + 1) {
                if !current_text.is_empty() {
                    pd.elements
                        .push(Argument::Text(Text::new(current_text.clone())));
                    current_text.clear();
                }
                if let Some((consumed, link)) = parse_external_link_at(input, idx) {
                    pd.elements.push(Argument::Link(link));
                    idx += consumed;
                    continue;
                } else {
                    current_text.push('[');
                    idx += 1;
                    continue;
                }
            }
        }

        // list line detection at line start
        let at_line_start = if idx == 0 {
            true
        } else {
            let prev = bytes[idx - 1];
            prev == b'\n' || prev == b'\r'
        };
        if at_line_start {
            // skip spaces
            let mut ws = 0usize;
            while idx + ws < len
                && bytes[idx + ws].is_ascii_whitespace()
                && bytes[idx + ws] != b'\n'
            {
                ws += 1;
            }
            if idx + ws < len {
                let ch = bytes[idx + ws] as char;
                if ch == '*' || ch == '#' || ch == ';' || ch == ':' {
                    if !current_text.is_empty() {
                        pd.elements
                            .push(Argument::Text(Text::new(current_text.clone())));
                        current_text.clear();
                    }
                    if let Some((consumed, list)) = parse_list_at(input, idx + ws) {
                        pd.elements.push(Argument::List(list));
                        idx = idx + ws + consumed;
                        continue;
                    }
                }
            }
        }

        // default: append character to current_text
        current_text.push(bytes[idx] as char);
        idx += 1;
    }

    if !current_text.is_empty() {
        pd.elements.push(Argument::Text(Text::new(current_text)));
    }

    Ok(pd)
}

/// Parse a template starting at `start` (expects "{{").
fn parse_template_at(input: &str, start: usize) -> Option<(usize, Template)> {
    let bytes = input.as_bytes();
    let mut idx = start;
    let len = bytes.len();
    if idx + 1 >= len || bytes[idx] != b'{' || bytes[idx + 1] != b'{' {
        return None;
    }
    idx += 2; // consume "{{"

    let mut depth = 1usize;
    let mut content = String::new();

    while idx < len {
        if idx + 1 < len && bytes[idx] == b'{' && bytes[idx + 1] == b'{' {
            depth += 1;
            content.push_str("{{");
            idx += 2;
            continue;
        } else if idx + 1 < len && bytes[idx] == b'}' && bytes[idx + 1] == b'}' {
            depth -= 1;
            if depth == 0 {
                idx += 2; // consume "}}"
                break;
            } else {
                content.push_str("}}");
                idx += 2;
                continue;
            }
        } else {
            content.push(bytes[idx] as char);
            idx += 1;
        }
    }

    if depth != 0 {
        return None;
    }

    match parse_template_content(&content) {
        Ok(tpl) => Some((idx - start, tpl)),
        Err(_) => None,
    }
}

/// Parse the inside of a template (without the surrounding braces).
fn parse_template_content(content: &str) -> Result<Template, String> {
    // Split top-level by '|'
    let parts = split_top_level(content, '|');
    if parts.is_empty() {
        return Err("empty template content".into());
    }
    let name = parts[0].trim().to_string();
    if name.is_empty() {
        return Err("empty template name".into());
    }

    let mut arguments: Vec<TemplateArgument> = Vec::new();
    for p in parts.into_iter().skip(1) {
        let trimmed = p.trim();
        if trimmed.is_empty() {
            // empty positional
            arguments.push(TemplateArgument {
                name: None,
                value: ParsedData::new(""),
            });
            continue;
        }
        if let Some(eq_pos) = find_top_level_char(trimmed, '=') {
            let (npart, vpart) = trimmed.split_at(eq_pos);
            let name = npart.trim().to_string();
            let val = vpart[1..].trim(); // skip '='
            let parsed_value = parse_wikitext_fragment(val)
                .map_err(|e| format!("failed to parse argument value: {}", e))?;
            arguments.push(TemplateArgument {
                name: Some(name),
                value: parsed_value,
            });
        } else {
            let parsed_value = parse_wikitext_fragment(trimmed)
                .map_err(|e| format!("failed to parse positional argument: {}", e))?;
            arguments.push(TemplateArgument {
                name: None,
                value: parsed_value,
            });
        }
    }

    Ok(Template { name, arguments })
}

/// Split by `sep` only at top level (not inside nested {{ }}, [[ ]], or <...> tags).
fn split_top_level(s: &str, sep: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut i = 0usize;
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut depth_brace = 0usize;
    let mut depth_bracket = 0usize;
    let mut in_tag = false;

    while i < len {
        let ch = bytes[i] as char;
        if ch == '{' && i + 1 < len && bytes[i + 1] == b'{' {
            depth_brace += 1;
            cur.push_str("{{");
            i += 2;
            continue;
        } else if ch == '}' && i + 1 < len && bytes[i + 1] == b'}' {
            depth_brace = depth_brace.saturating_sub(1);
            cur.push_str("}}");
            i += 2;
            continue;
        } else if ch == '[' && i + 1 < len && bytes[i + 1] == b'[' {
            depth_bracket += 1;
            cur.push_str("[[");
            i += 2;
            continue;
        } else if ch == ']' && i + 1 < len && bytes[i + 1] == b']' {
            depth_bracket = depth_bracket.saturating_sub(1);
            cur.push_str("]]");
            i += 2;
            continue;
        } else if ch == '<' {
            in_tag = true;
            cur.push(ch);
            i += 1;
            continue;
        } else if ch == '>' {
            in_tag = false;
            cur.push(ch);
            i += 1;
            continue;
        }

        if ch == sep && depth_brace == 0 && depth_bracket == 0 && !in_tag {
            parts.push(cur);
            cur = String::new();
            i += 1;
            continue;
        } else {
            cur.push(ch);
            i += 1;
            continue;
        }
    }

    parts.push(cur);
    parts
}

/// Find a top-level occurrence of `c` in `s` (not inside nested constructs).
fn find_top_level_char(s: &str, c: char) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0usize;
    let len = bytes.len();
    let mut depth_brace = 0usize;
    let mut depth_bracket = 0usize;
    let mut in_tag = false;
    while i < len {
        if i + 1 < len && bytes[i] == b'{' && bytes[i + 1] == b'{' {
            depth_brace += 1;
            i += 2;
            continue;
        } else if i + 1 < len && bytes[i] == b'}' && bytes[i + 1] == b'}' {
            depth_brace = depth_brace.saturating_sub(1);
            i += 2;
            continue;
        } else if i + 1 < len && bytes[i] == b'[' && bytes[i + 1] == b'[' {
            depth_bracket += 1;
            i += 2;
            continue;
        } else if i + 1 < len && bytes[i] == b']' && bytes[i + 1] == b']' {
            depth_bracket = depth_bracket.saturating_sub(1);
            i += 2;
            continue;
        } else if bytes[i] as char == '<' {
            in_tag = true;
            i += 1;
            continue;
        } else if bytes[i] as char == '>' {
            in_tag = false;
            i += 1;
            continue;
        }

        if (bytes[i] as char) == c && depth_brace == 0 && depth_bracket == 0 && !in_tag {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Parse an internal link `[[...]]` starting at `start`.
fn parse_internal_link_at(input: &str, start: usize) -> Option<(usize, Link)> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    if start + 1 >= len || bytes[start] != b'[' || bytes[start + 1] != b'[' {
        return None;
    }
    let mut idx = start + 2;
    let mut depth = 1usize;
    let mut content = String::new();

    while idx < len {
        if idx + 1 < len && bytes[idx] == b'[' && bytes[idx + 1] == b'[' {
            depth += 1;
            content.push_str("[[");
            idx += 2;
            continue;
        } else if idx + 1 < len && bytes[idx] == b']' && bytes[idx + 1] == b']' {
            depth -= 1;
            if depth == 0 {
                idx += 2;
                break;
            } else {
                content.push_str("]]");
                idx += 2;
                continue;
            }
        } else {
            content.push(bytes[idx] as char);
            idx += 1;
        }
    }

    if content.is_empty() {
        return None;
    }

    let mut splits = content.splitn(2, '|');
    let target = splits.next().unwrap().trim().to_string();
    let label = splits
        .next()
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| target.clone());
    Some((idx - start, Link::new_internal(target, label)))
}

/// Parse an external link `[http... label]` starting at `start`.
fn parse_external_link_at(input: &str, start: usize) -> Option<(usize, Link)> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    if start >= len || bytes[start] != b'[' {
        return None;
    }
    let mut idx = start + 1;
    let mut content = String::new();
    while idx < len {
        let ch = bytes[idx] as char;
        if ch == ']' {
            idx += 1;
            break;
        } else {
            content.push(ch);
            idx += 1;
        }
    }
    if content.is_empty() {
        return None;
    }
    let mut parts = content.splitn(2, ' ');
    let target = parts.next().unwrap().trim().to_string();
    let label = parts
        .next()
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| target.clone());
    Some((idx - start, Link::new_external(target, label)))
}

/// Parse a block of consecutive list lines starting at `start` (pointing to bullet char).
fn parse_list_at(input: &str, start: usize) -> Option<(usize, List)> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut idx = start;
    if idx >= len {
        return None;
    }
    let bullet = bytes[idx] as char;
    let mut entries: Vec<Argument> = Vec::new();
    let mut consumed = 0usize;

    while idx < len {
        let mut line_idx = idx;
        // skip leading spaces (but not newlines)
        while line_idx < len && bytes[line_idx].is_ascii_whitespace() && bytes[line_idx] != b'\n' {
            line_idx += 1;
        }
        if line_idx >= len {
            break;
        }
        if bytes[line_idx] as char != bullet {
            break;
        }
        line_idx += 1; // consume bullet
        let mut line = String::new();
        while line_idx < len {
            let ch = bytes[line_idx] as char;
            if ch == '\n' {
                line_idx += 1;
                break;
            } else {
                line.push(ch);
                line_idx += 1;
            }
        }
        // parse the line content as fragment
        if let Ok(pd) = parse_wikitext_fragment(line.trim()) {
            if pd.elements.len() == 1 {
                entries.push(pd.elements[0].clone());
            } else {
                // keep as text wrapper with raw content
                entries.push(Argument::Text(Text::new(pd.raw)));
            }
        } else {
            entries.push(Argument::Text(Text::new(line)));
        }
        consumed = line_idx - start;
        idx = line_idx;
    }

    let list_type = match bullet {
        '*' => ListType::Unordered,
        '#' => ListType::Ordered,
        ';' | ':' => ListType::Definition,
        other => ListType::Other(other.to_string()),
    };

    Some((consumed, List { list_type, entries }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_template() {
        let s = "{{Infobox|name=Test|value=42|pos1|pos2}}";
        let pd = parse_wikitext_fragment(s).expect("parse");
        assert_eq!(pd.elements.len(), 1);
        if let Argument::Template(t) = &pd.elements[0] {
            assert_eq!(t.name.to_lowercase(), "infobox");
            assert_eq!(t.arguments.len(), 4);
            assert!(t.get_named_arg("name").is_ok());
            let p0 = t.get_positional_arg(0).unwrap();
            assert_eq!(p0.raw, "pos1");
        } else {
            panic!("expected template");
        }
    }

    #[test]
    fn parse_links() {
        let s = "Before [[Page|Label]] middle [[Other]] after";
        let pd = parse_wikitext_fragment(s).expect("parse");
        let links = pd.get_links(None);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].label, "Label");
        assert_eq!(links[1].label, "Other");
    }

    #[test]
    fn nested_templates() {
        let s = "{{A|x={{B|1}}|y=foo}}";
        let pd = parse_wikitext_fragment(s).expect("parse");
        if let Argument::Template(t) = &pd.elements[0] {
            assert_eq!(t.name, "A");
            let x = t.get_named_arg("x").expect("x");
            if let Argument::Template(tb) = &x.elements[0] {
                assert_eq!(tb.name, "B");
            } else {
                panic!("expected nested template B");
            }
        } else {
            panic!("expected template A");
        }
    }

    #[test]
    fn lists_parsing() {
        let s = "* Item A\n* Item B\n# One\n";
        let pd = parse_wikitext_fragment(s).expect("parse");
        let mut found_lists = 0;
        for e in pd.elements {
            if let Argument::List(_) = e {
                found_lists += 1;
            }
        }
        assert!(found_lists >= 1);
    }
}
