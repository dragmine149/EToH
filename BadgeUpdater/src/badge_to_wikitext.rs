use std::{error::Error, str::FromStr};

use url::Url;

use crate::{
    ETOH_WIKI, clean_badge_name,
    definitions::{Badge, Data},
    wiki_api::RustClient,
};

struct BadgeResponse {
    badge: Badge,
    data: WikiData,
}

// TODO: expand out to the parsed format.
struct WikiData(String);

pub async fn process_data(client: &RustClient, url: &Url) -> Result<WikiData, Box<dyn Error>> {
    let mut badges: Vec<String> = vec![];
    let data: Data = client.0.get(*url).send().await?.json::<Data>().await?;

    client.get_text(
        data.data
            .iter()
            .map(|b| clean_badge_name(&b.name))
            .map(|n| format!("{:}{:}", ETOH_WIKI, n))
            .map(|n| Url::from_str(&n).unwrap())
            .collect::<Vec<Url>>(),
    );
}
