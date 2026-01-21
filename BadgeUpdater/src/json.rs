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
    Area(ExtendedArea),
    Other(Vec<OtherData>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jsonify {
    modify_date: DateTime<Utc>,
    categories: HashMap<String, Category>,
}

impl Jsonify {
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

                (area.name.to_owned(), Category::Area(category))
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

            (event.event_name.to_owned(), Category::Area(area))
        }));

        Self {
            modify_date: Utc::now(),
            categories,
        }
    }

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

    pub fn compare(&self, previous: &Self) -> Option<Vec<String>> {
        if self.modify_date == previous.modify_date {
            return None;
        }

        todo!()
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
