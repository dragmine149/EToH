//! The final module, convert everything to a json de/serializable struct

use crate::{
    definitions::{
        AreaInformation, BadgeOverwrite, Category, EventInfo, EventItem, ExtendedArea, Item,
        OtherData, Tower, WikiTower,
    },
    shrink_json_defs::ShrinkJson,
};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{
    Deserialize, Serialize,
    de::Visitor,
    ser::{SerializeMap, SerializeStruct},
};
use std::{collections::HashMap, fs};

/// Store information about everything we've been collecting.
/// Also allows for the data to be serialized/deserialized to and from json.
#[derive(Debug, Clone, Default)]
pub struct Jsonify {
    /// The data of last modification.
    pub modify_date: DateTime<Utc>,
    /// The actual data we store.
    pub categories: HashMap<String, Category>,
}

impl Jsonify {
    /// Parse all the information we've been gathering and store it in a massive struct.
    ///
    /// # Arguments
    /// * towers: List of towers which have passed
    /// * areas: List of every single area we've gotten from the towers.
    /// * events: List of every single event.
    /// * all_items: List of every single item and a potential tower relationship
    /// * mini: List of all mini towers which didn't get added in the towers list.
    /// * adventure: List of badge overwrites classififed as "adventure", aka those gotten from description via `[crate::hard_coded::area_from_description]`
    pub fn parse(
        towers: &[&WikiTower],
        areas: &[&AreaInformation],
        events: &[&EventInfo],
        all_items: &[(EventItem, Option<WikiTower>)],
        mini: &[&WikiTower],
        hard: &[BadgeOverwrite],
    ) -> Self {
        let mut categories = HashMap::<String, Category>::new();

        // pioritise towers first then everything else.
        towers.iter().chain(mini.iter()).for_each(|tower| {
            let (area_name, event_area) = match events
                .iter()
                .find(|e| e.area_name == tower.area || e.event_name == tower.area)
            {
                Some(ei) => (ei.area_name.to_owned(), Some(ei.event_name.to_owned())),
                None => (tower.area.to_owned(), None),
            };
            // log::debug!("old: {}, new: {}", tower.area, area_name);

            match categories.get_mut(&area_name) {
                Some(area) => match area {
                    Category::Area(extended_area) => extended_area.towers.push(Tower::from(tower)),
                    Category::Other(_) => {
                        unreachable!("Area from towers should not be of type other.")
                    }
                },
                None => {
                    let area = ExtendedArea {
                        towers: vec![Tower::from(tower)],
                        event_name: event_area,
                        ..Default::default()
                    };
                    categories.insert(area_name, Category::Area(Box::new(area)));
                }
            }
        });
        areas.iter().for_each(|area| {
            match categories.get_mut(&area.name) {
                Some(area_info) => match area_info {
                    Category::Area(extended_area) => {
                        extended_area.requirements =
                            area.requirements.to_owned().unwrap_or_default();
                        extended_area.parent = area.parent_area.to_owned();
                    }
                    Category::Other(_) => unreachable!("Areas should not be of type other!"),
                },
                None => {
                    // we'll let other part of the code insert them if we need to.

                    // categories.insert(
                    //     area.name.clone(),
                    //     Category::Area(Box::new(ExtendedArea::from(*area))),
                    // );
                }
            };
        });
        hard.iter().for_each(|a| {
            match categories.get_mut(&a.category) {
                Some(cat) => match cat {
                    Category::Other(other) => other.push(OtherData::from(&a)),
                    _ => unreachable!("Adventure should never be anything but other."),
                },
                None => {
                    categories.insert(
                        a.category.to_owned(),
                        Category::Other(vec![OtherData::from(&a)]),
                    );
                }
            };
        });
        events
            .iter()
            .for_each(|event| match categories.get_mut(&event.event_name) {
                Some(event_info) => match event_info {
                    Category::Area(extended_area) => {
                        extended_area.event_name = Some(event.event_name.clone());
                        extended_area.items = Some(
                            all_items
                                .iter()
                                .filter(|(item, _)| item.event_name == event.event_name)
                                .map(|(item, _)| Item::from(item))
                                .collect_vec(),
                        );
                        all_items
                            .iter()
                            .filter(|t| t.1.is_some())
                            .map(|t| t.1.as_ref().unwrap().to_owned())
                            .filter(|t| t.area == event.area_name)
                            .map(Tower::from)
                            .for_each(|t| {
                                if !extended_area.towers.contains(&t) {
                                    extended_area.towers.push(t);
                                }
                            });
                        extended_area.until = event.until;
                    }
                    Category::Other(_) => unreachable!("Event info shouldn't be of type other."),
                },
                None => {
                    let items = all_items
                        .iter()
                        .filter(|(item, _)| item.event_name == event.event_name)
                        .map(|(item, _)| Item::from(item))
                        .collect_vec();
                    let towers = all_items
                        .iter()
                        .filter(|t| t.1.is_some())
                        .map(|t| t.1.as_ref().unwrap().to_owned())
                        .filter(|t| t.area == event.area_name)
                        .map(Tower::from)
                        .collect_vec();

                    if items.is_empty() && towers.is_empty() {
                        // don't add it if we have nothing of worth to add.
                        return;
                    }

                    categories.insert(
                        event.area_name.clone(),
                        Category::Area(Box::new(ExtendedArea {
                            event_name: Some(event.event_name.to_owned()),
                            items: Some(items),
                            towers,
                            until: event.until,
                            ..Default::default()
                        })),
                    );
                }
            });

        Self {
            modify_date: Utc::now(),
            categories,
        }
    }

    /// Parse the skipped badges, this is done separately because... just because.
    ///
    /// # Arguments
    /// * &mut self: Link to itself for writing
    /// * overwrite: List of badges which we manually assign.
    pub fn parse_skipped(&mut self, overwrite: &[BadgeOverwrite]) -> &mut Self {
        for badge in overwrite {
            let cat = self.categories.get_mut(&badge.category);
            let data = OtherData {
                name: badge.name.to_owned(),
                ids: badge.badge_ids,
            };

            if let Some(category) = cat {
                match category {
                    // Category::Area(_) => unreachable!(
                    //     "... `{}` is of type area. id: ({:?})",
                    //     badge.category, badge.badge_ids
                    // ),
                    Category::Area(area) => {
                        match &mut area.other {
                            Some(other_data) => other_data.push(data),
                            None => area.other = Some(vec![data]),
                        };
                    }
                    Category::Other(other_data) => {
                        other_data.push(data);
                    } // _ => {}
                }
            } else {
                self.categories
                    .insert(badge.category.to_owned(), Category::Other(vec![data]));
            }
        }

        self
    }

    /// Clean up the hashmap by removing any category with no badge data at all.
    ///
    /// Even if it's an event category, if it has no badges we don't care.
    pub fn clean_up(&mut self) -> &mut Self {
        // return self;

        self.categories.retain(|_, cat| match cat {
            Category::Area(extended_area) => {
                if let Some(items) = &extended_area.items {
                    return !(items.is_empty() && extended_area.towers.is_empty());
                }
                !extended_area.towers.is_empty()
            }
            Category::Other(other_data) => !other_data.is_empty(),
        });

        self
    }

    /// Compare the current data structure to the old one.
    /// If changes have occurred, then list them.
    pub fn compare(&self, previous: &Self) -> Vec<String> {
        if self.modify_date == previous.modify_date {
            return vec![];
        }

        let mut changes = Vec::new();

        // Compare categories
        let self_keys: std::collections::HashSet<_> = self.categories.keys().collect();
        let prev_keys: std::collections::HashSet<_> = previous.categories.keys().collect();

        // Check for removed categories
        for removed in prev_keys.difference(&self_keys) {
            changes.push(format!("Removed category: {}", removed));
        }

        // Check for added categories
        for added in self_keys.difference(&prev_keys) {
            changes.push(format!("Added category: {}", added));
        }

        // Check for modified categories
        for common_key in self_keys.intersection(&prev_keys) {
            match (
                &self.categories[*common_key],
                &previous.categories[*common_key],
            ) {
                (Category::Area(self_area), Category::Area(prev_area)) => {
                    // Compare towers
                    let self_towers_map: std::collections::HashMap<_, _> = self_area
                        .towers
                        .iter()
                        .map(|t| (t.name.as_str(), t))
                        .collect();
                    let prev_towers_map: std::collections::HashMap<_, _> = prev_area
                        .towers
                        .iter()
                        .map(|t| (t.name.as_str(), t))
                        .collect();

                    let self_tower_names: std::collections::HashSet<_> =
                        self_towers_map.keys().copied().collect();
                    let prev_tower_names: std::collections::HashSet<_> =
                        prev_towers_map.keys().copied().collect();

                    for removed in prev_tower_names.difference(&self_tower_names) {
                        changes.push(format!(
                            "Removed tower '{}' from area '{}'",
                            removed, common_key
                        ));
                    }

                    for added in self_tower_names.difference(&prev_tower_names) {
                        changes.push(format!("Added tower '{}' to area '{}'", added, common_key));
                    }

                    // Check for modified towers
                    for common_tower_name in self_tower_names.intersection(&prev_tower_names) {
                        let self_tower = self_towers_map[common_tower_name];
                        let prev_tower = prev_towers_map[common_tower_name];

                        if self_tower.difficulty != prev_tower.difficulty {
                            changes.push(format!(
                                "Changed tower '{}' difficulty from {} to {} in area '{}'",
                                common_tower_name,
                                prev_tower.difficulty,
                                self_tower.difficulty,
                                common_key
                            ));
                        }

                        if self_tower.length != prev_tower.length {
                            changes.push(format!(
                                "Changed tower '{}' length from {:?} to {:?} in area '{}'",
                                common_tower_name, prev_tower.length, self_tower.length, common_key
                            ));
                        }

                        if self_tower.tower_type != prev_tower.tower_type {
                            changes.push(format!(
                                "Changed tower '{}' type from {:?} to {:?} in area '{}'",
                                common_tower_name,
                                prev_tower.tower_type,
                                self_tower.tower_type,
                                common_key
                            ));
                        }
                    }

                    // Compare items (for event areas)
                    match (&self_area.items, &prev_area.items) {
                        (Some(self_items), Some(prev_items)) => {
                            let self_items_map: std::collections::HashMap<_, _> =
                                self_items.iter().map(|i| (i.badges.clone(), i)).collect();
                            let prev_items_map: std::collections::HashMap<_, _> =
                                prev_items.iter().map(|i| (i.badges.clone(), i)).collect();

                            let self_item_badges: std::collections::HashSet<_> =
                                self_items_map.keys().cloned().collect();
                            let prev_item_badges: std::collections::HashSet<_> =
                                prev_items_map.keys().cloned().collect();

                            for removed in prev_item_badges.difference(&self_item_badges) {
                                let prev_item = prev_items_map[removed];
                                changes.push(format!(
                                    "Removed item `{} ({:?})` from event area `{}`",
                                    prev_item.name, removed, common_key
                                ));
                            }

                            for added in self_item_badges.difference(&prev_item_badges) {
                                let self_item = self_items_map[added];
                                changes.push(format!(
                                    "Added item `{} ({:?})` to event area `{}`",
                                    self_item.name, added, common_key
                                ));
                            }

                            // Check for modified items
                            for common_item_badges in
                                self_item_badges.intersection(&prev_item_badges)
                            {
                                let self_item = self_items_map[common_item_badges];
                                let prev_item = prev_items_map[common_item_badges];

                                if self_item.name != prev_item.name {
                                    changes.push(format!(
                                        "Renamed item from `{} ({:?})` to `{} ({:?})` in event area `{}`",
                                        prev_item.name, common_item_badges, self_item.name, common_item_badges, common_key
                                    ));
                                }

                                if self_item.tower_name != prev_item.tower_name {
                                    changes.push(format!(
                                        "Changed item `{} ({:?})` tower association from {:?} to {:?} in event area `{}`",
                                        self_item.name, common_item_badges, prev_item.tower_name, self_item.tower_name, common_key
                                    ));
                                }
                            }
                        }
                        (Some(items), None) => {
                            changes.push(format!(
                                "Added items to event area `{}`. Names: {:?}",
                                common_key,
                                items.iter().map(|i| &i.name).collect_vec()
                            ));
                        }
                        (None, Some(items)) => {
                            changes.push(format!(
                                "Removed items from event area `{}`. Names: {:?}",
                                common_key,
                                items.iter().map(|i| &i.name).collect_vec()
                            ));
                        }
                        (None, None) => {}
                    }
                }
                (Category::Other(self_other), Category::Other(prev_other)) => {
                    let self_names: std::collections::HashSet<_> =
                        self_other.iter().map(|o| o.name.as_str()).collect();
                    let prev_names: std::collections::HashSet<_> =
                        prev_other.iter().map(|o| o.name.as_str()).collect();

                    for removed in prev_names.difference(&self_names) {
                        changes.push(format!(
                            "Removed other data '{}' from category '{}'",
                            removed, common_key
                        ));
                    }

                    for added in self_names.difference(&prev_names) {
                        changes.push(format!(
                            "Added other data '{}' to category '{}'",
                            added, common_key
                        ));
                    }
                }
                (to, from) => {
                    changes.push(format!(
                        "Category '{}' type changed from {} to {}",
                        common_key,
                        match from {
                            Category::Area(_) => "Area",
                            Category::Other(_) => "Other",
                        },
                        match to {
                            Category::Area(_) => "Area",
                            Category::Other(_) => "Other",
                        },
                    ));
                }
            }
        }

        changes.sort();
        changes.iter_mut().for_each(|i| *i = format!("- {}", i));

        changes
    }

    pub fn shrinkfy(self) -> ShrinkJson {
        ShrinkJson::from(self)
    }
}

impl Serialize for Jsonify {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Jsonify", 2)?;
        s.serialize_field("modify_date", &(self.modify_date.timestamp()))?;
        s.serialize_field("categories", &SortedHashMap(self.categories.to_owned()))?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for Jsonify {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct("Jsonify", &["modify_date", "categories"], JsonifyVisitor)
    }
}

struct JsonifyVisitor;
impl<'de> Visitor<'de> for JsonifyVisitor {
    type Value = Jsonify;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct Jsonify")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut modify_date: Option<DateTime<Utc>> = None;
        let mut categories: Option<HashMap<String, Category>> = None;

        while let Some(key) = map.next_key()? {
            match key {
                "modify_date" => {
                    let timestamp: i64 = map.next_value()?;
                    modify_date = Some(
                        DateTime::<Utc>::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now),
                    );
                }
                "categories" => {
                    categories = Some(map.next_value()?);
                }
                _ => {
                    let _: serde::de::IgnoredAny = map.next_value()?;
                }
            }
        }

        Ok(Jsonify {
            modify_date: modify_date.unwrap_or_else(Utc::now),
            categories: categories.unwrap_or_default(),
        })
    }
}

/// Helper function for reading a `jsonc` file as `serde_json` doesn't work by default.
///
/// This is just an extension of [fs::read_to_string] but removes any lines starting with `//`
pub fn read_jsonc(path: &str) -> String {
    fs::read_to_string(path)
        .unwrap_or("{}".into())
        .lines()
        .filter(|line| !line.trim_start().contains("//"))
        .join("\n")
}

/// Source: <https://www.codestudy.net/blog/how-to-sort-hashmap-keys-when-serializing-with-serde/>
#[derive(Debug, Clone)]
pub struct SortedHashMap<K, V>(pub HashMap<K, V>);
impl<K, V> Serialize for SortedHashMap<K, V>
where
    K: Serialize + Ord,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Collect HashMap entries into a Vec for sorting
        let mut entries: Vec<_> = self.0.iter().collect();

        // Sort entries by key (ascending order by default)
        entries.sort_by_key(|&(key, _)| key);

        // Serialize as a map: start the map, write sorted entries, end the map
        let mut map = serializer.serialize_map(Some(entries.len()))?;
        for (key, value) in entries {
            map.serialize_entry(key, value)?; // Serialize key-value pair
        }
        map.end()
    }
}

impl<'de, K, V> Deserialize<'de> for SortedHashMap<K, V>
where
    K: Deserialize<'de> + std::cmp::Eq + std::hash::Hash,
    V: Deserialize<'de> + std::hash::Hash,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map = HashMap::deserialize(deserializer)?;
        Ok(SortedHashMap(map))
    }
}
