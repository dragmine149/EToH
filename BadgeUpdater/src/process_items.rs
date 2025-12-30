use itertools::Itertools;

use crate::{
    badge_to_wikitext::get_page_redirect,
    definitions::{
        AreaInformation, AreaRequirements, Badge, EventInfo, EventItem, Length, TowerType,
        WikiTower,
    },
    reqwest_client::RustClient,
    wikitext::{
        Argument, QueryType, Template, WikiText,
        enums::LinkType,
        parsed_data::{List, ParsedData},
    },
};

/// Get the difficulty provided by the template.
/// `original_difficulty` field is ignored as there can be many difficulties.
///
/// Also parses the difficulty into a number which we can use.
fn get_difficulty(template: &Template) -> Result<f64, String> {
    let query = template.get_named_args_query("difficulty", QueryType::StartsWith);
    let difficulty_text = query.first().ok_or("No difficulty found in tower")?;

    // match is required as there is more than one way.
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
            Argument::Table(_) => return Err(String::from("table in list in template!")),
            Argument::Text(text) => text.raw.clone(),
        },
        Argument::Text(text) => text.raw.clone(),
        Argument::Table(_) => return Err(String::from("Somehow a table in difficulty")),
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

/// Get the length field of the specified template.
fn get_length(template: &Template) -> Result<Length, String> {
    let query = template.get_named_args_query("length", QueryType::StartsWith);
    let length_text = query
        .first()
        // we have to deal with this, but some by default length is < 20 minutes hence we can ignore it.
        .ok_or("(warn ignore) No length found in tower")?;

    // just catching some loose ones.
    if length_text.raw.is_empty() {
        return Ok(Length::default());
    }

    let txt = match length_text
        .elements
        .first()
        .ok_or("No elements in length but not empty? ({:?})")?
    {
        Argument::Template(template) => match template.get_positional_arg(0) {
            // nice, we have number
            Ok(arg) => arg.raw.clone(),
            // yeah, sometimes `{{Length}}` exists which defaults to < 20 mins
            Err(_) => return Ok(Length::default()),
        },
        Argument::Link(_) => return Err(String::from("Somehow a link in Length")),
        Argument::List(_) => {
            return Err(String::from(
                "Somehow a List in Length (never seen this before)",
            ));
        }
        Argument::Table(_) => return Err(String::from("Somehow a table in Length")),
        Argument::Text(text) => text.raw.clone(),
    };

    // should avoid chases when length is provided but no length is realistically provided.
    // NOTE: We can't actually remove this as this prevents cases with comments and other stuff failing the parsing even though its valid.
    if !txt.chars().any(|c| c.is_numeric()) {
        return Ok(Length::default());
    }

    // parse and return.
    let v = txt
        .parse::<u16>()
        .map_err(|e| format!("Failed to parse number ({:?})", e))?;
    Ok(Length::from(v))
}

/// The type of tower box is more accurate than from the name.
/// This also covers for mini-towers and the rare (now removed) case of `Thanos Tower`
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

/// Area is more complicated than it looks.
fn get_area(template: &Template, tower_name: &str) -> Result<String, String> {
    let area_obj = template
        .get_named_args_query("found_in", QueryType::StartsWith)
        .first()
        .ok_or(format!("Failed to get area of {:?}", tower_name))?
        .elements
        .clone();

    // get the first element which passes our checks.
    //
    for elm in area_obj {
        match elm {
            Argument::Link(link) => return Ok(link.target.clone()),
            Argument::List(list) => {
                // log::debug!("{:?}", list);

                // yeah, this is annoying. We have to get the first entry as raw text
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
                // just to parse it to try and get the internal link on that object.
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

/// get_page_redirect but returns wikitext
/// TODO: move this?
pub async fn get_page_data(client: &RustClient, page: &str) -> Result<WikiText, String> {
    let data = get_page_redirect(client, page).await;
    if let Ok(res) = data {
        let mut wikitext = WikiText::parse(res.text);
        wikitext.set_page_name(res.name);
        return Ok(wikitext);
    }
    Err(format!("Failed to get {:?}", page))
}

/// Items have their own specific set of template which we need to deal with.
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
    // technically it could be found elsewhere but here is most likely.
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
    // got to check all the links though.
    for link in links {
        let mut wikitext = get_page_data(client, &link.target).await?;
        wikitext.set_page_name(Some(link.target));
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

/// Area requirements are semi unique.
///
/// NOTE: This affects the object directly instead of returning a new object.
fn parse_area_requirement(text: &str, reqs: &mut AreaRequirements) -> Result<(), String> {
    // custom regex to search for us.
    let (_total, _, _, count, _, diff, towers, _, area) = lazy_regex::regex_captures!(
        r"(?m)(\*|=|=\*)?(.*) (\d+) (\{\{Difficulty\|(.*)\|.*\|)?(\[?\[?Towers?)? ?(in.*\[\[(.*)\]\])?",
        text.split("<").next().ok_or("Failed to get first item??")?
    )
    .ok_or(format!("Invalid info (no matches): {:?}", text))?;
    log::debug!(
        "{:?}",
        lazy_regex::regex_captures!(
            r"(?m)(\*|=|=\*)?(.*) (\d+) (\{\{Difficulty\|(.*)\|.*\|)?(\[?\[?Towers?)? ?(in.*\[\[(.*)\]\])?",
            text.split("<").next().ok_or("Failed to get first item??")?
        )
    );
    let count = count
        .trim()
        .parse::<u64>()
        .map_err(|e| format!("Failed to parse count: {:?} ({:?})", e, count))?;
    // all the possible types.

    if !area.is_empty() {
        log::debug!("Require area: {:?}", area);
        reqs.areas.insert(
            area.to_owned(),
            AreaRequirements {
                points: count,
                ..Default::default()
            },
        );
        return Ok(());
    }
    if !towers.is_empty() {
        reqs.points = count;
        return Ok(());
    }
    reqs.difficulties.parse_difficulty(diff, count);
    Ok(())
}

/// Loop through all requirements in the list as there can be a couple..
///
/// This just helps separate code out though
fn get_requirements(list: &List) -> Result<AreaRequirements, String> {
    let mut reqs = AreaRequirements::default();
    // TODO: figure out why this is cutting off links.
    log::debug!("{:?}", list);
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

/// Get all requirements in the template.
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

    log::debug!("{:?}", requirements);
    match requirements {
        Argument::List(list) => get_requirements(&list),
        // If we just have a text object, it's probably just the one requirement hence we can parse that raw.
        Argument::Text(_) => {
            let mut reqs = AreaRequirements::default();
            let err = parse_area_requirement(
                &template.get_named_arg_raw("towers_required").map_err(|e| {
                    format!("Somehow failed to get raw after we just got it... {:?}", e)
                })?,
                &mut reqs,
            );
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

// Just like items, areas are also special.
pub async fn process_area(client: &RustClient, area: &str) -> Result<AreaInformation, String> {
    let wikitext = get_page_data(client, area).await?;
    let parsed = wikitext
        .get_parsed()
        .map_err(|e| format!("Failed to parse wikitext: {:?}", e))?;
    let template = parsed
        .get_template("ringinfobox")
        .map_err(|e| format!("Failed to get ringinfobox ({:?}) > {:?}", area, e))?;

    // parent is the most important one. It's easier to get the parent than the children.
    // we ignore any error as if it's an error, the wiki is incorrect.
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

/// The only template which requires this uniqueness of 2 versions
/// Parser can't deal with this (yes its two templates). Hence we just get one, return if successful else get the other.
fn get_event_template(data: &ParsedData, area: &str) -> Result<Template, String> {
    let normal = data
        .get_template("eventinfobox")
        .map_err(|e| format!("Failed to get eventinfobox ({:?}) > {:?}", area, e));
    // if let Ok(norm) = normal {
    if normal.is_ok() {
        return normal;
        // return Ok(norm);
    }
    data.get_template("event infobox")
        .map_err(|e| format!("Failed to get event infobox ({:?}) > {:?}", area, e))
}

/// Events are like areas mostly, but with less data so we only store a way to group them.
/// And besides, event areas can sometimes not follow convention making it harder to automate. So less work the better.
pub async fn process_event_area(client: &RustClient, area: &str) -> Result<EventInfo, String> {
    let wikitext = get_page_data(client, area).await?;
    // println!("{:?}", wikitext);
    let parsed = wikitext
        .get_parsed()
        .map_err(|e| format!("Failed to parse wikitext: {:?}", e))?;
    let template = get_event_template(&parsed, area)?;
    let name_text = template
        // realm is most likely
        .get_named_arg("realm")
        .map_err(|e| format!("Failed to get realm of area {:?} ({:?})", area, e))?
        // and because its a mixture of `{{icon template}} area \n (some text)`
        .elements
        .iter()
        // get the plain text
        .find(|elm| matches!(elm, Argument::Text(_)))
        .ok_or(format!("Failed to get text of realm of {:?}", area))?
        .as_text()
        .unwrap()
        .raw
        .clone();
    // and ignore any further lines
    let name = name_text.split("<br/>").next().unwrap().trim();

    Ok(EventInfo {
        area_name: name.to_owned(),
        event_name: area.to_owned(),
    })
}

pub fn process_event_item(
    text: &WikiText,
    badge: &Badge,
    event_areas: &Vec<&EventInfo>,
) -> Result<EventItem, String> {
    println!("____________________________________________________________");
    println!("BADGE: {:?}", badge.name);
    let links = text
        .get_parsed()
        .map_err(|e| format!("Failed to parse wikitext ({:?})", e))?
        .get_links(Some(LinkType::Internal));
    println!(
        "{:?}",
        links
            .iter()
            .filter(|e| e.target.starts_with("Category:"))
            .collect_vec()
    );
    println!(
        "{:?}",
        links
            .iter()
            .filter(|e| e.target.starts_with("Category:"))
            .filter(|link| {
                event_areas.iter().any(|e| {
                    link.target
                        .to_lowercase()
                        .contains(&e.event_name.to_lowercase())
                })
            })
            .collect_vec()
    );
    println!("{:?}", event_areas);
    println!("____________________________________________________________");
    let event = links
        .iter()
        .filter(|link| link.target.starts_with("Category"))
        .find(|link| {
            event_areas.iter().any(|e| {
                link.target
                    .to_lowercase()
                    .contains(&e.event_name.to_lowercase())
            })
        })
        .ok_or(format!(
            "Failed to get event area out of page categories ({:?}) ({:?})",
            badge.name,
            links.iter().map(|link| &link.target).collect_vec()
        ))?;
    Ok(EventItem {
        item_name: badge.name.to_owned(),
        event_name: event
            .target
            .split(":")
            .nth(1)
            .ok_or("Failed to get event name from category split")?
            .into(),
        badge_id: badge.id,
    })
}
