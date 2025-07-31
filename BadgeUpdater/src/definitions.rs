use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash};

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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Tower {
    pub name: String,
    pub difficulty: f64,
    pub badges: Vec<u64>,
    #[serde(rename = "type")]
    pub tower_type: Option<TowerType>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct TowerDifficulties {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub easy: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medium: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hard: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub difficult: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub challenging: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intense: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remorseless: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insane: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extreme: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terrifying: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catastrophic: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct AreaRequirements {
    pub difficulties: TowerDifficulties,
    pub points: u64,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct AreaInformation {
    pub name: String,
    pub requirements: AreaRequirements,
    #[serde(skip_serializing_if = "Option::is_none")]
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum TowerType {
    MiniTower,
    Tower,
    Citadel,
    Obelisk,
    Steeple,
    Invalid,
}

impl ToString for TowerType {
    fn to_string(&self) -> String {
        match self {
            TowerType::MiniTower => "Mini Tower".to_string(),
            TowerType::Tower => "Tower".to_string(),
            TowerType::Citadel => "Citadel".to_string(),
            TowerType::Obelisk => "Obelisk".to_string(),
            TowerType::Steeple => "Steeple".to_string(),
            TowerType::Invalid => "".to_string(),
        }
    }
}

impl From<&str> for TowerType {
    fn from(value: &str) -> Self {
        let value = value.trim().replace(" ", "").to_lowercase();
        match value.as_str() {
            "minitower" => Self::MiniTower,
            "tower" => Self::Tower,
            "citadel" => Self::Citadel,
            "obelisk" => Self::Obelisk,
            "steeple" => Self::Steeple,
            _ => Self::Invalid,
        }
    }
}

impl From<u8> for TowerType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::MiniTower,
            1 => Self::Steeple,
            2 => Self::Tower,
            3 => Self::Citadel,
            4 => Self::Obelisk,
            _ => Self::Invalid,
        }
    }
}

#[derive(Serialize, Debug, Deserialize)]
pub struct AreaMap {
    pub areas: HashMap<String, HashMap<String, Vec<String>>>,
}

impl AreaMap {
    pub fn get_area(&self, area: &String) -> (String, String) {
        for main in self.areas.iter() {
            for sub in main.1.iter() {
                if sub.1.contains(area) {
                    return (main.0.to_owned(), sub.0.to_owned());
                }
            }
        }

        (String::default(), String::default())
    }
}
