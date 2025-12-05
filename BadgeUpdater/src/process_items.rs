use crate::{
    badge_to_wikitext::get_page,
    definitions::{AreaInformation, AreaRequirements, Badge, Length, TowerType, WikiTower},
    reqwest_client::RustClient,
    wikitext::{Argument, QueryType, Template, WikiText, enums::LinkType, parsed_data::List},
};

/// Get the difficulty provided by the template.
/// `original_difficulty` field is ignored as there can be many difficulties.
fn get_difficulty(template: &Template) -> Result<f64, String> {
    let query = template.get_named_args_query("difficulty", QueryType::StartsWith);
    let difficulty_text = query.first().ok_or("No difficulty found in tower")?;
    match difficulty_text
        .elements
        .first()
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
    let length_text = query
        .first()
        .ok_or("(warn ignore) No length found in tower")?;

    if length_text.raw.is_empty() {
        return Ok(Length::default());
    }

    let txt = match length_text
        .elements
        .first()
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

fn get_area(template: &Template, tower_name: &str) -> Result<String, String> {
    let area_obj = template
        .get_named_args_query("found_in", QueryType::StartsWith)
        .first()
        .ok_or(format!("Failed to get area of {:?}", tower_name))?
        .elements
        .clone();
    for elm in area_obj {
        match elm {
            // Argument::Template(template) => todo!(),
            Argument::Link(link) => return Ok(link.target.clone()),
            Argument::List(list) => {
                log::debug!("{:?}", list);
                let wt = WikiText::parse(
                    list.entries
                        .first()
                        .ok_or(format!(
                            "Failed to get first entry of list ({:?}/found_in)",
                            tower_name
                        ))?
                        .as_text()
                        .ok_or(format!(
                            "Failed to translate into text (we checked this though...) ({:?})",
                            tower_name
                        ))?
                        .raw
                        .clone(),
                );
                return Ok(wt
                    .get_parsed()
                    .map_err(|e| format!("Failed to parse list entry: {:?} ({:?})", e, tower_name))?
                    .get_links(Some(LinkType::Internal))
                    .first()
                    .ok_or(format!("No links in first list entry ({:?})", tower_name))?
                    .target
                    .clone());
            }
            _ => {
                // log::warn!("Failed to deal with {:?} for {:?}", elm, tower_name);
                continue;
            }
        }
    }
    Err(format!("Failed to find a link for {:?}", tower_name))
}

/// Processes the tower provided into something else.
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

    let area = get_area(template, &badge.name)?;

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
    })
}

async fn get_page_data(client: &RustClient, page: &str) -> Result<WikiText, String> {
    let data = get_page(client, page).await;
    if let Ok(res) = data
        && let Ok(text) = res.text().await
    {
        let mut wikitext = WikiText::parse(text);
        wikitext.set_page_name(Some(page));
        return Ok(wikitext);
    }
    Err(format!("Failed to get {:?}", page))
}

#[allow(
    clippy::await_holding_refcell_ref,
    reason = "we specifically drop it, its fine. We can't do the workaround without complicating the code any further and we don't really need the parsed obj anymore."
)]
pub async fn process_item(
    client: &RustClient,
    text: &WikiText,
    badge: &Badge,
) -> Result<WikiTower, String> {
    let page_name = text.page_name();
    let parsed = text
        .get_parsed()
        .map_err(|e| format!("Failed to parse wikitext: {:?}", e))?;
    let template = parsed
        .get_template("iteminfobox")
        .map_err(|e| format!("Failed to get iteminfobox ({:?}) > {:?}", page_name, e))?;
    let links = template
        .get_named_arg("method_of_obtaining")
        .map_err(|e| {
            format!(
                "Failed to get method of obtaining on item template ({:?})",
                e
            )
        })?
        .get_links(Some(LinkType::Internal));

    drop(parsed);
    for link in links {
        let wikitext = get_page_data(client, &link.target).await?;
        let tower = process_tower(&wikitext, badge);
        if tower.is_ok() {
            return tower;
        }
    }
    Err(format!(
        "Failed to get a valid tower out of the links provided. ({:?})",
        page_name
    ))
}

fn parse_area_requirement(text: &str, reqs: &mut AreaRequirements) -> Result<(), String> {
    let (_, reqtype, count, _, diff) =
        lazy_regex::regex_captures!(r"(?m)\s?(\w+) (\d+) (\{\{Difficulty\|(\w+))?", text)
            .ok_or(format!("Invalid info (no matches): {:?}", text))?;
    let count = count
        .trim()
        .parse::<u64>()
        .map_err(|e| format!("Failed to parse count: {:?} ({:?})", e, count))?;
    match reqtype {
        "Obtain" => reqs.points = count,
        "Beat" => reqs.difficulties.parse_difficulty(diff, count),
        _ => return Err(format!("Invalid type: {:?}", reqtype)),
    };
    Ok(())
}

fn get_requirements(list: &List) -> Result<AreaRequirements, String> {
    let mut reqs = AreaRequirements::default();
    for entry in list.entries.iter() {
        let text = entry
            .as_text()
            .ok_or(format!(
                "Failed to parse requirement to text ({:?})",
                entry.to_wikitext()
            ))?
            .raw
            .clone();
        parse_area_requirement(&text, &mut reqs)?;
    }
    Ok(reqs)
}

fn get_all_requirements(template: &Template, area: &str) -> Result<AreaRequirements, String> {
    let requirements = template
        .get_named_arg("towers_required")
        .map_err(|e| {
            format!(
                "Failed to get towers_required for area (none required?) ({:?}) ({})",
                e, area
            )
        })?
        .get(0)
        .map_err(|e| format!("Failed to get elements (how??) ({:?})", e))?;

    match requirements {
        Argument::List(list) => get_requirements(&list),
        Argument::Text(text) => {
            let mut reqs = AreaRequirements::default();
            let err = parse_area_requirement(&text.raw, &mut reqs);
            if err.is_err() {
                log::warn!("{:?}", err);
                return Err(err.err().unwrap());
            }
            Ok(reqs)
        }
        _ => Err(format!(
            "Failed to get lists (ok... whats wrong here? ({:?})",
            template.to_wikitext()
        )),
    }
}

pub async fn process_area(client: &RustClient, area: &str) -> Result<AreaInformation, String> {
    let wikitext = get_page_data(client, area).await?;
    let parsed = wikitext
        .get_parsed()
        .map_err(|e| format!("Failed to parse wikitext: {:?}", e))?;
    // Garden of eshool has an annoying accent...
    // if area.to_lowercase().starts_with("garden") {
    //     log::warn!("{:#?}", wikitext);
    // }
    let template = parsed
        .get_template("ringinfobox")
        .map_err(|e| format!("Failed to get ringinfobox ({:?}) > {:?}", area, e))?;

    let parent = template
        .get_named_arg("realm")
        .map(|area| {
            area.get_links(Some(LinkType::Internal))
                .first()
                .unwrap()
                .label
                .to_owned()
        })
        .ok();

    let parsed_requirements = get_all_requirements(&template, area);

    // sub-areas are most likely to contain errors in this stage, hence we ignore them.
    if parsed_requirements.is_err() && parent.is_none() {
        log::warn!(
            "Error in requirements: {:?} ({:?})",
            parsed_requirements.as_ref().err().unwrap(),
            area
        );
    }

    Ok(AreaInformation {
        name: area.to_owned(),
        requirements: parsed_requirements.ok(),
        parent_area: parent,
    })
}
