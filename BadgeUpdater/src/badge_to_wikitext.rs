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
use url::Url;

pub async fn get_badges(
    client: &RustClient,
    url: Url,
    ignore: &[u64],
) -> Result<Vec<Badge>, ProcessError> {
    let mut data = RobloxBadgeData::default();
    let mut results = vec![];
    while let Some(next_page_cursor) = data.next_page_cursor {
        let mut url = url.clone();
        url.query_pairs_mut()
            .append_pair("cursor", &next_page_cursor);

        data = client.get(url).await?.json::<RobloxBadgeData>()?;
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
    let mut results = vec![];
    let mut searches = vec![];
    log::info!("Attempting page link search");
    println!("{:#?}", badges);
    for page in page_data {
        // it's kinda hard to return a `ErrorDetails` hence we have to return the main error.
        // we would have done this normally if using [mediawiki_api::get_pages] anyway.
        let page = page?;
        log::debug!("{:?}", page.title);
        println!("{:#?}", page);
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
        let content = page.get_content().unwrap().content.clone();
        let mut wt = WikiText::parse(&content);
        wt.set_page_name(Some(page.title.to_owned()));
        results.push(Ok(OkDetails(wt, entry_badge.clone())));
    }
    log::info!(
        "Pages found. Searching wiki for {} items. Found items: {}",
        searches.len(),
        results.len()
    );
    println!("{:#?}", results);
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

                let content = &page.get_content().unwrap().content;
                if search.check_ids(content) {
                    let mut wt = WikiText::parse(content);
                    wt.set_page_name(Some(page.title));
                    results.push(Ok(OkDetails(wt, search.to_owned())));
                    searched = true;
                    break;
                }
            }

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
