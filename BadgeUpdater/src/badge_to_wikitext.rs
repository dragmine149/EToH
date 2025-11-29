use async_recursion::async_recursion;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio::task::JoinHandle;

use url::Url;

use crate::{
    ETOH_WIKI, clean_badge_name,
    reqwest_client::{RustClient, RustError},
    wikitext::parser::WikiText,
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

#[derive(Debug)]
pub enum ProcessError {
    Reqwest(RustError),
    Process(String),
}
impl From<RustError> for ProcessError {
    fn from(value: RustError) -> Self {
        Self::Reqwest(value)
    }
}
impl From<reqwest::Error> for ProcessError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(RustError::from(value))
    }
}
impl From<reqwest_middleware::Error> for ProcessError {
    fn from(value: reqwest_middleware::Error) -> Self {
        Self::Reqwest(RustError::from(value))
    }
}
impl From<String> for ProcessError {
    fn from(value: String) -> Self {
        Self::Process(value)
    }
}
impl From<&str> for ProcessError {
    fn from(value: &str) -> Self {
        Self::Process(value.to_owned())
    }
}

pub async fn get_badges(
    client: RustClient,
    url: &Url,
) -> Result<Vec<JoinHandle<Result<WikiText, ProcessError>>>, Box<dyn Error>> {
    let mut data: Data = Data::default();
    let mut tasks = vec![];
    while let Some(next_page_cursor) = data.next_page_cursor {
        let mut url = url.clone();
        url.query_pairs_mut()
            .append_pair("cursor", &next_page_cursor);

        data = client.0.get(url).send().await?.json::<Data>().await?;

        for badge in data.data {
            tasks.push(tokio::spawn(process_data(
                client.clone(),
                badge.name,
                badge.id,
                true,
            )))
        }
    }
    Ok(tasks)
}

fn is_page_link(page: WikiText, badge: u64) -> Result<WikiText, String> {
    if page.raw.as_str().contains(&badge.to_string()) {
        Ok(page)
    } else {
        Err("No links to the specific badge were found.".into())
    }
}

#[async_recursion]
async fn process_data(
    client: RustClient,
    badge: String,
    badge_id: u64,
    search: bool,
) -> Result<WikiText, ProcessError> {
    let mut page_title = Some(clean_badge_name(&badge));
    log::debug!("Getting: {:?} ({:?})", page_title, badge_id);

    let mut clean = false;
    while let Some(ref redirect) = page_title {
        // initial response to get data
        let data = client
            .get(format!("{:}wiki/{:}?action=raw", ETOH_WIKI, redirect))
            .send()
            .await?;

        // Process success first as the rest has to loop anyway.
        // Normally i would do this last, but it's easier here.
        if data.status().is_success() {
            return Ok(is_page_link(
                WikiText::parse(&data.text().await?),
                badge_id,
            )?);
        }

        // retry, but clean it if we haven't already cleaned it.
        if !clean {
            page_title = Some(
                page_title
                    .unwrap_or_default()
                    .replace("-", " ")
                    .trim()
                    .to_string(),
            );
            clean = true;
            continue;
        }

        page_title = None;
    }

    // Assumption: Failed clean, check so we search.

    if search {
        let pages = client
            .get(format!(
                "{:}api.php?action=query&format=json&list=search&srsearch={:}&srlimit={:}",
                ETOH_WIKI, badge, 1
            ))
            .send()
            .await?
            .json::<WikiSearch>()
            .await?;

        for entry in pages.query.search {
            let search_page = process_data(client.clone(), entry.title, badge_id, false).await;
            if search_page.is_ok() {
                return Ok(search_page?);
            }
        }
    }
    Err("Failed to find page after searching!".into())
}
