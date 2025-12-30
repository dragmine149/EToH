use async_recursion::async_recursion;
use itertools::Itertools;
use reqwest::Response;
use std::{collections::HashMap, error::Error};
use tokio::task::JoinHandle;

use url::Url;

use crate::{
    ETOH_WIKI, clean_badge_name,
    definitions::{Badge, Data, ErrorDetails, OkDetails, PageDetails, ProcessError, WikiSearch},
    reqwest_client::{RustClient, RustError},
    wikitext::WikiText,
};

/// Returns a list of new threads which contain information on every single badge.
///
/// # Usage
/// ```rs
/// let badges = get_badges(&client, &url, &[]).await.unwrap();
/// for badge in badges {
///    // badge can be gotten after awaiting it.
///    println!("{:?}", badge.await);
/// }
/// ```
pub async fn get_badges(
    client: &RustClient,
    url: &Url,
    ignore: &[u64],
) -> Result<Vec<JoinHandle<Result<OkDetails, ErrorDetails>>>, Box<dyn Error>> {
    let mut data: Data = Data::default();
    let mut tasks = vec![];
    // keep going until we run out of cursor to check.
    while let Some(next_page_cursor) = data.next_page_cursor {
        let mut url = url.clone();
        url.query_pairs_mut()
            .append_pair("cursor", &next_page_cursor);

        data = client.0.get(url).send().await?.json::<Data>().await?;

        for badge in data.data {
            if ignore.contains(&badge.id) {
                continue;
            }
            tasks.push(tokio::spawn(pre_process(client.clone(), badge)));
        }
    }
    Ok(tasks)
}

/// Checks to see if the provided badge id is found on the page.
///
/// This is required when searching as page name is not always equal to badge name.
fn is_page_link(page: WikiText, badge: u64) -> Result<WikiText, String> {
    if page.text().contains(&badge.to_string()) {
        Ok(page)
    } else {
        Err("No links to the specific badge were found.".into())
    }
}

/// Wrapper for [process_data] just so we can handle the return type easier.
async fn pre_process(client: RustClient, badge: Badge) -> Result<OkDetails, ErrorDetails> {
    let result = process_data(client.clone(), &badge.name, badge.id, None).await;
    if result.is_err() {
        return Err(ErrorDetails(result.err().unwrap(), badge));
    }
    Ok(OkDetails(result.ok().unwrap(), badge))
}

/// Make a dedicated network reqwest to the wiki.
///
/// # Notes
/// - Will always return the raw text when possible with `?action=raw`
/// - Any form of fragments will be removed `#some_fragment` -> ``
async fn get_page(client: &RustClient, page_name: &str) -> Result<Response, RustError> {
    let mut page_name =
        Url::parse(&format!("{:}wiki/{:}", ETOH_WIKI, page_name)).expect("How is url invalid?");
    page_name.set_fragment(None);
    page_name.set_query(Some("action=raw"));

    log::debug!("Request to {:?}", page_name.as_str().replace("%20", " "));
    Ok(client.get(page_name).send().await?)
}

/// Gets the page by following every single (wiki) redirect that we come across.
#[async_recursion]
pub async fn get_page_redirect(
    client: &RustClient,
    page_name: &str,
) -> Result<PageDetails, RustError> {
    let data = get_page(client, page_name).await?;
    let text = data.error_for_status()?.text().await?;

    // got to have a redirect.
    if text.to_lowercase().contains("#redirect") {
        // if we have #redirect, there will be a match and if there isn't well the page is broken so we fix that externally.
        // under no circumstance should redirect be empty
        let matches = lazy_regex::regex_captures!(r"(?mi)#redirect \[\[(.+)\]\]", &text);
        if matches.is_none() {
            panic!("No matches for {:?} data: {:?}", page_name, text);
        }
        let (_, redirect) = matches.unwrap();
        log::debug!("Redirecting to {:?}", redirect);
        let redirect_result = get_page_redirect(client, redirect).await?;
        return Ok(PageDetails {
            text: redirect_result.text,
            name: Some(redirect_result.name.unwrap_or(redirect.to_owned())),
        });
    }

    Ok(PageDetails {
        text,
        ..Default::default()
    })
}

/// Attempts to:
/// - get the badge
/// - get the badge but cleaner
/// - get the badge but searching
///
/// Badge link is important and not always as simple to get, especially with some weird stuff in names sometimes.
#[async_recursion]
async fn process_data(
    client: RustClient,
    badge: &String,
    badge_id: u64,
    search: Option<&String>,
) -> Result<WikiText, ProcessError> {
    let mut page_title = clean_badge_name(badge);
    // log::debug!("Getting: {:?} ({:?})", page_title, badge_id);

    let mut page_data = get_page_redirect(&client, &page_title).await;
    if page_data.is_err() {
        // recheck but with cleaning the input.
        page_title = page_title
            .replace("-", " ")
            .replace("!", "")
            .trim()
            .to_string();
        page_data = get_page_redirect(&client, &page_title).await;
    }

    // cool, we can return early now that we have data.
    if let Ok(text) = page_data {
        let mut wikitext = WikiText::parse(text.text);
        wikitext.set_page_name(Some(text.name.unwrap_or(page_title.clone())));

        if search.is_some() && *search.unwrap() != page_title {
            // as we're searching a different page than the badge name, we just need to check to make sure there IS a link.
            return Ok(is_page_link(wikitext, badge_id)?);
        }
        return Ok(wikitext);
    }

    // if we aren't already in search mode
    if search.is_none() {
        // search the next 3 entries.
        let pages = client
            .get(format!(
                "{:}api.php?action=query&format=json&list=search&srsearch={:}&srlimit={:}",
                ETOH_WIKI, badge, 3
            ))
            .send()
            .await?
            .json::<WikiSearch>()
            .await?;

        // loop through each entry and return the first valid entry.
        // Normally this is the first entry, but there is always a chance it isn't.
        for entry in pages.query.search {
            // TODO: Sort out secret badges
            if entry.title.contains("/") {
                continue;
            }
            if entry.title == page_title {
                // something went wrong here
                log::error!("How entry is title?? {:?}", entry.title);
                continue;
            }

            let search_page =
                process_data(client.clone(), &entry.title, badge_id, Some(badge)).await;
            if search_page.is_ok() {
                return search_page;
            }
        }
    }
    Err("Failed to find the page after searching".into())
}

pub async fn get_annoying(
    client: &RustClient,
    badges: &[&Badge],
    annoying_links: &HashMap<String, String>,
) -> Vec<Result<OkDetails, ErrorDetails>> {
    let mut annoying = Vec::with_capacity(annoying_links.len());

    for (id, url) in annoying_links.iter() {
        let badge = badges
            .iter()
            .find(|b| b.id == id.parse::<u64>().expect("Failed to parse badge id"))
            .expect("Failed to find badge!")
            .to_owned();

        let data = get_page_redirect(client, url)
            .await
            .map(|ok| {
                let mut wt = WikiText::parse(ok.text);
                wt.set_page_name(Some(url));
                OkDetails(wt, badge.to_owned())
            })
            .map_err(|err| ErrorDetails(err.into(), badge.to_owned()));

        annoying.push(data);
    }

    annoying
}
