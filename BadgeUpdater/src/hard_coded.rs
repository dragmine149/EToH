//! Some things are easier done if we just hard code rather than guess...
//!
//! As much as 99% of this is hard coded, i've tried to keep it semi-dynamic by not referencing any specific names where possible.
//! But here, we need to be a bit more strict with what we do in order for it to work.

use crate::{
    ETOH_WIKI,
    badge_to_wikitext::get_page_data,
    definitions::{Badge, BadgeOverwrite, WikiTower},
    process_items::process_tower,
    reqwest_client::RustClient,
    wikitext::{Cell, WikiText, enums::LinkType},
};
use itertools::Itertools;

/// Inner function for [parse_mini_towers]. That was getting big so we split it out of readability.
///
/// Parses a single row in the specified table we search.
/// # Arguments
/// * client: Passed from [parse_mini_towers]
/// * badges: Passed from [parse_mini_towers]
/// * Ignore: Passed from [parse_mini_towers]
/// * data: The cell (row info) we actually look at.
///
/// # Returns
/// * None: We skip this row as it doesn't have anything useful for us.
/// * Some(Err): Something failed whilst trying to progress the row.
/// * Some(Ok): The result of the row after everything succeeded.
async fn parse_mini_row(
    client: &RustClient,
    data: Cell,
    ignore: &[String],
    badges: &[&[Badge; 2]],
) -> Option<Result<WikiTower, String>> {
    // no links, no page to link to. Aka, probably no badge.
    let links = data.inner.content.get_links(Some(LinkType::Internal));
    let target = links.first();
    if target.is_none() {
        // mini_towers.push(Err(format!("Failed to get link for {:?}", data)));
        return None;
    }
    let target = target.unwrap();
    if ignore.contains(&target.target) {
        // no need to push anything as we're ignoring it.
        log::debug!("Ignoring cell due to already processed");
        return None;
    }

    // and then basically get the page data like normal.
    let wikitext = get_page_data(client, &target.target.replace("?", "%3F")).await;
    if wikitext.is_err() {
        // println!("ERR: Failed to get wikidata");
        // println!("{:?}: {:?}", target.target, data);
        log::warn!("Failed to get wiki data for {:?}", target.target);
        return Some(Err(format!(
            "Failed to get wiki data for {:?}",
            target.target
        )));
    }
    let mut wikitext = wikitext.ok().unwrap();
    wikitext.set_page_name(Some(target.target.to_owned()));
    println!("{:?}", target.target);

    // check to see if the page contains our specific badges.
    let badge = badges.iter().find(|b| {
        // println!("{:?}/{:?}", b[0].id, b[1].id);
        // println!(
        //     "{:?}/{:?}",
        //     wikitext.text().contains(&b[0].id.to_string()),
        //     wikitext.text().contains(&b[1].id.to_string())
        // );

        // as we use `0` as a placeholder for no badge id. we have to make that check. `0` exists kinda a lot...
        (b[0].id > 0 && wikitext.text().contains(&b[0].id.to_string()))
            || (b[1].id > 0 && wikitext.text().contains(&b[1].id.to_string()))
    });

    // no badge mini tower.
    if badge.is_none() {
        // println!("{:?}", wikitext.text());
        return Some(Err(format!(
            "Failed to find badge id for {:?}",
            target.target
        )));
    }

    // Return that everything went well, after we get the tower data.
    Some(process_tower(&wikitext, badge.unwrap()))
}

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
    badges: &[&[Badge; 2]],
    ignore: &[String],
) -> Vec<Result<WikiTower, String>> {
    // get a list of mini towers.
    let response = client
        .get(format!("{}wiki/Mini_Tower?action=raw", ETOH_WIKI))
        .await
        .unwrap();
    let mini_towers = response.text().unwrap();

    let mini_wiki = WikiText::parse(mini_towers);
    let data = mini_wiki
        .get_parsed()
        .unwrap()
        // .map_err(|e| format!("{:?}", e))?
        .get_tables();
    let table = data.first().unwrap();
    // .ok_or("Failed to find table on mini tower page... (how!!??)")?;

    // println!("{:?}", table.get_headers());

    println!("Mini badges: {:#?}", badges);

    // for all rows in our table.
    let mut mini_towers = vec![];
    for row_id in 0..table.get_rows().len() {
        let cell = table.get_cell(row_id, "Name");
        let location = table.get_cell(row_id, "Location");
        // println!("row: {:?}, cell: {:?}", row_id, cell);
        log::debug!("Processing: {:?}", cell);

        // Make sure we have cell data, location data and it's not a cancelled tower.
        if let Some(data) = cell
            && let Some(loc) = location
            && loc.raw() != "Cancelled"
        {
            // pass it off to another function for row parsing.
            let result = parse_mini_row(client, data, ignore, badges).await;
            if let Some(res) = result {
                mini_towers.push(res);
            }
        }
    }

    mini_towers
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
pub fn area_from_description(badges: &[&[Badge; 2]]) -> Vec<Result<BadgeOverwrite, String>> {
    badges
        .iter()
        .map(|b| {
            let description = b[1].description.clone().unwrap_or_default();
            // the main regex, technically you could have descend to zone 10 which is techniaclly incorrect as you ascend.
            // Yeah, we don't care about that. Too much effort
            let (_, area) = lazy_regex::regex_captures!(
                r#"(?m)Beat enough towers to (?:de|a)scend to ((?:Ring \d)|(?:Zone \d))."#,
                &description
            )
            .ok_or("Failed to do regex")?;

            Ok(BadgeOverwrite {
                badge_ids: [b[0].id, b[1].id],
                category: "Adventure".to_owned(),
                name: format!("{} ({})", b[1].name.replace("\"", ""), area.to_owned()),
            })
        })
        .collect_vec()
}

/// Get badges which are related to beating a certain amount of towers.
///
/// Yeah, they all follow the same format and with the 400 towers badge it's like, fine. Just add them yourself.
pub fn progression(badges: &[&[Badge; 2]]) -> Vec<Result<BadgeOverwrite, String>> {
    badges
        .iter()
        .map(|b| {
            let (_, total) =
                lazy_regex::regex_captures!(r#"(?m)Beat (\d\d\d?) Towers"#, &b[1].name)
                    .ok_or("Failed to regex name for progression")?;
            Ok(BadgeOverwrite {
                badge_ids: [b[0].id, b[1].id],
                category: "Progression".to_owned(),
                name: format!("{} Towers Completed!", total),
            })
        })
        .collect_vec()
}
