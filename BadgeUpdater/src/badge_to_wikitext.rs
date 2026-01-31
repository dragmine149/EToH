//! Get badges and convert them to wikitext
//!
//! Everything here is related to the retrieval of data. Processing said data is done elsewhere.

use crate::{
    clean_badge_name,
    definitions::{Badge, Badges, ErrorDetails, OkDetails, ProcessError, RobloxBadgeData},
    mediawiki_api::{get_pages_limited, get_search},
    reqwest_client::RustClient,
    wikitext::WikiText,
};
use itertools::Itertools;

/// Get a list of badges from roblox.
///
/// # Arguments
/// * client: The client to use to ping the API.
/// * game_id: The id of the game to get the badges for.
/// * ignore: Any badges linked with these ids won't be returned in the final result.
pub async fn get_badges(
    client: &RustClient,
    game_id: &u64,
    ignore: &[u64],
) -> Result<Vec<Badge>, ProcessError> {
    let mut data = RobloxBadgeData::default();
    let mut results = vec![];
    while let Some(next_page_cursor) = data.next_page_cursor {
        data = client
            .get(format!(
                "https://badges.roblox.com/v1/universes/{}/badges?limit=100&cursor={}",
                game_id, next_page_cursor
            ))
            .await?
            .json::<RobloxBadgeData>()?;
        for b in data.data {
            results.push(b);
        }
    }
    Ok(results
        .iter()
        .filter(|b| !ignore.contains(&b.id))
        .map(|b| b.to_owned())
        .collect_vec())
}

/// Translate the badges into as many wiki pages as possible
///
/// # Arguments
/// * client: The client to use to ping the API.
/// * badges: The list of badges to parse.
pub async fn get_wiki_pages(
    client: &RustClient,
    badges: &[Badges],
) -> Result<Vec<Result<OkDetails, ErrorDetails>>, ProcessError> {
    let page_data = get_pages_limited(
        client,
        &badges
            .iter()
            .map(|b| clean_badge_name(b.annoying.as_ref().unwrap_or(&b.name)))
            .collect_vec(),
    )
    .await;

    // results is our global result.
    let mut results = vec![];
    // searches is a temporary list of badges that need to be sent via the search api to try for better results.
    let mut searches = vec![];
    log::info!("Attempting page link search");
    // println!("{:#?}", badges);
    for page in page_data {
        // it's kinda hard to return a `ErrorDetails` hence we have to return the main error.
        // we would have done this normally if using [mediawiki_api::get_pages] anyway.
        let page = page?;
        log::debug!("{:?}", page.title);
        // println!("{:#?}", page);
        let entry_badge = badges
            .iter()
            .find(|b| b.is_badge(&page))
            .unwrap_or_else(|| panic!("Failed to find a badge with name '{}'", page.title));

        if let Some(miss) = page.missing
            && miss
        {
            // don't have to worry about redirects because a missing page wouldn't have been redirected.
            // well, in theory at least.
            //
            // in theory, find should never fail either.
            searches.push(entry_badge);
            continue;
        }
        // if we have a link, parse and return it.
        let content = &page.get_content().unwrap().content;
        let mut wt = WikiText::parse(content);
        wt.set_page_name(Some(page.title.to_owned()));
        results.push(Ok(OkDetails(wt, entry_badge.clone())));
    }
    log::info!(
        "Pages found. Searching wiki for {} items. Found items: {}",
        searches.len(),
        results.len()
    );
    // println!("{:#?}", results);
    // same as above, but just for searching instead.
    for search in searches {
        let search_data = get_search(client, &clean_badge_name(&search.name), 4).await?;
        if let Some(searches) = search_data.query.search {
            let pages = get_pages_limited(
                client,
                &searches
                    .iter()
                    .filter(|s| !s.title.contains("/"))
                    .map(|s| &s.title)
                    .collect_vec(),
            )
            .await;

            let mut searched = false;
            for page in pages {
                let page = page?;
                // page should not be missing as it wouldn't be here...

                // if we find a link, we break out early to avoid the rest being searched.
                let content = &page.get_content().unwrap().content;
                if search.check_ids(content) {
                    let mut wt = WikiText::parse(content);
                    wt.set_page_name(Some(page.title));
                    results.push(Ok(OkDetails(wt, search.to_owned())));
                    searched = true;
                    break;
                }
            }

            // and for when we fail to break, we error.
            if !searched {
                results.push(Err(ErrorDetails(
                    "Failed to get badge by searching".into(),
                    search.to_owned(),
                )));
            }
        }
    }
    log::info!("Search completed, returning results");
    if results.is_empty() {
        Err("No pages were returned in API requests".into())
    } else {
        Ok(results)
    }
}
