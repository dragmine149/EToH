use crate::definitions::{
    AreaInformation, AreaRequirements, BadgeOverwrite, EventInfo, EventItem, Length, TowerType,
    WikiTower,
};
use chrono::{DateTime, FixedOffset, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/*
 * {
 * 	 "modified": String,
 *   "categories": {
 * 	   "some-area": {
 *       "requirements": AreaRequirements,
 *       "parent": Option<String>,
 *       "towers": Tower[],
 * 		 // we can only have items if we it's an event area. As such, event area will be determined by items
 *       "items": Option<Tower[]>,
 *     }
 * 	 }
 * }
 *
 *
 */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tower {
    /// The wiki name of the tower. We could go badge, but wiki will be easier i hope.
    pub name: String,
    /// First badge is primary badge
    pub badges: [u64; 2],
    pub difficulty: f64,
    pub length: Length,
    pub tower_type: TowerType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub badges: [u64; 2],
    pub tower_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtendedArea {
    requirements: AreaRequirements,
    parent: Option<String>,
    towers: Vec<Tower>,
    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<Vec<Item>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event_area_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtherData {
    name: String,
    ids: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Category {
    Area(Box<ExtendedArea>),
    Other(Vec<OtherData>),
}

/// Store information about everything we've been collecting.
/// Also allows for the data to be serialized/deserialized to and from json.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Jsonify {
    modify_date: DateTime<Utc>,
    categories: HashMap<String, Category>,
}

impl Jsonify {
    /// Parse all the information we've been gathering and store it in a massive struct.
    pub fn parse(
        towers: &[&WikiTower],
        areas: &[&AreaInformation],
        events: &[&EventInfo],
        all_items: &[&(EventItem, Option<WikiTower>)],
        mini: &[&WikiTower],
        adventure: &[&BadgeOverwrite],
    ) -> Self {
        let mut categories = areas
            .iter()
            .map(|area| {
                let mut category = ExtendedArea::from(area.to_owned());
                category.towers = towers
                    .iter()
                    .filter(|t| t.area == area.name)
                    .chain(mini.iter().filter(|m| m.area == area.name))
                    .map(Tower::from)
                    .collect_vec();

                (area.name.to_owned(), Category::Area(Box::new(category)))
            })
            .collect::<HashMap<String, Category>>();

        categories.extend(adventure.iter().map(|a| {
            (
                a.category.to_owned(),
                Category::Other(vec![OtherData::from(a)]),
            )
        }));
        categories.extend(events.iter().map(|event| {
            // println!(
            //     "{}///\n{:#?}\n",
            //     event.event_name,
            //     all_items
            //         .iter()
            //         .filter(|(item, _)| item.event_name == event.event_name)
            // );

            let area = ExtendedArea {
                event_area_name: Some(event.area_name.to_owned()),
                items: Some(
                    all_items
                        .iter()
                        .filter(|(item, _)| item.event_name == event.event_name)
                        .map(|(item, _)| Item::from(item))
                        .collect_vec(),
                ),
                towers: all_items
                    .iter()
                    .filter(|t| t.1.is_some())
                    .map(|t| t.1.as_ref().unwrap().to_owned())
                    .filter(|t| t.area == event.area_name)
                    .map(Tower::from)
                    .collect_vec(),
                until: event.until,
                ..Default::default()
            };

            (event.event_name.to_owned(), Category::Area(Box::new(area)))
        }));

        Self {
            modify_date: Utc::now(),
            categories,
        }
    }

    /// Parse the skipped badges, this is done separately because... just because.
    pub fn parse_skipped(&mut self, overwrite: &[BadgeOverwrite]) -> &mut Self {
        for badge in overwrite {
            let mut badge_ids = vec![badge.badge_id];
            badge.alt_ids.iter().for_each(|id| badge_ids.push(*id));

            let cat = self.categories.get_mut(&badge.category);
            if let Some(category) = cat {
                match category {
                    Category::Area(_) => unreachable!("..."),
                    Category::Other(other_data) => {
                        other_data.push(OtherData {
                            name: badge.name.to_owned(),
                            ids: badge_ids,
                        });
                    }
                }
            } else {
                self.categories.insert(
                    badge.category.to_owned(),
                    Category::Other(vec![OtherData {
                        name: badge.name.to_owned(),
                        ids: badge_ids,
                    }]),
                );
            }
        }

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
                                self_items.iter().map(|i| (i.name.as_str(), i)).collect();
                            let prev_items_map: std::collections::HashMap<_, _> =
                                prev_items.iter().map(|i| (i.name.as_str(), i)).collect();

                            let self_item_names: std::collections::HashSet<_> =
                                self_items_map.keys().copied().collect();
                            let prev_item_names: std::collections::HashSet<_> =
                                prev_items_map.keys().copied().collect();

                            for removed in prev_item_names.difference(&self_item_names) {
                                changes.push(format!(
                                    "Removed item '{}' from event area '{}'",
                                    removed, common_key
                                ));
                            }

                            for added in self_item_names.difference(&prev_item_names) {
                                changes.push(format!(
                                    "Added item '{}' to event area '{}'",
                                    added, common_key
                                ));
                            }

                            // Check for modified items
                            for common_item_name in self_item_names.intersection(&prev_item_names) {
                                let self_item = self_items_map[common_item_name];
                                let prev_item = prev_items_map[common_item_name];

                                if self_item.tower_name != prev_item.tower_name {
                                    changes.push(format!(
                                        "Changed item '{}' tower association from {:?} to {:?} in event area '{}'",
                                        common_item_name, prev_item.tower_name, self_item.tower_name, common_key
                                    ));
                                }
                            }
                        }
                        (Some(_), None) => {
                            changes.push(format!("Added items to event area '{}'", common_key));
                        }
                        (None, Some(_)) => {
                            changes.push(format!("Removed items from event area '{}'", common_key));
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
                _ => {
                    changes.push(format!("Category '{}' type changed", common_key));
                }
            }
        }

        changes
    }
}

impl From<&&WikiTower> for Tower {
    fn from(tower: &&WikiTower) -> Self {
        Tower {
            name: tower.page_name.to_owned(),
            badges: [tower.badge_id, 0],
            difficulty: tower.difficulty,
            length: tower.length,
            tower_type: tower.tower_type,
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
        let mut ids = vec![value.badge_id];
        ids.extend(value.alt_ids.iter());

        Self {
            name: value.name.to_owned(),
            ids,
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
