//! Core parsed-data types and wikitext parsing helpers.
//!
//! This module implements the primary data structures required by the spec:
//! - `Text`
//! - `Link`
//! - `Template` (+ `TemplateArgument`)
//! - `List`
//! - `TableCell` / `Table`
//! - `Argument` (top-level variant carrying the above)
//! - `ParsedData` (owner of parsed elements)
//!
//! It also provides a reasonably small, resilient parser implemented as
//! utility functions. Parsing is conservative: it only extracts top-level
//! templates, links, lists and simple tables, keeping the rest as `Text` nodes.
//!
//! The API is designed so `ParsedData` and its contained elements are fully
//! owned and can be cloned by callers as needed.

use crate::wikitext::enums::{LinkType, ListType, QueryType};
use crate::wikitext::errors::WtError;

/// Helper: check whether the byte slice starting at `pos` begins with
/// ASCII "http" or "https" (case-insensitive).
fn starts_with_http(bytes: &[u8], pos: usize) -> bool {
    let len = bytes.len();
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

pub use crate::wikitext::types::links::Link;

pub use crate::wikitext::types::templates::{Template, TemplateArgument};

/// Table types were moved into `types/table.rs`. Re-export the `TableCell` type
/// so existing code in this module can continue to reference it by name.
pub use super::types::table::TableCell;

/// `Table` was moved into `types/table.rs`. Re-export it here so other code
/// in this module continues to refer to `Table` without needing to change paths.
pub use super::types::table::Table;

/// A list node containing entries which are top-level arguments (text/templates/etc).
#[derive(Debug, Clone)]
pub struct List {
    pub list_type: ListType,
    pub entries: Vec<Argument>,
}

impl List {
    /// Reconstruct the list as wikitext. Uses a marker for list type.
    pub fn to_wikitext(&self) -> String {
        let marker = match &self.list_type {
            ListType::Unordered => "*",
            ListType::Ordered => "#",
            ListType::Definition => ";",
            ListType::Other(s) => s.as_str(),
        };
        let mut out = String::new();
        for entry in &self.entries {
            out.push_str(marker);
            out.push(' ');
            out.push_str(&entry.to_wikitext());
            out.push('\n');
        }
        out
    }
}

/// Top-level argument - variant for every kind of parsed component.
#[derive(Debug, Clone)]
pub enum Argument {
    Template(Template),
    Link(Link),
    List(List),
    Table(Table),
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
    pub fn into_table(self) -> Option<Table> {
        match self {
            Argument::Table(t) => Some(t),
            _ => None,
        }
    }

    /// Reconstruct this argument into wikitext.
    pub fn to_wikitext(&self) -> String {
        match self {
            Argument::Text(t) => t.raw.clone(),
            Argument::Link(l) => l.to_wikitext(),
            Argument::Template(t) => t.to_wikitext(),
            Argument::List(ls) => ls.to_wikitext(),
            Argument::Table(tb) => tb.to_wikitext(),
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

    /// Return top-level tables parsed in this fragment.
    pub fn get_tables(&self) -> Vec<Table> {
        let mut out: Vec<Table> = Vec::new();
        for elem in &self.elements {
            if let Argument::Table(tb) = elem {
                out.push(tb.clone());
            }
        }
        out
    }

    /// Find the first table whose title matches `title` (case-insensitive).
    /// If the provided title is empty, returns None.
    pub fn get_table_by_title(&self, title: &str) -> Option<Table> {
        if title.trim().is_empty() {
            return None;
        }
        let title_lc = title.to_lowercase();
        for elem in &self.elements {
            if let Argument::Table(tb) = elem
                && let Some(ref t) = tb.title
                && t.to_lowercase() == title_lc
            {
                return Some(tb.clone());
            }
        }
        None
    }

    /// Alias for `get_table_by_title`. Provided for API compatibility (search by name).
    pub fn get_table_by_name(&self, name: &str) -> Option<Table> {
        self.get_table_by_title(name)
    }

    /// Return nth top-level element (0-based). If out of bounds returns an error.
    pub fn get(&self, nth: usize) -> Result<Argument, WtError> {
        if nth < self.elements.len() {
            Ok(self.elements[nth].clone())
        } else {
            Err(WtError::index_oob(nth, self.elements.len()))
        }
    }

    /// Return the raw textual wikitext for the nth element. This provides a
    /// helper to obtain the "raw" value (as a string) rather than the
    /// structured `Argument`. Useful when caller wants to operate on plain
    /// wikitext or the original raw content of the argument.
    pub fn get_raw(&self, nth: usize) -> Result<String, WtError> {
        if nth < self.elements.len() {
            let elem = &self.elements[nth];
            match elem {
                Argument::Text(t) => Ok(t.raw.clone()),
                Argument::Link(l) => Ok(l.to_wikitext()),
                Argument::Template(tpl) => Ok(tpl.to_wikitext()),
                Argument::List(lst) => Ok(lst.to_wikitext()),
                Argument::Table(tb) => Ok(tb.to_wikitext()),
            }
        } else {
            Err(WtError::index_oob(nth, self.elements.len()))
        }
    }

    /// Reconstruct the wikitext for this ParsedData by concatenating element wikitexts.
    /// If there are no parsed elements, fall back to the original raw string.
    pub fn to_wikitext(&self) -> String {
        if self.elements.is_empty() {
            return self.raw.clone();
        }
        let mut out = String::new();
        for elem in &self.elements {
            out.push_str(&elem.to_wikitext());
        }
        out
    }
}

/// Parse a wikitext fragment into `ParsedData`.
///
/// The parser extracts top-level:
/// - templates ({{...}}) with nesting support
/// - internal links ([[...]])
/// - external links ([http... label])
/// - simple list blocks (lines starting with *, #, ;, :)
/// - simple tables ({| ... |})
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

        // detect table start "{|"
        if idx + 1 < len && bytes[idx] == b'{' && bytes[idx + 1] == b'|' {
            if !current_text.is_empty() {
                pd.elements
                    .push(Argument::Text(Text::new(current_text.clone())));
                current_text.clear();
            }
            if let Some((consumed, table)) =
                crate::wikitext::types::table::parse_table_at(input, idx)
            {
                pd.elements.push(Argument::Table(table));
                idx += consumed;
                continue;
            } else {
                // treat as literal "{|"
                current_text.push_str("{|");
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
                // Inspect the next Unicode scalar (char) safely instead of taking a raw byte.
                let ch = input[idx + ws..].chars().next().unwrap();
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

        // default: append next UTF-8 char to current_text
        let ch = input[idx..].chars().next().unwrap();
        current_text.push(ch);
        idx += ch.len_utf8();
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
            let ch = input[idx..].chars().next().unwrap();
            content.push(ch);
            idx += ch.len_utf8();
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
    // Operate on char boundaries to be UTF-8 safe. We iterate over the
    // char_indices so we can both examine characters and still return
    // byte-accurate positions when needed elsewhere.
    let mut parts = Vec::new();
    let mut cur = String::new();

    let chs: Vec<(usize, char)> = s.char_indices().collect();
    let mut i = 0usize;
    let n = chs.len();
    let mut depth_brace = 0usize;
    let mut depth_bracket = 0usize;
    let mut in_tag = false;

    while i < n {
        let (_byte_pos, ch) = chs[i];
        if ch == '{' && i + 1 < n && chs[i + 1].1 == '{' {
            depth_brace += 1;
            cur.push_str("{{");
            i += 2;
            continue;
        } else if ch == '}' && i + 1 < n && chs[i + 1].1 == '}' {
            depth_brace = depth_brace.saturating_sub(1);
            cur.push_str("}}");
            i += 2;
            continue;
        } else if ch == '[' && i + 1 < n && chs[i + 1].1 == '[' {
            depth_bracket += 1;
            cur.push_str("[[");
            i += 2;
            continue;
        } else if ch == ']' && i + 1 < n && chs[i + 1].1 == ']' {
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
    // Use char-aware iteration and return the byte index (from char_indices)
    // of the top-level occurrence of `c`. This avoids slicing at invalid
    // UTF-8 boundaries and ensures returned index can be used with split_at.
    let chs: Vec<(usize, char)> = s.char_indices().collect();
    let mut i = 0usize;
    let n = chs.len();
    let mut depth_brace = 0usize;
    let mut depth_bracket = 0usize;
    let mut in_tag = false;

    while i < n {
        let (byte_pos, ch) = chs[i];
        if ch == '{' && i + 1 < n && chs[i + 1].1 == '{' {
            depth_brace += 1;
            i += 2;
            continue;
        } else if ch == '}' && i + 1 < n && chs[i + 1].1 == '}' {
            depth_brace = depth_brace.saturating_sub(1);
            i += 2;
            continue;
        } else if ch == '[' && i + 1 < n && chs[i + 1].1 == '[' {
            depth_bracket += 1;
            i += 2;
            continue;
        } else if ch == ']' && i + 1 < n && chs[i + 1].1 == ']' {
            depth_bracket = depth_bracket.saturating_sub(1);
            i += 2;
            continue;
        } else if ch == '<' {
            in_tag = true;
            i += 1;
            continue;
        } else if ch == '>' {
            in_tag = false;
            i += 1;
            continue;
        }

        if ch == c && depth_brace == 0 && depth_bracket == 0 && !in_tag {
            return Some(byte_pos);
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
            let ch = input[idx..].chars().next().unwrap();
            content.push(ch);
            idx += ch.len_utf8();
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
        let ch = input[idx..].chars().next().unwrap();
        if ch == ']' {
            idx += ch.len_utf8();
            break;
        } else {
            content.push(ch);
            idx += ch.len_utf8();
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
    // Determine the bullet by reading the next UTF-8 char (handles multibyte chars safely).
    let bullet = input[idx..].chars().next().unwrap();
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
        // capture line content until newline (properly handling UTF-8 chars)
        let mut line = String::new();
        while line_idx < len {
            let ch = input[line_idx..].chars().next().unwrap();
            if ch == '\n' {
                line_idx += ch.len_utf8();
                break;
            } else {
                line.push(ch);
                line_idx += ch.len_utf8();
            }
        }
        // parse the line content as fragment
        if let Ok(pd) = parse_wikitext_fragment(line.trim()) {
            if pd.elements.len() == 1 {
                entries.push(pd.elements[0].clone());
            } else {
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
    fn parse_simple_template_and_link() {
        let s = "{{Infobox|name=Test|value=42|pos1|pos2}} Something [[Page|Label]]";
        let pd = parse_wikitext_fragment(s).expect("parse");
        // should contain a template and a link
        let tpl = pd.get_template("Infobox");
        assert!(tpl.is_ok());
        let links = pd.get_links(None);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].label, "Label");
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

    #[test]
    fn unicode_garden_of_eeshol() {
        // Ensure UTF-8 characters are preserved and parsed as a single Text element.
        let s = "Garden_of_Eeshöl";
        let pd = parse_wikitext_fragment(s).expect("parse");
        assert_eq!(pd.elements.len(), 1);
        if let Argument::Text(t) = &pd.elements[0] {
            assert_eq!(t.raw, "Garden_of_Eeshöl");
        } else {
            panic!("expected a Text element");
        }
    }

    #[test]
    fn internal_link_unicode_target() {
        let s = "[[Garden_of_Eeshöl]]";
        let pd = parse_wikitext_fragment(s).expect("parse");
        assert_eq!(pd.elements.len(), 1);
        if let Argument::Link(l) = &pd.elements[0] {
            assert_eq!(l.target, "Garden_of_Eeshöl");
            assert_eq!(l.label, "Garden_of_Eeshöl");
        } else {
            panic!("expected a Link element");
        }
    }

    #[test]
    fn simple_table_parse() {
        let s =
            "{| class=\"wikitable\"\n|+ Title\n! Header1 !! Header2\n|-\n| A || B\n| C || D\n|}";
        let pd = parse_wikitext_fragment(s).expect("parse table");
        assert_eq!(pd.elements.len(), 1);
        if let Argument::Table(tb) = &pd.elements[0] {
            assert_eq!(tb.class.as_deref(), Some("wikitable"));
            assert_eq!(tb.title.as_deref(), Some("Title"));
            // rows should include header row + two data rows
            assert!(tb.rows.len() >= 3);
            // header names present
            assert!(tb.headers.contains(&"Header1".to_string()));
            assert!(tb.headers.contains(&"Header2".to_string()));
            // check a specific cell raw content
            let maybe = tb.get_cell_by_index(1, 0);
            assert!(maybe.is_some());
            let c = maybe.unwrap();
            assert_eq!(c.content.raw, "A");
        } else {
            panic!("expected table");
        }
    }

    #[test]
    fn table_api_mini_tower_sample() {
        let s = r#"{| class="sortable mw-collapsible mw-collapsed wikitable" width="100%" style="text-align:center;"
! colspan="4" |Mini Tower List
|-
!Difficulty
!Name
!Location
!Difficulty Num
|-
| data-sort-value="3" |{{Difficulty|3}}
|TNF - [[Tower Not Found]]
|{{Emblem|R0}} [[Ring 0]]
|3.11
|-
| data-sort-value="1" |{{Difficulty|1}}
|NEAT - [[Not Even A Tower]]
|{{Emblem|R1}} [[Ring 1]]
|1.11
|-
| data-sort-value="3" |{{Difficulty|3}}
|TIPAT - [[This Is Probably A Tower]]
|{{Emblem|FR}} [[Forgotten Ridge]]
|3.61
|-
| data-sort-value="1" |{{Difficulty|1}}
|MAT - [[Maybe A Tower]]
|{{Emblem|R2}} [[Ring 2]]
|1.07
|-
| data-sort-value="5" |{{Difficulty|5}}
|NEAF - [[Not Even A Flower]]
|{{Emblem|GoE}} [[Garden of Eeshöl]]
|5.79
|}"#;

        let pd = parse_wikitext_fragment(s).expect("parse mini table");

        // ParsedData.get_tables()
        let tables = pd.get_tables();
        assert_eq!(tables.len(), 1, "expected a single table parsed");

        // ParsedData.get_table_by_name()
        let tb = pd
            .get_table_by_name("Mini Tower List")
            .expect("table should be findable by name");

        // Table.get_headers()
        let headers = tb.get_headers();
        assert_eq!(
            headers,
            vec![
                "Difficulty".to_string(),
                "Name".to_string(),
                "Location".to_string(),
                "Difficulty Num".to_string()
            ]
        );

        // Find first data row by locating a Difficulty template in the first column
        let mut data_row: Option<usize> = None;
        for (i, _row) in tb.get_rows().iter().enumerate() {
            if let Some(c) = tb.get_cell_by_index(i, 0) {
                if c.content.get_template("Difficulty").is_ok() {
                    data_row = Some(i);
                    break;
                }
            }
        }
        let r_idx = data_row.expect("should find a data row with a Difficulty template");

        // Table.get_row() -> Row and Row.raw()
        let row = tb.get_row(r_idx).expect("row wrapper available");
        let row_raw = row.raw();
        assert!(row_raw.contains("{{Difficulty|3}}"));
        assert!(row_raw.contains("TNF - [[Tower Not Found]]"));
        assert!(row_raw.contains("{{Emblem|R0}}"));
        assert!(row_raw.contains("3.11"));

        // Row.get_cell_from_col() -> Cell and Cell.raw
        let name_cell_row = row
            .get_cell_from_col("Name")
            .expect("name cell from row should exist");
        assert!(name_cell_row.raw().contains("TNF - [[Tower Not Found]]"));

        // Table.get_cell(row, col) -> Cell
        let name_cell_tbl = tb.get_cell(r_idx, "Name").expect("name cell via table");
        assert_eq!(name_cell_row.raw(), name_cell_tbl.raw());

        // Cell.get_class() (attributes)
        let diff_cell = tb.get_cell(r_idx, "Difficulty").expect("difficulty cell");
        assert_eq!(diff_cell.get_class(), "data-sort-value=\"3\"");

        // Cell.get_parsed() returns ParsedData and contains template + links
        let loc_cell = tb.get_cell(r_idx, "Location").expect("location cell");
        assert!(loc_cell.get_parsed().get_template("Emblem").is_ok());
        let links = loc_cell.get_parsed().get_links(None);
        assert!(links.iter().any(|l| l.label == "Ring 0"));

        // Ensure direct Table.get_cell_by_index still accessible for low-level checks
        let maybe = tb.get_cell_by_index(r_idx, 3);
        assert!(maybe.is_some());
        let c = maybe.unwrap();
        assert_eq!(c.content.raw, "3.11");
    }
    #[test]
    fn table_get_cell_numeric_index() {
        let s = r#"{| class="sortable mw-collapsible mw-collapsed wikitable" width="100%" style="text-align:center;"
! colspan="4" |Mini Tower List
|-
!Difficulty
!Name
!Location
!Difficulty Num
|-
| data-sort-value="3" |{{Difficulty|3}}
|TNF - [[Tower Not Found]]
|{{Emblem|R0}} [[Ring 0]]
|3.11
|}"#;

        let pd = parse_wikitext_fragment(s).expect("parse mini table for numeric index test");
        let tb = pd
            .get_table_by_name("Mini Tower List")
            .expect("table by name");

        // find data row
        let mut data_row: Option<usize> = None;
        for (i, _r) in tb.get_rows().iter().enumerate() {
            if let Some(c) = tb.get_cell_by_index(i, 0) {
                if c.content.get_template("Difficulty").is_ok() {
                    data_row = Some(i);
                    break;
                }
            }
        }
        let r_idx = data_row.expect("should find a data row");

        // Table.get_cell by numeric string index
        let c_by_str = tb.get_cell(r_idx, "0").expect("cell by numeric string");
        let c_by_idx = tb.get_cell_by_index(r_idx, 0).expect("cell by index");
        assert_eq!(c_by_str.raw(), c_by_idx.content.raw);

        // Row.get_cell_from_col by numeric string index
        let row = tb.get_row(r_idx).unwrap();
        let c_row_by_str = row
            .get_cell_from_col("3")
            .expect("cell by numeric col on row");
        let c_by_idx_3 = tb.get_cell_by_index(r_idx, 3).expect("cell by index 3");
        assert_eq!(c_row_by_str.raw(), c_by_idx_3.content.raw);
    }
}
