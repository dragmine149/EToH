mod badge_to_wikitext;
// mod cache;
mod definitions;
// mod json;
// mod parse_wikitext;
// mod pywiki;
mod reqwest_client;
// mod rust_wiki;
mod process_items;
mod wikitext;

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use dotenv::dotenv;
use lazy_regex::regex_replace;
use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    badge_to_wikitext::{ErrorDetails, OkDetails, get_badges},
    process_items::{WikiTower, process_tower},
    reqwest_client::RustClient,
};

// use crate::rust_wiki::{WikiTower, WikiTowerBuilder};

pub const BADGE_URL: &str = "https://badges.roblox.com/v1/universes/3264581003/badges?limit=100";
pub const OLD_BADGE_URL: &str =
    "https://badges.roblox.com/v1/universes/1055653882/badges?limit=100";
pub const ETOH_WIKI: &str = "https://jtoh.fandom.com/";

// fn get_badges(client: &Client, url: String) -> Result<Vec<Badge>, Box<dyn std::error::Error>> {
//     let mut badges: Vec<Badge> = vec![];
//     let mut data: Data = Data {
//         previous_page_cursor: None,
//         next_page_cursor: Some(String::new()),
//         data: vec![],
//     };

//     while let Some(next_page_cursor) = data.next_page_cursor {
//         let request_url = format!("{}&cursor={}", url, next_page_cursor);
//         // println!("Fetching badges from {}", request_url);
//         data = cache::reqwest_with_cache(client, &Url::parse(&request_url)?)?;
//         // let response = client.get(&request_url).send()?;
//         // println!("Response status: {}", response.status());

//         // data = response.json::<Data>()?;
//         badges.extend(data.data);
//     }

//     Ok(badges)
// }

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

fn compress_name(badge: &str) -> String {
    badge
        .replace("Tower of ", "")
        .replace("Citadel of ", "")
        .replace("Steeple of ", "")
}

/// Convert a list of badges to wikitower for later processing.
///
/// # Arguments
/// - badges -> List of badges (mutable) to use
///
/// # Returns
/// - Vec<WikiTower> -> WikiTower::default() pre-made.
// fn convert_basic_wikitower(badges: &mut [Badge]) -> Vec<WikiTower> {
//     // mappings for those annoying towers.
//     let mappings =
//         serde_json::from_str::<Mappings>(&fs::read_to_string("../mappings.json").unwrap())
//             .unwrap()
//             .mappings;

//     // the basic conversion to raw.
//     let raw = badges.iter_mut().map(|b| {
//         let name = clean_badge_name(&b.name);
//         WikiTowerBuilder::default()
//             .badge_name(b.name.clone())
//             .name(mappings.get(&name).unwrap_or(&name).to_owned())
//             .badges(vec![b.id])
//             .build()
//             .unwrap()
//     });

//     // removes those which have more than one and collapses them.
//     let mut deduped = HashMap::<String, WikiTower>::new();
//     for mut r in raw {
//         match deduped.get_mut(&r.name) {
//             Some(v) => v.badges.append(&mut r.badges),
//             None => {
//                 deduped.insert(r.name.to_owned(), r);
//             }
//         }
//     }
//     deduped
//         .values()
//         .map(|v| v.to_owned())
//         .collect::<Vec<WikiTower>>()
// }

/// Take an object and count how many passed/failed.
///
/// # Arguments
/// - obj -> A vector of objects to list through. (type is dynamic)
/// - pass_check -> The function to filter out objects which have passed.
/// - func_name -> Name of the function called before this
/// - file -> Optional path to store something to.
///
/// # Returns
/// - Vec<&'a K> -> A list to use in other places.
fn count_processed<'a, K, P, E>(
    obj: &'a [Result<K, E>],
    pass_check: P,
    func_name: &str,
    file: Option<&PathBuf>,
) -> Vec<&'a K>
where
    P: Fn(&Result<K, E>) -> bool,
    K: std::fmt::Debug,
    E: std::fmt::Debug + 'a,
{
    let mut passed: Vec<&K> = Vec::new();
    let mut failed: Vec<&E> = Vec::new();

    for item in obj.iter() {
        if pass_check(item) {
            passed.push(item.as_ref().ok().unwrap());
        } else {
            failed.push(item.as_ref().err().unwrap());
        }
    }

    match file {
        Some(path) => {
            use std::io::Write;
            match fs::OpenOptions::new().create(true).append(true).open(&path) {
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
        None => {
            log::error!("No path passed for appending: {:?}", file);
        }
    }

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

    passed
}

const DEBUG_PATH: &str = "./badges.temp.txt";

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv().ok();

    let path = PathBuf::from(DEBUG_PATH);
    if path.exists() {
        if let Err(e) = fs::remove_file(&path) {
            log::error!("Failed to remove debug file {:?}: {}", path, e);
        }
    }

    let client = RustClient::new(None, None);
    let url = Url::from_str(&format!("{:}?limit=100", BADGE_URL)).unwrap();

    let mut badges_vec = vec![];
    let raw = get_badges(client, &url).await.unwrap();
    for badge_fut in raw {
        badges_vec.push(badge_fut.await.unwrap());
    }

    let passed = count_processed(
        &badges_vec,
        |f: &Result<OkDetails, ErrorDetails>| f.is_ok(),
        "get_badges",
        Some(&path),
    );

    let tower_data = passed
        .iter()
        .map(|p| process_tower(&p.0, &p.1))
        .inspect(|x| println!("{:?}", x))
        .collect::<Vec<Result<WikiTower, String>>>();

    let processed = count_processed(
        &tower_data,
        |r: &Result<WikiTower, String>| r.is_ok(),
        "process_tower",
        Some(&path),
    );

    // The rest of the original code (commented-out legacy logic) remains unchanged.
}
