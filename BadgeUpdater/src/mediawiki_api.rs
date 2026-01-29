use crate::{
    definitions::{ProcessError, WikiAPI},
    reqwest_client::RustClient,
};
use itertools::Itertools;
use std::str::FromStr;
use url::{ParseError, Url};

/// Link to the wiki to append to pretty nmuch every single URL.
// pub const ETOH_WIKI: &str = "https://jtoh.fandom.com/";
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
        .append_pair("srlimit", &limit.clamp(1, 500).to_string())
        .finish();

    Ok(client.get(url).await?.json::<WikiAPI>()?)
}

pub async fn get_category<S: AsRef<str>>(
    client: &RustClient,
    category_name: &str,
    limit: u16,
) -> Result<WikiAPI, ProcessError> {
    let mut url = build_wiki_url()?;
    url.query_pairs_mut()
        .append_pair("list", "categorymembers")
        .append_pair("cmtitle", &format!("Category:{}", category_name))
        .append_pair("cmlimit", &limit.clamp(1, 500).to_string())
        .finish();

    Ok(client.get(url).await?.json::<WikiAPI>()?)
}

pub async fn get_pages_from_category<S: AsRef<str>>(
    client: &RustClient,
    category_name: &str,
    limit: u16,
) -> Result<WikiAPI, ProcessError> {
    let category = get_category::<&str>(client, category_name, limit).await?;
    if let Some(members) = category.query.categorymembers {
        let pages = get_pages(
            client,
            &members
                .iter()
                .map(|member| member.title.clone())
                .collect_vec(),
        )
        .await?;
        Ok(pages)
    } else {
        Err("No categorymembers found".into())
    }
}
