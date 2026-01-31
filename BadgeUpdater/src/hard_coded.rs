//! Some things are easier done if we just hard code rather than guess...
//!
//! As much as 99% of this is hard coded, i've tried to keep it semi-dynamic by not referencing any specific names where possible.
//! But here, we need to be a bit more strict with what we do in order for it to work.

use std::path::PathBuf;

use crate::{
    count_processed,
    definitions::{BadgeOverwrite, Badges, WikiTower},
    mediawiki_api::{get_pages, get_pages_limited},
    process_items::process_tower,
    reqwest_client::RustClient,
    wikitext::{WikiText, enums::LinkType},
};
use itertools::Itertools;

/// The main function for all hard coded inputs.
///
/// This is split out from [crate::main_processing] to allow for more limited control and readability.
///
/// # Arguments
/// * badges: A borrowed list of borrowed badges which we search through.
///   We also make sure to reduces the badges passed into the next function.
/// * debug_path: Path to provided to [crate::count_processed] for debugging.
///
/// # Returns
/// * A list of badges that we have managed to pass successfully.
/// * We also debug everything to `log::info!` and the provided file.
///
/// # Notes
/// This can be run first or last or whenever, last is recommended as most of the results will end in failure due to their nature.
pub fn process_hard_coded<'a>(
    badges: &'a [&'a Badges],
    debug_path: Option<&PathBuf>,
) -> Vec<BadgeOverwrite> {
    let mut results = vec![];

    log::info!("Processing description");
    let area_towers = area_from_description(badges);
    let (adventure_pass, adventure_fail) = count_processed(
        &area_towers,
        |a| a.is_ok(),
        "hard_coded::area_from_description",
        debug_path,
    );

    log::info!("Processing Progression");
    let adve_failed = adventure_fail.iter().map(|f| *f.1).collect_vec();
    let prog = progression(&adve_failed);
    let (prog_pass, prog_fail) =
        count_processed(&prog, |p| p.is_ok(), "hard_coded::progression", debug_path);

    log::info!("Processing Difficulties");
    let prog_failed = prog_fail.iter().map(|f| *f.1).collect_vec();
    let difficulties = tower_difficultied(&prog_failed);
    let (diff_pass, _diff_fail) = count_processed(
        &difficulties,
        |d| d.is_ok(),
        "hard_coded::tower_difficultied",
        debug_path,
    );

    results.extend(adventure_pass.iter().map(|a| (*a).clone()));
    results.extend(prog_pass.iter().map(|p| (*p).clone()));
    results.extend(diff_pass.iter().map(|d| (*d).clone()));
    results
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
                // println!("{:?}", data);
                // no links, no page to link to. Aka, probably no badge.
                let links = data.inner.content.get_links(Some(LinkType::Internal));
                // println!("{:?}", links);
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

#[derive(Debug)]
/// An error describing what happened and the badge related.
pub struct HardError<'a>(pub String, pub &'a &'a Badges);

/// Get the area link from the badge description.
///
/// # Arguments
/// * badges: The list of badges to map.
///
/// # Returns
/// A vector containing the following for each badge
/// * Ok(BadgeOverwrite) The badge, category already filled out like it came from overwrite.jsonc
/// * Err(String) Why it failed, or well this regex just didn't work.
pub fn area_from_description<'a>(
    badges: &'a [&'a Badges],
) -> Vec<Result<BadgeOverwrite, HardError<'a>>> {
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
            .ok_or(HardError(format!("Failed to do regex {}", b.name), b))?;

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
/// Yeah, they all follow the same format and with the 400 towers badge it's like, fine. Lets just add this.
pub fn progression<'a>(badges: &'a [&'a Badges]) -> Vec<Result<BadgeOverwrite, HardError<'a>>> {
    badges
        .iter()
        .map(|b| {
            let (_, total) =
                lazy_regex::regex_captures!(r#"(?m)Beat (\d\d\d?) Towers"#, &b.name).ok_or(
                    HardError("Failed to regex name for progression".to_owned(), b),
                )?;
            Ok(BadgeOverwrite {
                badge_ids: b.ids,
                category: "Progression".to_owned(),
                name: format!("{} Towers Completed!", total),
            })
        })
        .collect_vec()
}

/// Similar to [progression], read the title for the difficulty of beaten.
///
/// I doubt we'll get any more towers like this but eh.
pub fn tower_difficultied<'a>(
    badges: &'a [&'a Badges],
) -> Vec<Result<BadgeOverwrite, HardError<'a>>> {
    badges
        .iter()
        .map(|b| {
            let (_, difficulty) =
                lazy_regex::regex_captures!(r#"(?m)Beat Your First (.*) Tower"#, &b.name).ok_or(
                    HardError(
                        format!("Failed to regex name for difficulty ({})", b.name),
                        b,
                    ),
                )?;
            Ok(BadgeOverwrite {
                badge_ids: b.ids,
                category: "Difficulty".to_owned(),
                name: format!("First {}", difficulty),
            })
        })
        .collect_vec()
}
