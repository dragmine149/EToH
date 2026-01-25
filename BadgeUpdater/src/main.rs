pub mod badge_to_wikitext;
pub mod definitions;
pub mod hard_coded;
pub mod json;
pub mod process_items;
pub mod reqwest_client;
pub mod shrink_json_defs;
pub mod wikitext;

use crate::{
    badge_to_wikitext::{get_annoying, get_badges},
    definitions::{
        AreaInformation, BadgeOverwrite, ErrorDetails, EventInfo, EventItem, OkDetails, WikiTower,
        badges_from_map_value,
    },
    json::{Jsonify, read_jsonc},
    process_items::{get_event_areas, process_all_items, process_area, process_tower},
    reqwest_client::RustClient,
};
use dotenv::dotenv;
use itertools::Itertools;
use lazy_regex::regex_replace;
use std::{collections::HashMap, fs, io::Write, path::PathBuf, str::FromStr};
use url::Url;

// TODO: like we're doing for badges, make this a vector or something.

/// Link to the badge list of the new game.
pub const BADGE_URL: &str = "https://badges.roblox.com/v1/universes/3264581003/badges?limit=100";
/// Link to the badge list of the old game.
pub const OLD_BADGE_URL: &str =
    "https://badges.roblox.com/v1/universes/1055653882/badges?limit=100";
/// Link to the wiki to append to pretty nmuch every single URL.
pub const ETOH_WIKI: &str = "https://jtoh.fandom.com/";
pub const ETOH_WIKI_API: &str = "https://jtoh.fandom.com/api.php";

/// Some badges have unwanted data which either messes with fandom search or just breaks other things.
///
/// Here we clean it up, If possible, all badges should go through this function.
///
/// # Arguments
/// * badge: The name of the badge to clean up.
///
/// # Returns
/// * String: A brand new owned string element to do anything with.
pub fn clean_badge_name(badge: &str) -> String {
    // Start with a trimmed copy
    let mut s = badge.trim().to_string();

    // Remove leading "Beat the" (case-insensitive) variations
    s = regex_replace!(r"(?i)^\s*beat\s+the\s+", &s, "").to_string();
    // Remove leading "Beat" (case-insensitive) variations (annoying ToXYZ 3109062537716097)
    s = regex_replace!(r"(?i)^\s*beat\s+", &s, "").to_string();

    // Remove any parenthetical content like "(Unobtainable)", "(LE)", etc.
    s = regex_replace!(r"\s*\(.*", &s, "").to_string();

    // Remove question marks
    s = s.replace('?', "");

    // Collapse multiple spaces into one and trim again
    s = regex_replace!(r"\s{2,}", &s, " ").to_string();
    s.trim().to_string()
}

/// Special one-off function for formatting a datetime into a certain string.
///
/// TODO: Switch to something chorno::Datetime based?
pub fn fmt_secs(number: u64) -> String {
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
/// * obj: A vector of objects to list through. (type is dynamic)
/// * pass_check: The function to filter out objects which have passed.
/// * func_name: Name of the function called before this
/// * file: Optional path to store something to.
///
/// # Returns
/// * `Vec<&'a K>`: A list of borrowed entries where they have passed the check
/// * `Vec<&'a E>`: A list of borrowed entries where they have failed the check.
///
/// # Example
/// ```
/// let mut objs: Vec<Result<u64, String>> = vec![];
/// // some random function
/// populate_vec(&mut objs);
/// let (passed, failed) = count_processed(&objs, |o| o.is_ok(), "some function", None);
/// ```
pub fn count_processed<'a, K, P, E>(
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

/// The path to store the debug information, includes all towers passed and the errors.
pub const DEBUG_PATH: &str = "./badges.temp.txt";
/// The path where we store individual badges we have manually overwritten to make work.
pub const OVERWRITE_PATH: &str = "../overwrite.jsonc";
/// The path where we store those few badges which fandom won't search for us.
pub const ANNOYING_LINKS_PATH: &str = "../annoying_links.json";
/// The path where we store all the badges we just straight up ignore and never deal with.
pub const IGNORED_LIST_PATH: &str = "../ignored.jsonc";
/// The path of the final output, the result will always be written here pre-end-panic.
pub const OUTPUT_PATH: &str = "../badges.json";
/// The path to store any changes between this version and the previous days[^1] version
///
/// [^1]: days could be just yesterday or from weeks ago if nothing got updated.
pub const CHANGELOG_PATH: &str = "../changelog.md";
/// The path of the final output, after shrinking.
pub const SHRINK_PATH: &str = "../shrunk.json";

/// The main function of the program to call every other function like always.
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
    let client = RustClient::new(
        None,
        Some("Data2/BadgeUpdater (https://github.com/dragmine149/Etoh)"),
    );
    let url = Url::from_str(&format!("{:}?limit=100", BADGE_URL)).unwrap();

    let overwrites =
        badges_from_map_value(&serde_json::from_str(&read_jsonc(OVERWRITE_PATH)).unwrap())
            .unwrap_or_default();
    let annoying_links = serde_json::from_str::<HashMap<String, String>>(
        &fs::read_to_string(ANNOYING_LINKS_PATH).unwrap_or("{}".into()),
    )
    .unwrap_or_default();
    let ignored_list =
        serde_json::from_str::<HashMap<String, Vec<u64>>>(&read_jsonc(IGNORED_LIST_PATH))
            .unwrap_or_default();

    log::info!("Setup complete, starting searching");

    let (mut result, unprocessed) = main_processing(
        &client,
        &url,
        &path,
        &overwrites,
        &ignored_list,
        &annoying_links,
    )
    .await;
    result.parse_skipped(&overwrites).clean_up();
    // println!("{:?}", result);

    log::info!("Parsing completed, storing...");

    let previous =
        serde_json::from_str::<Jsonify>(&fs::read_to_string(OUTPUT_PATH).unwrap_or("{}".into()))
            .unwrap_or_default();

    fs::write(OUTPUT_PATH, serde_json::to_string(&result).unwrap()).unwrap();
    let change_log = result.compare(&previous);
    fs::write(CHANGELOG_PATH, change_log.join("\n")).unwrap();
    fs::write(
        SHRINK_PATH,
        serde_json::to_string(&result.shrinkfy()).unwrap(),
    )
    .unwrap();

    log::info!("Data stored, panicking if left overs then stopping.");

    if !unprocessed.is_empty() {
        panic!("There are still some items left in the list to process!");
    }
}

/// The main processing function which takes in the most basics and gives everything as something usable.
///
/// Separating this from main allows us to move it around easier and means main is smaller.
// #[allow(unused_variables, reason = "Will be used later")]
async fn main_processing(
    client: &RustClient,
    url: &Url,
    debug_path: &PathBuf,
    overwrites: &[BadgeOverwrite],
    ignored: &HashMap<String, Vec<u64>>,
    annoying_links: &HashMap<String, String>,
) -> (Jsonify, Vec<u64>) {
    let skip_ids = overwrites
        .iter()
        .flat_map(|bo| bo.badge_ids)
        .chain(ignored.values().flatten().copied())
        .collect_vec();
    println!("{:?}", overwrites);
    println!("{:#?}", skip_ids);

    // get a list of all the badges.
    let mut badges_vec = vec![];
    let raw = get_badges(client, url, &skip_ids)
        .await
        .expect("Failed to get badges from roblox api...");
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
    let (annoying_pass, _annoying_fail) = count_processed(
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

    let (tower_processed, _tower_processed_failed) = count_processed(
        &tower_data,
        |r: &Result<WikiTower, String>| r.is_ok(),
        "process_tower",
        Some(debug_path),
    );

    // process areas based off towers.
    // Unique is here to reduce double area checking
    let mut areas = vec![];
    for area in tower_processed.iter().map(|t| t.area.clone()).unique() {
        areas.push(process_area(client, &area).await);
    }

    let (area_processed, _area_failed) = count_processed(
        &areas,
        |a: &Result<AreaInformation, String>| a.is_ok(),
        "process_area",
        Some(debug_path),
    );

    // do the same but for the event based ones.
    let pre_event_areas = get_event_areas(client).await;
    let event_areas = if pre_event_areas.is_err() {
        log::error!(
            "Failed to get event areas: {:?}",
            pre_event_areas.err().unwrap()
        );
        vec![]
    } else {
        pre_event_areas.ok().unwrap()
    };

    let (event_processed, _event_failed) = count_processed(
        &event_areas,
        |a: &Result<EventInfo, String>| a.is_ok(),
        "process_event_area",
        Some(debug_path),
    );

    let success_area_count = area_processed.len() + event_processed.len();
    log::info!(
        "[area parsing] Total: {}. Passed: {}. Rate: {:.2}%",
        areas.len() + event_areas.len(),
        success_area_count,
        ((success_area_count as f64) / (areas.len() as f64)) * 100.0
    );

    // Process the items that we have by avoid any of the towers we processed so far.
    // println!("{:?}", event_processed);
    let mut items = vec![];
    for ele in passed.iter().filter(|p| {
        !tower_processed
            .iter()
            .any(|t| t.page_name.contains(&p.1.name))
    }) {
        items.push(process_all_items(client, &ele.0, &ele.1, &event_processed).await);
    }

    let (all_items_processed, _all_items_failed) = count_processed(
        &items,
        |e: &Result<(EventItem, Option<WikiTower>), String>| e.is_ok(),
        "process_all_items",
        Some(debug_path),
    );

    let failed_list = &failed.iter().map(|p| p.1.clone()).collect_vec();

    // okay, now we have to hard-code some stuff.
    let mini_towers = hard_coded::parse_mini_towers(
        client,
        failed_list,
        &tower_processed
            .iter()
            .map(|t| t.page_name.clone())
            .collect_vec(),
    )
    .await;
    let (mini_passed, _mini_failed) = count_processed(
        &mini_towers,
        |m| m.is_ok(),
        "hard_coded::parse_mini_towers",
        Some(debug_path),
    );

    let adventure_towers = hard_coded::area_from_description(failed_list);
    let (adventure_pass, _adventure_fail) = count_processed(
        &adventure_towers,
        |a| a.is_ok(),
        "hard_coded::area_from_description",
        Some(debug_path),
    );
    let adventure_ids = adventure_pass
        .iter()
        .flat_map(|a| a.badge_ids)
        .collect_vec();
    let success_ids = tower_processed
        .iter()
        .map(|s| s.badge_id)
        .chain(all_items_processed.iter().flat_map(|ei| ei.0.badges))
        .chain(mini_passed.iter().map(|mini| mini.badge_id))
        .collect_vec();
    let event_items_ids = all_items_processed
        .iter()
        .flat_map(|e| e.0.badges)
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
    // log::warn!("{}", success_ids.len());
    // log::warn!("{}", adventure_ids.len());
    // log::warn!("{}", event_items_ids.len());
    // log::warn!("{}", badges_vec.len());
    // log::warn!(
    //     "{}",
    //     ((success_ids.len() + adventure_ids.len() + event_items_ids.len()) / badges_vec.len())
    // );
    // log::warn!("{}", (skip_ids.len() / badges_vec.len()));
    // log::warn!(
    //     "{}",
    //     (success_ids.len()
    //         + adventure_ids.len()
    //         + event_items_ids.len()
    //         + skip_ids.len() / badges_vec.len())
    // );
    log::info!(
        "Badges processed: {} ({:.2}%). Badges Skipped: {} ({:.2}%). Total Processed: {} ({:.2}%). Total Total: {}",
        badges_vec.len() - unprocessed.len() - skip_ids.len(),
        ((badges_vec.len() - unprocessed.len() - skip_ids.len()) as f64 / badges_vec.len() as f64)
            * 100.0,
        skip_ids.len(),
        (skip_ids.len() as f64 / badges_vec.len() as f64) * 100.0,
        badges_vec.len() - unprocessed.len(),
        ((badges_vec.len() - unprocessed.len()) as f64 / badges_vec.len() as f64) * 100.0,
        badges_vec.len()
    );

    if !unprocessed.is_empty() {
        log::error!(
            "There are badges which have failed to been processed (total of: {:?} ({:.2}%))",
            unprocessed.len(),
            (unprocessed.len() as f64 / badges_vec.len() as f64) * 100.0
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

    (
        Jsonify::parse(
            &tower_processed,
            &area_processed,
            &event_processed,
            &all_items_processed,
            &mini_passed,
            &adventure_pass,
        ),
        unprocessed,
    )
}
