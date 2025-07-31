use crate::definitions::TowerType;

#[derive(Debug, Default)]
pub struct WIkiTower {
    pub tower_type: Option<TowerType>,
    pub location: Option<String>,
    pub difficulty: Option<f64>,
}

impl WIkiTower {
    fn built(&self) -> bool {
        self.tower_type.is_some() && self.location.is_some() && self.difficulty.is_some()
    }
}

pub fn parse_wiki_text(wikitext: &str) -> Option<WIkiTower> {
    let mut tower = WIkiTower::default();
    // println!("{:?}", wikitext);

    let mut parser = wikitext.split("\n");
    parser.find(|x| *x == "{{TowerInfobox");

    // println!("===========================v======================================================");

    while !tower.built() {
        let value = parser.next()?;
        let value = match value.strip_prefix("|") {
            Some(v) => v,
            None => continue,
        };
        // println!("{:?}", value);

        if value.starts_with("type_of_tower") {
            println!("{:?}", value.split("=").last());

            tower.tower_type = Some(
                value
                    .split("=")
                    .last()
                    .unwrap()
                    .trim()
                    .trim_end_matches("}}")
                    .trim_start_matches("[[")
                    .trim_end_matches("]]")
                    .into(),
            )
        }
        if value.starts_with("found_in") {
            let pos = value.find("[[")?;

            println!("{:?}", value.split_at(pos).1);

            tower.location = Some(
                value
                    .split_at(pos)
                    .1
                    .trim()
                    .trim_end_matches("}}")
                    .trim_start_matches("[[")
                    .trim_end_matches("]]")
                    .to_string(),
            )
        }
        if value.contains("DifficultyNum") && !value.contains("DifficultyNumNoLink") {
            println!("{:?}", value.split("DifficultyNum").last());

            tower.difficulty = Some(
                value
                    .split("DifficultyNum")
                    .last()
                    .unwrap()
                    .trim_end_matches("}}")
                    .trim_start_matches("|")
                    .parse()
                    .ok()?,
            )
        }
    }

    if !tower.built() {
        return None;
    }

    Some(tower)
}
