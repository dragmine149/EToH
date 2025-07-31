use crate::{
    definitions::{AreaInformation, Tower},
    parse_wikitext::WIkiTower,
};

impl From<WIkiTower> for Tower {
    fn from(value: WIkiTower) -> Self {
        Self {
            name: String::default(),
            difficulty: value.difficulty.unwrap(),
            badges: vec![],
            tower_type: value.tower_type,
        }
    }
}

pub struct TowerJSON {
    pub permanent: Vec<AreaInformation>,
    pub temporary: Vec<AreaInformation>,
    pub other: Vec<AreaInformation>,
}

impl TowerJSON {
    pub fn insert_tower(mut self, tower: WIkiTower, name: &str, badges: &[u64]) {
        let mut json_tower = Tower::from(tower);
        json_tower.badges = badges.to_vec();
        json_tower.name = name.to_owned();
    }
}
