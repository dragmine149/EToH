use futures::io::ReadToEnd;
use serde::{Deserialize, Serialize};
use std::{error::Error, str::FromStr};
use tokio::task::JoinHandle;

use url::Url;

use crate::{
    ETOH_WIKI, clean_badge_name,
    reqwest_client::{RustClient, RustError},
    wikitext::parser::{ParseResult, WikiText},
};

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BadgeUniverse {
    pub id: u64,
    pub name: String,
    pub root_place_id: u64,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BadgeStatistics {
    pub past_day_awarded_count: u64,
    pub awarded_count: u64,
    pub win_rate_percentage: f64,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Badge {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub display_name: String,
    pub display_description: Option<String>,
    pub enabled: bool,
    pub icon_image_id: u64,
    pub display_icon_image_id: u64,
    pub created: String,
    pub updated: String,
    pub statistics: BadgeStatistics,
    pub awarding_universe: BadgeUniverse,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub previous_page_cursor: Option<String>,
    pub next_page_cursor: Option<String>,
    pub data: Vec<Badge>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WikiSearchEntry {
    pub title: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct WikiSearchList {
    pub search: Vec<WikiSearchEntry>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct WikiSearch {
    pub query: WikiSearchList,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            previous_page_cursor: None,
            next_page_cursor: Some(String::new()),
            data: vec![],
        }
    }
}

pub async fn get_badges(
    client: RustClient,
    url: &Url,
) -> Result<Vec<JoinHandle<Result<WikiText, RustError>>>, Box<dyn Error>> {
    let mut data: Data = Data::default();
    let mut tasks = vec![];
    while let Some(next_page_cursor) = data.next_page_cursor {
        let mut url = url.clone();
        url.query_pairs_mut()
            .append_pair("cursor", &next_page_cursor);

        data = client.0.get(url).send().await?.json::<Data>().await?;

        for badge in data.data {
            tasks.push(tokio::spawn(process_data(client.clone(), badge.name)))
        }
    }
    Ok(tasks)
}

async fn process_data(client: RustClient, badge: String) -> Result<WikiText, RustError> {
    let mut page = WikiText {
        parsed: ParseResult::new(),
        redirect: Some(clean_badge_name(&badge)),
    };

    let mut clean = false;
    let mut search = false;
    while let Some(redirect) = page.get_redirect() {
        let data = client
            .get(format!("{:}wiki/{:}?action=raw", ETOH_WIKI, redirect))
            .send()
            .await?;

        if data.status().is_client_error() {
            if !clean {
                page.redirect = Some(
                    page.redirect
                        .unwrap_or_default()
                        .replace("-", " ")
                        .trim()
                        .to_string(),
                );
                clean = true;
                continue;
            }

            if !search {
                let pages = client
                    .get(format!(
                        "{:}api.php?action=query&format=json&list=search&srsearch={:}&srlimit={:}",
                        ETOH_WIKI, redirect, 3
                    ))
                    .send()
                    .await?
                    .json::<WikiSearch>()
                    .await?;

                search = true;
                continue;
            }
        }

        page = WikiText::parse(&data.text().await?);
    }

    Ok(page)
}
