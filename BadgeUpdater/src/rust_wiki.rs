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
        // gets the page.
        let result = self
            .pwb
            .call_method1("Page", (&self.site, page))?
            .call_method1("get", (false, true))?
            .extract::<String>()?;

        // if we have a redirect, always follow it.
        if result.starts_with("#redirect") {
            let redirect = self.parse_redirect(&result)?;
            let new = self.get_wiki_page(&redirect)?;

            // return the page data, and the redirect. Lowest level is more important
            return Ok((new.0, Some(new.1.unwrap_or(redirect))));
        }

        // return the page data, and the none saying we haven't redirected.
        Ok((result, None))
    }

    /// Parse the redirect of the page. Bit over the top but its needed
    ///
    /// # Arguments
    /// - redirect - The raw source of the redirect page.
    ///
    /// # Returns
    /// - Ok(String) - The new page to go to.
    /// - Err(dyn Error) - Any errors that might have happened
    fn parse_redirect(&self, redirect: &str) -> Result<String, Box<dyn error::Error>> {
        Ok(self
            .wtp
            .call_method1("parse", (redirect,))?
            .getattr("wikilinks")?
            .get_item(0)?
            .call_method0("plain_text")?
            .extract::<String>()?)
    }

    fn search_wiki(
        &self,
        page: &str,
        search_count: Option<u8>,
    ) -> Result<String, Box<dyn error::Error>> {
        let search_args = PyDict::new(self.site.py());
        search_args.set_item("total", search_count.unwrap_or(3));
        let pages = self
            .site
            .call_method("search", (page,), Some(&search_args))?;
        for page in pages.cast::<PyIterator>()? {
            let data = self.get_wiki_page(page?.call_method0("title")?.extract::<String>())?;
            let links = self.get_external_links(data.0)?;
            links.iter().any(|link|)
        }
    }

    fn get_external_links(
            &self,
            page_data: &str,
            page_name: &str,
        ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
            let parsed = self.wtp.call_method1("parse", (page_data,))?;
            let external_links = parsed.getattr("external_links")?;
            let py_list = external_links.cast::<PyList>()?.to_owned();

            let mut links: Vec<String> = Vec::new();

            for item in py_list.iter() {
                let text = match item.getattr("text") {
                    Ok(attr) => attr.extract::<String>().unwrap_or_default(),
                    Err(_) => String::new(),
                };
                links.push(text.replace("{{PAGENAME}}", page_name));
            }

            Ok(links)
        }
}
