//! Thin `WikiText` wrapper backed by the parser in `parsed_data`.
//!
//! This module implements the small, lazy API described in `spec.md`:
//! - `WikiText::parse(input) -> WikiText` (lazy parse, cached)
//! - `WikiText::get_parsed(&mut) -> Result<&ParsedData, WtError>`
//! - `page_name` getter/setter and `text()` accessor
//!
//! The implementation keeps ownership of all parsed data internally so callers
//! can clone or take ownership as needed.

use crate::wikitext::errors::WtError;
use crate::wikitext::parsed_data::{ParsedData, parse_wikitext_fragment};

/// Wrapper around a wikitext string that lazily parses on demand and caches
/// the `ParsedData`.
#[derive(Debug, Clone)]
pub struct WikiText {
    text: String,
    page_name: Option<String>,
    parsed: Option<ParsedData>,
}

impl WikiText {
    /// Create a new `WikiText` wrapper from `input`. Parsing is lazy and will
    /// only occur when `get_parsed` is called.
    pub fn parse<S: Into<String>>(input: S) -> Self {
        Self {
            text: input.into(),
            page_name: None,
            parsed: None,
        }
    }

    /// Return a reference to the cached `ParsedData`, parsing the underlying
    /// text on first access. The parsed value is cached so repeated calls do
    /// not re-parse unless you construct a fresh `WikiText`.
    ///
    /// Note: the method takes `&mut self` because parsing stores the cached
    /// result inside the struct.
    pub fn get_parsed(&mut self) -> Result<&ParsedData, WtError> {
        if self.parsed.is_none() {
            // Delegate to the parser in the parsed_data module. It already
            // returns a `WtError` on failure.
            let parsed = parse_wikitext_fragment(&self.text)?;
            self.parsed = Some(parsed);
        }
        // Safe unwrap: parsed is guaranteed to be Some by the code above.
        Ok(self.parsed.as_ref().unwrap())
    }

    /// Return a clone of the optional page name.
    pub fn page_name(&self) -> Option<String> {
        self.page_name.clone()
    }

    /// Set the optional page name. Accepts `None` to clear it.
    pub fn set_page_name<S: Into<String>>(&mut self, page_name: Option<S>) {
        self.page_name = page_name.map(|s| s.into());
    }

    /// Return the original text (owned `String`).
    pub fn text(&self) -> String {
        self.text.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wiki_text_lazy_parse_and_cache() {
        let mut wt = WikiText::parse("Plain text {{T|x=1}} trailing");
        assert_eq!(wt.page_name(), None);
        // before parsing, parsed is None internally; calling get_parsed parses it
        let pd = wt.get_parsed().expect("should parse");
        // parsed data should reflect the input (raw field contains original input)
        assert!(pd.raw.contains("Plain text"));
        // second call should return cached reference (no error)
        let _pd2 = wt.get_parsed().expect("cached");
    }

    #[test]
    fn page_name_setter_getter() {
        let mut wt = WikiText::parse("dummy");
        assert!(wt.page_name().is_none());
        wt.set_page_name(Some("TestPage"));
        assert_eq!(wt.page_name().as_deref(), Some("TestPage"));
        wt.set_page_name::<&str>(None);
        assert!(wt.page_name().is_none());
    }
}
