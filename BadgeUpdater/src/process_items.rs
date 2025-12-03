use crate::{
    definitions::{Badge, Length, TowerType},
    wikitext::{Argument, QueryType, Template, WikiText},
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
    let query = template.get_named_args_query("difficulty", QueryType::StartsWith);
    let difficulty_text = query.first().ok_or("No difficulty found in tower")?;
    match difficulty_text
        .elements.first()
        .ok_or("No elements in difficulty?")?
    {
        Argument::Template(template) => {
            template
                .get_positional_arg(0)
                .map_err(|e| format!("failed to get first arg ({})", e))?
                .raw
        }
        Argument::Link(_) => return Err(String::from("Somehow a link in difficulty")),
        Argument::List(list) => match list.entries.first().ok_or("List with no entries?")? {
            Argument::Template(template) => {
                template
                    .get_positional_arg(0)
                    .map_err(|e| format!("failed to get first arg ({})", e))?
                    .raw
            }
            Argument::Link(_) => return Err(String::from("Somehow a link in difficulty")),
            Argument::List(_) => return Err(String::from("Who made a list in a list?")),
            Argument::Text(text) => text.raw.clone(),
        },
        Argument::Text(text) => text.raw.clone(),
    }
    .parse::<f64>()
    .map_err(|e| {
        // log::debug!("{}", template);
        format!(
            "Failed to parse number ({} -> {:?})",
            difficulty_text.raw, e
        )
    })
}

fn get_length(template: &Template) -> Result<Length, String> {
    let query = template.get_named_args_query("length", QueryType::StartsWith);
    let length_text = query.first()
        .ok_or("(warn ignore) No length found in tower")?;

    if length_text.raw.is_empty() {
        return Ok(Length::default());
    }

    let txt = match length_text
        .elements.first()
        .ok_or("No elements in length but not empty? ({:?})")?
    {
        Argument::Template(template) => match template.get_positional_arg(0) {
            Ok(arg) => arg.raw.clone(),
            Err(_) => return Ok(Length::default()),
        },
        Argument::Link(_) => return Err(String::from("Somehow a link in Length")),
        Argument::List(_) => {
            return Err(String::from(
                "Somehow a List in Length (never seen this before)",
            ));
        }
        Argument::Text(text) => text.raw.clone(),
    };

    // should avoid chases when length is provided but no length is realistically provided.
    if !txt.chars().any(|c| c.is_numeric()) {
        return Ok(Length::default());
    }

    let v = txt
        .parse::<u16>()
        .map_err(|e| format!("Failed to parse number ({:?})", e))?;
    Ok(Length::from(v))
}

fn get_type(template: &Template) -> Result<TowerType, String> {
    let query = template.get_named_args_query("type_of_tower", QueryType::StartsWith);
    let type_text = query.first().ok_or("Failed to get type of tower")?;
    let txt = match type_text.get(0).map_err(|e| format!("{:?}", e))? {
        Argument::Text(text) => text.raw.clone(),
        Argument::Link(link) => link.label.clone(),
        _ => {
            return Err(format!(
                "Somehow another type of argument was in type: {:?}",
                type_text.raw
            ));
        }
    };
    Ok(TowerType::from(txt))
}

pub fn process_tower(text: &WikiText, badge: &Badge) -> Result<WikiTower, String> {
    // log::debug!("Tower: {:?}", text.page_name());
    let parsed = text
        .get_parsed()
        .map_err(|e| format!("Failed to parse wikitext: {:?}", e))?;
    let templates = parsed.get_template_query("towerinfobox", QueryType::StartsWith);
    let template = templates.first().ok_or(format!(
        "Failed to get towerinfobox ({:?})",
        text.page_name()
    ))?;

    let area = template
        .get_named_args_query("found_in", QueryType::StartsWith).first()
        .ok_or("Failed to get area of tower")?
        .raw
        .clone();
    let difficulty = match get_difficulty(template) {
        Ok(diff) => diff,
        Err(e) => {
            log::warn!("[Difficult/{}]: {:?}", badge.display_name, e);
            100.0
        }
    };
    let length = match get_length(template) {
        Ok(len) => len,
        Err(e) => {
            if !e.contains("(warn ignore)") {
                log::warn!("[Length/{}]: {:?}", badge.display_name, e);
            }
            Length::default()
        }
    };
    let tower_type = match get_type(template) {
        Ok(tp) => tp,
        Err(e) => {
            log::warn!("[Type/{}]: {:?}", badge.display_name, e);
            TowerType::default()
        }
    };
    let page_name = text.page_name().unwrap_or_default();

    Ok(WikiTower {
        badge_name: badge.name.to_owned(),
        badge_id: badge.id,
        area,
        difficulty,
        length,
        tower_type,
        page_name,
        ..Default::default()
    })
}
