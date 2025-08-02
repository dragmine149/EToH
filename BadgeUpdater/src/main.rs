mod cache;
mod definitions;
mod json;
mod parse_wikitext;

use std::fs;

use definitions::*;
use parse_wikitext::WIkiTower;
use reqwest::blocking::Client;
use url::Url;

use crate::json::TowerJSON;

const WIKI_BASE: &str = "https://jtoh.fandom.com/wiki";

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

fn scrap_wiki(client: &Client, badge_name: impl Into<String>) -> Option<WIkiTower> {
    let badge: String = badge_name.into();

    let url =
        Url::parse_with_params(&format!("{}/{}", WIKI_BASE, badge), &[("action", "raw")]).unwrap();

    let wikicache = cache::read_cache(&url);
    let wikitext = match wikicache {
        Some(wikicache) => wikicache,
        None => {
            let data = client.get(url.to_owned()).send().ok()?.text().ok()?;
            // println!("{data}");
            cache::write_cache(&url, &data).ok()?;
            // println!("e");
            data
        }
    };

    let new_badge = follow_redirect(&wikitext);
    if let Some(badge) = new_badge {
        return scrap_wiki(client, badge);
    }

    let mut wiki = parse_wikitext::parse_wiki_text(&wikitext)?;
    wiki.tower_name = badge;
    Some(wiki)
}

fn follow_redirect(wikitext: &str) -> Option<String> {
    match wikitext.starts_with("#REDIRECT") {
        true => {
            let tower_name = wikitext
                .split_once(" ")
                .unwrap()
                .1
                .replace("[[", "")
                .replace("]]", "");
            Some(tower_name)
        }
        false => None,
    }
}

fn process_badges(badge_list: &[u64], badges: Vec<Badge>) -> String {
    badges
        .iter()
        .filter(|badge| !badge_list.contains(&badge.id))
        .filter(|badge| !badge.name.to_lowercase().contains("placeholder"))
        .filter(|badge| badge.name != "Beat The Tower Of ...")
        .filter(|badge| badge.id != 2124560526) // The duplicate badge of Tower of Suffering Outside.
        .map(|badge| format!("{} - {}\n", badge.id, badge.name))
        .collect::<String>()
}

fn clean_badge_name(badge: &str) -> String {
    badge.trim().replace("Beat The", "").trim().to_string()
}

fn compress_name(badge: &str) -> String {
    badge
        .replace("Tower of ", "")
        .replace("Citadel of ", "")
        .replace("Steeple of ", "")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let mut badges = get_badges(
        &client,
        String::from("https://badges.roblox.com/v1/universes/3264581003/badges?limit=100"),
    )
    .unwrap();

    let mut data = TowerJSON::new();
    let map = serde_json::from_str::<AreaMap>(&fs::read_to_string("../area_info.json").unwrap())?;
    data.make_areas(&map);

    badges
        .iter_mut()
        .for_each(|b| b.name = clean_badge_name(&b.name));

    for badge in badges.iter() {
        println!("Tower: {:?}", badge.name);
        let wiki = scrap_wiki(&client, &badge.name);
        // println!("{:#?}", wiki);

        if wiki.is_none() {
            continue;
        }
        if data.has_tower(&badge.name) {
            data.add_tower_badge(&badge.name, badge.id);
        }
        let wiki = wiki.unwrap();
        let name = wiki.tower_name.to_owned();

        data.insert_tower(wiki, &compress_name(&name), badge.id, &map);
    }

    data.write_to_file("../tower_data.json".into())
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
