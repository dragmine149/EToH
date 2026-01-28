//! Get badges and convert them to wikitext
//!
//! Everything here is related to the retrieval of data. Processing said data is done elsewhere.

use crate::{
    ETOH_WIKI, ETOH_WIKI_API, clean_badge_name,
    definitions::{
        Badge, BadgeDetails, BadgeError, ErrorDetails, OkDetails, PageDetails, ProcessError,
        RobloxBadgeData, WikiAPI,
    },
    reqwest_client::{ResponseBytes, RustClient, RustError},
    wikitext::WikiText,
};
use async_recursion::async_recursion;
use std::collections::HashMap;
use tokio::task::JoinHandle;
use url::Url;

/// Returns a list of new threads which contain information on every single badge.
///
/// # Arguments
///   * client: The client to use for web requests
///   * url: The URL to request the badges from
///   * ignore: A list of badge ids to ignore ass they have already been processed.
///   * callback: A function to call to process the badge upon receiving the data.
///   * callback_args: Additional arguments to send to the function.
///     TODO: Make it optional to send args
///
/// # Usage
/// ```
/// // NOTE: Please do something with client and badge... You have them for a reason.
/// let badges = get_badges(&client, &url, &[], async |client, badge| badge, 0).await.unwrap();
/// for badge in badges {
///    // badge can be gotten after awaiting it.
///    println!("{:?}", badge.await);
/// }
/// ```
/// # Returns
/// A complex result object which can be split up into the following.
///   * Outer result
///     * Ok: All requests to roblox succeeded and this is the data after the callback function.
///     * Err: **ANY** request to roblox failed and this is the details why. Do note that any previous succeeded reqwests will be dropped.
///   * Inner result: This depends on what the callback function returns for us.
///
pub async fn get_badges<F, Fut, O, E, A>(
    client: &RustClient,
    url: &Url,
    ignore: &[u64],
    callback: F,
    callback_args: A,
) -> Result<Vec<JoinHandle<Result<O, E>>>, ProcessError>
where
    F: Fn(RustClient, Badge, A) -> Fut,
    A: Send + Clone + 'static,
    Fut: Future<Output = Result<O, E>> + Send + 'static,
    E: Send + 'static,
    O: Send + 'static,
{
    let mut data: RobloxBadgeData = RobloxBadgeData::default();
    let mut tasks = vec![];
    // keep going until we run out of cursor to check.
    while let Some(next_page_cursor) = data.next_page_cursor {
        let mut url = url.clone();
        url.query_pairs_mut()
            .append_pair("cursor", &next_page_cursor);

        data = client.get(url).await?.json::<RobloxBadgeData>()?;

        for badge in data.data {
            if ignore.contains(&badge.id) {
                continue;
            }

            // we have to clone the client here so that each thread has their own client.
            let client = client.clone();
            let args = callback_args.clone();
            tasks.push(tokio::spawn(callback(client, badge, args)));
        }
    }
    Ok(tasks)
}

/// Smaller version of [get_badges] but just returns the badges and nothing special.
///
/// # Arguments
///   * client: The client to use for web requests
///   * url: The URL to request the badges from
///   * ignore: A list of badge ids to ignore ass they have already been processed.
///
/// # Usage
/// ```
/// let badges = get_badges(&client, &url, &[]).await.unwrap();
/// ```
///
/// # Returns
///   * Ok: All requests to roblox succeeded.
///   * Err: **ANY** request to roblox failed and this is the details why. Do note that any previous succeeded reqwests will be dropped.
///
pub async fn get_quick_badges(
    client: &RustClient,
    url: &Url,
    ignore: &[u64],
) -> Result<Vec<Badge>, ProcessError> {
    let mut data: RobloxBadgeData = RobloxBadgeData::default();
    let mut badges = vec![];
    // keep going until we run out of cursor to check.
    while let Some(next_page_cursor) = data.next_page_cursor {
        let mut url = url.clone();
        url.query_pairs_mut()
            .append_pair("cursor", &next_page_cursor);

        data = client.get(url).await?.json::<RobloxBadgeData>()?;
        badges.extend(
            data.data
                .iter()
                .filter(|b| !ignore.contains(&b.id))
                .cloned(),
        )
    }
    Ok(badges)
}

/// Wrapper for [get_badges] which just does [pre_process] as the callback.
pub async fn get_badges_wiki(
    client: &RustClient,
    url: &Url,
    ignore: &[u64],
) -> Result<Vec<JoinHandle<Result<OkDetails, ErrorDetails>>>, ProcessError> {
    get_badges(
        client,
        url,
        ignore,
        async |client, badge, _| pre_process(client, badge).await,
        0,
    )
    .await
}

/// Combines [get_badges], [get_quick_badges] and [pre_process] to get every single badge and the associated wiki data.
///
/// Note: Wikidata will be based off the newest badge, aka the one at the end of the array.
pub async fn get_all_badges_wiki_edition(
    client: &RustClient,
    url: &[&Url; 2],
    ignore: &[u64],
) -> Result<Vec<JoinHandle<Result<BadgeDetails, BadgeError>>>, ProcessError> {
    let old = get_quick_badges(client, url[0], ignore).await?;
    // got to pass in old to each individual thread semi-annoyingly...
    get_badges(
        client,
        url[1],
        ignore,
        async |client, badge, old| {
            // we searach by name as that is the most reliable.
            // Not same name, probably not same badge.
            let relationship = &old
                .iter()
                .find(|o| o.name == badge.name)
                .cloned()
                .unwrap_or_default();
            let details = pre_process(client, badge).await;
            details
                .map(|ok| BadgeDetails(ok.0, [relationship.to_owned(), ok.1]))
                .map_err(|err| BadgeError(err.0, [relationship.to_owned(), err.1]))
        },
        old,
    )
    .await
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
///
/// also attaches details about the badge to the return type.
///
/// # Arguments
/// * client: An owned version of the client, used to get data from the wiki.
/// * badge: An owned version of the badge, used to find the page and confirm it.
pub async fn pre_process(client: RustClient, badge: Badge) -> Result<OkDetails, ErrorDetails> {
    let result = process_data(&client, &badge.name, badge.id, None).await;
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
async fn get_page(client: &RustClient, page_name: &str) -> Result<ResponseBytes, RustError> {
    let mut page_name =
        Url::parse(&format!("{:}wiki/{:}", ETOH_WIKI, page_name)).expect("How is url invalid?");
    page_name.set_fragment(None);
    page_name.set_query(Some("action=raw"));

    log::debug!("Request to {:?}", page_name.as_str().replace("%20", " "));
    client.get(page_name).await
}

/// Gets the page by following every single (wiki) redirect that we come across.
///
/// Note: Use [get_page_data] instead. This will provide better return types and support.
///
/// # Arguments
/// * client: A reference to the client.
/// * page_name: The page to get information for.
///
/// # Returns
/// * Ok(PageDetails): Information about the page after all redirects.
/// * Err(RustError): Information about errors.
#[async_recursion]
async fn get_page_redirect(client: &RustClient, page_name: &str) -> Result<PageDetails, RustError> {
    let data = get_page(client, page_name).await?;
    let text = data.text()?;

    // got to have a redirect.
    if text.to_lowercase().contains("#redirect") {
        // if we have #redirect, there will be a match and if there isn't well the page is broken so we fix that externally.
        // under no circumstance should redirect be empty
        //
        // Technically, `#redirect` can be commented out but eh, thats a very rare issue.
        let (_, redirect) = lazy_regex::regex_captures!(r"(?mi)#redirect \[\[(.+)\]\]", &text)
            .unwrap_or_else(|| panic!("No matches for {:?} data: {:?}", page_name, text));
        log::debug!("Redirecting to {:?}", redirect);
        let redirect_result = get_page_redirect(client, redirect).await?;
        // returns the redirect version. Page name is included although we pioritise the redirect as thats more useful for back-tracking.
        return Ok(PageDetails {
            text: redirect_result.text,
            name: Some(redirect_result.name.unwrap_or(redirect.to_owned())),
        });
    }

    // just return the bog standard text. Thats all we need to worry about.
    Ok(PageDetails {
        text: text.to_owned(),
        ..Default::default()
    })
}

/// Attempts to get the badge and if that fails attempts again but just with some slightly different options.
///
/// Recommendation: Use [pre-process] instead as that has less parameters and better returns.
///
/// Badge link is important and not always as simple to get, especially with some weird stuff in names sometimes.
#[async_recursion]
async fn process_data(
    client: &RustClient,
    badge: &String,
    badge_id: u64,
    search: Option<&String>,
) -> Result<WikiText, ProcessError> {
    // clean the badge name, this does pre-processing and make it actually usable.
    let mut clean_badge = clean_badge_name(badge);
    // log::debug!("Getting: {:?} ({:?})", page_title, badge_id);

    // initial search of the page.
    let mut page_data = get_page_redirect(client, &clean_badge).await;
    if page_data.is_err() {
        // recheck directly after a failed attempt, but replace some of the separators with other separators.
        clean_badge = clean_badge.replace("-", " ").trim().to_string();
        page_data = get_page_redirect(client, &clean_badge).await;
    }

    // cool, we can return early now that we have data.
    if let Ok(text) = page_data {
        let mut wikitext = WikiText::parse(text.text);
        wikitext.set_page_name(Some(text.name.unwrap_or(clean_badge.clone())));

        if search.is_some() && *search.unwrap() != clean_badge {
            // as we're searching a different page than the badge name, we just need to check to make sure there IS a link.
            return Ok(is_page_link(wikitext, badge_id)?);
        }
        return Ok(wikitext);
    }

    // We can only search once per loop, searching more would just end up stuck in infinite loop.
    if search.is_none() {
        // search the next X entries.
        let pages = client
            .get(format!(
                "{:}?action=query&format=json&list=search&srsearch={:}&srlimit={:}",
                ETOH_WIKI_API, clean_badge, 4
            ))
            .await?
            .json::<WikiAPI>()?;
        println!("{:?} ({:?}) ->\n{:#?}", clean_badge, badge, pages);

        let search_list = pages
            .query
            .search
            .ok_or("A search object was not returned by the API!")?;

        // loop through each entry and return the first valid entry.
        // Normally this is the first entry, but there is always a chance it isn't.
        for entry in search_list {
            // Subpages just don't count at all. They just annoy stuff.
            // If the wiki has a lot of subpages which we need to check, then we'll deal with it in the future.
            if entry.title.contains("/") {
                // This could be useful for secret routes, but half of them don't follow convention.
                continue;
            }
            if entry.title == clean_badge {
                // something went wrong here
                // If entry title is badge, we should have already got it earlier in a direct attack.
                log::error!("How entry is title?? {:?}", entry.title);
                continue;
            }

            // run this function again but with our search entry.
            let search_page = process_data(client, &entry.title, badge_id, Some(badge)).await;
            if search_page.is_ok() {
                return search_page;
            }
        }
    }

    // if all else fails, we just have to admit defeat.
    Err("Failed to find the page after searching".into())
}

/// There are some (rare) badges where fandom just does not want to link the search for us.
/// Hence we have to do it ourselves.
///
/// Compared to other functions though, we need to insert the return data back into the main list (as this is basically a search bypass).
/// So its slightly more annoying. as per the name...
pub async fn get_annoying(
    client: &RustClient,
    badges: &[&[Badge; 2]],
    annoying_links: &HashMap<String, String>,
) -> Vec<Result<BadgeDetails, BadgeError>> {
    // pre-make vector. Easier to make new than having to worry about lifetime and hashmap.
    let mut annoying = Vec::with_capacity(annoying_links.len());

    for (id, url) in annoying_links.iter() {
        // we just need to make sure its a valid badge.
        // we could just not worry about invalid ones but eh, easier.
        let badge = badges
            .iter()
            .find(|b| b[1].id == id.parse::<u64>().expect("Failed to parse badge id"))
            .unwrap_or_else(|| panic!("Failed to find badge! {}", id))
            .to_owned();

        // now we get the data. And map the ok and err separately. It's mostly just adding the badge though.
        let data = get_page_redirect(client, url)
            .await
            .map(|ok| {
                let mut wt = WikiText::parse(ok.text);
                wt.set_page_name(Some(url));
                BadgeDetails(wt, badge.to_owned())
            })
            .map_err(|err| BadgeError(err.into(), badge.to_owned()));

        annoying.push(data);
    }

    annoying
}

/// Wrapper for [get_page_redirect] but returns a more favourable `Result<WikiText, String>` instead.
pub async fn get_page_data(client: &RustClient, page: &str) -> Result<WikiText, String> {
    log::debug!("Page: {}", page);
    let data = get_page_redirect(client, page).await;
    if let Ok(res) = data {
        let mut wikitext = WikiText::parse(res.text);
        wikitext.set_page_name(res.name);
        return Ok(wikitext);
    }
    Err(format!("Failed to get {:?}", page))
}
