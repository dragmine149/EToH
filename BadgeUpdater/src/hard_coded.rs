use crate::{definitions::ProcessError, reqwest_client::RustClient, wikitext::WikiText};

pub async fn parse_mini_towers(client: &RustClient) -> Result<(), String> {
    let mini_towers = client
        .get("https://jtoh.fandom.com/wiki/Mini_Tower?action=raw")
        .send()
        .await
        .map_err(|e| format!("{:?}", e))?
        .text()
        .await
        .map_err(|e| format!("{:?}", e))?;

    let mini_wiki = WikiText::parse(mini_towers);
    let data = mini_wiki
        .get_parsed()
        .map_err(|e| format!("{:?}", e))?
        .get_tables();
    let table = data
        .get(0)
        .ok_or("Failed to find table on mini tower page... (how!!??)")?;

    println!("{:?}", table.get_headers());

    for row_id in 0..table.get_rows().len() {
        let cell = table.get_cell(row_id, "Name");
        println!("row: {:?}, cell: {:?}", row_id, cell);
    }

    // .get_table_by_title("Mini Tower List");
    // println!("{:#?}", table);

    Ok(())
}
