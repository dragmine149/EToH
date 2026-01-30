use crate::{
    definitions::{ProcessError, WikiAPI, WikiPageEntry},
    reqwest_client::RustClient,
};
use itertools::Itertools;
use std::str::FromStr;
use url::{ParseError, Url};

/// Link to the wiki to append to pretty nmuch every single URL.
// pub const ETOH_WIKI: &str = "https://jtoh.fandom.com/";
/// Link to the wiki API as it's slightly different and can't just use the same URL...
const ETOH_WIKI_API: &str = "https://jtoh.fandom.com/api.php";

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

pub async fn get_pages_limited<S: AsRef<str>>(
    client: &RustClient,
    pages: &[S],
) -> Vec<Result<WikiPageEntry, ProcessError>> {
    let mut result_pages = Vec::with_capacity(pages.len());
    for chunk in pages.chunks(50) {
        let page = get_pages(client, chunk).await;
        match page {
            Ok(p) => p.query.pages.unwrap().iter().for_each(|page| {
                // println!("{:?}", p.query.redirects);

                let mut owned_page = page.to_owned();
                let redirect_from = if let Some(norm) = p.query.normalized.as_ref() {
                    norm.iter()
                        .find(|n| n.to == page.title)
                        .map(|f| f.from.to_owned())
                } else {
                    None
                };
                let redirect_from = if redirect_from.is_none() {
                    if let Some(redirect) = p.query.redirects.as_ref() {
                        // println!("Redirect! {:?}", redirect);
                        redirect
                            .iter()
                            .find(|r| r.to == page.title)
                            .map(|f| f.from.to_owned())
                    } else {
                        None
                    }
                } else {
                    redirect_from
                };

                owned_page.redirected = redirect_from;
                // if page.title == "Tower of High Quality Fishing Boat" {
                //     println!("{:#?}", owned_page);
                // }

                result_pages.push(Ok(owned_page));
            }),
            Err(e) => chunk.iter().for_each(|c| {
                result_pages.push(Err(format!("{:?} on {:?}", e, c.as_ref()).into()));
            }),
        }
    }
    result_pages
}

pub async fn get_pages<S: AsRef<str>>(
    client: &RustClient,
    pages: &[S],
) -> Result<WikiAPI, ProcessError> {
    let mut url = build_wiki_url()?;
    url.query_pairs_mut()
        .append_pair("prop", "revisions")
        .append_pair("titles", &pages.iter().map(|s| s.as_ref()).join("|"))
        .append_pair("rvprop", "content")
        .append_pair("rvslots", "main")
        .append_pair("redirects", "1")
        .finish();

    log::debug!("pages: {}", url);
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
