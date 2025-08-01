use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};

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
    #[serde(rename = "n")]
    pub name: String,
    #[serde(rename = "d")]
    pub difficulty: f64,
    #[serde(rename = "b")]
    pub badges: Vec<u64>,
    #[serde(rename = "t")]
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
    #[serde(rename = "ds")]
    pub difficulties: TowerDifficulties,
    #[serde(rename = "p")]
    pub points: u64,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct AreaInformation {
    #[serde(rename = "n")]
    pub name: String,
    #[serde(rename = "r")]
    pub requirements: AreaRequirements,
    #[serde(skip_serializing_if = "Option::is_none", rename = "s")]
    pub sub_area: Option<String>,
    #[serde(rename = "t")]
    pub towers: Vec<Tower>,
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

#[derive(Debug, Clone, Copy)]
pub enum TowerType {
    MiniTower,
    Tower,
    Citadel,
    Obelisk,
    Steeple,
    Invalid,
}

impl Serialize for TowerType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value: u8 = (*self).into();
        serializer.serialize_u8(value)
    }
}

impl<'de> Deserialize<'de> for TowerType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Ok(TowerType::from(value))
    }
}

impl Display for TowerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TowerType::MiniTower => "Mini Tower".to_string(),
                TowerType::Tower => "Tower".to_string(),
                TowerType::Citadel => "Citadel".to_string(),
                TowerType::Obelisk => "Obelisk".to_string(),
                TowerType::Steeple => "Steeple".to_string(),
                TowerType::Invalid => "".to_string(),
            }
        )
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

impl From<TowerType> for u8 {
    fn from(value: TowerType) -> Self {
        match value {
            TowerType::MiniTower => 0,
            TowerType::Tower => 1,
            TowerType::Citadel => 2,
            TowerType::Obelisk => 3,
            TowerType::Steeple => 4,
            TowerType::Invalid => 0b11110000,
        }
    }
}

// impl Serialize for TowerType {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         let mut s = serializer.serialize_u8(self)
//     }
// }

#[derive(Serialize, Debug, Deserialize)]
pub struct AreaMap {
    pub areas: HashMap<String, HashMap<String, Vec<String>>>,
}

impl AreaMap {
    pub fn get_area(&self, area: &String) -> Option<(String, String)> {
        for main in self.areas.iter() {
            for sub in main.1.iter() {
                if sub.1.contains(area) {
                    return Some((main.0.to_owned(), sub.0.to_owned()));
                }
            }
        }

        None
    }
}
