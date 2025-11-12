use derive_builder::Builder;
use pyo3::{
    Bound, PyAny, PyResult, Python,
    types::{PyAnyMethods, PyDict, PyIterator, PyList, PyListMethods, PyModule},
};
use regex::Regex;
use serde::Serialize;
use std::{collections::HashMap, error};

use crate::definitions::{Length, TowerType};

/// The wiki tower object containing all the information.
#[derive(Debug, Clone, Default, Builder)]
pub struct WikiTower {
    pub name: String,
    pub area: Option<String>,
    pub length: Length,
    pub difficulty: Option<f32>,
    pub tower_type: TowerType,
    pub badges: Vec<u64>,
    /// This is here because the name can be (in some cases), different from the actual tower.
    pub badge_name: String,
    /// This is here so we can have a direct link in the UI.
    pub wiki_link: String,
}

impl Serialize for WikiTower {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let csv = format!(
            "{},{},{},{:?},{}{},{}",
            self.name,
            self.area.to_owned().unwrap_or_default(),
            match self.difficulty {
                Some(diff) => diff.to_string(),
                None => String::new(),
            },
            self.badges,
            (self.length as u8).to_string(),
            self.tower_type,
            if self.wiki_link == self.name {
                String::new()
            } else {
                self.wiki_link.to_owned()
            }
        );
        serializer.serialize_str(&csv)
    }
}

struct WikiConverter<'a> {
    pwb: Bound<'a, PyModule>,
    site: Bound<'a, PyAny>,
    wtp: Bound<'a, PyModule>,
}

/// Overall function for setting up python and badges.
///
/// # Arguments
/// - &[WikiTower] -> Pre-made list of wikitowers to fill out. This includes all badges and reduces work afterwards
///
/// # Returns
/// - OK
/// 	- Vec<WikiTower> -> The data which has been converted
/// 	- Vec<Vec<String>> -> Which badges failed at every step of the process.
/// - Err -> Just some kind of python error.
pub fn parse_badges(
    badges: &[WikiTower],
) -> Result<(Vec<WikiTower>, Vec<Vec<String>>), pyo3::PyErr> {
    Python::initialize();
    Python::attach(|py| -> PyResult<(Vec<WikiTower>, Vec<Vec<String>>)> {
        // import pywikibot and setup required site data
        let pwb = py.import("pywikibot")?;
        let site = pwb.call_method1("Site", ("en", "etoh"))?;

        // import wikitextparser
        let wtp = py.import("wikitextparser")?;

        let data = WikiConverter { pwb, site, wtp };

        // Get all the badges (TODO: return result)
        Ok(data.get_wiki_pages(&badges))
    })
}

impl WikiConverter<'_> {
    /// Get the raw data of the wiki page.
    ///
    /// Will automatically follow all redirects as long as the page starts with `#redirect`
    ///
    /// # Arguments
    /// - pwb - Python module directly linking to pywikibot
    /// - site - BaseSite provided from `pwb.Site`
    /// - badge - The badge to search for under `pwb.Page`
    ///
    /// # Returns
    /// - Ok
    /// 	- String - The raw data of the page
    /// 	- Option - If any redirects were followed (and if so, the name)
    /// - Err(dyn Error) - Any errors that might have happened
    fn get_wiki_page(&self, page: &str) -> Result<(String, Option<String>), Box<dyn error::Error>> {
        let result = self
            .pwb
            .call_method1("Page", (&self.site, page))?
            .call_method1("get", (false, true))?
            .extract::<String>()?;
        if result.starts_with("#redirect") {
            let redirect = &self.parse_redirect(&result)?;
            return Ok((self.get_wiki_page(redirect)?.0, Some(redirect.to_owned())));
        }
        Ok((result, None))
    }
}
