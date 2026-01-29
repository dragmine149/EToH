use crate::{
    definitions::{ProcessError, WikiAPI},
    reqwest_client::RustClient,
};
use itertools::Itertools;
use std::str::FromStr;
use url::{ParseError, Url};

/// Link to the wiki to append to pretty nmuch every single URL.
pub const ETOH_WIKI: &str = "https://jtoh.fandom.com/";
/// Link to the wiki API as it's slightly different and can't just use the same URL...
const ETOH_WIKI_API: &str = "https://jtoh.fandom.com/api.php";

// https://jtoh.fandom.com/api.php?&prop=revisions&titles=ToBT|Tower%20of%20Bent%20Trauma&rvprop=content&rvslots=main
// "{:}?&list=search&srsearch={:}&srlimit={:}",
// "{}?&list=categorymembers&titles={}&cmtitle=Category%3A{}&cmlimit=500",

/// Build the basic wiki api url of everything we need.
fn build_wiki_url() -> Result<Url, ParseError> {
    let mut url = Url::from_str(ETOH_WIKI_API)?;
    url.query_pairs_mut()
        .append_pair("action", "query")
        .append_pair("format", "json")
        .append_pair("formatversion", "2")
        .finish();
    Ok(url)
}

pub async fn get_pages<S: AsRef<str>>(
    client: &RustClient,
    pages: &[S],
) -> Result<WikiAPI, ProcessError> {
    let mut url = build_wiki_url()?;
    url.query_pairs_mut()
        .append_pair("prop", "revision")
        .append_pair("titles", &pages.iter().map(|s| s.as_ref()).join("|"))
        .append_pair("rvprop", "content")
        .append_pair("rvslots", "main")
        .append_pair("redirect", "1")
        .finish();

    Ok(client.get(url).await?.json::<WikiAPI>()?)
}

pub async fn get_search<S: AsRef<str>>(
    client: &RustClient,
    search: S,
    limit: u16,
) -> Result<WikiAPI, ProcessError> {
    let mut url = build_wiki_url()?;
    url.query_pairs_mut()
        .append_pair("list", "search")
        .append_pair("srsearch", search.as_ref())
        .append_pair("srlimit", &limit.min(1).max(500).to_string())
        .finish();

    Ok(client.get(url).await?.json::<WikiAPI>()?)
}

/// Make a URL for querying the wiki for the specified category.
pub fn category_url(category_name: &str) -> String {
    let url = format!(
        "{}?action=query&format=json&list=categorymembers&titles={}&formatversion=2&cmtitle=Category%3A{}&cmlimit=500",
        ETOH_WIKI_API, category_name, category_name
    );
    log::debug!("Build url: {}", url);
    url
}
