use derive_builder::Builder;
use pyo3::{
    Bound, PyAny, PyResult, Python,
    types::{PyAnyMethods, PyDict, PyIterator, PyList, PyListMethods, PyModule},
};
use regex::Regex;
use serde::Serialize;
use std::{collections::HashMap, error};

use crate::definitions::TowerType;

/// The wiki tower object containing all the information.
#[derive(Debug, Clone, Default, Builder)]
pub struct WikiTower {
    pub name: String,
    pub area: String,
    pub length: u8,
    pub difficulty: f32,
    pub tower_type: TowerType,
    pub badges: Vec<u64>,
    /// This is here because the name can be (in some cases), different from the actual tower.
    pub badge_name: String,
    /// This is here so we can have a direct link in the UI.
    pub wiki_link: String,
}

impl WikiTower {
    pub fn set_area(&mut self, area: Option<&str>) {
        self.area = area.unwrap_or("Unknown Area").to_owned()
    }

    pub fn set_length(&mut self, length: Option<u8>) {
        self.length = length.unwrap_or(0)
    }

    pub fn set_difficulty(&mut self, difficulty: Option<f32>) {
        self.difficulty = difficulty.unwrap_or_default()
    }

    pub fn set_type(&mut self, tower_type: Option<TowerType>) {
        self.tower_type = tower_type.unwrap_or_default()
    }
}

impl Serialize for WikiTower {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let csv = format!(
            "{},{},{},{},{:?},{},{}",
            self.name,
            self.area,
            self.length,
            self.difficulty,
            self.badges,
            self.tower_type,
            self.wiki_link
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
