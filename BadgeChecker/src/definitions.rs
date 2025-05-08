use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgeUniverse {
    pub id: u64,
    pub name: String,
    pub root_place_id: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgeStatistics {
    pub past_day_awarded_count: u64,
    pub awarded_count: u64,
    pub win_rate_percentage: f64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Badge {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub display_name: String,
    pub display_description: Option<String>,
    pub enabled: bool,
    pub icon_image_id: u64,
    pub display_icon_image_id: u64,
    pub created: String,
    pub updated: String,
    pub statistics: BadgeStatistics,
    pub awarding_universe: BadgeUniverse,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub previous_page_cursor: Option<String>,
    pub next_page_cursor: Option<String>,
    pub data: Vec<Badge>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tower {
    pub name: String,
    pub difficulty: f64,
    pub badges: Vec<u64>,
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
    pub difficulties: TowerDifficulties,
    pub points: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AreaInformation {
    pub name: String,
    pub requirements: AreaRequirements,
    pub sub_area: Option<String>,
    pub towers: Vec<Tower>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TowerSchema {
    pub areas: HashMap<String, Vec<AreaInformation>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OtherBadge {
    pub name: String,
    pub category: String,
    pub badges: Vec<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OtherSchema {
    pub data: Vec<OtherBadge>,
}
