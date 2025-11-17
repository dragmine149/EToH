use pyo3::{
    Bound, PyAny, PyResult, Python,
    types::{PyAnyMethods, PyDict, PyIterator, PyList, PyListMethods, PyModule},
};
use regex::Regex;
use std::error;

struct WikiConverter<'a> {
    pwb: Bound<'a, PyModule>,
    site: Bound<'a, PyAny>,
    wtp: Bound<'a, PyModule>,
}

#[derive(Debug, Clone)]
pub struct WikiTower {
    pub name: String,
    pub area: String,
    pub length: u8,
    pub difficulty: f32,
    pub badges: Vec<u64>,
}

/// Parse the badges names in order to get wiki data back.
pub fn parse_badges(
    badges: &Vec<String>,
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
        Ok(data.get_wiki_pages(badges))
    })
}

impl WikiConverter<'_> {
    /// Clean up the name a bit more to try for better results.
    ///
    /// # Arguments
    /// - badge - The name of the badge to clean
    ///
    /// # Returns
    /// - String - The clean name.
    fn cleaner_name(&self, badge: &str) -> String {
        badge.replace("-", " ")
    }

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

    /// Searches the wiki in order for attempting to find a link to the badge.
    /// Returns the first match.
    ///
    /// # Arguments
    /// - badge - The badge to search for.
    ///
    /// # Returns
    /// - Ok(String) - The wikidata
    /// - Err() - Anything
    fn search_wikipage(&self, badge: &str) -> Result<String, Box<dyn std::error::Error>> {
        // make all the search results
        let kwargs = PyDict::new(self.site.py());
        kwargs.set_item("total", 1)?;
        let binding = self.site.call_method("search", (badge,), Some(&kwargs))?;
        let bind = match binding.cast::<PyIterator>() {
            Ok(it) => it,
            Err(_err) => return Err("cast error".into()),
        };
        // log::debug!("iter: {:?}", bind);
        // Loop through them all
        let page = bind
            // .unwrap()
            .clone()
            .filter(|item| item.is_ok())
            .map(|item| item.ok().unwrap())
            // get page details
            .map(
                |item| -> Result<(String, String), Box<dyn std::error::Error>> {
                    // log::debug!("Attempting to get: {:?}", item);
                    Ok((
                        item.call_method0("title")?.extract::<String>()?,
                        item.call_method0("get")?.extract::<String>()?,
                    ))
                },
            )
            .flatten()
            // parse the page for external links linking to our desired badge.
            .map(|page| -> Option<String> {
                let result = match match match self.wtp.call_method1("parse", (page.1.to_owned(),))
                {
                    Ok(it) => it,
                    Err(_err) => return None,
                }
                .getattr("external_links")
                {
                    Ok(it) => it,
                    Err(_err) => return None,
                }
                .cast::<PyList>()
                {
                    Ok(it) => it,
                    Err(_err) => return None,
                }
                .iter()
                .any(|link| {
                    let text = match link.getattr("text") {
                        Ok(it) => it,
                        Err(_err) => return false,
                    }
                    .extract::<String>()
                    .unwrap_or_default()
                    .replace("{{PAGENAME}}", &page.0);
                    // lower case and double check just in case.
                    text.to_lowercase().contains(&badge.to_lowercase())
                        || badge.to_lowercase().contains(&text.to_lowercase())
                });
                if result { Some(page.1) } else { None }
            }).find(|a| a.is_some());
        if let Some(p) = page {
            return Ok(p.unwrap());
        }
        Err("No link found".into())
    }

    /// Due to multiple fields having different types... Get the raw data for that field.
    ///
    /// # Arguments
    /// - item - The item to call the `get_arg` method for.
    /// - name - The base name to look for. Will auto-add `1` and `<!--1-->` if needed
    ///
    /// # Returns
    /// - Ok(Bound<'a, PyAny>) - The object in question
    /// - Err - Failure to find the object and the page needs to be looked at.
    fn get_raw<'a>(
        &self,
        item: &'a Bound<'a, PyAny>,
        name: &'a str,
        ignore_miss: Option<bool>,
    ) -> Result<Bound<'a, PyAny>, String> {
        // println!("Getting arg for {:?}", name);
        // println!("{:?}", item);
        let result = item.call_method1("get_arg", (name,));
        if let Ok(arg) = result
            && !arg.is_none() {
                // println!("Normal");
                return Ok(arg);
            }
        let result = item.call_method1("get_arg", (name.to_owned() + "1",));
        if let Ok(arg) = result
            && !arg.is_none() {
                // println!("+1");
                return Ok(arg);
            }
        let result = item.call_method1("get_arg", (name.to_owned() + "<!--1-->",));
        if let Ok(arg) = result
            && !arg.is_none() {
                // println!("+comment");
                return Ok(arg);
            }

        if !ignore_miss.unwrap_or_default() {
            log::error!("Uncaught case!: {:?}", name);
            // panic!("An uncaught case somehow");
            // None
            return Err(format!(
                "{:} was not a valid argument in the template!",
                name
            ));
        }
        Ok(item.py().None().as_any().bind(item.py()).to_owned())
    }

    /// Process the wiki page and convert it into useable template data.
    ///
    /// # Arguments
    /// - page_data - The raw source of the page we're looking at
    ///
    /// # Returns
    /// - Ok(Bound<'_, PyAny>) - The python object in question. (wikitextparser.Template)
    /// - Err(dyn Error) - Any errors that might have happened
    fn get_template_data(
        &self,
        page_data: &str,
        template_name: &str,
    ) -> Result<Bound<'_, PyAny>, Box<dyn std::error::Error>> {
        let binding = self
            .wtp
            .call_method1("parse", (page_data,))?
            .getattr("templates")?;
        // with all the templates as a list
        let list = match binding.cast::<PyList>() {
            Ok(it) => it,
            Err(_err) => return Err("Some error".into()),
        };
        Ok(list
            .iter()
            // attempts to get those starting with our template.
            .filter(|template| -> bool {
                // log::debug!(
                //     "template: {:?}. looking: {:?}",
                //     template.getattr("name"),
                //     template_name
                // );
                match template.getattr("name") {
                    Ok(it) => it,
                    Err(_) => return false,
                }
                .extract::<String>()
                .unwrap_or_default()
                .trim()
                .to_lowercase()
                .starts_with(&template_name.to_lowercase())
            })
            // .map(|a| {
            //     log::debug!("{:?}", a);
            //     a
            // })
            .next()
            .ok_or("Item is none")?
            .as_any()
            .to_owned()
            .to_owned())
    }

    fn process_tower(
        &self,
        tower_data: Bound<'_, PyAny>,
        tower_name: String,
    ) -> Result<WikiTower, Box<dyn std::error::Error>> {
        // log::debug!("difficulty");
        let difficulty = self
            .get_raw(&tower_data, "difficulty", None)?
            .getattr("value")?
            .extract::<String>()?;
        let diff = Regex::new(r"[\d.]+")?
            .captures(&difficulty)
            .unwrap()
            .get(0)
            .unwrap()
            .as_str()
            .parse::<f32>()?;

        let removed = self.get_raw(&tower_data, "date_removed", Some(true))?;
        let ar = if removed.is_none() {
            // log::debug!("area");
            let area = self
                .get_raw(&tower_data, "found_in", None)?
                .getattr("value")?
                .extract::<String>()?;
            self.wtp
                .call_method1("parse", (area,))?
                .call_method0("plain_text")?
                .extract::<String>()?
                .lines()
                .next()
                .unwrap()
                .trim()
                .to_string()
        } else {
            "Removed Towers".to_string()
        };

        // log::debug!("length");
        let length = self.get_raw(&tower_data, "length", Some(true))?;
        let len = if length.is_none() {
            0
        } else if let Ok(template) = self
            .wtp
            .call_method1("parse", (length.getattr("value")?.extract::<String>()?,))?
            .getattr("templates")?
            .get_item(0)
        {
            if let Ok(arg) = template.getattr("arguments")?.get_item(0) {
                arg.getattr("value")?
                    .extract::<String>()?
                    .parse::<u8>()
                    .unwrap_or_default()
            } else {
                0
            }
        } else {
            0
        };
        // let len = 0;

        // log::debug!("completed");
        Ok(WikiTower {
            name: tower_name,
            difficulty: diff,
            badges: vec![],
            area: ar,
            length: len,
        })
    }

    fn get_page_data(
        &self,
        badges: &Vec<String>,
    ) -> (Vec<(String, Option<String>)>, Vec<String>, Vec<String>) {
        log::info!("Attempting to get page data");
        let mut pages = badges
            .iter()
            .map(|badge| {
                // log::debug!("Badge: {:?}", badge);
                let data = self.get_wiki_page(badge);
                match data {
                    Ok(d) => (d.1.unwrap_or(badge.to_owned()), Some(d.0)),
                    Err(_err) => {
                        // panic!("{:?}", _err);
                        (badge.to_owned(), None)
                    }
                }
            })
            .collect::<Vec<(String, Option<String>)>>();

        // starts keeping track of failed badges to do different stuff to later.
        let failed = pages
            .iter()
            .filter(|p| p.1.is_none())
            .map(|p| p.0.to_owned())
            .collect::<Vec<String>>();

        log::info!(
            "Pages parsed: {:?}. Pages failed: {:?}",
            pages.iter().filter(|p| p.1.is_some()).count(),
            failed.len()
        );

        // NOTE: This whole section of code should only be used during prod and not testing as it checks an additional like 1k+ pages (100+ badges search)
        // NOTE: Skipping it during testing is fine as the rest that don't need to be researched should hit most issues if any.

        
        // let mut failed2 = vec![];
        // if !cfg!(debug_assertions) {
        log::info!("Searching wiki...");
        // attempts to search the wiki to add some more badges
        let mut searched = failed
            .iter()
            .map(|fail| {
                // log::debug!("{:?}", fail);
                (fail.to_owned(), self.search_wikipage(fail).ok())
            })
            .collect::<Vec<(String, Option<String>)>>();

        let failed2: Vec<String> = searched
            .iter()
            .filter(|s| s.1.is_none())
            .map(|s| s.0.to_owned())
            .collect::<Vec<String>>();

        log::info!(
            "Wiki searched: {:?}. Wiki Failed: {:?}\nTotal passed: {:?}. Total failed: {:?}",
            searched.iter().filter(|s| s.1.is_some()).count(),
            failed2.len(),
            pages.iter().filter(|p| p.1.is_some()).count()
                + searched.iter().filter(|s| s.1.is_some()).count(),
            failed2.len()
        );

        pages.append(&mut searched);
        // }

        // NOTE: End of section of prod-only code.

        (pages, failed, failed2)
    }

    // fn process_item(&self, item_data: Bound<'_, PyAny>, item_name: &str) {
    //     self.get_raw(item_data, "method_of_obtaining", None)?
    // }

    /// Main loop for the wiki pages, also filters out badges which we can't relate to a page.
    ///
    /// # Arguments
    /// - badges - A list of badges to convert into tower information. Ids are added afterwards
    ///
    /// # Returns
    /// - Tuple
    /// 	- Vec<WikiTower> - All of the details which were successfully parsed
    /// 	- Vec<String> - Any towers which couldn't be parsed for whatever reason.
    pub fn get_wiki_pages(&self, badges: &Vec<String>) -> (Vec<WikiTower>, Vec<Vec<String>>) {
        log::info!("Parsing badges");
        // get the pages, returning an error or the page/badge_name followed by data.
        let (mut pages, failed, failed2) = self.get_page_data(badges);
        log::info!("Cleaning failed and trying again");
        let (mut new_pages, failed3, failed4) = self.get_page_data(
            &failed2
                .iter()
                .map(|f| (f, self.cleaner_name(f)))
                // makes sure we have actually made a difference, otherwise we're probably just going to be wasting time...
                .filter(|f| *f.0 != f.1)
                .map(|f| f.1)
                .collect::<Vec<String>>(),
        );
        pages.append(&mut new_pages);

        log::info!("Getting template data");
        // attempts to get the templates for the given badges that have passed the previous step.
        let badges_templates = pages
            .iter()
            .filter(|p| p.1.is_some())
            .map(|data| {
                // log::debug!("Badge: {:?}", data.0);
                (
                    data.0.to_owned(),
                    self.get_template_data(&data.1.to_owned().unwrap(), "towerinfobox"),
                )
            })
            .collect::<Vec<(String, Result<Bound<'_, PyAny>, Box<dyn std::error::Error>>)>>();

        // once again, keeep track of anything that might have provided errors
        let failed5 = badges_templates
            .iter()
            .filter(|t| t.1.is_err())
            .map(|t| {
                // log::debug!("{:?}", t.1.as_ref().err().unwrap());
                t.0.to_owned()
            })
            .collect::<Vec<String>>();

        log::info!(
            "Templates parsed: {:?}. Templates failed: {:?}",
            badges_templates.iter().filter(|p| p.1.is_ok()).count(),
            failed5.len()
        );

        log::info!("Making WikiTower");
        // parse all the templates to get the actual data we want.
        let towers = badges_templates
            .iter()
            .filter(|t| t.1.is_ok())
            .map(|t| {
                let a = self.process_tower(t.1.as_ref().ok().unwrap().clone(), t.0.to_owned());
                // log::debug!("{:?}", t);
                (t.0.to_owned(), a)
            })
            .collect::<Vec<(String, Result<WikiTower, Box<dyn std::error::Error>>)>>();

        // final error catch
        let failed6 = towers
            .iter()
            .filter(|t| t.1.is_err())
            .map(|t| t.0.to_owned())
            .collect::<Vec<String>>();

        log::info!(
            "WikiTowers: {:?}. Failed towers: {:?}",
            towers.iter().filter(|p| p.1.is_ok()).count(),
            failed6.len()
        );

        log::info!("Checking item data");
        let item_templates = pages
            .iter()
            .filter(|p| !failed5.contains(&p.0))
            .map(|data| {
                (
                    data.0.to_owned(),
                    self.get_template_data(&data.1.to_owned().unwrap(), "ItemInfoBox"),
                )
            })
            .collect::<Vec<(String, Result<Bound<'_, PyAny>, Box<dyn std::error::Error>>)>>();

        let failed7 = item_templates
            .iter()
            .filter(|t| t.1.is_err())
            .map(|t| {
                // log::debug!("{:?}", t.1.as_ref().err().unwrap());
                t.0.to_owned()
            })
            .collect::<Vec<String>>();

        log::info!(
            "Item parsed: {:?}. Item failed: {:?}",
            item_templates.iter().filter(|p| p.1.is_ok()).count(),
            failed7.len()
        );

        log::info!("Making WikiTower - Item Edition");

        // return a tuple of the result.
        (
            towers
                .iter()
                .filter(|t| t.1.is_ok())
                .map(|t| t.1.as_ref().ok().unwrap().to_owned())
                .collect::<Vec<WikiTower>>(),
            vec![failed, failed2, failed3, failed4, failed5, failed6, failed7],
        )
    }
}
