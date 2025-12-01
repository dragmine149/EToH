use crate::{
    definitions::{Badge, Length, TowerType},
    wikitext::{
        Template,
        parser::WikiText,
        parts::{ArgPart, ArgQueryKind, ArgQueryResult, MatchType, parts_to_plain},
    },
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
    // Prefer nested template's first positional value, fallback to textual parts
    let num_text = match template.query_arg(
        "difficulty",
        MatchType::StartsWith,
        ArgQueryKind::NestedFirstPositionalText,
    ) {
        Some(ArgQueryResult::Text(s)) => s,
        Some(ArgQueryResult::Part(p)) => p.to_plain().trim().to_string(),
        Some(ArgQueryResult::Parts(ps)) => parts_to_plain(ps).trim().to_string(),
        None => return Err("Failed to get difficulty of the tower".to_string()),
    };

    num_text.parse::<f64>().map_err(|e| {
        log::debug!("{}", template);
        format!("Failed to parse number ({} -> {:?})", num_text, e)
    })
}

fn get_length(template: &Template) -> Result<Length, String> {
    let txt = match template.query_arg(
        "length",
        MatchType::StartsWith,
        ArgQueryKind::NestedFirstPositionalText,
    ) {
        Some(ArgQueryResult::Text(s)) => s,
        Some(ArgQueryResult::Part(p)) => p.to_plain().trim().to_string(),
        Some(ArgQueryResult::Parts(ps)) => parts_to_plain(ps).trim().to_string(),
        None => return Err("Failed to get length of the tower".to_string()),
    };

    let v = txt
        .parse::<u16>()
        .map_err(|e| format!("Failed to parse number ({:?})", e))?;
    Ok(Length::from(v))
}

fn get_type(template: &Template) -> Result<TowerType, String> {
    // Use parts-level query and inspect first element's variant
    let first =
        match template.query_arg("type_of_tower", MatchType::StartsWith, ArgQueryKind::Parts) {
            Some(ArgQueryResult::Parts(ps)) => {
                if let Some(p) = ps.get(0) {
                    p
                } else {
                    return Err("Failed to get type of tower".to_string());
                }
            }
            Some(ArgQueryResult::Part(p)) => p,
            Some(ArgQueryResult::Text(t)) => return Ok(TowerType::from(t)),
            None => return Err("Failed to get type of tower".to_string()),
        };

    match first {
        ArgPart::Text(t) => Ok(TowerType::from(t.to_owned())),
        ArgPart::InternalLink { target, .. } => Ok(TowerType::from(target.to_owned())),
        _ => Err("Invalid argpart type".to_string()),
    }
}

pub fn process_tower(text: &WikiText, badge: &Badge) -> Result<WikiTower, String> {
    let template = text
        .get_template_startswith("towerinfobox")
        .ok_or(String::from("Failed to find towerinfobox in template"))?;

    let area = template
        .get_argument_startswith("found_in")
        .ok_or("Failed to get area of tower")?
        .value_plain();
    let difficulty = match get_difficulty(&template) {
        Ok(diff) => diff,
        Err(e) => {
            log::warn!("[Difficult/{}]: {:?}", badge.display_name, e);
            100.0
        }
    };
    let length = match get_length(&template) {
        Ok(len) => len,
        Err(e) => {
            log::warn!("[Length/{}]: {:?}", badge.display_name, e);
            Length::default()
        }
    };
    let tower_type = match get_type(&template) {
        Ok(tp) => tp,
        Err(e) => {
            log::warn!("[Type/{}]: {:?}", badge.display_name, e);
            TowerType::default()
        }
    };
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
