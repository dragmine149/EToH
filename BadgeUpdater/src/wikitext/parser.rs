use crate::wikitext::parts::{ArgPart, Argument, MatchType, Template};
use serde::{Deserialize, Serialize};
use url::Url;

/// Result of parsing: top-level templates found in the wikitext.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ParseResult {
    pub templates: Vec<Template>,
}

impl ParseResult {
    pub fn new() -> Self {
        ParseResult {
            templates: Vec::new(),
        }
    }

    /// Unified lookup: find the first top-level template whose name matches
    /// `name` according to `mode` (case-insensitive).
    pub fn get_template_by(&self, name: &str, mode: MatchType) -> Option<&Template> {
        let target = name.trim().to_lowercase();
        if target.is_empty() {
            return None;
        }
        for t in &self.templates {
            let cand = t.name.trim().to_lowercase();
            match mode {
                MatchType::Exact => {
                    if cand == target {
                        return Some(t);
                    }
                }
                MatchType::StartsWith => {
                    if cand.starts_with(&target) {
                        return Some(t);
                    }
                }
            }
        }
        None
    }

    /// Backwards-compatible: exact-match lookup.
    pub fn get_template_by_name(&self, name: &str) -> Option<&Template> {
        self.get_template_by(name, MatchType::Exact)
    }

    /// Backwards-compatible: starts-with lookup.
    pub fn get_template_startswith(&self, name: &str) -> Option<&Template> {
        self.get_template_by(name, MatchType::StartsWith)
    }
}

/// Parse top-level templates from the input wikitext.
///
/// This returns only templates that are not nested inside another template;
/// nested templates are represented inside their parent's argument parts as
/// `ArgPart::Template`.
pub fn parse_templates(input: &str) -> ParseResult {
    let mut res = ParseResult::new();
    let bytes = input.as_bytes();
    let mut i: usize = 0;

    while i + 1 < bytes.len() {
        if bytes[i] == b'{' && bytes[i + 1] == b'{' {
            // Found a template opening. Find its matching closing by tracking depth.
            let start = i + 2;
            i = start;
            let mut depth = 1usize;

            while i + 1 < bytes.len() && depth > 0 {
                if bytes[i] == b'{' && bytes[i + 1] == b'{' {
                    depth += 1;
                    i += 2;
                    continue;
                }
                if bytes[i] == b'}' && bytes[i + 1] == b'}' {
                    depth -= 1;
                    i += 2;
                    continue;
                }
                // advance by current char
                let ch = input[i..].chars().next().unwrap();
                i += ch.len_utf8();
            }

            if depth == 0 {
                let end = i - 2; // end is exclusive for body slice
                if let Some(body) = input.get(start..end) {
                    if let Some(tpl) = parse_single_template(body) {
                        res.templates.push(tpl);
                    }
                }
                // continue scanning from i (already advanced past '}}')
                continue;
            } else {
                // malformed template; stop scanning to avoid infinite loop
                break;
            }
        }

        // Otherwise advance one char
        let ch = input[i..].chars().next().unwrap();
        i += ch.len_utf8();
    }

    res
}

/// Attempt to parse a redirect target from the provided wikitext.
///
/// This looks for a leading "#REDIRECT [[Target]]" (case-insensitive),
/// using the first non-empty line. Returns the target page title (text
/// before any pipe) if present.
fn parse_redirect(input: &str) -> Option<String> {
    for line in input.lines() {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }
        // Check for case-insensitive "#redirect"
        let low = l.to_lowercase();
        if low.starts_with("#redirect") {
            // Prefer forms that include [[...]]
            if let Some(start) = l.find("[[") {
                if let Some(rel_end) = l[start + 2..].find("]]") {
                    let inner = &l[start + 2..start + 2 + rel_end];
                    let target = inner.split('|').next().unwrap_or("").trim().to_string();
                    if !target.is_empty() {
                        return Some(target);
                    }
                }
            }
            // Fallback: "#REDIRECT: Target page" style (rare)
            if let Some(colon_pos) = l.find(':') {
                let rest = l[colon_pos + 1..].trim();
                if !rest.is_empty() {
                    return Some(rest.to_string());
                }
            }
            // If we matched #redirect but no target found, return None
            return None;
        }
        // If the first non-empty line isn't a redirect, stop scanning.
        break;
    }
    None
}

/// Unified in-memory representation for parsed wikitext.
///
/// This holds the raw page text and lazily computed artifacts:
/// - parsed templates (`ParseResult`)
/// - redirect target (if any)
/// - external links found on the page
///
/// Computation is deferred until the corresponding getter is called.
/// Caches are protected by a `Mutex` so the type is safe to share across threads.
#[derive(Debug, Clone)]
pub struct WikiText {
    /// Raw page text (canonical source of truth)
    pub raw: String,

    /// Name of the page for future reference. Optional as we might only parse part of a page sometimes.
    pub page_name: Option<String>,

    /// Cached parsed templates (set once when computed)
    parsed_cache: std::sync::OnceLock<ParseResult>,

    /// Cached redirect: OnceLock holds Option<String> (None means no redirect)
    redirect_cache: std::sync::OnceLock<Option<String>>,

    /// Cached external links
    external_links_cache: std::sync::OnceLock<Vec<Url>>,
}

impl WikiText {
    /// Create a lazy `WikiText` wrapper around the raw text.
    /// Nothing is parsed at construction time.
    pub fn parse(input: &str) -> Self {
        WikiText {
            raw: input.to_string(),
            page_name: None,
            parsed_cache: std::sync::OnceLock::new(),
            redirect_cache: std::sync::OnceLock::new(),
            external_links_cache: std::sync::OnceLock::new(),
        }
    }

    /// Lazily obtain the parsed templates. Parsing is performed at most once and cached.
    pub fn get_parsed(&self) -> ParseResult {
        if let Some(parsed_ref) = self.parsed_cache.get() {
            return parsed_ref.clone();
        }
        // Compute and set (set can fail if another thread set it concurrently; ignore result)
        let parsed = parse_templates(&self.raw);
        let _ = self.parsed_cache.set(parsed.clone());
        parsed
    }

    /// Ensure the parsed templates are present and return a reference to the stored ParseResult.
    /// This returns a borrow into the internal cache (no cloning) and is useful for
    /// returning `&Template` references from callers.
    fn ensure_parsed_ref(&self) -> &ParseResult {
        if let Some(parsed_ref) = self.parsed_cache.get() {
            return parsed_ref;
        }
        let parsed = parse_templates(&self.raw);
        // set the cache (ignore Err on concurrent set)
        let _ = self.parsed_cache.set(parsed);
        // now it must be present
        self.parsed_cache.get().expect("parsed_cache should be set")
    }

    /// Convenience: return a reference to the first top-level template with exact `name`
    /// (case-insensitive). This borrows from the cached parsed result so no cloning occurs.
    pub fn get_template_by_name(&self, name: &str) -> Option<&Template> {
        if name.trim().is_empty() {
            return None;
        }
        let parsed = self.ensure_parsed_ref();
        parsed.get_template_by_name(name)
    }

    /// Convenience: return a reference to the first top-level template whose name starts with
    /// `name` (case-insensitive). Borrows from cached parsed result.
    pub fn get_template_startswith(&self, name: &str) -> Option<&Template> {
        if name.trim().is_empty() {
            return None;
        }
        let parsed = self.ensure_parsed_ref();
        parsed.get_template_startswith(name)
    }

    /// Return the redirect target (page name) if the page is a redirect.
    /// The redirect detection is computed lazily and cached.
    pub fn get_redirect(&self) -> Option<String> {
        if let Some(opt) = self.redirect_cache.get() {
            return opt.clone();
        }
        let computed = parse_redirect(&self.raw);
        let _ = self.redirect_cache.set(computed.clone());
        computed
    }

    /// Return all external links found on the page as a vector of Urls.
    /// This is computed lazily and cached.
    pub fn get_external_links(&self) -> Vec<Url> {
        if let Some(links_ref) = self.external_links_cache.get() {
            return links_ref.clone();
        }

        // Collect external links from the raw page parts
        let mut links: Vec<Url> = Vec::new();
        collect_external_links_from_parts(&parse_parts(&self.raw), &mut links);

        // Also traverse parsed templates (which will be computed lazily by get_parsed)
        let parsed = self.get_parsed();
        for tpl in &parsed.templates {
            collect_external_links_from_template(tpl, &mut links);
        }

        // Deduplicate while preserving order
        let mut seen = std::collections::HashSet::new();
        links.retain(|u| seen.insert(u.as_str().to_string()));

        let _ = self.external_links_cache.set(links.clone());
        links
    }
}

/// Recursively collect external links from a slice of ArgPart values into `out`.
fn collect_external_links_from_parts(parts: &[ArgPart], out: &mut Vec<Url>) {
    for part in parts {
        match part {
            ArgPart::ExternalLink { url, .. } => out.push(url.clone()),
            ArgPart::Template(tpl) => collect_external_links_from_template(tpl, out),
            _ => {}
        }
    }
}

/// Recursively collect external links from a Template (including nested templates).
fn collect_external_links_from_template(tpl: &Template, out: &mut Vec<Url>) {
    for arg in &tpl.args {
        for part in &arg.value {
            match part {
                ArgPart::ExternalLink { url, .. } => out.push(url.clone()),
                ArgPart::Template(nested) => collect_external_links_from_template(nested, out),
                _ => {}
            }
        }
    }
}

/// Parse a single template body (without outer `{{` `}}`) into a Template
/// with its arguments parsed into `Argument` items.
fn parse_single_template(body: &str) -> Option<Template> {
    let parts = split_top_level_pipes(body);
    if parts.is_empty() {
        return None;
    }
    let name = parts[0].trim().to_string();
    let mut tpl = Template::new(name);

    for raw_arg in parts.into_iter().skip(1) {
        let (maybe_key, val_str) = split_first_equals_top_level(&raw_arg);
        let value_parts = parse_parts(&val_str);
        if let Some(k) = maybe_key {
            // store the raw right-hand side in the Argument so callers that need
            // to inspect raw text (e.g. list parsing) can access it.
            tpl.push_arg(Argument {
                name: Some(k.trim().to_string()),
                value: value_parts,
                raw: Some(val_str.clone()),
            });
        } else {
            // positional argument: no name, hold the parsed parts directly
            tpl.push_arg(Argument {
                name: None,
                value: value_parts,
                raw: Some(val_str.clone()),
            });
        }
    }

    Some(tpl)
}

/// Split on top-level '|' characters, ignoring pipes inside nested templates,
/// links, or angle tags. Returns list of parts (untrimmed).
fn split_top_level_pipes(s: &str) -> Vec<String> {
    let bytes = s.as_bytes();
    let mut out: Vec<String> = Vec::new();
    let mut buf = String::new();
    let mut i = 0usize;
    let mut brace_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut angle_depth = 0usize;

    while i < bytes.len() {
        // detect '{{'
        if i + 1 < bytes.len() && bytes[i] == b'{' && bytes[i + 1] == b'{' {
            brace_depth += 1;
            buf.push_str("{{");
            i += 2;
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b'}' && bytes[i + 1] == b'}' {
            if brace_depth > 0 {
                brace_depth -= 1;
            }
            buf.push_str("}}");
            i += 2;
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b'[' && bytes[i + 1] == b'[' {
            bracket_depth += 1;
            buf.push_str("[[");
            i += 2;
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b']' && bytes[i + 1] == b']' {
            if bracket_depth > 0 {
                bracket_depth -= 1;
            }
            buf.push_str("]]");
            i += 2;
            continue;
        }
        if bytes[i] == b'<' {
            angle_depth += 1;
            let ch = s_char_at(s, i);
            buf.push(ch);
            i += ch.len_utf8();
            continue;
        }
        if bytes[i] == b'>' {
            if angle_depth > 0 {
                angle_depth -= 1;
            }
            let ch = s_char_at(s, i);
            buf.push(ch);
            i += ch.len_utf8();
            continue;
        }

        if bytes[i] == b'|' && brace_depth == 0 && bracket_depth == 0 && angle_depth == 0 {
            out.push(buf);
            buf = String::new();
            i += 1;
            continue;
        }

        let ch = s_char_at(s, i);
        buf.push(ch);
        i += ch.len_utf8();
    }

    out.push(buf);
    out
}

/// Find the first top-level '=' (not inside nested constructs) and split
/// into (Some(left), right) or (None, whole).
fn split_first_equals_top_level(s: &str) -> (Option<String>, String) {
    let bytes = s.as_bytes();
    let mut i = 0usize;
    let mut brace_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut angle_depth = 0usize;

    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'{' && bytes[i + 1] == b'{' {
            brace_depth += 1;
            i += 2;
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b'}' && bytes[i + 1] == b'}' {
            if brace_depth > 0 {
                brace_depth -= 1;
            }
            i += 2;
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b'[' && bytes[i + 1] == b'[' {
            bracket_depth += 1;
            i += 2;
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b']' && bytes[i + 1] == b']' {
            if bracket_depth > 0 {
                bracket_depth -= 1;
            }
            i += 2;
            continue;
        }
        if bytes[i] == b'<' {
            angle_depth += 1;
            let ch = s_char_at(s, i);
            i += ch.len_utf8();
            continue;
        }
        if bytes[i] == b'>' {
            if angle_depth > 0 {
                angle_depth -= 1;
            }
            let ch = s_char_at(s, i);
            i += ch.len_utf8();
            continue;
        }

        let ch = s_char_at(s, i);
        if ch == '=' && brace_depth == 0 && bracket_depth == 0 && angle_depth == 0 {
            let left = s[..i].to_string();
            let right = s[i + ch.len_utf8()..].to_string();
            return (Some(left), right);
        }
        i += ch.len_utf8();
    }

    (None, s.to_string())
}

/// Parse a string value into a vector of ArgPart, recognizing nested templates,
/// internal links `[[...]]`, external links `[http://... label]`, and bare URLs.
fn parse_parts(s: &str) -> Vec<ArgPart> {
    let bytes = s.as_bytes();
    let mut parts: Vec<ArgPart> = Vec::new();
    let mut buf = String::new();
    let mut i = 0usize;

    let mut flush_buf = |buf: &mut String, parts: &mut Vec<ArgPart>| {
        if !buf.is_empty() {
            parts.push(ArgPart::Text(buf.clone()));
            buf.clear();
        }
    };

    while i < bytes.len() {
        // nested template
        if i + 1 < bytes.len() && bytes[i] == b'{' && bytes[i + 1] == b'{' {
            flush_buf(&mut buf, &mut parts);
            let start = i + 2;
            i = start;
            let mut depth = 1usize;
            while i + 1 < bytes.len() && depth > 0 {
                if bytes[i] == b'{' && bytes[i + 1] == b'{' {
                    depth += 1;
                    i += 2;
                    continue;
                }
                if bytes[i] == b'}' && bytes[i + 1] == b'}' {
                    depth -= 1;
                    i += 2;
                    continue;
                }
                let ch = s_char_at(s, i);
                i += ch.len_utf8();
            }
            if depth == 0 {
                let end = i - 2;
                if let Some(inner) = s.get(start..end) {
                    if let Some(nested_tpl) = parse_single_template(inner) {
                        parts.push(ArgPart::Template(nested_tpl));
                        continue;
                    } else {
                        // fallback: treat as text including braces
                        parts.push(ArgPart::Text(format!("{{{{{}}}}}", inner)));
                        continue;
                    }
                }
            } else {
                // malformed: push literal "{{"
                buf.push_str("{{");
                continue;
            }
        }

        // internal link
        if i + 1 < bytes.len() && bytes[i] == b'[' && bytes[i + 1] == b'[' {
            flush_buf(&mut buf, &mut parts);
            let (consumed, link_part) = parse_internal_link(&s[i..]);
            if consumed == 0 {
                // malformed; treat "[[" as literal
                buf.push_str("[[");
                i += 2;
                continue;
            }
            parts.push(link_part);
            i += consumed;
            continue;
        }

        // external link using single brackets
        if bytes[i] == b'[' {
            flush_buf(&mut buf, &mut parts);
            let (consumed, maybe_part) = parse_external_link(&s[i..]);
            if consumed == 0 || maybe_part.is_none() {
                // not an external link -> treat '[' as text
                buf.push('[');
                i += 1;
                continue;
            }
            parts.push(maybe_part.unwrap());
            i += consumed;
            continue;
        }

        // bare URL heuristic
        if is_url_start_at(s, i) {
            flush_buf(&mut buf, &mut parts);
            let (consumed, url_str) = parse_bare_url(&s[i..]);
            if consumed > 0 {
                if let Ok(url) = Url::parse(&url_str) {
                    parts.push(ArgPart::ExternalLink { url, label: None });
                } else {
                    // if parse fails, put as text
                    parts.push(ArgPart::Text(url_str));
                }
                i += consumed;
                continue;
            }
        }

        // default: accumulate char
        let ch = s_char_at(s, i);
        buf.push(ch);
        i += ch.len_utf8();
    }

    if !buf.is_empty() {
        parts.push(ArgPart::Text(buf));
    }

    parts
}

/// Parse an internal link starting at s[0..], which must begin with `[[`.
/// Returns (consumed_bytes, ArgPart::InternalLink) or (0, dummy) on failure.
fn parse_internal_link(s: &str) -> (usize, ArgPart) {
    let bytes = s.as_bytes();
    if bytes.len() < 2 || !(bytes[0] == b'[' && bytes[1] == b'[') {
        return (0, ArgPart::Text(String::new()));
    }
    let mut i = 2usize;
    let mut depth = 1usize;
    let mut buf = String::new();

    while i + 1 <= bytes.len() && depth > 0 {
        if i + 1 < bytes.len() && bytes[i] == b'[' && bytes[i + 1] == b'[' {
            depth += 1;
            buf.push_str("[[");
            i += 2;
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b']' && bytes[i + 1] == b']' {
            depth -= 1;
            if depth == 0 {
                i += 2;
                break;
            }
            buf.push_str("]]");
            i += 2;
            continue;
        }
        let ch = s_char_at(s, i);
        buf.push(ch);
        i += ch.len_utf8();
    }

    if depth != 0 {
        return (0, ArgPart::Text(String::new()));
    }

    // split on the first top-level '|' inside the link text
    let mut target = buf.trim().to_string();
    let mut label: Option<String> = None;
    if let Some(pos) = find_top_level_char(&target, '|') {
        label = Some(target[pos + 1..].trim().to_string());
        target = target[..pos].trim().to_string();
    }

    (i, ArgPart::InternalLink { target, label })
}

/// Parse an external link of the form `[url label]`. Returns (consumed_bytes, Option<ArgPart>).
/// If parsing fails, returns (0, None).
fn parse_external_link(s: &str) -> (usize, Option<ArgPart>) {
    let bytes = s.as_bytes();
    if bytes.is_empty() || bytes[0] != b'[' {
        return (0, None);
    }
    let mut i = 1usize;
    let mut buf = String::new();
    while i < bytes.len() {
        if bytes[i] == b']' {
            i += 1;
            break;
        }
        let ch = s_char_at(s, i);
        buf.push(ch);
        i += ch.len_utf8();
    }
    if buf.is_empty() {
        return (0, None);
    }

    // split on first whitespace for URL vs label
    if let Some(ws) = find_first_whitespace(&buf) {
        let url_part = buf[..ws].trim();
        let label = buf[ws..].trim().to_string();
        if let Ok(url) = Url::parse(url_part) {
            return (
                i,
                Some(ArgPart::ExternalLink {
                    url,
                    label: if label.is_empty() { None } else { Some(label) },
                }),
            );
        } else {
            return (0, None);
        }
    } else {
        // no label: the bracket contains only a URL
        let url_part = buf.trim();
        if let Ok(url) = Url::parse(url_part) {
            return (i, Some(ArgPart::ExternalLink { url, label: None }));
        } else {
            return (0, None);
        }
    }
}

/// Heuristic: detect if a URL-like token starts at index `i` in `s`.
fn is_url_start_at(s: &str, i: usize) -> bool {
    let rest = &s[i..].to_lowercase();
    rest.starts_with("http://")
        || rest.starts_with("https://")
        || rest.starts_with("ftp://")
        || rest.starts_with("mailto:")
}

/// Parse a bare URL from the start of s (stopping at whitespace or common punctuation).
/// Returns (consumed_bytes, url_string).
fn parse_bare_url(s: &str) -> (usize, String) {
    let mut url = String::new();
    let mut consumed = 0usize;
    for (idx, ch) in s.char_indices() {
        if ch.is_whitespace() || ch == ')' || ch == ',' || ch == ']' || ch == '}' || ch == '>' {
            break;
        }
        url.push(ch);
        consumed = idx + ch.len_utf8();
    }
    if consumed == 0 && !s.is_empty() {
        consumed = s.len();
        url = s.to_string();
    }
    (consumed, url)
}

/// Find first top-level occurrence of a character (not inside nested `{{}}` or `[[ ]]`).
fn find_top_level_char(s: &str, target: char) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0usize;
    let mut brace_depth = 0usize;
    let mut bracket_depth = 0usize;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'{' && bytes[i + 1] == b'{' {
            brace_depth += 1;
            i += 2;
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b'}' && bytes[i + 1] == b'}' {
            if brace_depth > 0 {
                brace_depth -= 1;
            }
            i += 2;
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b'[' && bytes[i + 1] == b'[' {
            bracket_depth += 1;
            i += 2;
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b']' && bytes[i + 1] == b']' {
            if bracket_depth > 0 {
                bracket_depth -= 1;
            }
            i += 2;
            continue;
        }
        let ch = s_char_at(s, i);
        if ch == target && brace_depth == 0 && bracket_depth == 0 {
            return Some(i);
        }
        i += ch.len_utf8();
    }
    None
}

/// Find byte index of first whitespace in `s`, or None.
fn find_first_whitespace(s: &str) -> Option<usize> {
    for (i, ch) in s.char_indices() {
        if ch.is_whitespace() {
            return Some(i);
        }
    }
    None
}

/// Helper: read the char at byte index i. Assumes i < s.len() and points to a char boundary.
fn s_char_at(s: &str, i: usize) -> char {
    s[i..].chars().next().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wikitext::parts::{
        ArgPart, ArgQueryKind, ArgQueryResult, MatchType, parts_to_plain,
    };
    use url::Url;

    #[test]
    fn parse_top_level_simple() {
        let s = "{{Infobox|name=Tower|difficulty=5}}";
        let r = parse_templates(s);
        assert_eq!(r.templates.len(), 1);
        let t = &r.templates[0];
        assert_eq!(t.name.to_lowercase(), "infobox");
        // basic value access via query_arg (FirstText) should match old semantics
        match t.query_arg("name", MatchType::Exact, ArgQueryKind::FirstText) {
            Some(ArgQueryResult::Text(s)) => assert_eq!(s, "Tower"),
            _ => panic!("expected name text"),
        }
        match t.query_arg("difficulty", MatchType::Exact, ArgQueryKind::FirstText) {
            Some(ArgQueryResult::Text(s)) => assert_eq!(s, "5"),
            _ => panic!("expected difficulty text"),
        }
    }

    #[test]
    fn parse_nested_template_inside_arg() {
        let s = "{{T|a={{Sub|x=y}}|b=val}}";
        let r = parse_templates(s);
        assert_eq!(r.templates.len(), 1);
        let t = &r.templates[0];
        assert_eq!(t.name, "T");
        let a = t.get_argument("a").expect("expected a");
        // a should contain a nested template as its first part
        assert!(matches!(a.value.first(), Some(ArgPart::Template(_))));
        // query the nested first positional as text via the unified API
        match t.query_arg(
            "a",
            MatchType::Exact,
            ArgQueryKind::NestedFirstPositionalText,
        ) {
            Some(ArgQueryResult::Text(s)) => assert_eq!(s, "y"),
            _ => panic!("expected nested first positional text"),
        }
    }

    #[test]
    fn parse_internal_and_external_links_in_arg() {
        let s = "{{X|a=[[Page|Label]] b=[http://ex.com Ex] c=[[Only]]}}";
        let r = parse_templates(s);
        assert_eq!(r.templates.len(), 1);
        let t = &r.templates[0];
        let a = t.get_argument("a").unwrap();
        // value plain should contain Label and Ex
        let plain = parts_to_plain(&a.value);
        assert!(plain.contains("Label"));
        assert!(plain.contains("Ex"));

        // Also exercise query_arg to fetch parts
        match t.query_arg("a", MatchType::Exact, ArgQueryKind::Parts) {
            Some(ArgQueryResult::Parts(ps)) => {
                let plain2 = parts_to_plain(ps);
                assert!(plain2.contains("Label"));
            }
            _ => panic!("expected parts"),
        }
    }

    #[test]
    fn external_link_url_parsed() {
        let s = "{{X|link=[https://example.org foo]}}";
        let r = parse_templates(s);
        let t = &r.templates[0];
        let a = t.get_argument("link").unwrap();
        // should be exactly one part and be ExternalLink
        assert_eq!(a.value.len(), 1);
        match &a.value[0] {
            ArgPart::ExternalLink { url, label } => {
                assert_eq!(url.as_str(), "https://example.org/");
                assert_eq!(label.as_ref().map(String::as_str), Some("foo"));
            }
            _ => panic!("expected external link"),
        }

        // also via query_arg
        match t.query_arg("link", MatchType::Exact, ArgQueryKind::Parts) {
            Some(ArgQueryResult::Parts(ps)) => {
                assert_eq!(ps.len(), 1);
                match &ps[0] {
                    ArgPart::ExternalLink { url, label } => {
                        assert_eq!(url.as_str(), "https://example.org/");
                        assert_eq!(label.as_ref().map(String::as_str), Some("foo"));
                    }
                    _ => panic!("expected external link"),
                }
            }
            _ => panic!("expected parts"),
        }
    }

    #[test]
    fn parse_bare_url_into_external_link() {
        let s = "{{X|u=http://rust-lang.org}}";
        let r = parse_templates(s);
        let t = &r.templates[0];
        let a = t.get_argument("u").unwrap();
        // plain should include the URL (may not include trailing slash)
        let plain = parts_to_plain(&a.value);
        assert!(plain.contains("http://rust-lang.org"));

        // also ensure query_arg FirstText returns the expected fragment
        match t.query_arg("u", MatchType::Exact, ArgQueryKind::FirstText) {
            Some(ArgQueryResult::Text(s)) => assert!(s.contains("http://rust-lang.org")),
            _ => panic!("expected URL text"),
        }
    }

    #[test]
    fn query_arg_nested_first_positional_text_handles_leading_space() {
        let s = "{{towerinfobox|difficulty= {{DifficultyNum|4.67}}|original_difficulty = {{DifficultyName|3}}}}";
        let r = parse_templates(s);
        let t = &r.templates[0];

        // starts-with match should find 'difficulty' and return nested first positional text "4.67"
        match t.query_arg(
            "difficulty",
            MatchType::StartsWith,
            ArgQueryKind::NestedFirstPositionalText,
        ) {
            Some(ArgQueryResult::Text(s)) => assert!(s.contains("4.67")),
            other => panic!("expected nested numeric text, got {:?}", other),
        }

        // exact match on original_difficulty should find nested template too
        match t.query_arg(
            "original_difficulty",
            MatchType::Exact,
            ArgQueryKind::NestedFirstPositionalText,
        ) {
            Some(ArgQueryResult::Text(s)) => assert!(s.contains("3")),
            other => panic!("expected nested numeric text, got {:?}", other),
        }
    }
}
