mod badge_to_wikitext;
mod cache;
mod definitions;
mod json;
mod parse_wikitext;
mod pywiki;
mod rust_wiki;
mod wiki_api;

use std::{collections::HashMap, fs};

use definitions::*;
use dotenv::dotenv;
use lazy_regex::regex_replace;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::rust_wiki::{WikiTower, WikiTowerBuilder};

pub const BADGE_URL: &str = "https://badges.roblox.com/v1/universes/3264581003/badges?limit=100";
pub const OLD_BADGE_URL: &str =
    "https://badges.roblox.com/v1/universes/1055653882/badges?limit=100";
pub const ETOH_WIKI: &str = "https://jtoh.fandom.com/wiki/";

#[derive(Debug, Serialize, Deserialize)]
struct Mappings {
    mappings: HashMap<String, String>,
}

fn get_badges(client: &Client, url: String) -> Result<Vec<Badge>, Box<dyn std::error::Error>> {
    let mut badges: Vec<Badge> = vec![];
    let mut data: Data = Data {
        previous_page_cursor: None,
        next_page_cursor: Some(String::new()),
        data: vec![],
    };

    while let Some(next_page_cursor) = data.next_page_cursor {
        let request_url = format!("{}&cursor={}", url, next_page_cursor);
        // println!("Fetching badges from {}", request_url);
        data = cache::reqwest_with_cache(client, &Url::parse(&request_url)?)?;
        // let response = client.get(&request_url).send()?;
        // println!("Response status: {}", response.status());

        // data = response.json::<Data>()?;
        badges.extend(data.data);
    }

    Ok(badges)
}

fn clean_badge_name(badge: &str) -> String {
    let trimmed = badge
        .trim()
        .replace("Beat The", "")
        .replace("Beat the", "")
        .replace("beat The", "")
        .replace("beat the", "")
        .replace("(Unobtainable)", "")
        .replace("(LE)", "")
        .trim()
        .to_string();
    regex_replace!(r"\(.*"i, &trimmed, "").to_string()
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
fn convert_basic_wikitower(badges: &mut [Badge]) -> Vec<WikiTower> {
    // mappings for those annoying towers.
    let mappings =
        serde_json::from_str::<Mappings>(&fs::read_to_string("../mappings.json").unwrap())
            .unwrap()
            .mappings;

    // the basic conversion to raw.
    let raw = badges.iter_mut().map(|b| {
        let name = clean_badge_name(&b.name);
        WikiTowerBuilder::default()
            .badge_name(b.name.clone())
            .name(mappings.get(&name).unwrap_or(&name).to_owned())
            .badges(vec![b.id])
            .build()
            .unwrap()
    });

    // removes those which have more than one and collapses them.
    let mut deduped = HashMap::<String, WikiTower>::new();
    for mut r in raw {
        match deduped.get_mut(&r.name) {
            Some(v) => v.badges.append(&mut r.badges),
            None => {
                deduped.insert(r.name.to_owned(), r);
            }
        }
    }
    deduped
        .values()
        .map(|v| v.to_owned())
        .collect::<Vec<WikiTower>>()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    dotenv().ok();

    let client = Client::new();

    let mut badges = get_badges(
        &client,
        String::from("https://badges.roblox.com/v1/universes/3264581003/badges?limit=100"),
    )
    .unwrap();
    let mut other = get_badges(
        &client,
        String::from("https://badges.roblox.com/v1/universes/1055653882/badges?limit=100"),
    )
    .unwrap();
    badges.append(&mut other);
    drop(other);
    // let other_data =
    //     serde_json::from_str::<OtherMap>(&fs::read_to_string("../other_data.json").unwrap())?;
    // let other_ids = other_data
    //     .data
    //     .iter()
    //     .flat_map(|b| b.badges.clone())
    //     .collect::<Vec<u64>>();

    let mut towers = convert_basic_wikitower(&mut badges);

    let result = rust_wiki::parse_badges(&mut towers).unwrap();
    println!("{:#?}", result.0);
    result.1.iter().for_each(|r| {
        println!("{:?}", r);
        println!("============================================");
    });
    Ok(())

    // let mut data = TowerJSON::new();
    // let map = serde_json::from_str::<AreaMap>(&fs::read_to_string("../area_info.json").unwrap())?;
    // // data.make_areas(&map);
    // data.load_map(&map);
    // let mut badge_map =
    //     serde_json::from_str::<BadgeMap>(&fs::read_to_string("../badge_map.json").unwrap())?;
    // badge_map.parse();

    // let mut other_notices = vec![];
    // for badge in badges.iter_mut() {
    //     if let Some(name) = badge_map.get_badge(&badge.id) {
    //         badge.name = name.to_owned();
    //     }
    //     let res = parse_badge(badge, &mut data, &map, &client);
    //     if res.is_err() {
    //         other_notices.push(parse_other(badge, &other_ids, &other_data.ignored));
    //     }
    // }
    // for mut badge in badge_map.use_unused() {
    //     let res = parse_badge(&mut badge, &mut data, &map, &client);
    //     if res.is_err() {
    //         other_notices.push(parse_other(&badge, &other_ids, &other_data.ignored));
    //     }
    // }

    // data.write_to_file("../tower_data.json".into())?;

    // println!();
    // println!();
    // println!();
    // let items = other_notices.iter().filter(|n| !n.is_empty());
    // items.clone().for_each(|n| println!("{:}", n));
    // if items.count() > 0 {
    //     panic!("Items to deal with!");
    // }
    // Ok(())

    // let old_badges = get_badges(
    //     &Client::new(),
    //     String::from("https://badges.roblox.com/v1/universes/1055653882/badges?limit=100"),
    // )
    // .unwrap();

    // let used_tower_badges = serde_json::from_str::<TowerSchema>(
    //     &std::fs::read_to_string("../data/tower_data.json").unwrap(),
    // )
    // .unwrap();
    // let used_badges = serde_json::from_str::<OtherSchema>(
    //     &std::fs::read_to_string("../data/other_data.json").unwrap(),
    // )
    // .unwrap();

    // // Process tower badges

    // let badge_list = used_tower_badges
    //     .areas
    //     .iter()
    //     .flat_map(|(_, area)| {
    //         area.iter()
    //             .flat_map(|info| info.towers.iter().flat_map(|tower| tower.badges.to_vec()))
    //     })
    //     .chain(
    //         used_badges
    //             .data
    //             .iter()
    //             .flat_map(|other| other.badges.to_vec()),
    //     )
    //     .collect::<Vec<u64>>();

    // let dupes = badge_list
    //     .iter()
    //     .filter(|badge_id| {
    //         badge_list
    //             .iter()
    //             .filter(|badge| badge == badge_id)
    //             .collect::<Vec<&u64>>()
    //             .len()
    //             > 1
    //     })
    //     .collect::<Vec<&u64>>();

    // // println!("{:?}", badge_list.collect::<Vec<u64>>());

    // let unused = process_badges(&badge_list, badges);
    // let old_unused = process_badges(&badge_list, old_badges);

    // if !dupes.is_empty() {
    //     println!();
    //     println!();
    //     println!("Duplicate entries found:\n{:?}", dupes);
    // }

    // if !old_unused.is_empty() {
    //     println!();
    //     println!();
    //     println!("Old unused badges found (somehow): \n{}", old_unused);
    // }

    // if !unused.is_empty() {
    //     println!();
    //     println!();
    //     panic!("Unused badges found:\n{}", unused);
    // }
}
