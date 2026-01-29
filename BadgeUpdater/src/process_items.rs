//! Every single function (bar those in [crate::hard_coded]) which help with processing the data provided by the wiki.
//!
//! As much as this could be structured, not worth it. Also 90% of the processing happens here.

use chrono::DateTime;
use itertools::Itertools;

use crate::{
    definitions::{
        AreaInformation, AreaRequirements, Badges, EventInfo, EventItem, Length, ProcessError,
        TowerType, WikiTower,
    },
    mediawiki_api::get_pages_from_category,
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
pub fn get_difficulty(template: &Template) -> Result<f64, String> {
    let query = template.get_named_args_query("difficulty", QueryType::StartsWith);
    let difficulty_text = query.first().ok_or("No difficulty found in tower")?;

    // match is required as there is more than one way.
    match difficulty_text
        .elements
        .first()
        .ok_or("No elements in difficulty?")?
    {
        // assume format is like {{difficulty|`some difficulty`}}
        Argument::Template(template) => {
            template
                .get_positional_arg(0)
                .map_err(|e| format!("failed to get first arg ({})", e))?
                .raw
        }
        // some towers have more than one listed difficulty, an old difficulty or two different versions.
        // we just care about the first one as thats probably accurate.
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
        // easy, raw text.
        Argument::Text(text) => text.raw.clone(),
        // never seen these
        Argument::Link(_) => return Err(String::from("Somehow a link in difficulty")),
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
pub fn get_length(template: &Template) -> Result<Length, String> {
    let query = template.get_named_args_query("length", QueryType::StartsWith);
    let length_text = query
        .first()
        // we have to deal with this, but by default length is < 20 minutes hence we can ignore it.
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
        Argument::List(_) => {
            return Err(String::from(
                "Somehow a List in Length (never seen this before)",
            ));
        }
        Argument::Text(text) => text.raw.clone(),
        // never seen these
        Argument::Link(_) => return Err(String::from("Somehow a link in Length")),
        Argument::Table(_) => return Err(String::from("Somehow a table in Length")),
    };

    // should avoid chases when length is provided but no length is realistically provided.
    // i haven't seen length being defined as the name itself, just the time (20/30/45/etc). Hence we should be fine.
    //
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
/// This also covers for mini-towers and the rare case of `Thanos Tower` (or similar mixed up tower names)
pub fn get_type(template: &Template) -> Result<TowerType, String> {
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

/// Area is more complicated than it looks. As towers have been moved between areas, and some events rely on towers instead of event-area specific.
pub fn get_area(template: &Template, tower_name: &str) -> Result<String, String> {
    // basic area get function.
    let area_obj = template
        .get_named_args_query("found_in", QueryType::StartsWith)
        .first()
        .ok_or(format!("Failed to get area of {:?}", tower_name))?
        .elements
        .clone();

    // get the first element which passes our checks.
    for elm in area_obj {
        match elm {
            // link has the most likely to work.
            Argument::Link(link) => return Ok(link.target.clone()),
            Argument::List(list) => {
                //log::debug!("{:?}", list);

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
                //log::warn!("Failed to deal with {:?} for {:?}", elm, tower_name);
                continue;
            }
        }
    }
    Err(format!("Failed to find a link for {:?}", tower_name))
}

/// Processes the tower provided into something else.
///
/// Aka, a function which does many things in one.
pub fn process_tower(text: &WikiText, badge: &Badges) -> Result<WikiTower, String> {
    //log::debug!("Tower: {:?}", text.page_name());

    // Got to get the tower first.
    let parsed = text
        .get_parsed()
        .map_err(|e| format!("Failed to parse wikitext: {:?}", e))?;
    let templates = parsed.get_template_query("towerinfobox", QueryType::StartsWith);
    let template = templates.first().ok_or(format!(
        "Failed to get towerinfobox ({:?})",
        text.page_name()
    ))?;

    // these are solved in their own function, so we just have to deal with any errors.
    let area = get_area(template, &badge.name)?;

    let difficulty = match get_difficulty(template) {
        Ok(diff) => diff,
        Err(e) => {
            log::warn!("[Difficult/{}]: {:?}", badge.name, e);
            100.0
        }
    };
    let length = match get_length(template) {
        Ok(len) => len,
        Err(e) => {
            if !e.contains("(warn ignore)") {
                log::warn!("[Length/{}]: {:?}", badge.name, e);
            }
            Length::default()
        }
    };
    let tower_type = match get_type(template) {
        Ok(tp) => tp,
        Err(e) => {
            log::warn!("[Type/{}]: {:?}", badge.name, e);
            TowerType::default()
        }
    };
    let page_name = text.page_name().unwrap_or_default();

    Ok(WikiTower {
        badge_ids: badge.ids,
        area,
        difficulty,
        length,
        tower_type,
        page_name,
    })
}

/// Items are special, ever since the purgatory update you no longer get items from normal towers. Or well, key items.
/// Henceforth, we can assume everything will be linked to an event.
///
/// If this changes in the future, well... whatever deal with it then (thats half this codebase).
#[allow(
    clippy::await_holding_refcell_ref,
    reason = "We do drop it though.. kinda. Point being, it's dropped its fine. hopefully..."
)]
pub fn process_all_items(
    text: &WikiText,
    badge: &Badges,
    areas: &[&EventInfo],
) -> Result<(EventItem, Vec<String>), String> {
    let parsed = text
        .get_parsed()
        .map_err(|e| format!("Failed to parse wikitext ({:?})", e))?;
    let links = parsed.get_links(Some(LinkType::Internal));

    // Got to be a valid event based page first. A Err(String) is returned if it is not.
    // checking the linked categories is kinda the easiest way to find if it's an event, even if it might be unneseccary.
    let event_link = links
        .iter()
        .filter(|link| link.target.starts_with("Category"))
        .find(|link| {
            areas.iter().any(|e| {
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

    // Templates are nice, because we can link a tower.
    let tower_link = match parsed.get_template("iteminfobox") {
        Ok(template) => {
            match template.get_named_arg("method_of_obtaining") {
                Ok(obtain) => obtain
                    .get_links(Some(LinkType::Internal))
                    .iter()
                    .map(|link| link.target.to_owned())
                    .collect_vec(),
                Err(_) => vec![],
            }
            // if let Ok(obtain) = template.get_named_arg("method_of_obtaining") {

            // check all our links for the tower. As there are many links in one box.
            // for link in obtain.get_links(Some(LinkType::Internal)) {
            //     // If this fails, then the rest will probably fail.
            //     let mut wikitext = get_page_data(client, &link.target).await?;
            //     wikitext.set_page_name(Some(link.target.to_owned()));
            //     let tower = process_tower(&wikitext, badge);
            //     if tower.is_ok() {
            //         tower_link = tower.ok();
            //         break;
            //         // } else {
            //         //     log::error!(
            //         //         "Failed to get link tower: {:?} err: {:?}",
            //         //         link,
            //         //         tower.err()
            //         //     );
            //     }
            // }
            // }
        }
        Err(_) => vec![],
    };

    Ok((
        EventItem {
            item_name: badge.name.to_owned(),
            event_name: event_link.label.replace("Category:", "").trim().to_owned(),
            badges: badge.ids,
            tower_name: None,
        },
        tower_link,
    ))
}

/// Area requirements are semi unique...
///
/// We simultaneously got to deal with:
/// * single line requirements
/// * list requirements
/// * semi-different wording
/// * and the rare occasion of the extra message.
///
/// # Notes
/// * This affects the object directly instead of returning a new object.
/// * Even if the towers required is a list, we only process one line at a time. The regex would get too complex otherwise. If you want to do multiple, use [get_requirements]
pub fn parse_area_requirement(text: &str, reqs: &mut AreaRequirements) -> Result<(), String> {
    // custom regex to search for us. See https://regex101.com/r/UHVWCZ/3 for test data and details on regex.
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
    // if we can't process the count, we can't do much.
    let count = count
        .trim()
        .parse::<u64>()
        .map_err(|e| format!("Failed to parse count: {:?} ({:?})", e, count))?;

    // If towers are required in a specific area to be beaten.
    // Rare, but with purgatorio still a thing.
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
    // if we find the word `Towers` then it's not going to be a difficulty.
    if !towers.is_empty() {
        reqs.points = count;
        return Ok(());
    }
    reqs.difficulties.parse_difficulty(diff, count);
    Ok(())
}

/// Loop through all requirements in the list as there can be a couple..
pub fn get_requirements(list: &List) -> Result<AreaRequirements, String> {
    let mut reqs = AreaRequirements::default();
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
///
/// Compared to [get_requirements]. This takes in an even higher level object for processing.
pub fn get_all_requirements(template: &Template, area: &str) -> Result<AreaRequirements, String> {
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
        // got to go through the whole list...
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
            "Failed to get something useful for requirements (ok... whats wrong here? ({:?})",
            template.to_wikitext()
        )),
    }
}

/// Areas are special with their data, jusst like towers and items.
pub fn process_area(wikitext: &WikiText, area: &str) -> Result<AreaInformation, String> {
    // typical fetch from wiki and then get the specific template.
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

    // offloaded*
    let parsed_requirements = get_all_requirements(&template, area);

    // sub-areas are most likely to contain errors in this stage, hence we ignore them.
    // Aka, sub areas don't really have requirements as they are normally unlocked with the parent. Yes the route there might be unique but
    // thats not our problem.
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
///
/// This isn't meant to be used outside of [process_event_area]
fn get_event_template(data: &ParsedData) -> Result<Template, String> {
    let normal = data
        .get_template("eventinfobox")
        .map_err(|e| format!("Failed to get eventinfobox. reason: {:?}", e));
    // if let Ok(norm) = normal {
    if normal.is_ok() {
        return normal;
        // return Ok(norm);
    }
    data.get_template("event infobox")
        .map_err(|e| format!("Failed to get event infobox reason: {:?}", e))
}

/// Ping the category API to get all of the events.
///
/// Due to naming, links, targets and other factors. It's kinda hard to get a reliable list of all the events without going at it directly.
pub async fn get_event_areas(
    client: &RustClient,
) -> Result<Vec<Result<EventInfo, String>>, ProcessError> {
    let category_pages = get_pages_from_category::<&'static str>(client, "Events", 500).await?;

    if let Some(category_data) = category_pages.query.pages {
        return Ok(category_data
            .iter()
            .map(|page| {
                let wt = WikiText::from(page);
                process_event_area(&wt)
            })
            .collect_vec());
    }

    unreachable!("Event category should have pages")
}

/// Events are like areas mostly, but with less data so we only store a way to group them.
/// And besides, event areas can sometimes not follow convention making it harder to automate.
pub fn process_event_area(event_area: &WikiText) -> Result<EventInfo, String> {
    let area = event_area.page_name().unwrap();

    let parsed = event_area
        .get_parsed()
        .map_err(|e| format!("Failed to parse wikitext: {:?}", e))?;
    let template = get_event_template(&parsed)
        .map_err(|e| format!("{} (area: {})", e, event_area.page_name().unwrap()))?;
    let name_text = template
        // realm is most likely
        .get_named_arg("realm")
        .map_err(|e| format!("Failed to get realm of area {:?} ({:?})", area, e))?
        // and because its a mixture of `{{icon template}} area \n (some text)`
        .elements
        .iter()
        // get the plain text
        // .find(|elm| matches!(elm, Argument::Text(_)))
        .find(|elm| matches!(elm, Argument::List(_) | Argument::Text(_)))
        .map(|elm| match elm {
            Argument::List(list) => list
                .entries
                .first()
                .unwrap()
                .as_text()
                .unwrap()
                .raw
                .split_once("}}")
                .unwrap()
                .1
                .trim()
                .to_owned(),
            Argument::Text(text) => text.raw.clone(),
            _ => "??? How is this very weird edge case returned?".into(), // this shouldn't be returned...
        })
        .ok_or(format!("Failed to get text of realm of {:?}", area))?;
    // and ignore any further lines
    let name = name_text.split("<br/>").next().unwrap().trim();

    // if we have a countdown template, the event is ongoing. we can use this to our advantage.
    let countdown = parsed.get_template("Countdown");
    let until = if let Ok(count) = countdown {
        // i've tried to follow what was said in the countdown template, but that isn't the truth so some small adjustments (hopefully not breaking) had to be made.
        let end_date = count
            .get_named_arg("enddate")
            .map_err(|e| format!("Invalid countdown template ({}): {:}", area, e))?
            .raw;
        let end_time = count.get_named_arg("endtime");
        let endyear = count
            .get_named_arg("endyear")
            .map_err(|e| format!("Invalid countdown template ({}): {:}", area, e))?
            .raw;
        // this is weird and probably a quirk of js `Date()` object...
        // we just try our best to deal with it. Would love to not deal with it, but then have to offset other things and can't confirm stuff..
        let timezone_offset = count
            .get_named_arg("timezone")
            .map(|t| t.raw)
            .map(|t| {
                let split = t.split_once(":").unwrap();
                let start = if split.0.len() < 3 {
                    let mut chars = split.0.chars();
                    format!("{}0{}", chars.next().unwrap(), chars.next().unwrap())
                } else {
                    split.0.to_string()
                };
                format!("{}:{}", start, split.1)
            })
            .unwrap_or("-06:00".into());

        // format a string, our way.
        let timestamp = format!(
            "{} {} {} {}",
            end_time.map(|t| t.raw).unwrap_or("00:00:00".into()),
            endyear,
            end_date,
            timezone_offset
        );

        // and then try to parse it correctly.
        let dt = DateTime::parse_from_str(&timestamp, "%H:%M:%S %Y %B %-d %:z").map_err(|e| {
            format!(
                "Failed to convert {} countdown! ({}) {:?}",
                area, timestamp, e
            )
        })?;

        Some(dt)
    } else {
        None
    };

    Ok(EventInfo {
        area_name: name.to_owned(),
        event_name: area.to_owned(),
        until,
    })
}
