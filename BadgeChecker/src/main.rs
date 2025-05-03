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

fn default() -> Option<u64> {
    None
}

#[derive(Debug, Deserialize, Serialize)]
// #[serde(rename_all = "camelCase")]
pub struct Tower {
    pub difficulty: Option<f64>,
    #[serde(default = "default")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge_id: Option<u64>,
    pub old_id: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TowerDifficulties {
    pub easy: Option<u64>,
    pub medium: Option<u64>,
    pub hard: Option<u64>,
    pub difficult: Option<u64>,
    pub challenging: Option<u64>,
    pub intense: Option<u64>,
    pub remorseless: Option<u64>,
    pub insane: Option<u64>,
    pub extreme: Option<u64>,
    pub terrifying: Option<u64>,
    pub catastrophic: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AreaRequirements {
    pub tower_difficulties: TowerDifficulties,
    pub tower_points: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AreaInformation {
    pub requirements: AreaRequirements,
    pub sub_area: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ring {
    pub area_information: AreaInformation,
    #[serde(flatten)]
    pub towers: HashMap<String, Tower>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Zone {
    pub area_information: AreaInformation,
    #[serde(flatten)]
    pub towers: HashMap<String, Tower>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub area_information: AreaInformation,
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
pub struct TowerSchemaParent {
    pub data: TowerSchema,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BadgeCategory {
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    badge: Option<HashMap<String, Tower>>,
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<HashMap<String, BadgeCategory>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OtherSchema {
    // #[serde(rename = "$schema")]
    // _schema: String,
    data: HashMap<String, BadgeCategory>,
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

    let used_tower_badges = serde_json::from_str::<TowerSchemaParent>(
        &std::fs::read_to_string("./data/tower_data.json").unwrap(),
    )
    .unwrap();
    let used_badges = serde_json::from_str::<OtherSchema>(
        &std::fs::read_to_string("./data/other_data.json").unwrap(),
    )
    .unwrap();
    let mut badge_list: Vec<u64> = Vec::new();

    // Process tower badges
    for (_, ring) in used_tower_badges.data.rings.iter() {
        for (_, tower) in ring.towers.iter() {
            let id = tower.badge_id;
            if let Some(id) = id {
                badge_list.push(id);
            }
        }
    }
    for (_, zone) in used_tower_badges.data.zones.iter() {
        for (_, tower) in zone.towers.iter() {
            let id = tower.badge_id;
            if let Some(id) = id {
                badge_list.push(id);
            }
        }
    }
    for (_, event) in used_tower_badges.data.events.iter() {
        for (_, tower) in event.towers.iter() {
            let id = tower.badge_id;
            if let Some(id) = id {
                badge_list.push(id);
            }
        }
    }

    // Process other badges
    fn process_badge_category(category: &BadgeCategory, badge_list: &mut Vec<u64>) {
        if let Some(badges) = &category.badge {
            for (_, tower) in badges.iter() {
                if let Some(id) = tower.badge_id {
                    badge_list.push(id);
                }
            }
        }
        if let Some(subcategories) = &category.category {
            for (_, subcategory) in subcategories.iter() {
                process_badge_category(subcategory, badge_list);
            }
        }
    }

    for (_, category) in used_badges.data.iter() {
        process_badge_category(category, &mut badge_list);
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
