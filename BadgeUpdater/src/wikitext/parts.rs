use serde::{Deserialize, Serialize};
use url::Url;

/// A part of an argument's value.
///
/// Examples:
/// - Text("hello")
/// - Template(Template { name: "T", args: [...] })
/// - InternalLink { target: "Page", label: Some("Label") }
/// - ExternalLink { url: Url::parse("http://...").unwrap(), label: None }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ArgPart {
    Text(String),
    Template(Template),
    InternalLink {
        target: String,
        label: Option<String>,
    },
    ExternalLink {
        url: Url,
        label: Option<String>,
    },
}

impl ArgPart {
    /// Convert this part to a human-friendly plain text fragment.
    /// - Text -> returned as-is
    /// - Template -> `Template::to_plain()`
    /// - InternalLink -> label or target
    /// - ExternalLink -> label or url.as_str()
    pub fn to_plain(&self) -> String {
        match self {
            ArgPart::Text(s) => s.clone(),
            ArgPart::Template(t) => t.to_plain(),
            ArgPart::InternalLink { target, label } => label
                .as_ref()
                .map(|l| l.clone())
                .unwrap_or_else(|| target.clone()),
            ArgPart::ExternalLink { url, label } => label
                .as_ref()
                .map(|l| l.clone())
                .unwrap_or_else(|| url.as_str().to_string()),
        }
    }
}

/// Template argument: either positional (no name) or named (key = value).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Argument {
    pub name: Option<String>,
    pub value: Vec<ArgPart>,
}

impl Argument {
    /// Create a simple positional argument from text.
    pub fn positional<T: Into<String>>(text: T) -> Self {
        Argument {
            name: None,
            value: vec![ArgPart::Text(text.into())],
        }
    }

    /// Create a named argument from key and parts.
    pub fn named<T: Into<String>>(key: T, parts: Vec<ArgPart>) -> Self {
        Argument {
            name: Some(key.into()),
            value: parts,
        }
    }

    /// Render this argument's value to plain text by concatenating parts.
    pub fn value_plain(&self) -> String {
        parts_to_plain(&self.value)
    }

    /// Return the first meaningful `ArgPart` of this argument's value, skipping
    /// any leading `Text` parts that are pure whitespace. This is useful to
    /// handle values like `difficulty= {{DifficultyNum|4.67}}` where a leading
    /// space becomes a `Text` part; callers usually want the nested template.
    pub fn first_meaningful_part(&self) -> Option<&ArgPart> {
        for part in &self.value {
            match part {
                ArgPart::Text(t) => {
                    if !t.trim().is_empty() {
                        return Some(part);
                    } else {
                        // skip pure-whitespace text parts
                        continue;
                    }
                }
                _ => return Some(part),
            }
        }
        None
    }

    /// Convenience: return the first meaningful part as plain text. If the
    /// part is a nested template or link, its `to_plain()` will be used.
    pub fn first_meaningful_text(&self) -> Option<String> {
        self.first_meaningful_part()
            .map(|p| p.to_plain().trim().to_string())
    }
}

/// A parsed template (name and arguments).
///
/// Examples
/// ```rust,ignore
/// // Typical usage (the real entry point for parsing is in `parser::parse_templates`)
/// use crate::wikitext::parts::{Template, Argument, ArgPart, MatchType, ArgQueryKind};
///
/// // Build a small template programatically for demonstration:
/// let mut nested = Template::new(\"DifficultyNum\");
/// nested.push_arg(Argument::positional(\"4.67\"));
///
/// let mut tpl = Template::new(\"towerinfobox\");
/// tpl.push_arg(Argument::named(\"difficulty\", vec![ArgPart::Template(nested.clone())]));
///
/// // Query the `difficulty` argument for the nested template's first positional value:
/// if let Some(crate::wikitext::parts::ArgQueryResult::Text(val)) = tpl.query_arg(
///     \"difficulty\",
///     MatchType::Exact,
///     ArgQueryKind::NestedFirstPositionalText,
/// ) {
///     assert_eq!(val, \"4.67\"); // demonstrates extracting the inner scalar
/// }
/// ```
///
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub args: Vec<Argument>,
}

/// How to match argument names when querying a template.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchType {
    Exact,
    StartsWith,
}

/// What kind of query to perform against an argument's value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgQueryKind {
    /// Return the full parts slice (&[ArgPart]).
    Parts,
    /// Return the first meaningful part (skips leading whitespace-only Text parts).
    FirstPart,
    /// Return the first meaningful part rendered to plain text (String).
    FirstText,
    /// If the first meaningful part is a nested template, return that template's
    /// first positional argument as plain text. Otherwise fallback to FirstText.
    NestedFirstPositionalText,
}

/// Result of querying an argument. Borrowed where possible to avoid cloning.
pub enum ArgQueryResult<'a> {
    Parts(&'a [ArgPart]),
    Part(&'a ArgPart),
    Text(String),
}

impl Template {
    /// Create a new template with given name and empty args.
    pub fn new<T: Into<String>>(name: T) -> Self {
        Template {
            name: name.into(),
            args: Vec::new(),
        }
    }

    /// Add an argument (positional or named).
    pub fn push_arg(&mut self, arg: Argument) {
        self.args.push(arg);
    }

    /// Convert template to a brief plain representation:
    /// `Name: arg1, key=value, ...`
    pub fn to_plain(&self) -> String {
        if self.args.is_empty() {
            return self.name.clone();
        }
        let mut pieces = Vec::with_capacity(self.args.len());
        for a in &self.args {
            if let Some(k) = &a.name {
                pieces.push(format!("{}={}", k, a.value_plain()));
            } else {
                pieces.push(a.value_plain());
            }
        }
        format!("{}: {}", self.name, pieces.join(", "))
    }

    /// Internal helper: find an argument by name using the provided `MatchType`.
    fn find_arg(&self, name: &str, mode: MatchType) -> Option<&Argument> {
        let target = name.trim().to_lowercase();
        if target.is_empty() {
            return None;
        }
        for a in &self.args {
            if let Some(k) = &a.name {
                let key = k.trim().to_lowercase();
                match mode {
                    MatchType::Exact => {
                        if key == target {
                            return Some(a);
                        }
                    }
                    MatchType::StartsWith => {
                        if key.starts_with(&target) {
                            return Some(a);
                        }
                    }
                }
            }
        }
        None
    }

    /// Unified high-level query API. Returns either a borrowed parts/part or an owned String
    /// depending on the requested `kind`.
    pub fn query_arg<'a>(
        &'a self,
        name: &str,
        mode: MatchType,
        kind: ArgQueryKind,
    ) -> Option<ArgQueryResult<'a>> {
        let arg = self.find_arg(name, mode)?;
        match kind {
            ArgQueryKind::Parts => Some(ArgQueryResult::Parts(arg.value.as_slice())),
            ArgQueryKind::FirstPart => arg.first_meaningful_part().map(ArgQueryResult::Part),
            ArgQueryKind::FirstText => arg
                .first_meaningful_part()
                .map(|p| ArgQueryResult::Text(p.to_plain().trim().to_string())),
            ArgQueryKind::NestedFirstPositionalText => {
                if let Some(ArgPart::Template(tpl)) = arg.first_meaningful_part() {
                    if let Some(pos0) = tpl.args.get(0) {
                        return Some(ArgQueryResult::Text(pos0.value_plain().trim().to_string()));
                    }
                }
                // fallback to FirstText
                arg.first_meaningful_part()
                    .map(|p| ArgQueryResult::Text(p.to_plain().trim().to_string()))
            }
        }
    }

    /// Get first argument with a matching name (case-insensitive).
    pub fn get_argument(&self, name: &str) -> Option<&Argument> {
        self.find_arg(name, MatchType::Exact)
    }

    /// Get first argument which starts with a matching name (case-insensitive).
    pub fn get_argument_startswith(&self, name: &str) -> Option<&Argument> {
        self.find_arg(name, MatchType::StartsWith)
    }

    /// Convenience: get argument value by name as plain text (if present).
    pub fn get_arg_value(&self, name: &str) -> Option<String> {
        // delegate to query API: return FirstText for exact match
        match self.query_arg(name, MatchType::Exact, ArgQueryKind::FirstText) {
            Some(ArgQueryResult::Text(s)) => Some(s),
            Some(ArgQueryResult::Part(p)) => Some(p.to_plain().trim().to_string()),
            Some(ArgQueryResult::Parts(ps)) => Some(parts_to_plain(ps).trim().to_string()),
            None => None,
        }
    }

    /// Convenience: return the underlying `ArgPart` slice for an argument
    /// matching `name` (case-insensitive). This makes it simple to access
    /// structured parts (e.g. nested templates or links) without dealing with
    /// `Argument` manually.
    pub fn get_arg_parts(&self, name: &str) -> Option<&[ArgPart]> {
        match self.query_arg(name, MatchType::Exact, ArgQueryKind::Parts) {
            Some(ArgQueryResult::Parts(ps)) => Some(ps),
            _ => None,
        }
    }

    /// Like `get_arg_parts` but matches argument names that start with `name`.
    pub fn get_arg_parts_startswith(&self, name: &str) -> Option<&[ArgPart]> {
        match self.query_arg(name, MatchType::StartsWith, ArgQueryKind::Parts) {
            Some(ArgQueryResult::Parts(ps)) => Some(ps),
            _ => None,
        }
    }

    /// (Removed) Use `query_arg(name, mode, ArgQueryKind::FirstPart)` directly.
    /// This helper was removed in favor of the unified `query_arg` API.

    /// Convenience: return the first meaningful part (exact match).
    pub fn get_arg_first_part(&self, name: &str) -> Option<&ArgPart> {
        match self.query_arg(name, MatchType::Exact, ArgQueryKind::FirstPart) {
            Some(ArgQueryResult::Part(p)) => Some(p),
            _ => None,
        }
    }

    /// Convenience: return the first meaningful part (starts-with match).
    pub fn get_arg_first_part_startswith(&self, name: &str) -> Option<&ArgPart> {
        match self.query_arg(name, MatchType::StartsWith, ArgQueryKind::FirstPart) {
            Some(ArgQueryResult::Part(p)) => Some(p),
            _ => None,
        }
    }

    /// (Removed) Use `query_arg(name, mode, ArgQueryKind::FirstText)` directly.
    /// The unified `query_arg` API should be used instead of this separate helper.

    /// Convenience: first meaningful text (exact match).
    pub fn get_arg_first_text(&self, name: &str) -> Option<String> {
        match self.query_arg(name, MatchType::Exact, ArgQueryKind::FirstText) {
            Some(ArgQueryResult::Text(s)) => Some(s),
            Some(ArgQueryResult::Part(p)) => Some(p.to_plain().trim().to_string()),
            Some(ArgQueryResult::Parts(ps)) => Some(parts_to_plain(ps).trim().to_string()),
            None => None,
        }
    }

    /// Convenience: first meaningful text (starts-with match).
    pub fn get_arg_first_text_startswith(&self, name: &str) -> Option<String> {
        match self.query_arg(name, MatchType::StartsWith, ArgQueryKind::FirstText) {
            Some(ArgQueryResult::Text(s)) => Some(s),
            Some(ArgQueryResult::Part(p)) => Some(p.to_plain().trim().to_string()),
            Some(ArgQueryResult::Parts(ps)) => Some(parts_to_plain(ps).trim().to_string()),
            None => None,
        }
    }
}

/// Convert a slice of ArgPart into plain text by concatenating `to_plain()` for each part.
pub fn parts_to_plain(parts: &[ArgPart]) -> String {
    let mut out = String::new();
    for p in parts {
        out.push_str(&p.to_plain());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argpart_to_plain_texts() {
        assert_eq!(ArgPart::Text("hello".into()).to_plain(), "hello");
    }

    #[test]
    fn internal_link_to_plain() {
        let p = ArgPart::InternalLink {
            target: "Page".into(),
            label: Some("Label".into()),
        };
        assert_eq!(p.to_plain(), "Label");
        let p2 = ArgPart::InternalLink {
            target: "OnlyPage".into(),
            label: None,
        };
        assert_eq!(p2.to_plain(), "OnlyPage");
    }

    #[test]
    fn external_link_to_plain() {
        let url = Url::parse("http://example.com").unwrap();
        let p = ArgPart::ExternalLink {
            url: url.clone(),
            label: Some("Ex".into()),
        };
        assert_eq!(p.to_plain(), "Ex");
        let p2 = ArgPart::ExternalLink {
            url: url.clone(),
            label: None,
        };
        assert_eq!(p2.to_plain(), "http://example.com/");
    }

    #[test]
    fn argument_plain_and_lookup() {
        let mut t = Template::new("T");
        t.push_arg(Argument::positional("pos1"));
        t.push_arg(Argument::named("named", vec![ArgPart::Text("val".into())]));
        t.push_arg(Argument::named(
            "link",
            vec![ArgPart::InternalLink {
                target: "A".into(),
                label: Some("Label".into()),
            }],
        ));

        assert_eq!(t.get_arg_value("named").unwrap(), "val");
        assert_eq!(t.get_arg_value("link").unwrap(), "Label");
        assert!(t.get_argument("doesnotexist").is_none());
    }

    #[test]
    fn nested_template_to_plain() {
        let mut sub = Template::new("Sub");
        sub.push_arg(Argument::positional("x"));
        let mut top = Template::new("Top");
        top.push_arg(Argument::named(
            "nested",
            vec![ArgPart::Template(sub.clone())],
        ));
        assert!(top.to_plain().contains("Sub"));
        assert_eq!(top.get_arg_value("nested").unwrap(), "Sub: x");
    }

    #[test]
    fn parts_to_plain_concatenates() {
        let parts = vec![
            ArgPart::Text("See ".into()),
            ArgPart::InternalLink {
                target: "P".into(),
                label: Some("page".into()),
            },
            ArgPart::Text(" now".into()),
        ];
        assert_eq!(parts_to_plain(&parts), "See page now");
    }

    #[test]
    fn external_link_url_type() {
        let url = Url::parse("https://rust-lang.org").unwrap();
        let p = ArgPart::ExternalLink {
            url: url.clone(),
            label: None,
        };
        match p {
            ArgPart::ExternalLink {
                url: u,
                label: None,
            } => {
                assert_eq!(u, url);
            }
            _ => panic!("wrong variant"),
        }
    }
}
