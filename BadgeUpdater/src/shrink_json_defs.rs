//! As much as json is good, we do want to shrink it a bit to save on space as much as possible.
//!
//! And because we still want a decent version of ease of use, we... just have to duplicate stuff.
//!
//! Note: As much as deserialize exists in this file, they technically don't need to as we never read the shrunk json.
//! Eh, practice.

use chrono::{DateTime, FixedOffset, Utc};
use itertools::Itertools;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor, ser::SerializeStruct};
use std::collections::HashMap;

use crate::{
    definitions::{
        AreaRequirements, Category, ExtendedArea, Item, Length, OtherData, Tower,
        TowerDifficulties, TowerType,
    },
    json::{Jsonify, SortedHashMap},
};

/// Helper function for serde, skips if it's default value.
fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    *value == T::default()
}

/// Helper function for serde as we can't have multiple skip_ifs.
///
/// If [is_default] or [Option::None] skip.
fn is_default_or_none<T: Default + PartialEq>(value: &Option<T>) -> bool {
    if let Some(v) = value {
        return is_default(v);
    }
    true
}

#[derive(Debug, Clone, Default)]
pub struct ShrinkJson {
    modify_date: DateTime<Utc>,
    categories: HashMap<String, ShrinkCategory>,
}

impl Serialize for ShrinkJson {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Jsonify", 2)?;
        s.serialize_field("m", &(self.modify_date.timestamp()))?;
        s.serialize_field("c", &SortedHashMap(self.categories.to_owned()))?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for ShrinkJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct("Jsonify", &["m", "c"], ShrinkJsonVisitor)
    }
}

struct ShrinkJsonVisitor;
impl<'de> Visitor<'de> for ShrinkJsonVisitor {
    type Value = ShrinkJson;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct Jsonify")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut modify_date: Option<DateTime<Utc>> = None;
        let mut categories: Option<HashMap<String, ShrinkCategory>> = None;

        while let Some(key) = map.next_key()? {
            match key {
                "m" => {
                    let timestamp: i64 = map.next_value()?;
                    modify_date = Some(
                        DateTime::<Utc>::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now),
                    );
                }
                "c" => {
                    categories = Some(map.next_value()?);
                }
                _ => {
                    let _: serde::de::IgnoredAny = map.next_value()?;
                }
            }
        }

        Ok(ShrinkJson {
            modify_date: modify_date.unwrap_or_else(Utc::now),
            categories: categories.unwrap_or_default(),
        })
    }
}

impl From<Jsonify> for ShrinkJson {
    fn from(value: Jsonify) -> Self {
        Self {
            modify_date: value.modify_date,
            categories: value
                .categories
                .iter()
                .map(|c| (c.0.to_owned(), ShrinkCategory::from(c.1)))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShrinkTower {
    /// Note: Name is also wiki page.
    pub name: String,
    pub badges: [u64; 2],
    pub difficulty: f64,
    pub length: Length,
    pub tower_type: TowerType,
}

impl Serialize for ShrinkTower {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let name = match self.tower_type {
            TowerType::MiniTower => self.name.to_owned(),
            TowerType::Steeple => self.name.replace("Steeple of", ""),
            TowerType::Tower => self.name.replace("Tower of", ""),
            TowerType::Citadel => self.name.replace("Citadel of", ""),
            TowerType::Obelisk => self.name.replace("Obelisk of", ""),
        };

        serializer.serialize_str(&format!(
            "{},{},{},{},{},{}",
            name.trim(),
            self.badges[0],
            self.badges[1],
            self.difficulty,
            self.length as u8,
            self.tower_type as u8,
        ))
    }
}
impl<'de> Deserialize<'de> for ShrinkTower {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(TowerVisitor)
    }
}

struct TowerVisitor;
impl<'de> Visitor<'de> for TowerVisitor {
    type Value = ShrinkTower;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("A CSV of 6 elements. (name,badge,badge,diff,len,type)")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let mut items = v.split(",");
        let name = items.next().unwrap().to_string();
        let badges = [
            items.next().unwrap().parse::<u64>().unwrap(),
            items.next().unwrap().parse::<u64>().unwrap(),
        ];
        let difficulty = items.next().unwrap().parse::<f64>().unwrap();
        let length = Length::from(items.next().unwrap().parse::<u8>().unwrap());
        let tower_type = TowerType::from(items.next().unwrap().parse::<u8>().unwrap());

        Ok(ShrinkTower {
            name,
            badges,
            difficulty,
            length,
            tower_type,
        })
    }

    // fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    // where
    //     A: serde::de::SeqAccess<'de>,
    // {
    //     let name = seq.next_element()?.unwrap();
    //     let badges = [seq.next_element()?.unwrap(), seq.next_element()?.unwrap()];
    //     let difficulty = seq.next_element()?.unwrap();
    //     let length = seq.next_element()?.unwrap();
    //     let tower_type = seq.next_element()?.unwrap();
    //     let wiki_page = seq.next_element()?.unwrap();

    //     Ok(Tower {
    //         name,
    //         badges,
    //         difficulty,
    //         length,
    //         tower_type,
    //         wiki_page,
    //     })
    // }
}

impl From<&Tower> for ShrinkTower {
    fn from(value: &Tower) -> Self {
        Self {
            name: value.name.to_owned(),
            badges: value.badges,
            difficulty: value.difficulty,
            length: value.length,
            tower_type: value.tower_type,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShrinkItem {
    pub name: String,
    pub badges: [u64; 2],
    pub tower_name: Option<String>,
}

impl From<&Item> for ShrinkItem {
    fn from(value: &Item) -> Self {
        Self {
            name: value.name.to_owned(),
            badges: value.badges.to_owned(),
            tower_name: value.tower_name.to_owned(),
        }
    }
}

impl Serialize for ShrinkItem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!(
            "{},{},{},{}",
            self.name,
            self.badges[0],
            self.badges[1],
            self.tower_name.as_ref().unwrap_or(&String::default())
        ))
    }
}

impl<'de> Deserialize<'de> for ShrinkItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ShrinkItemVisitor)
    }
}

struct ShrinkItemVisitor;
impl<'de> Visitor<'de> for ShrinkItemVisitor {
    type Value = ShrinkItem;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Expecting csv of (name,badge,badge,(optional)tower_name)")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let mut items = v.split(",");
        let name = items.next().unwrap().to_owned();
        let badges = [
            items.next().unwrap().parse::<u64>().unwrap(),
            items.next().unwrap().parse::<u64>().unwrap(),
        ];
        let tower = items.next().unwrap();
        let tower = if tower.is_empty() {
            None
        } else {
            Some(tower.to_owned())
        };

        Ok(ShrinkItem {
            name,
            badges,
            tower_name: tower,
        })
    }
}

/// All of these have an is_default check.
///
/// As much as i would like to put it on the parent object, i can't without probably writing [serde_derive::Serialize]
/// for more structs.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ShrinkExtendedArea {
    #[serde(skip_serializing_if = "is_default")]
    #[serde(rename = "r")]
    pub requirements: ShrunkAreaRequirements,
    #[serde(skip_serializing_if = "is_default")]
    #[serde(rename = "p")]
    pub parent: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    #[serde(rename = "t")]
    pub towers: Vec<ShrinkTower>,
    #[serde(skip_serializing_if = "is_default_or_none", rename = "i")]
    pub items: Option<Vec<ShrinkItem>>,
    #[serde(skip_serializing_if = "is_default_or_none", rename = "e")]
    pub event_area_name: Option<String>,
    #[serde(skip_serializing_if = "is_default_or_none", rename = "u")]
    pub until: Option<DateTime<FixedOffset>>,
}

impl From<&Box<ExtendedArea>> for ShrinkExtendedArea {
    fn from(value: &Box<ExtendedArea>) -> Self {
        Self {
            requirements: value.requirements.to_owned().into(),
            parent: value.parent.to_owned(),
            towers: value.towers.iter().map(ShrinkTower::from).collect_vec(),
            items: value
                .items
                .as_ref()
                .map(|v| v.iter().map(ShrinkItem::from).collect_vec()),
            event_area_name: value.event_area_name.to_owned(),
            until: value.until.to_owned(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq)]
pub struct ShrunkAreaRequirements {
    #[serde(rename = "d")]
    pub difficulties: ShrunkTowerDifficulties,
    #[serde(rename = "p")]
    pub points: u64,
    #[serde(rename = "a", skip_serializing_if = "HashMap::is_empty")]
    pub areas: HashMap<String, ShrunkAreaRequirements>,
}

impl From<AreaRequirements> for ShrunkAreaRequirements {
    fn from(value: AreaRequirements) -> Self {
        Self {
            difficulties: value.difficulties.into(),
            points: value.points,
            areas: value
                .areas
                .iter()
                .map(|c| (c.0.to_owned(), ShrunkAreaRequirements::from(c.1.to_owned())))
                .collect(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ShrunkTowerDifficulties {
    // pub easy: Option<u64>,
    pub medium: Option<u64>,
    pub hard: Option<u64>,
    pub difficult: Option<u64>,
    pub challenging: Option<u64>,
    pub intense: Option<u64>,
    pub remorseless: Option<u64>,
    // pub insane: Option<u64>,
    // pub extreme: Option<u64>,
    // pub terrifying: Option<u64>,
    // pub catastrophic: Option<u64>,
}

impl Serialize for ShrunkTowerDifficulties {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        fn assign_slice(slice: &mut [u8; 3], data: Option<u64>, index: u8) {
            if let Some(d) = data {
                if slice[1] == 0 {
                    slice[0] = index;
                    slice[1] = d as u8;
                    return;
                }
                slice[2] = d as u8;
            }
        }

        // format: [offset, first, second]
        let mut data = [0_u8, 0_u8, 0u8];
        assign_slice(&mut data, self.medium, 0);
        assign_slice(&mut data, self.hard, 1);
        assign_slice(&mut data, self.difficult, 2);
        assign_slice(&mut data, self.challenging, 3);
        assign_slice(&mut data, self.intense, 4);
        assign_slice(&mut data, self.remorseless, 5);

        let result = ((data[0] as u16) << 6) + ((data[1] as u16) << 3) + (data[2] as u16);
        println!("{:?} -> {:?}", data, result);
        serializer.serialize_u16(result)
    }
}
impl<'de> Deserialize<'de> for ShrunkTowerDifficulties {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u16(ShrunkTowerDifficultiesVisitor)
    }
}
struct ShrunkTowerDifficultiesVisitor;
impl<'de> Visitor<'de> for ShrunkTowerDifficultiesVisitor {
    type Value = ShrunkTowerDifficulties;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("A u16 number. The bytes are what we care about though.")
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let offset = v >> 6;
        let first = Some((v - offset >> 3) as u64);
        let second = v - offset - first.unwrap() as u16;
        let second = if second > 0 {
            Some(second as u64)
        } else {
            None
        };

        let mut res = ShrunkTowerDifficulties::default();
        match offset {
            0 => {
                res.medium = first;
                res.hard = second;
            }
            1 => {
                res.hard = first;
                res.difficult = second;
            }
            2 => {
                res.difficult = first;
                res.challenging = second;
            }
            3 => {
                res.challenging = first;
                res.intense = second;
            }
            4 => {
                res.intense = first;
                res.remorseless = second;
            }
            5 => {
                res.remorseless = first;
            }
            _ => {
                return Err(serde::de::Error::custom(format!(
                    "Invalid value for offset: {}",
                    offset,
                )));
            }
        }

        Ok(res)
    }
}

impl From<TowerDifficulties> for ShrunkTowerDifficulties {
    fn from(value: TowerDifficulties) -> Self {
        Self {
            // easy: value.easy,
            medium: value.medium,
            hard: value.hard,
            difficult: value.difficult,
            challenging: value.challenging,
            intense: value.intense,
            remorseless: value.remorseless,
            // insane: value.insane,
            // extreme: value.extreme,
            // terrifying: value.terrifying,
            // catastrophic: value.catastrophic,
        }
    }
}

/// Store information about badges which we can't categories elsewhere.
///
/// Data normally from [overwrite.jsonc]
#[derive(Debug, Clone)]
pub struct ShrinkOtherData {
    /// Our custom name of the badge
    pub name: String,
    /// the ids, `[old badge, new badge]`
    pub ids: [u64; 2],
}

impl Serialize for ShrinkOtherData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{},{},{}", self.name, self.ids[0], self.ids[1]))
    }
}
impl<'de> Deserialize<'de> for ShrinkOtherData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ShrinkOtherDataVisitor)
    }
}

struct ShrinkOtherDataVisitor;
impl<'de> Visitor<'de> for ShrinkOtherDataVisitor {
    type Value = ShrinkOtherData;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Expected a CSV of `name,badge,badge`")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let mut items = v.split(",");
        let name = items.next().unwrap();
        let badges = [
            items.next().unwrap().parse::<u64>().unwrap(),
            items.next().unwrap().parse::<u64>().unwrap(),
        ];
        Ok(ShrinkOtherData {
            name: name.to_owned(),
            ids: badges,
        })
    }
}

impl From<&OtherData> for ShrinkOtherData {
    fn from(value: &OtherData) -> Self {
        Self {
            name: value.name.to_owned(),
            ids: value.ids.to_owned(),
        }
    }
}

/// An enum to contain both the area info and the data info.
///
/// They are all a category even if something different sometimes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ShrinkCategory {
    Area(Box<ShrinkExtendedArea>),
    Other(Vec<ShrinkOtherData>),
}

impl From<&Category> for ShrinkCategory {
    fn from(value: &Category) -> Self {
        match value {
            Category::Area(extended_area) => {
                Self::Area(Box::new(ShrinkExtendedArea::from(extended_area)))
            }
            Category::Other(other_data) => {
                Self::Other(other_data.iter().map(ShrinkOtherData::from).collect_vec())
            }
        }
    }
}
