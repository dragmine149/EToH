use async_recursion::async_recursion;
use reqwest::Response;
use std::error::Error;
use tokio::task::JoinHandle;

use url::Url;

use crate::{
    ETOH_WIKI, clean_badge_name,
    definitions::{Badge, Data, ErrorDetails, OkDetails, ProcessError, WikiSearch},
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

pub async fn get_page(client: &RustClient, page_name: &str) -> Result<Response, RustError> {
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
async fn process_data(
    client: RustClient,
    badge: &String,
    badge_id: u64,
    search: Option<&String>,
) -> Result<WikiText, ProcessError> {
    let mut page_title = Some(clean_badge_name(badge));
    log::debug!("Getting: {:?} ({:?})", page_title, badge_id);

    let mut clean = false;
    while let Some(ref redirect) = page_title {
        // initial response to get data
        let data = get_page(&client, redirect).await?;
        println!(
            "{:?}, {:?}, {:?}",
            data.url().as_str(),
            badge_id,
            data.status().as_str()
        );

        // Process success first as the rest has to loop anyway.
        // Normally i would do this last, but it's easier here.
        if data.status().is_success() {
            let mut page = WikiText::parse(&data.text().await?);
            page.set_page_name(Some(redirect.to_owned()));
            if search.is_some() && search.unwrap() != redirect {
                return Ok(is_page_link(page, badge_id)?);
            }
            return Ok(page);
        }

        // retry, but clean it if we haven't already cleaned it.
        if !clean {
            page_title = Some(
                page_title
                    .unwrap_or_default()
                    .replace("-", " ")
                    .replace("!", "")
                    .trim()
                    .to_string(),
            );
            clean = true;
            continue;
        }

        page_title = None;
    }

    // Assumption: Failed clean, check so we search.

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
            // if entry.title.contains("/") {
            //     continue;
            // }

            let search_page =
                process_data(client.clone(), &entry.title, badge_id, Some(badge)).await;
            if search_page.is_ok() {
                return search_page;
            }
        }
    }
    Err("Failed to find page after searching!".into())
}
