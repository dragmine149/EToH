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

struct Template<'b> {
    template: Bound<'b, PyAny>,
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
    /// - pwb -> Python module directly linking to pywikibot
    /// - site -> BaseSite provided from `pwb.Site`
    /// - badge -> The badge to search for under `pwb.Page`
    ///
    /// # Returns
    /// - Ok
    /// 	- String -> The raw data of the page
    /// 	- Option -> If any redirects were followed (and if so, the name)
    /// - Err(dyn Error) -> Any errors that might have happened
    fn get_wiki_page(&self, page: &str) -> Result<(String, String), Box<dyn error::Error>> {
        // gets the page.
        let result = self
            .pwb
            .call_method1("Page", (&self.site, page))?
            .call_method1("get", (false, true))?
            .extract::<String>()?;

        // if we have a redirect, always follow it.
        if result.starts_with("#redirect") {
            let redirect = self.parse_redirect(&result)?;
            return self.get_wiki_page(&redirect);
        }

        // return the page data, and the none saying we haven't redirected.
        Ok((result, page.to_owned()))
    }

    /// Parse the redirect of the page. Bit over the top but its needed
    ///
    /// # Arguments
    /// - redirect -> The raw source of the redirect page.
    ///
    /// # Returns
    /// - Ok(String) -> The new page to go to.
    /// - Err(dyn Error) -> Any errors that might have happened
    fn parse_redirect(&self, redirect: &str) -> Result<String, Box<dyn error::Error>> {
        Ok(self
            .wtp
            .call_method1("parse", (redirect,))?
            .getattr("wikilinks")?
            .get_item(0)?
            .call_method0("plain_text")?
            .extract::<String>()?)
    }

    /// Search the wiki to try and find our page.
    ///
    /// # Arguments
    /// - page -> The page to look for
    /// - search_count -> How many pages to search. Default 3.
    ///
    /// # Returns
    /// - Ok((String, String)) -> The result of `get_wiki_page` as it contains both title and data already processed
    /// - Err(dyn Error) -> Something went wrong.
    fn search_wiki(
        &self,
        page: &str,
        search_count: Option<u8>,
    ) -> Result<(String, String), Box<dyn error::Error>> {
        let search_args = PyDict::new(self.site.py());
        search_args.set_item("total", search_count.unwrap_or(3));
        let pages = self
            .site
            .call_method("search", (page,), Some(&search_args))?;
        let iter = match pages.cast::<PyIterator>() {
            Ok(v) => v.to_owned(),
            Err(_) => return Err("Failed to cast into iterator".into()),
        };
        for search_result in iter {
            let title = search_result?.call_method0("title")?.extract::<String>()?;
            let data = self.get_wiki_page(&title)?;
            let links = self.get_external_links(&data.0, &data.1)?;
            if links.iter().any(|link| {
                page.to_lowercase().contains(link) || link.contains(&page.to_lowercase())
            }) {
                return Ok(data);
            }
        }
        Err("No page found during searching with a link.".into())
    }

    /// Parse the page and look for the links in the page.
    ///
    /// # Arguments
    /// - page_data -> Data of the page (wikitext)
    /// - page_name -> Name of the page, used in replacing `{{PAGENAME}}`
    ///
    /// # Returns
    /// - Ok(Vec<String>) -> A vector of all of the links found. Links that have no "text" are filtered out.
    /// - Err(dyn Error) -> Something happened to cause an error.
    fn get_external_links(
        &self,
        page_data: &str,
        page_name: &str,
    ) -> Result<Vec<String>, Box<dyn error::Error>> {
        let parsed = self.wtp.call_method1("parse", (page_data,))?;
        let external_links = parsed.getattr("external_links")?;
        let list = match external_links.cast::<PyList>() {
            Ok(v) => v.to_owned(),
            Err(_) => return Err("Failed to cast into list.".into()),
        };
        Ok(list
            .iter()
            .map(|item| match item.getattr("text") {
                Ok(attr) => attr.extract::<String>().unwrap_or_default(),
                Err(_) => String::new(),
            })
            .map(|link| link.replace("{{PAGENAME}}", page_name).to_lowercase())
            .collect::<Vec<String>>())
    }

    fn process_tower(
        &self,
        tower_obj: &mut WikiTower,
        page_data: &str,
    ) -> Result<(), Box<dyn error::Error>> {
        // get the main template object.
        let template = Template::new_from_name(&self.wtp, page_data, "towerinfobox")?;
        // let template = self.get_template_from_name(page_data, "towerinfobox")?;

        // get the difficulty of the tower.
        let difficulty = template.get_argument_by_name("difficulty")?;
        tower_obj.difficulty = Regex::new(r"[\d.]+")
            .unwrap()
            .captures(&difficulty)
            .unwrap()
            .get(0)
            .unwrap()
            .as_str()
            .parse::<f32>()
            .ok();

        let removed = template.get_argument_by_name("date_removed");
        tower_obj.area = if removed.is_err() {
            Some("Removed Towers".to_string())
        } else {
            let area = template.get_argument_by_name("found_in")?;
            Some(
                self.wtp
                    .call_method1("parse", (area,))?
                    .call_method0("plain_text")?
                    .extract::<String>()?
                    .lines()
                    .next()
                    .unwrap()
                    .trim()
                    .to_string(),
            )
        };
        let length = template.get_argument_by_name("length");
        tower_obj.length = Length::from(length.unwrap_or_default().parse::<u16>()?);

        Ok(())
    }
}

impl Template<'_> {
    /// Get the template on the page with the provided name. Returns **first instance**
    ///
    /// # Arguments
    /// - page_data -> Data of the page, see `get_wiki_page` for a possible way.
    /// - name -> Name of the template to find.
    ///
    /// # Returns
    /// - Ok(Bound<'_, PyAny>) -> The template still as the python object.
    /// - Err(dyn Error) -> Some errored happened whilst making the list of templates. (not whilst filtering)
    pub fn new_from_name<'b>(
        wtp: &Bound<'b, PyModule>,
        page_data: &str,
        name: &str,
    ) -> Result<Template<'b>, Box<dyn error::Error>> {
        let templates = wtp
            .call_method1("parse", (page_data,))?
            .getattr("templates")?;
        let template_list = match templates.cast::<PyList>() {
            Ok(v) => v.to_owned(),
            Err(_) => return Err("Failed to cast into pylist".into()),
        };
        for template in template_list {
            let template_name = match template.getattr("name") {
                Ok(v) => v.extract::<String>().unwrap_or_default(),
                Err(_) => continue,
            };
            if template_name.trim().eq_ignore_ascii_case(name.trim()) {
                return Ok(Template { template });
            }
        }
        Err("Failed to find template in page".into())
    }

    /// Search for an argument in the template.
    ///
    /// Normally, we could just do. `.get_arg(arg_name)` but due to the wiki not being consistent.. things like `found_in`, `found_in1` and `found_in<!--1-->` are all possible.
    /// Hence the requirement to do a mini filter search.
    ///
    /// # Arguments
    /// - template_data -> The template data to search through, an object of `wtp.Template` or gotten from `get_template_from_name`
    /// - name -> The name of the argument to query against.
    ///
    /// # Returns
    /// - Ok(String) -> The name of the argument once we have succesffully found it.
    /// - Err(dyn Error) -> No argument found or failed to cast into list.
    pub fn argument_exists(&self, argument: &str) -> Result<String, Box<dyn error::Error>> {
        let arguments = match self.template.getattr("arguments")?.cast::<PyList>() {
            Ok(v) => v.to_owned(),
            Err(_) => return Err("Failed in casting to list".into()),
        };
        let name = argument.trim().to_lowercase();
        for arg in arguments {
            let arg_name = match arg.getattr("name") {
                Ok(v) => v
                    .extract::<String>()
                    .unwrap_or_default()
                    .trim()
                    .to_lowercase(),
                Err(_) => continue,
            };
            if arg_name.contains(&name) || name.contains(&arg_name) {
                return Ok(arg_name);
            }
        }
        Err("Failed to find any hint towards the name provided in the template".into())
    }

    /// Get the value of the argument on the template.
    ///
    /// Unlike [`get_argument_by_name`] this gets the exact value. See [`get_argument_by_name`] for an aproximate guess.
    ///
    /// # Arguments
    /// - argument -> The argument to get
    ///
    /// # Returns
    /// - Ok(String) -> The value of that argument extracted
    /// - pyo3::PyErr -> Something failed in python whilst trying to extract the argument.
    pub fn get_argument(&self, argument: &str) -> Result<String, pyo3::PyErr> {
        self.template
            .call_method1("get_arg", (argument,))?
            .getattr("value")?
            .extract::<String>()
    }

    /// A short for both [get_argument] and [argument_exists][^note]
    ///
    /// [^note]: If argument_exists returns an error, then it'll default to passing `""` to [get_argument], hence causing another different error.
    ///
    /// # Arguments
    /// - argument -> The argument to get
    ///
    /// # Returns
    /// - Ok(String) -> The value of that argument extracted
    /// - pyo3::PyErr -> Something failed in python whilst trying to extract the argument.
    pub fn get_argument_by_name(&self, argument: &str) -> Result<String, pyo3::PyErr> {
        self.get_argument(&self.argument_exists(argument).unwrap_or_default())
    }
}
