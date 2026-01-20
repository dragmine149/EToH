use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::{
    collections::HashMap,
    error::Error,
    fmt::{Debug, Display},
};

use crate::{reqwest_client::RustError, wikitext::WikiText};

// #[derive(Debug, Deserialize, Serialize, Default, Clone)]
// #[serde(rename_all = "camelCase")]
// pub struct BadgeUniverse {
//     pub id: u64,
//     pub name: String,
//     pub root_place_id: u64,
// }

// #[derive(Debug, Deserialize, Serialize, Default, Clone)]
// #[serde(rename_all = "camelCase")]
// pub struct BadgeStatistics {
//     pub past_day_awarded_count: u64,
//     pub awarded_count: u64,
//     pub win_rate_percentage: f64,
// }

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Badge {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub display_name: String,
    pub display_description: Option<String>,
    // pub enabled: bool,
    pub icon_image_id: u64,
    pub display_icon_image_id: u64,
    pub created: String,
    pub updated: String,
    // pub statistics: BadgeStatistics,
    // pub awarding_universe: BadgeUniverse,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub previous_page_cursor: Option<String>,
    pub next_page_cursor: Option<String>,
    pub data: Vec<Badge>,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            previous_page_cursor: None,
            next_page_cursor: Some(String::new()),
            data: vec![],
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WikiCategoryMember {
    pub pageid: u32,
    pub ns: usize,
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WikiCategoryQuery {
    // pub pages: []
    pub categorymembers: Vec<WikiCategoryMember>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WikiCategory {
    pub batchcomplete: bool,
    pub query: WikiCategoryQuery,
}

#[derive(Debug, Default)]
pub struct WikiTower {
    pub badge_name: String,
    pub badge_id: u64,
    pub page_name: String,
    pub difficulty: f64,
    pub area: String,
    pub length: Length,
    pub tower_type: TowerType,
}

#[derive(Debug, Clone)]
pub struct EventInfo {
    pub area_name: String,
    pub event_name: String,
}

#[derive(Debug, Clone)]
pub struct EventItem {
    pub item_name: String,
    pub event_name: String,
    // badge id is still required when there is no tower_name.
    pub badges: [u64; 2],
    pub tower_name: Option<String>,
}

pub enum GlobalArea {
    Area(AreaInformation),
    Event(EventInfo),
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
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
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct AreaRequirements {
    #[serde(rename = "ds")]
    pub difficulties: TowerDifficulties,
    #[serde(rename = "p")]
    pub points: u64,
    pub areas: HashMap<String, AreaRequirements>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AreaInformation {
    #[serde(rename = "n")]
    pub name: String,
    #[serde(rename = "r")]
    pub requirements: Option<AreaRequirements>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "s")]
    pub parent_area: Option<String>,
}

impl Default for AreaInformation {
    fn default() -> Self {
        Self {
            name: "Unknown area".to_string(),
            requirements: None,
            parent_area: None,
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TowerType {
    MiniTower,
    Steeple,
    #[default]
    Tower,
    Citadel,
    Obelisk,
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
            _ => Self::default(),
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
            _ => Self::default(),
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
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum Length {
    /// <= 20 mins
    #[default]
    CakeWalk,
    /// 20+ mins
    VeryShort,
    /// 30+ mins
    Short,
    /// 45+ mins
    Medium,
    /// 60+ mins
    Long,
    /// 90+ mins
    VeryLong,
    /// INHUMANELY
    Inhumanely,
}

impl From<u8> for Length {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::CakeWalk,
            1 => Self::VeryShort,
            2 => Self::Short,
            3 => Self::Medium,
            4 => Self::Long,
            5 => Self::VeryLong,
            6 => Self::Inhumanely,
            _ => Self::default(),
        }
    }
}

impl From<Length> for u8 {
    fn from(value: Length) -> Self {
        match value {
            Length::CakeWalk => 0,
            Length::VeryShort => 1,
            Length::Short => 2,
            Length::Medium => 3,
            Length::Long => 4,
            Length::VeryLong => 5,
            Length::Inhumanely => 6,
        }
    }
}

impl From<u16> for Length {
    fn from(value: u16) -> Self {
        match value {
            20 => Self::VeryShort,
            30 => Self::Short,
            45 => Self::Medium,
            60 => Self::Long,
            90 => Self::VeryLong,
            u16::MAX => Self::Inhumanely,
            _ => Self::default(),
        }
    }
}
impl From<Length> for u16 {
    fn from(value: Length) -> Self {
        match value {
            Length::CakeWalk => 0,
            Length::VeryShort => 20,
            Length::Short => 30,
            Length::Medium => 45,
            Length::Long => 60,
            Length::VeryLong => 90,
            Length::Inhumanely => u16::MAX,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OtherData {
    pub name: String,
    pub category: String,
    pub badges: Vec<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OtherMap {
    pub data: Vec<OtherData>,
    pub ignored: Vec<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WikiSearchEntry {
    pub title: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct WikiSearchList {
    pub search: Vec<WikiSearchEntry>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct WikiSearch {
    pub query: WikiSearchList,
}

#[derive(Debug)]
#[allow(
    dead_code,
    reason = "i use these for debugging, ik i don't use them rust because we don't need to use them and panic is just not worth it"
)]
pub enum ProcessError {
    Reqwest(RustError),
    Process(String),
}
impl From<RustError> for ProcessError {
    fn from(value: RustError) -> Self {
        Self::Reqwest(value)
    }
}
impl From<reqwest::Error> for ProcessError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(RustError::from(value))
    }
}
impl From<reqwest_middleware::Error> for ProcessError {
    fn from(value: reqwest_middleware::Error) -> Self {
        Self::Reqwest(RustError::from(value))
    }
}
impl From<String> for ProcessError {
    fn from(value: String) -> Self {
        Self::Process(value)
    }
}
impl From<&str> for ProcessError {
    fn from(value: &str) -> Self {
        Self::Process(value.to_owned())
    }
}

#[derive(Debug)]
#[allow(dead_code, reason = "i use these for debugging")]
pub struct ErrorDetails(pub ProcessError, pub Badge);
pub struct OkDetails(pub WikiText, pub Badge);

impl Debug for OkDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "OkDetails(")?;
        writeln!(
            f,
            "\tWikiText {{ text: --ignored--, page_name: {:?} }},",
            self.0.page_name()
        )?;
        for line in format!("{:#?}", self.1).lines() {
            writeln!(f, "\t{}", line)?;
        }

        // writeln!(f, "\t{:#?}", self.1)?;
        write!(f, ")")
    }
}

#[derive(Debug, Default)]
pub struct PageDetails {
    pub text: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BadgeOverwrite {
    pub badge_id: u64,
    pub alt_ids: Vec<u64>,
    pub category: String,
    pub name: String,
}

// Serialiser and deseriliser for BadgeOverwrite written by GPT-5 mini

/// For nicer error messages when deserializing arrays
#[derive(Debug)]
struct BadFormat(&'static str);

impl Display for BadFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bad format: {}", self.0)
    }
}

impl Error for BadFormat {}

impl<'de> Deserialize<'de> for BadgeOverwrite {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // We expect a map entry like "123": [ "Category", "Name", "456", "789" ]
        // But serde gives us a whole value. We'll accept either:
        //  - a map with a single string key mapping to an array (the top-level entry)
        //  - or directly the array along with an externally provided badge id (not used here)
        let v = Value::deserialize(deserializer)?;

        match v {
            Value::Object(map) => {
                // Expect exactly one key => array value
                if map.len() != 1 {
                    return Err(serde::de::Error::custom(
                        "expected an object with a single badge id entry",
                    ));
                }
                let (k, val) = map.into_iter().next().unwrap();
                let badge_id = k.parse::<u64>().map_err(|_| {
                    serde::de::Error::custom(format!("badge id key is not a u64: {}", k))
                })?;
                let arr = match val {
                    Value::Array(a) => a,
                    _ => {
                        return Err(serde::de::Error::custom(
                            "expected badge entry to be an array",
                        ));
                    }
                };
                parse_array(badge_id, arr).map_err(serde::de::Error::custom)
            }
            other => Err(serde::de::Error::custom(format!(
                "expected top-level object, got {}",
                other
            ))),
        }
    }
}

impl Serialize for BadgeOverwrite {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // serialize as an object with single string key -> array
        use serde_json::Value as Jv;

        let mut arr: Vec<Jv> = Vec::with_capacity(2 + self.alt_ids.len());
        arr.push(Jv::String(self.category.clone()));
        arr.push(Jv::String(self.name.clone()));
        for id in &self.alt_ids {
            arr.push(Jv::String(id.to_string()));
        }

        let mut map = serde_json::map::Map::new();
        map.insert(self.badge_id.to_string(), Jv::Array(arr));
        Jv::Object(map).serialize(serializer)
    }
}

fn parse_array(
    badge_id: u64,
    arr: Vec<Value>,
) -> Result<BadgeOverwrite, Box<dyn std::error::Error>> {
    if arr.len() < 2 {
        return Err(Box::new(BadFormat(
            "array must contain at least category and name",
        )));
    }

    // category and name must be strings
    let category = match &arr[0] {
        Value::String(s) => s.clone(),
        _ => return Err(Box::new(BadFormat("category must be a string"))),
    };
    let name = match &arr[1] {
        Value::String(s) => s.clone(),
        _ => return Err(Box::new(BadFormat("name must be a string"))),
    };

    // rest are ids (strings that parse to u64 or numbers)
    let mut alt_ids = Vec::with_capacity(arr.len().saturating_sub(2));
    for v in arr.into_iter().skip(2) {
        match v {
            Value::String(s) => {
                let id = s
                    .parse::<u64>()
                    .map_err(|_| BadFormat("alt id strings must parse to u64"))?;
                alt_ids.push(id);
            }
            Value::Number(n) => {
                if let Some(u) = n.as_u64() {
                    alt_ids.push(u);
                } else {
                    return Err(Box::new(BadFormat("alt id number not u64")));
                }
            }
            _ => return Err(Box::new(BadFormat("alt id must be string or number"))),
        }
    }

    Ok(BadgeOverwrite {
        badge_id,
        alt_ids,
        category,
        name,
    })
}

// Helper functions to convert a whole map <String, Array> -> Vec<BadgeOverwrite>
pub fn badges_from_map_value(v: &Value) -> Result<Vec<BadgeOverwrite>, Box<dyn std::error::Error>> {
    match v {
        Value::Object(map) => {
            let mut out = Vec::with_capacity(map.len());
            for (k, val) in map.iter() {
                let badge_id = k
                    .parse::<u64>()
                    .map_err(|_| BadFormat("map key must be a string representable as u64"))?;
                let arr = match val {
                    Value::Array(a) => a.clone(),
                    _ => return Err(Box::new(BadFormat("map value must be an array"))),
                };
                out.push(parse_array(badge_id, arr)?);
            }
            Ok(out)
        }
        _ => Err(Box::new(BadFormat("expected top-level object/map"))),
    }
}
