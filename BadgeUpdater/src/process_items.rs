use crate::{
    definitions::{Badge, Length, TowerType},
    wikitext::{Template, parser::WikiText, parts::ArgPart},
};

#[derive(Debug, Default)]
pub struct WikiTower {
    pub badge_name: String,
    pub badge_id: u64,
    pub page_name: String,
    pub difficulty: f64,
    pub area: String,
    pub length: Length,
    pub tower_type: TowerType,
}

/// Get the difficulty provided by the template.
/// `original_difficulty` field is ignored as there can be many difficulties.
fn get_difficulty(template: &Template) -> Result<f64, String> {
    Ok(match template
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
    .map_err(|e| format!("Failed to parse number ({:?})", e))?)
}

fn get_length(template: &Template) -> Result<Length, String> {
    Ok(Length::from(
        match template
            .get_arg_parts_startswith("length")
            .ok_or("Failed to get length of the tower")?
            .get(0)
        {
            Some(ArgPart::Template(nested)) => Ok(nested
                .args
                .get(0)
                .ok_or("Failed to get arg of length")?
                .value_plain()),
            Some(ArgPart::Text(t)) => Ok(t.to_owned()),
            _ => Err("Invalid argpart type"),
        }?
        .parse::<u16>()
        .map_err(|e| format!("Failed to parse number ({:?})", e))?,
    ))
}

fn get_type(template: &Template) -> Result<TowerType, String> {
    Ok(TowerType::from(match template
        .get_arg_parts_startswith("type_of_tower")
        .ok_or("Failed to get type of tower")?
        .get(0)
    {
        Some(ArgPart::Text(t)) => Ok(t.to_owned()),
        Some(ArgPart::InternalLink { target, label }) => Ok(target.to_owned()),
        _ => Err("Invalid argpart type"),
    }?))
}

pub fn process_tower(text: &WikiText, badge: &Badge) -> Result<WikiTower, String> {
    let template = text
        .get_template_startswith("towerinfobox")
        .ok_or(String::from("Failed to find towerinfobox in template"))?;

    let area = template
        .get_argument_startswith("found_in")
        .ok_or("Failed to get area of tower")?
        .value_plain();
    let difficulty = get_difficulty(&template).unwrap_or(100.0);
    let length = get_length(&template).unwrap_or_default();
    let tower_type = get_type(&template).unwrap_or_default();
    let page_name = text.page_name.clone().unwrap_or_default().to_owned();

    Ok(WikiTower {
        badge_name: badge.name.to_owned(),
        badge_id: badge.id,
        area: area,
        difficulty: difficulty,
        length: length,
        tower_type: tower_type,
        page_name: page_name,
        ..Default::default()
    })
}
