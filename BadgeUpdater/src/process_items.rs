use crate::{definitions::Badge, wikitext::{parser::WikiText, parts::ArgPart}};

#[derive(Debug, Default)]
pub struct WikiTower {
    badge_name: String,
    badge_id: u64,
    page_name: String,
    difficulty: f64,
    area: String,
    length: u64,
    tower_type: String,
}

pub fn process_tower(text: &WikiText, badge: &Badge) -> Result<WikiTower, String> {
    let template = text
        .get_parsed()
        .templates
        .iter()
        .find(|t| t.name.to_lowercase().starts_with("towerinfobox"))
        .ok_or(String::from("Failed to find towerinfobox in template"))?;

    let area = template
        .get_argument_startswith("area")
        .ok_or("Failed to get area of tower")?
        .value_plain();
    let difficulty = WikiText::parse(template
        .get_argument_startswith("difficulty")
        .ok_or("Failed to get difficulty of tower")?.value_plain()).get_parsed().templates.iter().next().ok_or("Somehow no templates in difficulty")?.args.iter().

    Ok(WikiTower {
        badge_name: badge.name.to_owned(),
        badge_id: badge.id,
        area: area,..Default::default(),
    })
}
