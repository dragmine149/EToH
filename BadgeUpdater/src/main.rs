mod badge_to_wikitext;
mod definitions;
// mod json;
mod hard_coded;
mod process_items;
mod reqwest_client;
mod wikitext;

use crate::{
    badge_to_wikitext::{get_annoying, get_badges, get_page_redirect},
    definitions::{
        AreaInformation, BadgeOverwrite, ErrorDetails, EventInfo, EventItem, GlobalArea, OkDetails,
        WikiTower, badges_from_map_value,
    },
    process_items::{
        process_area, process_event_area, process_event_item, process_item, process_tower,
    },
    reqwest_client::RustClient,
};
use dotenv::dotenv;
use itertools::Itertools;
use lazy_regex::regex_replace;
use std::{collections::HashMap, fs, io::Write, path::PathBuf, str::FromStr};
use url::Url;

pub const BADGE_URL: &str = "https://badges.roblox.com/v1/universes/3264581003/badges?limit=100";
pub const OLD_BADGE_URL: &str =
    "https://badges.roblox.com/v1/universes/1055653882/badges?limit=100";
pub const ETOH_WIKI: &str = "https://jtoh.fandom.com/";

fn clean_badge_name(badge: &str) -> String {
    // Start with a trimmed copy
    let mut s = badge.trim().to_string();

    // Remove leading "Beat the" (case-insensitive) variations
    s = regex_replace!(r"(?i)^\s*beat\s+the\s+", &s, "").to_string();

    // Remove any parenthetical content like "(Unobtainable)", "(LE)", etc.
    s = regex_replace!(r"\s*\(.*", &s, "").to_string();

    // Remove question marks
    s = s.replace('?', "");

    // Collapse multiple spaces into one and trim again
    s = regex_replace!(r"\s{2,}", &s, " ").to_string();
    s.trim().to_string()
}

fn fmt_secs(number: u64) -> String {
    let (hour, minute, second) = (number / 3600, (number % 3600) / 60, number % 60);
    [hour, minute, second]
        .iter()
        .zip(["h", "min", "s"])
        .filter_map(|(&v, u)| (v != 0).then_some(format!("{}{}", v, u)))
        .collect::<Vec<_>>()
        .join(" + ")
}

/// Take an object and count how many passed/failed.
///
/// We use references as we don't really care about the list and it saves having to reassign just for a debug.
///
/// # Arguments
/// - obj -> A vector of objects to list through. (type is dynamic)
/// - pass_check -> The function to filter out objects which have passed.
/// - func_name -> Name of the function called before this
/// - file -> Optional path to store something to.
///
/// # Returns
/// - Vec<&'a K> -> A list to use in other places.
///
/// # Example
/// ```rs
/// let mut objs: Vec<Result<u64, String>> = vec![];
/// populate_vec(objs);
/// let (passed, failed) = count_processed(&objs, |o| o.is_ok(), "some function", None);
/// ```
fn count_processed<'a, K, P, E>(
    obj: &'a [Result<K, E>],
    pass_check: P,
    func_name: &str,
    file: Option<&PathBuf>,
) -> (Vec<&'a K>, Vec<&'a E>)
where
    P: Fn(&Result<K, E>) -> bool,
    K: std::fmt::Debug,
    E: std::fmt::Debug + 'a,
{
    // theses lists are required not just for counting, but for also outputting.
    let mut passed: Vec<&K> = Vec::new();
    let mut failed: Vec<&E> = Vec::new();

    for item in obj.iter() {
        if pass_check(item) {
            passed.push(item.as_ref().ok().unwrap());
        } else {
            failed.push(item.as_ref().err().unwrap());
        }
    }

    // output to file.
    // If we don't have a file, then we don't really care about writing.
    if let Some(path) = file {
        match fs::OpenOptions::new().create(true).append(true).open(path) {
            Ok(mut fh) => {
                if let Err(e) = writeln!(fh, "{:?} passed:\n{:#?}\n", func_name, passed) {
                    log::error!("Failed to append passed items to {:?}: {}", path, e);
                }
                if let Err(e) = writeln!(fh, "{:?} failed:\n{:#?}\n", func_name, failed) {
                    log::error!("Failed to append failed items to {:?}: {}", path, e);
                }
            }
            Err(e) => {
                log::error!("Failed to open file {:?} for appending: {}", path, e);
            }
        }
    }

    // log the data we wanted to log.
    log::info!(
        "[{}] Total: {}. Passed: {}. Rate: {:.2}%",
        func_name,
        obj.len(),
        passed.len(),
        if obj.is_empty() {
            0.0
        } else {
            (passed.len() as f64 / obj.len() as f64) * 100.0
        }
    );

    // might as well return both lists.
    (passed, failed)
}

const DEBUG_PATH: &str = "./badges.temp.txt";
const OVERWRITE_PATH: &str = "../overwrite.jsonc";
const ANNOYING_LINKS_PATH: &str = "../annoying_links.json";
const IGNORED_LIST: &str = "../ignored.jsonc";

#[tokio::main]
async fn main() {
    // setup
    env_logger::init();
    dotenv().ok();

    // file setup
    let path = PathBuf::from(DEBUG_PATH);
    if path.exists()
        && let Err(e) = fs::remove_file(&path)
    {
        log::error!("Failed to remove debug file {:?}: {}", path, e);
    }

    // client and original url setup.
    let client = RustClient::new(None, None);
    let url = Url::from_str(&format!("{:}?limit=100", BADGE_URL)).unwrap();

    let overwrites = badges_from_map_value(
        &serde_json::from_str(
            // &fs::read_to_string(OVERWRITE_PATH).expect("Failed to read overwrite path"),
            &fs::read_to_string(OVERWRITE_PATH).unwrap_or("{}".into()),
        )
        .unwrap(),
    )
    .unwrap_or_default();
    let annoying_links = serde_json::from_str::<HashMap<String, String>>(
        &fs::read_to_string(ANNOYING_LINKS_PATH).unwrap_or("{}".into()),
    )
    .unwrap_or_default();
    let ignored_list = serde_json::from_str::<HashMap<String, Vec<u64>>>(
        &fs::read_to_string(IGNORED_LIST)
            .unwrap_or("{}".into())
            .lines()
            .filter(|line| !line.trim_start().contains("//"))
            .join("\n"),
    )
    .unwrap_or_default();

    log::info!("Setup complete, starting searching");

    main_processing(
        &client,
        &url,
        &path,
        &overwrites,
        &ignored_list,
        &annoying_links,
    )
    .await
}

/// The main processing function which takes in the most basics and gives everything as something usable.
#[allow(unused_variables, reason = "Will be used later")]
async fn main_processing(
    client: &RustClient,
    url: &Url,
    debug_path: &PathBuf,
    overwrites: &[BadgeOverwrite],
    ignored: &HashMap<String, Vec<u64>>,
    annoying_links: &HashMap<String, String>,
) {
    // Written by T3 Chat (Gemini 3 Flash)
    let skip_ids = overwrites
        .iter()
        .flat_map(|bo| std::iter::once(bo.badge_id).chain(bo.alt_ids.iter().copied()))
        .chain(ignored.values().flatten().map(|id| *id))
        .collect_vec();
    println!("{:?}", overwrites);
    println!("{:#?}", skip_ids);

    // get a list of all the badges.
    let mut badges_vec = vec![];
    let raw = get_badges(client, url, &skip_ids).await.unwrap();
    for badge_fut in raw {
        badges_vec.push(badge_fut.await.unwrap());
    }

    log::info!("Skipped {:?} badges due to overwrites file", skip_ids.len());
    // process the badges to get the passed and failed ones..
    let (passed, failed) = count_processed(
        &badges_vec,
        |f: &Result<OkDetails, ErrorDetails>| f.is_ok(),
        "get_badges",
        Some(debug_path),
    );

    let annoying = get_annoying(
        client,
        &badges_vec
            .iter()
            .map(|r| match r {
                Ok(ok) => &ok.1,
                Err(err) => &err.1,
            })
            .collect_vec(),
        annoying_links,
    )
    .await;
    let (annoying_pass, annoying_fail) = count_processed(
        &annoying,
        |a: &Result<OkDetails, ErrorDetails>| a.is_ok(),
        "get_annoying",
        Some(debug_path),
    );

    // start processing towers.
    let tower_data = passed
        .iter()
        .chain(annoying_pass.iter())
        .map(|p| process_tower(&p.0, &p.1))
        // .inspect(|x| println!("{:?}", x))
        .collect::<Vec<Result<WikiTower, String>>>();

    let (tower_processed, tower_processed_failed) = count_processed(
        &tower_data,
        |r: &Result<WikiTower, String>| r.is_ok(),
        "process_tower",
        Some(debug_path),
    );

    // process items now we now which towers have passed.
    let mut items = vec![];
    for ele in passed.iter().filter(|p| {
        !tower_processed
            .iter()
            .any(|t| t.badge_name.contains(&p.1.name))
    }) {
        items.push(process_item(client, &ele.0, &ele.1).await);
    }
    let (item_processed, items_failed) = count_processed(
        &items,
        |i: &Result<WikiTower, String>| i.is_ok(),
        "process_item",
        Some(debug_path),
    );

    // combine the both
    let mut success = vec![];
    tower_processed.iter().for_each(|i| success.push(i));
    item_processed.iter().for_each(|i| success.push(i));
    log::info!(
        "[badge to tower] Total: {}. Passed: {}. Rate: {:.2}%",
        badges_vec.len(),
        success.len(),
        ((success.len() as f64) / (badges_vec.len() as f64)) * 100.0
    );

    // process areas based off towers.
    // Unique is here to reduce double area checking
    let areas_list = success.clone().into_iter().map(|t| t.area.clone()).unique();
    let mut areas = vec![];
    for area in areas_list.clone() {
        areas.push(process_area(client, &area).await);
    }

    let (area_processed, area_failed) = count_processed(
        &areas,
        |a: &Result<AreaInformation, String>| a.is_ok(),
        "process_area",
        Some(debug_path),
    );

    // do the same but for the event based ones.
    let mut event_areas = vec![];
    for ele in areas_list.filter(|a| area_failed.iter().any(|f| f.contains(a))) {
        event_areas.push(process_event_area(client, &ele).await);
    }

    let (event_processed, event_failed) = count_processed(
        &event_areas,
        |a: &Result<EventInfo, String>| a.is_ok(),
        "process_event_area",
        Some(debug_path),
    );

    // combine them.
    let mut area_success: Vec<GlobalArea> = vec![];
    area_processed
        .iter()
        .for_each(|i| area_success.push(GlobalArea::Area((*i).clone())));
    event_processed
        .iter()
        .for_each(|i| area_success.push(GlobalArea::Event((*i).clone())));
    log::info!(
        "[area parsing] Total: {}. Passed: {}. Rate: {:.2}%",
        areas.len(),
        area_success.len(),
        ((area_success.len() as f64) / (areas.len() as f64)) * 100.0
    );

    // println!("[");
    // event_processed.iter().for_each(|x| println!("    {:?}", x));
    // println!("]");

    // println!("{:?}", event_processed);
    let mut event_items = vec![];
    for ele in passed
        .iter()
        .filter(|p| {
            !tower_processed
                .iter()
                .any(|t| t.badge_name.contains(&p.1.name))
        })
        .filter(|p| {
            !item_processed
                .iter()
                .any(|i| i.badge_name.contains(&p.1.name))
        })
    {
        event_items.push(process_event_item(&ele.0, &ele.1, &event_processed));
    }

    let (event_items_processed, event_items_failed) = count_processed(
        &event_items,
        |e: &Result<EventItem, String>| e.is_ok(),
        "process_event_item",
        Some(debug_path),
    );

    let failed_list = &failed.iter().map(|p| p.1.clone()).collect_vec();

    // okay, now we have to hard-code some stuff.
    let mini_towers = hard_coded::parse_mini_towers(
        client,
        failed_list,
        &success.iter().map(|t| t.page_name.clone()).collect_vec(),
    )
    .await;
    let (mini_passed, mini_failed) = count_processed(
        &mini_towers,
        |m| m.is_ok(),
        "hard_coded::parse_mini_towers",
        Some(debug_path),
    );

    mini_passed.iter().for_each(|m| success.push(m));
    log::info!(
        "[badge to tower w/hard] Total: {}. Passed: {}. Rate: {:.2}%",
        badges_vec.len(),
        success.len(),
        ((success.len() as f64) / (badges_vec.len() as f64)) * 100.0
    );

    let adventure_towers = hard_coded::area_from_description(failed_list);
    let (adventure_pass, adventure_fail) = count_processed(
        &adventure_towers,
        |a| a.is_ok(),
        "area_from_description",
        Some(debug_path),
    );
    let adventure_ids = adventure_pass.iter().map(|a| a.badge_id).collect_vec();
    let success_ids = success.iter().map(|s| s.badge_id).collect_vec();
    let event_items_ids = event_items_processed
        .iter()
        .map(|e| e.badge_id)
        .collect_vec();
    let mut unprocessed = badges_vec
        .iter()
        .map(|v| {
            if v.is_ok() {
                v.as_ref().ok().unwrap().1.id
            } else {
                v.as_ref().err().unwrap().1.id
            }
        })
        .filter(|id| !success_ids.contains(id))
        .filter(|id| !adventure_ids.contains(id))
        .filter(|id| !event_items_ids.contains(id))
        .collect_vec();
    unprocessed.sort();

    if !unprocessed.is_empty() {
        log::error!(
            "There are badges which have failed to been processed (total of: {:?})",
            unprocessed.len()
        );
    } else {
        log::info!("All badges processed!");
    }

    match fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(debug_path)
    {
        Ok(mut fh) => {
            if let Err(e) = writeln!(fh, "Unprocessed badges:") {
                log::error!("Failed to append passed items to {:?}: {}", debug_path, e);
            }
            if let Err(e) = writeln!(fh, "{:#?}", unprocessed) {
                log::error!("Failed to append failed items to {:?}: {}", debug_path, e);
            }
        }
        Err(e) => {
            log::error!("Failed to open file {:?} for appending: {}", debug_path, e);
        }
    }
}
