use crate::{
    definitions::Badge,
    wikitext::{parser::WikiText, parts::ArgPart},
};

#[derive(Debug, Default)]
pub struct WikiTower {
    pub badge_name: String,
    pub badge_id: u64,
    pub page_name: String,
    pub difficulty: f64,
    pub area: String,
    pub length: u64,
    pub tower_type: String,
}

pub fn process_tower(text: &WikiText, badge: &Badge) -> Result<WikiTower, String> {
    let template = text
        .get_template_startswith("towerinfobox")
        .ok_or(String::from("Failed to find towerinfobox in template"))?;

    let area = template
        .get_argument_startswith("area")
        .ok_or("Failed to get area of tower")?
        .value_plain();
    let difficulty = match template
        .get_arg_parts_startswith("difficulty")
        .ok_or("Failed to get difficulty of the tower")?
        .get(0)
    {
        Some(ArgPart::Template(nested)) => Ok(nested
            .args
            .get(0)
            .ok_or("Failed to get difficultynum arg")?
            .value_plain()),
        Some(ArgPart::Text(t)) => Ok(t.to_owned()),
        _ => Err("Invalid argpart type"),
    }?
    .parse::<f64>()
    .map_err(|e| format!("Failed to parse number ({:?})", e))?;

    Ok(WikiTower {
        badge_name: badge.name.to_owned(),
        badge_id: badge.id,
        area: area,
        difficulty: difficulty,
        ..Default::default()
    })
}
