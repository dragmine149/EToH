//! A list of pretty much every struct, impl From methods for use in the rest of the program.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use crate::{clean_badge_name, reqwest_client::RustError, wikitext::WikiText};

//=================================================
// Roblox Badge API results
//=================================================

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

/// Store information about the badge which we get from roblox.
///
/// Note: Roblox actually gives us way more data, we just drop it as we don't use it.
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Badge {
    /// The id of the badge and the identifier.
    pub id: u64,
    /// The name of the badge, used most often.
    pub name: String,
    /// The description of the badge as that can be semi-useful.
    pub description: Option<String>,
    // pub display_name: String,
    // pub display_description: Option<String>,
    // pub enabled: bool,
    // pub icon_image_id: u64,
    // pub display_icon_image_id: u64,
    // pub created: String,
    // pub updated: String,
    // pub statistics: BadgeStatistics,
    // pub awarding_universe: BadgeUniverse,
}

/// Extended Wrapper for [Badge]
///
/// Includes:
/// * multiple ids
/// * annoying references
/// Which allow for more use and way easier expandability.
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Badges {
    pub ids: [u64; 2],
    pub name: String,
    pub description: Option<String>,
    pub annoying: Option<String>,
}

impl Badges {
    /// Returns if the badge contains any of the ids in the badge.
    pub fn check_ids(&self, content: &str) -> bool {
        self.ids
            .iter()
            .any(|id| *id > 0 && content.contains(&id.to_string()))
    }
    /// Returns if any of the ids in the provided list are in this badge.
    pub fn check_all_ids(&self, ids: &[u64]) -> bool {
        self.ids.iter().any(|id| id > &0 && ids.contains(id))
    }

    /// Wrapper for [clean_badge_name] taking [Self::name] as an argument instead.
    pub fn clean_name(&self) -> String {
        clean_badge_name(&self.name)
    }

    /// Big `or` check to link a page title with this badge name.
    ///
    /// Has to check simple things, to more complex things like cleaning, redirects and annoyances.
    pub fn is_badge(&self, page: &WikiPageEntry) -> bool {
        // nice and easy check.
        self.name == page.title
        	// checks to see if we redirected to this page.
            || (page.redirected.is_some() && page.redirected.as_ref().unwrap() == &self.name)
            // checks to see if this was under the annoying category.
            || (self.annoying.is_some() && self.annoying.as_ref().unwrap() == &page.title)
            // checks to see if we used the clean name instead
            || (self.clean_name() == page.title)
            // checks to see if we used the clean name instead, and the page title requires cleaning.
            || (self.clean_name() == clean_badge_name(&page.title))
            // checks to see if we used the clean name instead, and we got redirected
            || (page.redirected.is_some() && page.redirected.as_ref().unwrap() == &self.clean_name())
    }
}
/// Store information about the overview of the data roblox gives us.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RobloxBadgeData {
    // pub previous_page_cursor: Option<String>,
    /// The key to getting the next set of badges
    pub next_page_cursor: Option<String>,
    /// All of the badges and their data which roblox provides.
    pub data: Vec<Badge>,
}

impl Default for RobloxBadgeData {
    fn default() -> Self {
        Self {
            // previous_page_cursor: None,
            next_page_cursor: Some(String::new()),
            data: vec![],
        }
    }
}
//=================================================
// Wiki API Results
//=================================================

/// The wrapper for anything returned by the wiki api.
#[derive(Debug, Deserialize)]
pub struct WikiAPI {
    /// The main result of the query.
    pub query: WikiQuery,
}

/// The results of the api reqwest to the wiki..
#[derive(Debug, Deserialize)]
pub struct WikiQuery {
    /// Is this a category request? If so, these are the members
    pub categorymembers: Option<Vec<WikiCategoryMember>>,
    /// Is this a search request? If so, these are the results
    pub search: Option<Vec<WikiSearchEntry>>,
    pub pages: Option<Vec<WikiPageEntry>>,
    pub redirects: Option<Vec<Redirection>>,
    pub normalized: Option<Vec<Redirection>>,
}

/// Information about the particular member from the search.
///
/// NOTE: There are more fields, just not used. So we drop them.
#[derive(Debug, Deserialize, Serialize)]
pub struct WikiCategoryMember {
    // pub pageid: u32,
    // pub ns: usize,
    /// The title of the page (we want this!)
    pub title: String,
}

/// Stores information about the individual search entry we received from the api.
#[derive(Debug, Serialize, Deserialize)]
pub struct WikiSearchEntry {
    /// The title of the page this entry points to.
    pub title: String,
}

/// Stores information about any redirects that might have happened.
///
/// Note: This works for redirects and normalizations.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Redirection {
    /// The page we used in the query
    pub from: String,
    /// Where it got redirected to.
    pub to: String,
}

/// Stores the main information related to a page.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WikiPageEntry {
    // pub page_id: u64,
    /// The title of the page.
    pub title: String,
    /// The list of revisions on that page
    pub revisions: Option<Vec<Revision>>,
    /// Is the page missing or not (aka doesn't exist)
    pub missing: Option<bool>,

    /// Custom additional field to show the redirection path provided by [Redirection], if any.
    /// The API does not return this.
    pub redirected: Option<String>,
}

impl WikiPageEntry {
    /// Shortcut for getting the content of the page as we have to go through revisions...
    pub fn get_content(&self) -> Option<&RevisionContent> {
        if let Some(revision) = &self.revisions
            && let Some(first) = revision.first()
            && let Some(main) = first.slots.get("main")
        {
            return Some(main);
        }

        None
    }
}

impl From<&WikiPageEntry> for WikiText {
    fn from(value: &WikiPageEntry) -> Self {
        let mut wt = WikiText::parse(&value.get_content().unwrap().content);
        wt.set_page_name(Some(value.title.to_owned()));
        wt
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Revision {
    pub slots: HashMap<String, RevisionContent>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RevisionContent {
    pub contentmodel: String,
    pub contentformat: String,
    pub content: String,
}

/// Wrapper for raw wikitext of the page.
///
/// Used when continuously redirecting to get the page.
#[derive(Debug, Default)]
pub struct PageDetails {
    /// The text of the page we just got.
    pub text: String,
    /// The name of the page after redirects.
    pub name: Option<String>,
}

//=================================================
// Collection of information from processing.
//=================================================

/// Everything the wikitext for a specific page can give us for a tower.
///
/// As much as we could also store the badge name, decided against that as page name gives us more accurate details.
#[derive(Debug, Default)]
pub struct WikiTower {
    /// The badge id related to this tower.
    pub badge_ids: [u64; 2],
    /// The name of the related page.
    pub page_name: String,
    /// The difficulty of the tower. (as `1?X.YZ`)
    pub difficulty: f64,
    /// The area this tower belongs to.
    pub area: String,
    /// The "average" length of the tower.
    pub length: Length,
    /// The type this tower is.
    pub tower_type: TowerType,
}

/// Information about the event.
#[derive(Debug, Clone)]
pub struct EventInfo {
    /// The name of the custom area where the event is taking place.
    pub area_name: String,
    /// The codename of the event, normally just the season and year.
    pub event_name: String,
    /// When the event will finish, if it's ongoing.
    pub until: Option<DateTime<FixedOffset>>,
}

/// Information about a specific item gotten from the event.
#[derive(Debug, Clone)]
pub struct EventItem {
    /// The name of the item received
    pub item_name: String,
    /// The name of the event the item was gained from
    pub event_name: String,
    /// The badges linking to the item. (as not every item has a tower)
    pub badges: [u64; 2],
    /// The tower (if any) that is required to be completed for this item.
    ///
    /// TODO: NOTE: This does not yet include multi-tower items. It's a todo at some point.
    pub tower_name: Option<String>,
}

/// A list of every single difficulty and how many towers of that difficulty area required to be completed before the area is unlocked.
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
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

impl TowerDifficulties {
    /// Convert the difficulty provided to the corresponding member.
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
            not_a_valid_difficulty => {
                eprintln!("Not a valid difficulty! {:?}", not_a_valid_difficulty);
            }
        }
    }
}

/// Stores information about the requirements for getting to the area.
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct AreaRequirements {
    /// The difficulties required.
    pub difficulties: TowerDifficulties,
    /// How many towers
    pub points: u64,
    /// Any specific areas that require for this area to be unlocked, and the requirements in that area.
    pub areas: HashMap<String, AreaRequirements>,
}

/// Store basic information about a certain area.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AreaInformation {
    /// The name of the area
    #[serde(rename = "n")]
    pub name: String,
    #[serde(rename = "r")]
    /// The requirements to access this area. Is optional as some areas don't need anything.
    pub requirements: Option<AreaRequirements>,
    /// If this area is a sub area, and if so what parent area does it come under.
    ///
    /// This is better and easier than trying to keep a list of sub areas.
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

/// The type of the tower.
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

/// How long does the tower take to complete for an average player in one-shot (no major failures)
///
/// True lengths aren't used as otherwise all towers would be like Cakewalk length.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
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
    /// INHUMANELY (very very rare and kinda like way too long...)
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
//=================================================
// Information to do with stuff hapapened during
// processing.
//=================================================

/// Extra information about what happened whilst we were trying to process the wikitext.
#[derive(Debug)]
#[allow(
    dead_code,
    reason = "i use these for debugging, ik i don't use them rust because we don't need to use them and panic is just not worth it"
)]
pub enum ProcessError {
    /// Was this a network reqwest error (or related)
    Reqwest(RustError),
    /// Or was it an error in our code, or the page data itself?
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
impl From<serde_json::Error> for ProcessError {
    fn from(value: serde_json::Error) -> Self {
        Self::Process(value.to_string())
    }
}
impl From<url::ParseError> for ProcessError {
    fn from(value: url::ParseError) -> Self {
        Self::Process(value.to_string())
    }
}

/// Struct used for containing error details
#[derive(Debug)]
#[allow(dead_code, reason = "i use these for debugging")]
pub struct ErrorDetails(
    /// Information, aka reason of why the error happened.
    pub ProcessError,
    /// The badge we were processing at the time.
    pub Badges,
);
/// Struct used for when things go correct.
pub struct OkDetails(
    /// The wikitext, aka data returned
    pub WikiText,
    /// The full details of the badge to keep them together.
    pub Badges,
);

impl Debug for OkDetails {
    /// Custom debug formatter function to reduce output.
    ///
    /// Raw Wikitext is kinda big, some pages being > 100kb, hence we just ignore that so we aren't filling up the debug file with too much waste.
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

//=================================================
// Data in a more suitable format for jsonification.
//=================================================

/// Any badges which we can't find we have to overwrite ourselves. This keeps track of that.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BadgeOverwrite {
    /// The badge ids related in format of `[old_game, new_gaame]`.
    ///
    /// If we have to overwrite we'll probably have to overwrite both.
    pub badge_ids: [u64; 2],
    /// The category the badge belongs to.
    pub category: String,
    /// Our custom name, we could link it to the actual badge, but that requires rework of some other system.
    ///
    /// And besides, this allows us to provide a mini reminder to the user.
    pub name: String,
}

// Serialiser and deseriliser for BadgeOverwrite written by GPT-5 mini
impl<'de> Deserialize<'de> for BadgeOverwrite {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // We expect a map entry like "123": [ "Category", "Name", "456" ]
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
        // serialize as an object with single string key -> array        use serde_json::Value as Jv;
        let arr = vec![
            Value::String(self.category.clone()),
            Value::String(self.name.clone()),
            Value::String(self.badge_ids[1].to_string()),
        ];

        let mut map = serde_json::map::Map::new();
        map.insert(self.badge_ids[0].to_string(), Value::Array(arr));
        Value::Object(map).serialize(serializer)
    }
}

/// parse the array provided by serde_json `overwrite.jsonc` for storage.
fn parse_array(badge_id: u64, arr: Vec<Value>) -> Result<BadgeOverwrite, String> {
    if arr.len() < 2 {
        return Err("array must contain at least category and name".into());
    }
    // println!("arr: {:?}", arr);

    // convert each individual values.
    let category = match &arr[0] {
        Value::String(s) => s.clone(),
        _ => return Err("category must be a string".into()),
    };
    let name = match &arr[1] {
        Value::String(s) => s.clone(),
        _ => return Err("name must be a string".into()),
    };
    let old_id = if arr.len() > 2 {
        match &arr[2] {
            Value::Number(number) => number
                .as_u64()
                .ok_or("Failed to convert third value into number")?,
            Value::String(num) => num
                .parse::<u64>()
                .map_err(|e| format!("old_id must be a number {}", e))?,
            _ => return Err("old_id must be a number or a string parsable to a number".into()),
        }
    } else {
        0
    };

    Ok(BadgeOverwrite {
        badge_ids: [old_id, badge_id],
        category,
        name,
    })
}

/// Helper functions to convert a whole map <String, Array> -> `Vec<BadgeOverwrite>`
pub fn badges_from_map_value(v: &Value) -> Result<Vec<BadgeOverwrite>, Box<dyn std::error::Error>> {
    match v {
        Value::Object(map) => {
            let mut out = Vec::with_capacity(map.len());
            for (k, val) in map.iter() {
                let badge_id = k
                    .parse::<u64>()
                    .map_err(|e| format!("map key must be a string representable as u64 {}", e))?;
                let arr = match val {
                    Value::Array(a) => a.clone(),
                    _ => return Err("map value must be an array".into()),
                };
                out.push(parse_array(badge_id, arr)?);
            }
            Ok(out)
        }
        _ => Err("expected top-level object/map".into()),
    }
}

/// Custom tower object we use when turning the data into json.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tower {
    /// The wiki name of the tower. We could go badge, but wiki will be easier i hope.
    pub name: String,
    /// Badges linked to the tower `[Old Badge, New Badge]`
    pub badges: [u64; 2],
    /// The difficutlty of the tower.
    pub difficulty: f64,
    /// How long it takes to complete the tower.
    pub length: Length,
    /// The type the tower is.
    pub tower_type: TowerType,
    /// A link to the wiki, used if the page != name.
    pub wiki_page: Option<String>,
}

/// Custom item object we use when turning the data into json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    /// The name of the item.
    pub name: String,
    /// Badges linked to the item `[old badge, new badge]`
    pub badges: [u64; 2],
    /// The name of the related tower... if we have a related tower.
    pub tower_name: Option<String>,
}

/// [AreaInformation] just with some additional information
///
/// Name is not included as thats part of the key, this is the value.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtendedArea {
    /// Requirements for the area
    pub requirements: AreaRequirements,
    /// Is this area a sub-area, and if so what area is the parent.
    pub parent: Option<String>,
    /// The towers contained in this sub area.
    pub towers: Vec<Tower>,
    /// A list of items in this sub area. Also confirms it's an event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<Item>>,
    /// A list of other badges related, but aren't towers of items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other: Option<Vec<OtherData>>,
    /// The name of the event if not the sub area name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_name: Option<String>,
    /// If the event is ongoing, when will it finish.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<DateTime<FixedOffset>>,
}

/// Store information about badges which we can't categories elsewhere.
///
/// Data normally from [overwrite.jsonc]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtherData {
    /// Our custom name of the badge
    pub name: String,
    /// the ids, `[old badge, new badge]`
    pub ids: [u64; 2],
}

/// An enum to contain both the area info and the data info.
///
/// They are all a category even if something different sometimes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Category {
    Area(Box<ExtendedArea>),
    Other(Vec<OtherData>),
}

impl From<&&WikiTower> for Tower {
    fn from(tower: &&WikiTower) -> Self {
        Tower {
            // The page name is most likely the tower name. Yes this does mean we don't have the badge name,
            // but eh that wasn't very useful to begin with.
            // Other badges are different though...
            name: tower.page_name.to_owned(),
            badges: tower.badge_ids,
            difficulty: tower.difficulty,
            length: tower.length,
            tower_type: tower.tower_type,
            wiki_page: Some(tower.page_name.to_owned()),
        }
    }
}
impl From<&WikiTower> for Tower {
    fn from(value: &WikiTower) -> Self {
        Self::from(&value)
    }
}

impl From<&AreaInformation> for ExtendedArea {
    fn from(value: &AreaInformation) -> Self {
        Self {
            requirements: value.requirements.to_owned().unwrap_or_default(),
            parent: value.parent_area.to_owned(),
            ..Default::default()
        }
    }
}

impl From<&&BadgeOverwrite> for OtherData {
    fn from(value: &&BadgeOverwrite) -> Self {
        Self {
            name: value.name.to_owned(),
            ids: value.badge_ids.to_owned(),
        }
    }
}

impl From<&EventItem> for Item {
    fn from(value: &EventItem) -> Self {
        Self {
            name: value.item_name.to_owned(),
            badges: value.badges,
            tower_name: value.tower_name.to_owned(),
        }
    }
}

//=================================================
