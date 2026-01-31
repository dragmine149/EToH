use crate::{
    definitions::{ProcessError, WikiAPI, WikiPageEntry},
    reqwest_client::RustClient,
};
use itertools::Itertools;
use std::str::FromStr;
use url::{ParseError, Url};

/// Link to the wiki API. Everything goes through here.
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

/// Wrapper for [get_pages] that only returns a list of page entries instead.
///
/// Unlike [get_pages], we follow the api limits and do reqwests in chunks of 50.
///
/// # Arguments
/// * client: The client to use to ping the API
/// * pages: A list of pages to get the information from.
///
/// # Returns
/// * A list of pages, the same size as our input.
pub async fn get_pages_limited<S: AsRef<str>>(
    client: &RustClient,
    pages: &[S],
) -> Vec<Result<WikiPageEntry, ProcessError>> {
    // we know we're going to get a page per input so we can pre-set.
    let mut result_pages = Vec::with_capacity(pages.len());
    for chunk in pages.chunks(50) {
        // get our pages like normal.
        let page = get_pages(client, chunk).await;
        match page {
            Ok(p) => p.query.pages.unwrap().iter().for_each(|page| {
                // println!("{:?}", p.query.redirects);

                // got to own the page so we can return it.
                let mut owned_page = page.to_owned();

                // detect if we have been redirected/normalised and make sure to add that to the information.
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
            // as much as this might be useless, it allows for none-blocking.
            Err(e) => chunk.iter().for_each(|c| {
                result_pages.push(Err(format!("{:?} on {:?}", e, c.as_ref()).into()));
            }),
        }
    }
    result_pages
}

/// Ping the api with the list of pages provided and return the results.
///
/// NOTE: It is highly recommended to use [get_pages_limited] instead unless you know what you are doing.
///
/// # Arguments
/// * client: The client to use to ping the API
/// * pages: A list of pages to get the information from.
///
/// # Returns
/// * Ok -> The results from the request.
/// * Err -> Some failed during the request OR there are two many pages to process.
pub async fn get_pages<S: AsRef<str>>(
    client: &RustClient,
    pages: &[S],
) -> Result<WikiAPI, ProcessError> {
    if pages.len() > 50 {
        return Err(
            "There are too many pagess to get. Please follow the API limits and stay within 50!"
                .into(),
        );
    }

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

/// Get the search results for a specific query.
///
/// # Arguments
/// * client: The client to use to ping the API
/// * search: Query to search for.
/// * limit: How many results to show. Clamps between 1 and 500
///
/// # Returns
/// * A [WikiAPI] if success, or [ProcessError] on failure.
///
/// # Limits
/// Due to API limits, search can only return a max of 500 entries. We do not allow for next-page-searching as normally after the first page, all results become trash anyway.
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

/// Get the information for a certain category.
///
/// # Arguments
/// * client: The client to use to ping the API
/// * category_name: Category to get info for.
/// * limit: How many results to show. Clamps between 1 and 500
///
/// # Returns
/// * A [WikiAPI] if success, or [ProcessError] on failure.
///
/// # Limits
/// Due to API limits, category can only return a max of 500 entries. We do not allow for next-page-searching as normally after the first page, all results become trash anyway.
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

/// Extension and wrapper for [get_category].
///
/// Does a request with [get_category], and then a further request to [get_pages_limited] to get all the pages in said category.
/// # Arguments
/// * client: The client to use to ping the API
/// * category_name: Category to get info for.
/// * limit: How many results to show. Clamps between 1 and 500
///
/// # Returns
/// * Result from [get_pages_limited] or [ProcessError] if something happened before it could finish.
///
pub async fn get_pages_from_category<S: AsRef<str>>(
    client: &RustClient,
    category_name: &str,
    limit: u16,
) -> Result<Vec<Result<WikiPageEntry, ProcessError>>, ProcessError> {
    let category = get_category::<&str>(client, category_name, limit).await?;
    if let Some(members) = category.query.categorymembers {
        let pages = get_pages_limited(
            client,
            &members
                .iter()
                .map(|member| member.title.clone())
                .collect_vec(),
        )
        .await;
        Ok(pages)
    } else {
        Err("No categorymembers found".into())
    }
}
