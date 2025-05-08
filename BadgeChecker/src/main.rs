mod definitions;
use definitions::*;
use reqwest::blocking::Client;

// const URL: &str = "https://badges.roblox.com/v1/universes/3264581003/badges?limit=100";

fn get_badges(client: &Client, url: String) -> Result<Vec<Badge>, reqwest::Error> {
    let mut badges: Vec<Badge> = vec![];
    let mut data: Data = Data {
        previous_page_cursor: None,
        next_page_cursor: Some(String::new()),
        data: vec![],
    };

    while let Some(next_page_cursor) = data.next_page_cursor {
        let request_url = format!("{}&cursor={}", url, next_page_cursor);
        println!("Fetching badges from {}", request_url);
        let response = client.get(&request_url).send()?;
        println!("Response status: {}", response.status());

        data = response.json::<Data>()?;
        badges.extend(data.data);
    }

    Ok(badges)
}

fn main() {
    let badges = get_badges(
        &Client::new(),
        String::from("https://badges.roblox.com/v1/universes/3264581003/badges?limit=100"),
    )
    .unwrap();
    let old_badges = get_badges(
        &Client::new(),
        String::from("https://badges.roblox.com/v1/universes/1055653882/badges?limit=100"),
    )
    .unwrap();

    let used_tower_badges = serde_json::from_str::<TowerSchema>(
        &std::fs::read_to_string("../data/tower_data.json").unwrap(),
    )
    .unwrap();
    let used_badges = serde_json::from_str::<OtherSchema>(
        &std::fs::read_to_string("../data/other_data.json").unwrap(),
    )
    .unwrap();

    // Process tower badges

    let badge_list = used_tower_badges
        .areas
        .iter()
        .flat_map(|(_, area)| {
            area.iter()
                .flat_map(|info| info.towers.iter().flat_map(|tower| tower.badges.to_vec()))
        })
        .chain(
            used_badges
                .data
                .iter()
                .flat_map(|other| other.badges.to_vec()),
        )
        .collect::<Vec<u64>>();

    // println!("{:?}", badge_list.collect::<Vec<u64>>());

    let unused = badges
        .iter()
        .filter(|badge| !badge_list.contains(&badge.id))
        .map(|badge| format!("{} - {}\n", badge.id, badge.name))
        .collect::<String>();

    let old_unused = old_badges
        .iter()
        .filter(|badge| !badge_list.contains(&badge.id))
        .map(|badge| format!("{} - {}\n", badge.id, badge.name))
        .collect::<String>();

    if !old_unused.is_empty() {
        println!();
        println!();
        println!("Old unused badges found (somehow): \n{}", old_unused);
    }

    if !unused.is_empty() {
        println!();
        println!();
        panic!("Unused badges found:\n{}", unused);
    }
}
