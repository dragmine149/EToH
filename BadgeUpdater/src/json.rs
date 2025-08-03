use std::{collections::HashMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    definitions::{AreaInformation, AreaMap, Tower},
    parse_wikitext::WIkiTower,
};

impl From<&WIkiTower> for Tower {
    fn from(value: &WIkiTower) -> Self {
        Self {
            name: String::default(),
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
    #[serde(skip)]
    towers: HashMap<String, Tower>,
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
        self.areas.insert("Uncategorised".to_string(), vec![]);
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

    pub fn insert_tower(&mut self, tower: WIkiTower, name: &str, badge: u64, map: &AreaMap) {
        let mut json_tower = Tower::from(&tower);
        json_tower.badges.push(badge);
        json_tower.name = name.to_owned();

        self.towers.insert(name.to_owned(), json_tower.to_owned());
        let area = map.get_area(&tower.location);
        println!("Area: {:?}", area);
        self.areas
            .get_mut(&area)
            .unwrap()
            .iter_mut()
            .find(|v| v.name == tower.location)
            .unwrap()
            .towers
            .push(json_tower);
    }

    pub fn has_area(&self, area: &String, map: &AreaMap) -> bool {
        self.areas
            .get(&map.get_area(area))
            .unwrap()
            .iter()
            .find(|v| v.name == *area)
            .is_some()
    }
    pub fn add_area(&mut self, area: AreaInformation, map: &AreaMap) {
        self.areas
            .get_mut(&map.get_area(&area.name))
            .unwrap()
            .push(area);
    }

    pub fn add_tower_badge(&mut self, name: &str, badge: u64) {
        self.towers.get_mut(name).unwrap().badges.push(badge)
    }

    pub fn has_tower(&self, name: &str) -> bool {
        self.towers.contains_key(name)
    }

    pub fn write_to_file(&self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let data = serde_json::to_string(&self)?;
        Ok(fs::write(path, data)?)
    }
}
