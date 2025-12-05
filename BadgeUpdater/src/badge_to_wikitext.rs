use async_recursion::async_recursion;
use reqwest::Response;
use std::error::Error;
use tokio::task::JoinHandle;

use url::Url;

use crate::{
    ETOH_WIKI, clean_badge_name,
    definitions::{Badge, Data, ErrorDetails, OkDetails, PageDetails, ProcessError, WikiSearch},
    reqwest_client::{RustClient, RustError},
    wikitext::WikiText,
};

pub async fn get_badges(
    client: &RustClient,
    url: &Url,
) -> Result<Vec<JoinHandle<Result<OkDetails, ErrorDetails>>>, Box<dyn Error>> {
    let mut data: Data = Data::default();
    let mut tasks = vec![];
    while let Some(next_page_cursor) = data.next_page_cursor {
        let mut url = url.clone();
        url.query_pairs_mut()
            .append_pair("cursor", &next_page_cursor);

        data = client.0.get(url).send().await?.json::<Data>().await?;

        for badge in data.data {
            tasks.push(tokio::spawn(pre_process(client.clone(), badge)));
        }
    }
    Ok(tasks)
}

fn is_page_link(page: WikiText, badge: u64) -> Result<WikiText, String> {
    if page.text().contains(&badge.to_string()) {
        Ok(page)
    } else {
        Err("No links to the specific badge were found.".into())
    }
}

async fn pre_process(client: RustClient, badge: Badge) -> Result<OkDetails, ErrorDetails> {
    let result = process_data(client.clone(), &badge.name, badge.id, None).await;
    if result.is_err() {
        return Err(ErrorDetails(result.err().unwrap(), badge));
    }
    Ok(OkDetails(result.ok().unwrap(), badge))
}

async fn get_page(client: &RustClient, page_name: &str) -> Result<Response, RustError> {
    log::debug!(
        "Request to \"{:}wiki/{:}?action=raw\"",
        ETOH_WIKI,
        page_name
    );
    Ok(client
        .get(format!("{:}wiki/{:}?action=raw", ETOH_WIKI, page_name))
        .send()
        .await?)
}

#[async_recursion]
pub async fn get_page_redirect(
    client: &RustClient,
    page_name: &str,
) -> Result<PageDetails, RustError> {
    let data = get_page(client, page_name).await?;
    let text = data.error_for_status()?.text().await?;

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
    if let Ok(text) = page_data {
        let mut wikitext = WikiText::parse(text.text);
        wikitext.set_page_name(Some(text.name.unwrap_or(page_title.clone())));
        if search.is_some() && *search.unwrap() != page_title {
            return Ok(is_page_link(wikitext, badge_id)?);
        }
        return Ok(wikitext);
    }

    if search.is_none() {
        let pages = client
            .get(format!(
                "{:}api.php?action=query&format=json&list=search&srsearch={:}&srlimit={:}",
                ETOH_WIKI, badge, 3
            ))
            .send()
            .await?
            .json::<WikiSearch>()
            .await?;

        for entry in pages.query.search {
            if entry.title.contains("/") {
                continue;
            }
            if entry.title == page_title {
                log::warn!("How entry is title?? {:?}", entry.title);
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
