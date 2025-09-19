use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fmt::Display,
};

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BadgeUniverse {
    pub id: u64,
    pub name: String,
    pub root_place_id: u64,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BadgeStatistics {
    pub past_day_awarded_count: u64,
    pub awarded_count: u64,
    pub win_rate_percentage: f64,
}

#[derive(Debug, Deserialize, Serialize, Default)]
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

#[derive(Debug, Clone)]
pub struct Tower {
    // #[serde(rename = "n")]
    pub name: String,
    // #[serde(rename = "d")]
    pub difficulty: f64,
    // #[serde(rename = "b")]
    pub badges: Vec<u64>,
    // #[serde(rename = "t")]
    pub tower_type: Option<TowerType>,
}

impl Serialize for Tower {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut csv = format!("{},{},{:?}", self.name, self.difficulty, self.badges);
        if let Some(ttype) = self.tower_type {
            csv = format!("{csv},{}", ttype as u8);
        }

        serializer.serialize_str(&csv)
    }
}
impl<'de> Deserialize<'de> for Tower {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.split(',').collect();

        if parts.len() < 3 {
            return Err(serde::de::Error::custom("invalid tower format"));
        }

        let name = parts[0].to_string();
        let difficulty = parts[1].parse().map_err(serde::de::Error::custom)?;

        let badges_str = parts[2].trim_start_matches('[').trim_end_matches(']');
        let badges = badges_str
            .split_whitespace()
            .filter_map(|s| s.trim_matches(',').parse().ok())
            .collect();

        let tower_type = if parts.len() > 3 {
            Some(TowerType::from(parts[3]))
        } else {
            None
        };

        Ok(Tower {
            name,
            difficulty,
            badges,
            tower_type,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct TowerDifficulties {
    #[serde(skip_serializing_if = "Option::is_none", rename = "e")]
    pub easy: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "m")]
    pub medium: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "h")]
    pub hard: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "d")]
    pub difficult: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "c")]
    pub challenging: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "i")]
    pub intense: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "r")]
    pub remorseless: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "s")]
    pub insane: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "x")]
    pub extreme: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "t")]
    pub terrifying: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "a")]
    pub catastrophic: Option<u64>,
}

impl TowerDifficulties {
    pub fn parse_difficulty(&mut self, difficulty: &str, count: u64) {
        match difficulty.to_lowercase().trim() {
            "easy" => self.easy = Some(count),
            "medium" => self.medium = Some(count),
            "hard" => self.hard = Some(count),
            "difficult" => self.difficult = Some(count),
            "challenging" => self.challenging = Some(count),
            "intense" => self.intense = Some(count),
            "remorseless" => self.remorseless = Some(count),
            "insane" => self.insane = Some(count),
            "extreme" => self.extreme = Some(count),
            "terrifying" => self.terrifying = Some(count),
            "catastrophic" => self.catastrophic = Some(count),
            inv => {
                println!("Not a valid difficulty! {:?}", inv);
            }
        }
    }

    pub fn types() -> impl Iterator<Item = String> {
        vec![
            String::from("easy"),
            String::from("medium"),
            String::from("hard"),
            String::from("difficult"),
            String::from("challenging"),
            String::from("intense"),
            String::from("remorseless"),
            String::from("insane"),
            String::from("extreme"),
            String::from("terrifying"),
            String::from("catastrophic"),
        ]
        .into_iter()
    }
    pub fn find_type(text: &str) -> Option<String> {
        // get rid of "Difficulty" as this causes some issues.
        let text = text.replace("Difficulty", "").replace("difficulty", "");
        for diff in Self::types() {
            if text.contains(&diff) {
                return Some(diff.clone());
            }
        }

        None
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct AreaRequirements {
    #[serde(rename = "ds")]
    pub difficulties: TowerDifficulties,
    #[serde(rename = "p")]
    pub points: u64,
}

#[derive(Debug, Deserialize, Serialize)]
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

impl Default for AreaInformation {
    fn default() -> Self {
        Self {
            name: "Unknown area".to_string(),
            requirements: AreaRequirements::default(),
            sub_area: None,
            towers: vec![],
        }
    }
}

impl AreaInformation {
    pub fn sort(&mut self) {
        self.towers.sort_by(|a, b| {
            if a.difficulty >= b.difficulty {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        });
    }
}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct OtherBadge {
//     pub name: String,
//     pub category: String,
//     pub badges: Vec<u64>,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct OtherSchema {
//     pub data: Vec<OtherBadge>,
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TowerType {
    MiniTower,
    Steeple,
    Tower,
    Citadel,
    Obelisk,
    Invalid,
}

impl Default for TowerType {
    fn default() -> Self {
        Self::Invalid
    }
}

impl Serialize for TowerType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(u8::from(*self))
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
impl From<String> for TowerType {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
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
            TowerType::Steeple => 1,
            TowerType::Tower => 2,
            TowerType::Citadel => 3,
            TowerType::Obelisk => 4,
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
    pub areas: HashMap<String, Vec<String>>,
}

impl AreaMap {
    pub fn get_area(&self, area: &str) -> String {
        for main in self.areas.iter() {
            println!("{:?}", main);
            if main
                .1
                .iter()
                .map(|v| v.trim().to_lowercase())
                .any(|v| v == *area.trim().to_lowercase())
            {
                return main.0.to_string();
            }
        }

        "other".to_string()
    }

    pub fn key_loop(&self) -> std::collections::hash_map::Keys<'_, String, Vec<String>> {
        self.areas.keys()
    }
}

#[derive(Serialize, Debug, Deserialize)]
pub struct BadgeMap {
    pub badges: HashMap<String, Vec<u64>>,
    #[serde(skip)]
    badge_map: HashMap<u64, String>,
    #[serde(skip)]
    unused: HashSet<u64>,
}

impl BadgeMap {
    pub fn parse(&mut self) {
        self.badges.iter().for_each(|b| {
            b.1.iter().for_each(|id| {
                self.badge_map.insert(*id, b.0.to_owned());
                self.unused.insert(*id);
            });
        });
    }

    pub fn get_badge(&mut self, id: &u64) -> Option<&String> {
        self.unused.remove(id);
        self.badge_map.get(id)
    }

    pub fn use_unused(&mut self) -> impl Iterator<Item = Badge> {
        self.unused.clone().into_iter().map(|b| Badge {
            id: b,
            name: self.badge_map.get(&b).unwrap().to_owned(),
            ..Default::default()
        })

        // self.unused.iter().clone().map(|b| Badge {
        //     id: *b,
        //     name: self.get_badge(b).unwrap().to_owned(),
        //     ..Default::default()
        // })
    }
}
