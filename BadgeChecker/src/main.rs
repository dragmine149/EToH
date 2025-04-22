use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct BadgeUniverse {
    id: u64,
    name: String,
    root_place_id: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct BadgeStatistics {
    past_day_awarded_count: u64,
    awarded_count: u64,
    win_rate_percentage: f64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Badge {
    id: u64,
    name: String,
    description: Option<String>,
    display_name: String,
    display_description: Option<String>,
    enabled: bool,
    icon_image_id: u64,
    display_icon_image_id: u64,
    created: String,
    updated: String,
    statistics: BadgeStatistics,
    awarding_universe: BadgeUniverse,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Data {
    previous_page_cursor: Option<String>,
    next_page_cursor: Option<String>,
    data: Vec<Badge>,
}

#[derive(Debug, Deserialize, Serialize)]
// #[serde(rename_all = "camelCase")]
pub struct Tower {
    pub difficulty: Option<f64>,
    pub badge_id: u64,
    // #[serde(skip_serializing_if = "Option::is_none")]
    pub old_id: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ring {
    #[serde(flatten)]
    pub towers: HashMap<String, Tower>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Zone {
    #[serde(flatten)]
    pub towers: HashMap<String, Tower>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    #[serde(flatten)]
    pub towers: HashMap<String, Tower>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TowerSchema {
    pub rings: HashMap<String, Ring>,
    pub zones: HashMap<String, Zone>,
    pub events: HashMap<String, Event>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OtherSchema {
    data: HashMap<String, Tower>,
}

// const URL: &str = "https://badges.roblox.com/v1/universes/3264581003/badges?limit=100";

fn get_badges(client: &Client, url: String) -> Result<Vec<Badge>, reqwest::Error> {
    let mut badges: Vec<Badge> = vec![];
    let mut data: Data = Data {
        previous_page_cursor: None,
        next_page_cursor: Some(String::new()),
        data: vec![],
    };

    while let Some(next_page_cursor) = data.next_page_cursor {
        let request_url = format!("{}&cursor={}", url, next_page_cursor);
        println!("Fetching badges from {}", request_url);
        let response = client.get(&request_url).send()?;
        println!("Response status: {}", response.status());

        data = response.json::<Data>()?;
        badges.extend(data.data);
    }

    Ok(badges)
}

fn main() {
    let badges = get_badges(
        &Client::new(),
        String::from("https://badges.roblox.com/v1/universes/3264581003/badges?limit=100"),
    )
    .unwrap();
    // let old_badges = get_badges(
    //     &Client::new(),
    //     String::from("https://badges.roblox.com/v1/universes/1055653882/badges?limit=100"),
    // )
    // .unwrap();

    let used_tower_badges = serde_json::from_str::<TowerSchema>(
        &std::fs::read_to_string("./data/tower_data.json").unwrap(),
    )
    .unwrap();
    let used_badges = serde_json::from_str::<HashMap<String, Tower>>(
        &std::fs::read_to_string("./data/other_data.json").unwrap(),
    )
    .unwrap();
    let mut badge_list: Vec<u64> = Vec::new();

    // Process tower badges
    for (_, ring) in used_tower_badges.rings.iter() {
        for (_, tower) in ring.towers.iter() {
            badge_list.push(tower.badge_id);
        }
    }
    for (_, zone) in used_tower_badges.zones.iter() {
        for (_, tower) in zone.towers.iter() {
            badge_list.push(tower.badge_id);
        }
    }
    for (_, event) in used_tower_badges.events.iter() {
        for (_, tower) in event.towers.iter() {
            badge_list.push(tower.badge_id);
        }
    }

    // Process other badges
    for (_, tower) in used_badges.iter() {
        badge_list.push(tower.badge_id);
    }
    // used_tower_badges.iter().map(|badge| badge.id).collect::<Vec<_>>();
    // used_badges.iter().map(|badge| badge.id).collect::<Vec<_>>();

    // old_badges.iter().map(|badge| badge.id);
    let unused = badges
        .iter()
        .filter(|badge| !badge_list.contains(&badge.id))
        .map(|badge| format!("{} - {}\n", badge.id, badge.name))
        .collect::<String>();
    // .flatten();

    if !unused.is_empty() {
        println!();
        println!();
        panic!("Unused badges found:\n{}", unused);
    }
}
