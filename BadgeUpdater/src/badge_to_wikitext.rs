//! Get badges and convert them to wikitext
//!
//! Everything here is related to the retrieval of data. Processing said data is done elsewhere.

use crate::{
    clean_badge_name,
    definitions::{Badge, Badges, ErrorDetails, OkDetails, ProcessError, RobloxBadgeData},
    mediawiki_api::{get_pages, get_search},
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
    let page_data = get_pages(
        client,
        &badges
            .iter()
            .map(|b| b.annoying.as_ref().unwrap_or(&b.name))
            .collect_vec(),
    )
    .await?;
    let mut results = vec![];
    let mut searches = vec![];
    if let Some(ref pages) = page_data.query.pages {
        for page in pages {
            let entry_badge = badges.iter().find(|b| b.name == page.title).unwrap();

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
    }
    for search in searches {
        let search_data = get_search(client, &clean_badge_name(&search.name), 4).await?;
        if let Some(searches) = search_data.query.search {
            let pages = get_pages(
                client,
                &searches
                    .iter()
                    .filter(|s| !s.title.contains("/"))
                    .map(|s| &s.title)
                    .collect_vec(),
            )
            .await?;
            for page in pages.query.pages.unwrap() {
                // page should not be missing as it wouldn't be here...

                let content = &page.get_content().unwrap().content;
                if search.check_ids(content) {
                    let mut wt = WikiText::parse(content);
                    wt.set_page_name(Some(page.title));
                    results.push(Ok(OkDetails(wt, search.to_owned())));
                    break;
                }
            }
        }
    }
    if results.is_empty() {
        Err("No pages were returned in API requests".into())
    } else {
        Ok(results)
    }
}
