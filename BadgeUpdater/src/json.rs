use crate::definitions::{
    AreaInformation, AreaRequirements, BadgeOverwrite, EventInfo, EventItem, Length, TowerType,
    WikiTower,
};
use chrono::{DateTime, Utc};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    requirements: AreaRequirements,
    parent: Option<String>,
    towers: Vec<Tower>,
    items: Option<Vec<Item>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jsonify {
    modify_date: DateTime<Utc>,
    categories: HashMap<String, Category>,
}

impl Jsonify {
    pub fn parse(
        skip_ids: &[u64],
        towers: &[&WikiTower],
        areas: &[&AreaInformation],
        events: &[&EventInfo],
        event_items: &[&(EventItem, Option<WikiTower>)],
        mini: &[&WikiTower],
        adventure: &[&BadgeOverwrite],
    ) -> Self {
        let categories = areas
            .iter()
            .map(|area| {
                let category = Category::from(area.to_owned());
                (area.name.to_owned(), category)
            })
            .collect();

        Self {
            modify_date: Utc::now(),
            categories,
        }
    }

    pub fn parse_skipped(
        &mut self,
        overwrite: &[BadgeOverwrite],
        annoyed: &HashMap<String, String>,
    ) -> &mut Self {
        todo!();
        self
    }

    pub fn shrink(&self) -> Self {
        todo!()
    }

    pub fn stringify(&self) -> String {
        serde_json::to_string(self).expect("Failed to convert jsonify to string!")
    }

    pub fn compare(&self, previous: &Self) -> Self {
        todo!()
    }
}

impl From<&WikiTower> for Tower {
    fn from(tower: &WikiTower) -> Self {
        Tower {
            name: tower.page_name.to_owned(),
            badges: [tower.badge_id, 0],
            difficulty: tower.difficulty,
            length: tower.length,
            tower_type: tower.tower_type,
        }
    }
}

impl From<&AreaInformation> for Category {
    fn from(value: &AreaInformation) -> Self {
        Self {
            requirements: value.requirements.to_owned().unwrap_or_default(),
            parent: value.parent_area.to_owned(),
            towers: vec![],
            items: None,
        }
    }
}
