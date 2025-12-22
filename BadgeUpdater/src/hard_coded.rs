use crate::{
    definitions::{Badge, WikiTower},
    process_items::{get_page_data, process_tower},
    reqwest_client::RustClient,
    wikitext::{WikiText, enums::LinkType},
};

/// Minitowers have their own unique page we can fill out when normal stuff fails.
///
/// ignore is there so we don't try getting stuff again.
pub async fn parse_mini_towers(
    client: &RustClient,
    badges: &[Badge],
    ignore: &[String],
) -> Vec<Result<WikiTower, String>> {
    let mini_towers = client
        .get("https://jtoh.fandom.com/wiki/Mini_Tower?action=raw")
        .send()
        .await
        .unwrap()
        // .map_err(|e| format!("{:?}", e))?
        .text()
        .await
        .unwrap();
    // .map_err(|e| format!("{:?}", e))?;

    let mini_wiki = WikiText::parse(mini_towers);
    let data = mini_wiki
        .get_parsed()
        .unwrap()
        // .map_err(|e| format!("{:?}", e))?
        .get_tables();
    let table = data.first().unwrap();
    // .ok_or("Failed to find table on mini tower page... (how!!??)")?;

    println!("{:?}", table.get_headers());

    let mut mini_towers = vec![];
    for row_id in 0..table.get_rows().len() {
        let cell = table.get_cell(row_id, "Name");
        log::debug!("Processing: {:?}", cell);
        println!("row: {:?}, cell: {:?}", row_id, cell);
        if let Some(data) = cell {
            let links = data.inner.content.get_links(Some(LinkType::Internal));
            let target = links.first();
            if target.is_none() {
                // mini_towers.push(Err(format!("Failed to get link for {:?}", data)));
                continue;
            }
            let target = target.unwrap();
            if ignore.contains(&target.target) {
                // no need to push anything as we're ignoring it.
                log::debug!("Ignoring cell due to already processed");
                continue;
            }

            let wikitext = get_page_data(client, &target.target).await;

            if wikitext.is_err() {
                mini_towers.push(Err(format!("Failed to get wiki data for {:?}", data)));
                continue;
            }
            let mut wikitext = wikitext.ok().unwrap();
            wikitext.set_page_name(Some(target.target.to_owned()));

            let badge = badges.iter().find(|b| {
                // println!("{:?}", b.id);
                wikitext.text().contains(&b.id.to_string())
            });

            if badge.is_none() {
                mini_towers.push(Err(format!("Failed to find badge id for {:?}", data)));
                println!("{:?}", wikitext.text());
                continue;
            }

            mini_towers.push(process_tower(&wikitext, badge.unwrap()));
        }
    }

    mini_towers
}
