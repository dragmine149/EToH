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
}

/// A parsed template (name and arguments).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub args: Vec<Argument>,
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

    /// Get first argument with a matching name (case-insensitive).
    pub fn get_argument(&self, name: &str) -> Option<&Argument> {
        let target = name.trim().to_lowercase();
        if target.is_empty() {
            return None;
        }
        for a in &self.args {
            if let Some(k) = &a.name {
                if k.trim().to_lowercase() == target {
                    return Some(a);
                }
            }
        }
        None
    }

    /// Get first argument which starts with a matching name (case-insensitive).
    pub fn get_argument_startswith(&self, name: &str) -> Option<&Argument> {
        let target = name.trim().to_lowercase();
        if target.is_empty() {
            return None;
        }
        for a in &self.args {
            if let Some(k) = &a.name {
                if k.trim().to_lowercase().starts_with(&target) {
                    return Some(a);
                }
            }
        }
        None
    }

    /// Convenience: get argument value by name as plain text (if present).
    pub fn get_arg_value(&self, name: &str) -> Option<String> {
        self.get_argument(name)
            .map(|a| a.value_plain().trim().to_string())
    }

    /// Convenience: return the underlying `ArgPart` slice for an argument
    /// matching `name` (case-insensitive). This makes it simple to access
    /// structured parts (e.g. nested templates or links) without dealing with
    /// `Argument` manually. Example:
    ///
    /// ```rust,ignore
    /// // assuming `tpl` is a parsed `Template`
    /// if let Some(parts) = tpl.get_arg_parts("difficulty") {
    ///     // parts: &[ArgPart]; you can inspect parts[0] etc.
    /// }
    /// ```
    pub fn get_arg_parts(&self, name: &str) -> Option<&[ArgPart]> {
        self.get_argument(name).map(|a| a.value.as_slice())
    }

    /// Like `get_arg_parts` but matches argument names that start with `name`.
    /// Handy when argument keys use prefixes or variants (e.g. `original_difficulty`).
    pub fn get_arg_parts_startswith(&self, name: &str) -> Option<&[ArgPart]> {
        self.get_argument_startswith(name)
            .map(|a| a.value.as_slice())
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
