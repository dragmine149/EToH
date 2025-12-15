mod badge_to_wikitext;
mod definitions;
// mod json;
mod process_items;
mod reqwest_client;
mod wikitext;

use dotenv::dotenv;
use itertools::Itertools;
use lazy_regex::regex_replace;
use std::{fs, path::PathBuf, str::FromStr};
use url::Url;

use crate::{
    badge_to_wikitext::get_badges,
    definitions::{AreaInformation, ErrorDetails, EventInfo, GlobalArea, OkDetails, WikiTower},
    process_items::{process_area, process_event_area, process_item, process_tower},
    reqwest_client::RustClient,
};

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
        use std::io::Write;
        match fs::OpenOptions::new().create(true).append(true).open(path) {
            Ok(mut fh) => {
                if let Err(e) = writeln!(fh, "{:#?}\n", passed) {
                    log::error!("Failed to append passed items to {:?}: {}", path, e);
                }
                if let Err(e) = writeln!(fh, "{:#?}\n", failed) {
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

    // get a list of all the badges.
    let mut badges_vec = vec![];
    let raw = get_badges(&client, &url).await.unwrap();
    for badge_fut in raw {
        badges_vec.push(badge_fut.await.unwrap());
    }

    // process the badges to get the passed and failed ones..
    let (passed, failed) = count_processed(
        &badges_vec,
        |f: &Result<OkDetails, ErrorDetails>| f.is_ok(),
        "get_badges",
        Some(&path),
    );

    // start processing towers.
    let tower_data = passed
        .iter()
        .map(|p| process_tower(&p.0, &p.1))
        .inspect(|x| println!("{:?}", x))
        .collect::<Vec<Result<WikiTower, String>>>();

    let (tower_processed, tower_processed_failed) = count_processed(
        &tower_data,
        |r: &Result<WikiTower, String>| r.is_ok(),
        "process_tower",
        Some(&path),
    );

    // process items now we now which towers have passed.
    let mut items = vec![];
    for ele in passed.iter().filter(|p| {
        !tower_processed
            .iter()
            .any(|t| t.badge_name.contains(&p.1.name))
    }) {
        items.push(process_item(&client, &ele.0, &ele.1).await);
    }
    let (item_processed, items_failed) = count_processed(
        &items,
        |i: &Result<WikiTower, String>| i.is_ok(),
        "process_item",
        Some(&path),
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
        areas.push(process_area(&client, &area).await);
    }

    let (area_processed, area_failed) = count_processed(
        &areas,
        |a: &Result<AreaInformation, String>| a.is_ok(),
        "process_area",
        Some(&path),
    );

    // do the same but for the event based ones.
    let mut event_areas = vec![];
    for ele in areas_list.filter(|a| area_failed.iter().any(|f| f.contains(a))) {
        event_areas.push(process_event_area(&client, &ele).await);
    }

    let (event_processed, event_failed) = count_processed(
        &event_areas,
        |a: &Result<EventInfo, String>| a.is_ok(),
        "process_event_area",
        Some(&path),
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
}
