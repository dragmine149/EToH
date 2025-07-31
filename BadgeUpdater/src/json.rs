use std::{collections::HashMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    definitions::{AreaInformation, AreaMap, AreaRequirements, Tower, TowerDifficulties},
    parse_wikitext::WIkiTower,
};

impl From<&WIkiTower> for Tower {
    fn from(value: &WIkiTower) -> Self {
        Self {
            name: String::default(),
            difficulty: value.difficulty.unwrap(),
            badges: vec![],
            tower_type: value.tower_type,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TowerJSON {
    pub permanent: HashMap<String, AreaInformation>,
    pub temporary: HashMap<String, AreaInformation>,
    pub other: HashMap<String, AreaInformation>,

    #[serde(skip)]
    towers: HashMap<String, Tower>,
}

impl TowerJSON {
    pub fn make_areas(&mut self, map: &AreaMap) {
        for a in map.areas.get("permanent").unwrap() {
            let mut area_info = AreaInformation::default();
            area_info.name = a.0.to_owned();
            self.permanent.insert(a.0.to_owned(), area_info);
        }
    }

    pub fn insert_tower(&mut self, tower: WIkiTower, name: &str, badge: u64, map: &AreaMap) {
        let mut json_tower = Tower::from(&tower);
        json_tower.badges.push(badge);
        json_tower.name = name.to_owned();

        self.towers.insert(name.to_owned(), json_tower.to_owned());
        let area = map.get_area(&tower.location.unwrap());
        match area.0.as_str() {
            "permanent" => self
                .permanent
                .get_mut(&area.1)
                .unwrap()
                .towers
                .push(json_tower),
            "temporary" => self
                .temporary
                .get_mut(&area.1)
                .unwrap()
                .towers
                .push(json_tower),
            "other" => self.other.get_mut(&area.1).unwrap().towers.push(json_tower),
            _ => return,
        }
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
