//! Template parsing helpers and Template data types.
//!
//! This module contains the `Template` and `TemplateArgument` types together
//! with conservative parsing utilities for templates of the form
//! `{{Name|arg|name=value|...}}`.
//!
//! The implementation is intentionally defensive: it only splits top-level
//! separators and preserves nested constructs by delegating to the project's
//! `parse_wikitext_fragment` for argument values.

use crate::wikitext::enums::QueryType;
use crate::wikitext::errors::WtError;
use crate::wikitext::parsed_data::ParsedData;
use crate::wikitext::parsed_data::parse_wikitext_fragment;

/// Template argument value - represented as `ParsedData` so it may contain
/// nested templates/links/lists/etc.
#[derive(Debug, Clone)]
pub struct TemplateArgument {
    pub name: Option<String>,
    pub value: ParsedData,
}

impl TemplateArgument {
    /// Reconstruct the argument as wikitext: either `name=value` or a positional value.
    pub fn to_wikitext(&self) -> String {
        let val = self.value.to_wikitext();
        if let Some(ref n) = self.name {
            format!("{}={}", n, val)
        } else {
            val
        }
    }
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
                && n.eq_ignore_ascii_case(name)
            {
                return Ok(arg.value.clone());
            }
        }
        Err(WtError::not_found(format!(
            "Named argument '{}' not found in template '{}'",
            name, self.name
        )))
    }

    /// Convenience: return the raw string value (the `ParsedData.raw`) of the
    /// first named argument matching `name`.
    pub fn get_named_arg_raw(&self, name: &str) -> Result<String, WtError> {
        self.get_named_arg(name).map(|pd| pd.raw)
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

    /// Convenience: return raw strings of all named args matching `query`.
    pub fn get_named_args_query_raw(&self, query: &str, qtype: QueryType) -> Vec<String> {
        self.get_named_args_query(query, qtype)
            .into_iter()
            .map(|pd| pd.raw)
            .collect()
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

    /// Convenience: return the raw string value of the positional argument.
    pub fn get_positional_arg_raw(&self, pos: usize) -> Result<String, WtError> {
        self.get_positional_arg(pos).map(|pd| pd.raw)
    }

    /// Reconstruct a wikitext representation of this template.
    /// Produces `{{Name|arg|name=value|...}}` approximating the original.
    pub fn to_wikitext(&self) -> String {
        let mut s = String::new();
        s.push_str("{{");
        s.push_str(&self.name);
        for arg in &self.arguments {
            s.push('|');
            s.push_str(&arg.to_wikitext());
        }
        s.push_str("}}");
        s
    }
}

/// Parse a template starting at `start` (expects "{{").
///
/// Returns (consumed_bytes, Template) on success. This routine is conservative:
/// it counts nested braces when locating the end of the template and then
/// delegates to `parse_template_content` to split the inner content.
pub fn parse_template_at(input: &str, start: usize) -> Option<(usize, Template)> {
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
///
/// This splits top-level arguments by '|' and supports named `name=value` args.
/// Splitting is top-level aware: '|' characters inside nested {{ }} or [[ ]]
/// or tags <...> will be ignored.
pub fn parse_template_content(content: &str) -> Result<Template, String> {
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
///
/// This implementation operates on char boundaries and keeps a simple stack for
/// double-brace and double-bracket constructs.
pub fn split_top_level(s: &str, sep: char) -> Vec<String> {
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
///
/// Returns the byte index of the occurrence suitable for `split_at`.
pub fn find_top_level_char(s: &str, c: char) -> Option<usize> {
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
