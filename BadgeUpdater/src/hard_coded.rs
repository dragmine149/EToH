//! Some things are easier done if we just hard code rather than guess...
//!
//! As much as 99% of this is hard coded, i've tried to keep it semi-dynamic by not referencing any specific names where possible.
//! But here, we need to be a bit more strict with what we do in order for it to work.

use crate::{
    definitions::{BadgeOverwrite, Badges, WikiTower},
    mediawiki_api::{get_pages, get_pages_limited},
    process_items::process_tower,
    reqwest_client::RustClient,
    wikitext::{WikiText, enums::LinkType},
};
use itertools::Itertools;

/// Mini-towers (and most towers types) have their own unique page listing them all, hence we can fallback on that as badge name != mini tower name.
///
/// # Arguments
/// * client: Normal client to perform network reqwests
/// * badges: List of badges to search through once we get page data.
///   This means we can link them instead of the reverse.
/// * Ignore: List of mini tower names which we have already got a match before.
///   If we have a match, no point in trying to process it again, so we just ignore it.
pub async fn parse_mini_towers(
    client: &RustClient,
    badges: &[&Badges],
    ignore: &[String],
) -> Vec<Result<WikiTower, String>> {
    // get a list of mini towers.
    let mini_towers = get_pages(client, &["Mini_Tower"]).await.unwrap();
    let content = mini_towers
        .query
        .pages
        .unwrap()
        .first()
        .unwrap()
        .get_content()
        .unwrap()
        .content
        .clone();

    let mini_wiki = WikiText::parse(&content);
    let data = mini_wiki
        .get_parsed()
        .unwrap()
        // .map_err(|e| format!("{:?}", e))?
        .get_tables();
    let table = data.first().unwrap();
    // .ok_or("Failed to find table on mini tower page... (how!!??)")?;

    // println!("{:?}", table.get_headers());

    println!("Mini badges: {:#?}", badges);
    println!("Ignoring: {:#?}", ignore);

    let rows = (0..table.get_rows().len())
        .filter_map(|row_id| {
            let cell = table.get_cell(row_id, "Name");
            let location = table.get_cell(row_id, "Location");

            if let Some(data) = cell
                && let Some(loc) = location
                && loc.raw() != "Cancelled"
            {
                println!("{:?}", data);
                // no links, no page to link to. Aka, probably no badge.
                let links = data.inner.content.get_links(Some(LinkType::Internal));
                println!("{:?}", links);
                if links.is_empty() {
                    return None;
                }

                let target = links.first().unwrap();
                // if target.is_none() {
                //     // mini_towers.push(Err(format!("Failed to get link for {:?}", data)));
                //     return None;
                // }
                // let target = target.unwrap();
                if ignore.contains(&target.target) {
                    // no need to push anything as we're ignoring it.
                    log::debug!("Ignoring cell due to already processed");
                    return None;
                }

                return Some(target.target.clone());
            }
            None
        })
        .collect_vec();
    let mini_towers = get_pages_limited(client, &rows).await;
    mini_towers
        .iter()
        .map(|tower| {
            match tower {
                Err(e) => Err(format!("{:?}", e)),
                Ok(tower) => {
                    let content = tower.get_content().unwrap().content.clone();
                    let mut wikitext = WikiText::parse(&content);
                    wikitext.set_page_name(Some(tower.title.to_owned()));
                    // println!("{:?}", tower.title);

                    let badge = badges.iter().find(|b| b.check_ids(&content));

                    // no badge mini tower.
                    if badge.is_none() {
                        // println!("{:?}", wikitext.text());
                        return Err(format!("Failed to find badge id for {:?}", tower.title));
                    }

                    // Return that everything went well, after we get the tower data.
                    process_tower(&wikitext, badge.unwrap())
                }
            }
        })
        .collect_vec()
}

/// Get the area link from the badge description.
///
/// In most cases, this doesn't work. However, for most know cases of Rings/Zones badges, this works perfectly.
/// Ofc, this will need to be updated for new world but thats a future problem (as the worlds don't exist).
///
/// NOTE: This is intended to be run on a small subset of the original list. As very few badges will actual work here.
///
/// # Arguments
/// * badges: The list of badges to map.
///
/// # Returns
/// A vector containing the following for each badge
/// * Ok(BadgeOverwrite) The badge, category already filled out like it came from overwrite.jsonc
/// * Err(String) Why it failed, or well this regex just didn't work.
pub fn area_from_description(badges: &[&Badges]) -> Vec<Result<BadgeOverwrite, String>> {
    // println!("afd badges: {:?}", badges);
    badges
        .iter()
        .map(|b| {
            let description = b.description.clone().unwrap_or_default();
            // the main regex, technically you could have descend to zone 10 which is techniaclly incorrect as you ascend.
            // Yeah, we don't care about that. Too much effort
            let (_, area) = lazy_regex::regex_captures!(
                r#"(?m)(?:de|a)scend to ((?:Ring \d)|(?:Zone \d))."#,
                &description
            )
            .ok_or(format!("Failed to do regex {}", b.name))?;

            Ok(BadgeOverwrite {
                badge_ids: b.ids,
                category: "Adventure".to_owned(),
                name: format!("{} ({})", b.name.replace("\"", ""), area.to_owned()),
            })
        })
        .collect_vec()
}

/// Get badges which are related to beating a certain amount of towers.
///
/// Yeah, they all follow the same format and with the 400 towers badge it's like, fine. Just add them yourself.
pub fn progression(badges: &[&Badges]) -> Vec<Result<BadgeOverwrite, String>> {
    badges
        .iter()
        .map(|b| {
            let (_, total) = lazy_regex::regex_captures!(r#"(?m)Beat (\d\d\d?) Towers"#, &b.name)
                .ok_or("Failed to regex name for progression")?;
            Ok(BadgeOverwrite {
                badge_ids: b.ids,
                category: "Progression".to_owned(),
                name: format!("{} Towers Completed!", total),
            })
        })
        .collect_vec()
}
