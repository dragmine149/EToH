use std::{collections::HashMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    definitions::{AreaInformation, AreaMap, Tower},
    parse_wikitext::WIkiTower,
};

impl From<&WIkiTower> for Tower {
    fn from(value: &WIkiTower) -> Self {
        Self {
            name: value.tower_name.clone(),
            difficulty: value.difficulty,
            badges: vec![],
            tower_type: Some(value.tower_type),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TowerJSON {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub areas: HashMap<String, Vec<AreaInformation>>,
}

impl TowerJSON {
    pub fn new() -> Self {
        TowerJSON {
            schema: "tds.json".to_string(),
            ..Default::default()
        }
    }
    pub fn load_map(&mut self, map: &AreaMap) {
        map.key_loop().for_each(|k| {
            self.areas.insert(k.to_string(), vec![]);
        });
        self.areas.insert("other".to_string(), vec![]);
    }

    // pub fn make_areas(&mut self, map: &AreaMap) {
    //     for main in map.areas.iter() {
    //         for a in main.1 {
    //             let area_info = AreaInformation {
    //                 name: a.0.to_owned(),
    //                 ..Default::default()
    //             };
    //             if !self.areas.contains_key(main.0) {
    //                 self.areas.insert(main.0.to_owned(), vec![]);
    //             }
    //             self.areas.get_mut(main.0).unwrap().push(area_info);
    //         }
    //     }
    // }

    pub fn add_tower(&mut self, tower: WIkiTower, badge: u64, map: &AreaMap) {
        let area = map.get_area(&tower.location);
        let tower_list: &mut Vec<Tower> = self
            .areas
            .get_mut(&area)
            .unwrap()
            .iter_mut()
            .find(|v| v.name == tower.location)
            .unwrap()
            .towers
            .as_mut();
        if let Some(stored) = tower_list.iter_mut().find(|v| v.name == tower.tower_name) {
            stored.badges.push(badge);
            return;
        }

        let mut json_tower = Tower::from(&tower);
        json_tower.badges.push(badge);
        tower_list.push(json_tower);
    }

    // pub fn insert_tower(&mut self, tower: WIkiTower, name: &str, badge: u64, map: &AreaMap) {
    //     let mut json_tower = Tower::from(&tower);
    //     json_tower.badges.push(badge);
    //     json_tower.name = name.to_owned();

    //     let area = map.get_area(&tower.location);
    //     println!("Area: {:?}", area);
    //     let towers: &mut Vec<Tower> = self
    //         .areas
    //         .get_mut(&area)
    //         .unwrap()
    //         .iter_mut()
    //         .find(|v| v.name == tower.location)
    //         .unwrap()
    //         .towers
    //         .as_mut();
    //     if let Some(stored_tower) = towers
    //         .iter_mut()
    //         .find(|v| v.name.trim().to_lowercase() == tower.tower_name.trim().to_lowercase())
    //     {
    //         stored_tower.badges.push(badge);
    //     } else {
    //         towers.push(json_tower);
    //     }
    // }

    pub fn has_area(&self, area: &String, map: &AreaMap) -> bool {
        self.areas
            .get(&map.get_area(area))
            .unwrap()
            .iter()
            .any(|v| v.name == *area)
    }
    pub fn add_area(&mut self, area: AreaInformation, map: &AreaMap) {
        self.areas
            .get_mut(&map.get_area(&area.name))
            .unwrap()
            .push(area);
    }

    // pub fn add_tower_badge(&mut self, name: &str, badge: u64, main_area: &str, tower_area: &str) {
    //     // self.towers.get_mut(name).unwrap().badges.push(badge);
    //     self.areas
    //         .get_mut(main_area)
    //         .unwrap()
    //         .iter_mut()
    //         .find(|v| v.name == tower_area)
    //         .unwrap()
    //         .towers
    //         .iter_mut()
    //         .find(|v| v.name == name)
    //         .unwrap()
    //         .badges
    //         .push(badge);
    // }

    pub fn write_to_file(&mut self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        // no point including it as its basically just temp dead weight.
        // if self.areas.get("other").unwrap_or(&vec![]).len() == 0 {
        //     self.areas.remove("other");
        // }
        // Sort each area's list
        self.areas
            .iter_mut()
            .for_each(|a| a.1.iter_mut().for_each(|s| s.sort()));

        // Build an ordered "areas" object with keys in the desired order:
        // ("permanent", "temporary", "other"), followed by any remaining keys
        // in alphabetical order.
        let mut areas_map = serde_json::Map::new();

        let preferred_order = ["permanent", "temporary", "other"];
        for &k in preferred_order.iter() {
            if let Some(v) = self.areas.get(k) {
                let value = serde_json::to_value(v)?;
                areas_map.insert(k.to_string(), value);
            }
        }

        // Insert remaining keys sorted alphabetically
        let mut remaining_keys: Vec<&String> = self
            .areas
            .keys()
            .filter(|k| !preferred_order.contains(&k.as_str()))
            .collect();
        remaining_keys.sort();
        for k in remaining_keys {
            if let Some(v) = self.areas.get(k) {
                let value = serde_json::to_value(v)?;
                areas_map.insert(k.to_string(), value);
            }
        }

        // Build the final root object with the schema and ordered areas
        let mut root = serde_json::Map::new();
        root.insert(
            "$schema".to_string(),
            serde_json::Value::String(self.schema.clone()),
        );
        root.insert("areas".to_string(), serde_json::Value::Object(areas_map));

        let data = serde_json::to_string(&serde_json::Value::Object(root))?;
        Ok(fs::write(path, data)?)
    }
}
