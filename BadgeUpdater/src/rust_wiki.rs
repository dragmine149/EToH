use derive_builder::Builder;
use pyo3::{
    Bound, PyAny, PyResult, Python,
    types::{PyAnyMethods, PyDict, PyIterator, PyList, PyListMethods, PyModule},
};
use regex::Regex;
use std::{
    env, error, fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::definitions::{Length, TowerType};

/// The wiki tower object containing all the information.
#[derive(Debug, Clone, Default, Builder)]
pub struct WikiTower {
    pub name: String,
    #[builder(default)]
    pub area: Option<String>,
    #[builder(default)]
    pub length: Length,
    #[builder(default)]
    pub difficulty: Option<f32>,
    #[builder(default)]
    pub tower_type: TowerType,
    pub badges: Vec<u64>,
    /// This is here because the name can be (in some cases), different from the actual tower.
    pub badge_name: String,
    /// This is here so we can have a direct link in the UI.
    #[builder(default)]
    pub wiki_link: String,

    /// Move private-ish items
    #[builder(default)]
    is_item: bool,
    #[builder(default)]
    has_tower: bool,
}

struct WikiData {
    pub wiki_tower: WikiTower,
    pub wiki_link: String,
    pub page_data: String,
}

struct WikiConverter<'a> {
    pwb: Bound<'a, PyModule>,
    site: Bound<'a, PyAny>,
    wtp: Bound<'a, PyModule>,
}

struct Template<'b> {
    template: Bound<'b, PyAny>,
}

struct ExternalLinks(Vec<ExternalLink>);

struct ExternalLink {
    pub url: String,
    pub text: String,
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
    badges: &mut Vec<WikiTower>,
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
        Ok(data.process_wiki_towers(badges))
    })
}

impl WikiConverter<'_> {
    /// Checks the modification date of a file to see if we should use cache or not.
    ///
    /// # Environment Variables
    /// - cache -> The cache path.
    ///
    /// # Arguments
    /// - cache_file -> Name of the cache.
    /// - cache_age -> Age of the cache in seconds, defaults to 86400.
    ///
    /// # Returns
    /// - Ok(PathBuf) -> We should use the cache and a path to said cache.
    /// - Err(dyn Error) -> We shouldn't use the cache and the reason why.
    fn use_cache(
        &self,
        cache_file: &str,
        cache_age: Option<u64>,
    ) -> Result<PathBuf, Box<dyn error::Error>> {
        let cache_dir = env::var("cache")?;
        let cache_path = Path::new(&cache_dir);
        let cache_path = cache_path.join(cache_file);
        let modified = fs::metadata(&cache_path)?
            .modified()?
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        if now > modified + cache_age.unwrap_or(86400) {
            Err("Cache is invalid.".into())
        } else {
            Ok(cache_path)
        }
    }

    /// Get the raw data of the wiki page.
    ///
    /// Will automatically follow all redirects as long as the page starts with `#redirect`
    ///
    /// # Arguments
    /// - page -> The page title to get the data for.
    /// - cache -> How long since the modified time of the cache. (Default to 1d, `86400`)
    ///
    /// # Returns
    /// - Ok
    /// 	- String -> The raw data of the page
    /// 	- String -> The page name, due to redirects potentially being followed.
    /// - Err(dyn Error) -> Any errors that might have happened
    fn get_wiki_page(
        &self,
        page: &str,
        cache: Option<u64>,
    ) -> Result<(String, String), Box<dyn error::Error>> {
        // gets the page.
        let result = if let Ok(path) = self.use_cache(page, cache) {
            fs::read_to_string(&path).unwrap()
        } else {
            let web_request = self
                .pwb
                .call_method1("Page", (&self.site, page))?
                .call_method1("get", (false, true))?
                .extract::<String>()?;

            fs::write(page, &web_request).ok().unwrap();
            web_request
        };

        // if we have a redirect, always follow it.
        if result.starts_with("#redirect") {
            let redirect = self.parse_redirect(&result)?;
            return self.get_wiki_page(&redirect, cache);
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
            let data = self.get_wiki_page(&title, None)?;
            let links = ExternalLinks::new(&self.wtp, &data.0)?;
            if links.might_contain(page, Some(page)) {
                return Ok(data);
            }
        }
        Err("No page found during searching with a link.".into())
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

    /// Process an item.
    ///
    /// Some badges are of items, but they are obtained from towers, hence we can categorise these better. We just need to see how we get them
    /// and attempt to find a tower link.
    ///
    /// Ofc, this adds some extra logic but eh later code can deal with that.
    ///
    /// # Arguments
    /// - item_obj -> The tower template, just like [`process_tower`]
    /// - item_page -> The raw page data for the item. Required to get and check if it is an item
    ///
    /// # Returns
    /// - OK(()) -> Doesn't actually return anything, just modifies the item itself directly.
    /// - Err(dyn Error) -> Something happened preventing this item from being checked.
    fn process_item(
        &self,
        item_obj: &mut WikiTower,
        item_page: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // attempts to find the item first
        let template = Template::new_from_name(&self.wtp, item_page, "iteminfobox")?;
        item_obj.is_item = true;
        // this seems to be the best one to look at.
        let obtain = template.get_argument_by_name("method_of_obtaining")?;
        for link in ExternalLinks::new(&self.wtp, &obtain)?.0 {
            // searching all the pages might not be the most efficient, but eh.
            // at least it'll break early due to failure to pass.
            let wiki_page = self.get_wiki_page(&link.text, None);
            if wiki_page.is_err() {
                continue;
            }

            let tower = self.process_tower(item_obj, &wiki_page.unwrap().0);
            if tower.is_ok() {
                item_obj.has_tower = true;
                return Ok(());
            }
        }
        Err("No tower associated with item.".into())
    }

    /// Take an object and count how many passed/failed.
    ///
    /// # Arguments
    /// - obj -> A vector of objects to list through. (type is dynamic)
    /// - fail_check -> The function to filter out objects which have failed.
    /// - name_func -> Function to get the name of the failed objects for later debugging.
    ///
    /// # Returns
    /// - Tuple
    /// 	- Maths
    /// 		- usize -> The number passed
    /// 		- usize -> The number failed
    /// 		- f64 -> Percent of passed over total
    /// 	- Vec<String> -> A vector of the names which have failed.
    fn count_processed<K, P, N>(
        &self,
        obj: &[K],
        fail_check: P,
        name_func: N,
    ) -> ((usize, usize, f64), Vec<String>)
    where
        P: FnMut(&&K) -> bool,
        N: Fn(&K) -> String,
    {
        let failed = obj
            .iter()
            .filter(fail_check)
            .map(name_func)
            .collect::<Vec<String>>();
        let fail_count = failed.len();
        let pass_count = obj.len() - fail_count;
        let percent = (pass_count as f64) / (obj.len() as f64);
        ((pass_count, fail_count, percent), failed)
    }

    /// Attempt to get tower data by searching and page name.
    ///
    /// TODO: Rework so its on an individual basis
    ///
    /// # Arguments
    /// - towers -> List of towers to process
    ///
    /// # Returns
    /// - Vec<WikiData> - List of towers and the data for the page.
    /// - Vec<Vec<String>> - List of list of names of failed.
    fn get_and_search_wiki(&self, towers: &[WikiTower]) -> (Vec<WikiData>, Vec<Vec<String>>) {
        let mut data = towers
            .iter()
            .map(|tower| -> WikiData {
                let wiki_data = self.get_wiki_page(&tower.name, None);
                if wiki_data.is_err() {
                    return WikiData {
                        wiki_tower: tower.to_owned(),
                        wiki_link: String::new(),
                        page_data: String::new(),
                    };
                }
                let wiki_data = wiki_data.unwrap();

                WikiData {
                    wiki_tower: tower.to_owned(),
                    wiki_link: wiki_data.1,
                    page_data: wiki_data.0,
                }
            })
            .collect::<Vec<WikiData>>();

        let maths = self.count_processed(
            &data,
            |w| w.page_data.is_empty(),
            |w| w.wiki_tower.name.to_owned(),
        );
        log::info!(
            "Pages parsed: {:?}. Pages failed: {:?}. Success: {:?}",
            maths.0.0,
            maths.0.1,
            maths.0.2
        );

        log::info!("Searching for failed entries");
        data.iter_mut().for_each(|tower| {
            let result = self.search_wiki(&tower.wiki_tower.name, Some(1));
            if result.is_err() {
                return;
            }
            let result = result.unwrap();
            tower.page_data = result.0;
            tower.wiki_tower.wiki_link = result.1;
        });

        let maths2 = self.count_processed(
            &data,
            |w| w.page_data.is_empty(),
            |w| w.wiki_tower.name.to_owned(),
        );
        log::info!(
            "Pages parsed: {:?}. Pages failed: {:?}. Success: {:?}",
            maths2.0.0,
            maths2.0.1,
            maths2.0.2
        );

        (data, vec![maths.1, maths2.1])
    }

    pub fn process_wiki_towers(
        &self,
        towers: &mut Vec<WikiTower>,
    ) -> (Vec<WikiTower>, Vec<Vec<String>>) {
        log::info!("Processing towers...");
        let simple_get = self.get_and_search_wiki(towers);
        let mut failed_list = simple_get.1;
        let mut success = simple_get.0;
        let mut advanced_list = success
            .iter()
            .filter(|simple| simple.page_data.is_empty())
            .map(|simple| simple.wiki_tower.to_owned())
            .collect::<Vec<WikiTower>>();
        advanced_list.iter_mut().for_each(|t| t.clean_name());
        let mut advanced_get = self.get_and_search_wiki(&advanced_list);
        failed_list.append(&mut advanced_get.1);
        success.append(&mut advanced_get.0);
        let success = success
            .iter()
            .map(|simple| simple.wiki_tower.to_owned())
            .collect::<Vec<WikiTower>>();

        (success, failed_list)

        // log::info!("Processing templates.")
    }
}

impl ExternalLinks {
    /// Parse the page and look for the links in the page.
    ///
    /// # Arguments
    /// - wtp -> wikitextparser required for parsing data.
    /// - page_data -> Data of the page (wikitext)
    ///
    /// # Returns
    /// - Ok(ExternalLinks) -> An external links struct to use, just made up of parsed links.
    /// - Err(dyn Error) -> Something happened to cause an error.
    pub fn new<'c>(
        wtp: &Bound<'c, PyModule>,
        page_data: &str,
    ) -> Result<ExternalLinks, Box<dyn std::error::Error>> {
        let parsed = wtp.call_method1("parse", (page_data,))?;
        let external_links = parsed.getattr("external_links")?;
        let list = match external_links.cast::<PyList>() {
            Ok(v) => v.to_owned(),
            Err(_) => return Err("Failed to cast into list.".into()),
        };
        Ok(ExternalLinks(
            list.iter()
                .map(|link| ExternalLink::from(link))
                .collect::<Vec<ExternalLink>>(),
        ))
    }

    /// Returns the list but only text of the items.
    ///
    /// # Arguments
    /// - page_name -> An optional argument which allows replacing `{{PAGENAME}}` (a template) with the page name provided.
    ///
    /// # Returns
    /// - Vec<String> -> A list of strings.
    pub fn text_list(&self, page_name: Option<&str>) -> Vec<String> {
        self.0
            .iter()
            .map(|item| item.text_page(page_name))
            .collect::<Vec<String>>()
    }

    /// Returns the list but only url of the items.
    ///
    /// # Returns
    /// - Vec<String> -> A list of strings.
    pub fn url_list(&self) -> Vec<String> {
        self.0
            .iter()
            .map(|item| item.url.to_owned())
            .collect::<Vec<String>>()
    }

    /// Checks to see if a link might contain the provided value (in full)
    ///
    /// This checks **BOTH** url and text fields.
    ///
    /// Useful for checking if pages contain a link back to another page, a badge for example.
    ///
    /// # Arguments
    /// - value -> The value to search for (will be trimmed and forced lowercase)
    /// - page_name -> Optional page name to pass into [text_list].
    ///
    /// # Returns
    /// - bool -> Does the link contain it or not.
    pub fn might_contain(&self, value: &str, page_name: Option<&str>) -> bool {
        let value = value.trim().to_lowercase();
        self.0.iter().any(|link| {
            let text = link.text_page(page_name);
            text.contains(&value)
                || value.contains(&text)
                || link.url.contains(&value)
                || value.contains(&link.url)
        })
    }
}

impl From<&Bound<'_, PyAny>> for ExternalLink {
    fn from(value: &Bound<'_, PyAny>) -> Self {
        let text = match value.getattr("text") {
            Ok(attr) => attr.extract::<String>().unwrap_or_default(),
            Err(_) => String::new(),
        };
        let url = match value.getattr("url") {
            Ok(attr) => attr.extract::<String>().unwrap_or_default(),
            Err(_) => String::new(),
        };
        Self { url, text }
    }
}
impl From<Bound<'_, PyAny>> for ExternalLink {
    fn from(value: Bound<'_, PyAny>) -> Self {
        Self::from(&value)
    }
}

impl ExternalLink {
    /// Convert text to a string and replace stuff if need be.
    ///
    /// # Arguments
    /// - page_name -> An optional string to replace `{{PAGENAME}}` with.
    ///
    /// # Returns
    /// - String -> The usable text.
    pub fn text_page(&self, page_name: Option<&str>) -> String {
        if let Some(name) = page_name {
            self.text.replace("{{PAGENAME}}", name)
        } else {
            self.text.to_owned()
        }
    }
}

impl Template<'_> {
    /// Get the template on the page with the provided name. Returns **first instance**
    ///
    /// # Arguments
    /// - wtp -> Wikitextparser python module required for parsing data.
    /// - page_data -> Data of the page, see `get_wiki_page` for a possible way.
    /// - name -> Name of the template to find.
    ///
    /// # Returns
    /// - Ok(Template) -> A template struct to use.
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

impl WikiTower {
    /// Clean the name to attempt to get better results whilst getting the wiki page.
    ///
    /// Modifies itself because if success, you most likely want this anyway. And besides, we have badge name in case of emergency.
    fn clean_name(&mut self) {
        self.name = self.name.replace("-", " ");
    }
}
