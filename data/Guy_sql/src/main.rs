use itertools::Itertools;
use rusqlite::Connection;
use rusqlite::Result;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug)]
struct Tower {
    difficulty: f64,
    badge_id: u64,
}

impl PartialEq for Tower {
    fn eq(&self, other: &Self) -> bool {
        self.difficulty == other.difficulty
    }
}

impl Eq for Tower {}

impl PartialOrd for Tower {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.difficulty.partial_cmp(&other.difficulty)
    }
}

impl Ord for Tower {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

fn row_to_json(badge_id: u64, name: String, difficulty: f64) -> (String, Tower) {
    (
        name,
        Tower {
            difficulty,
            badge_id,
        },
    )
}

fn main() -> Result<()> {
    let mut data: Vec<(u64, String, f64)> = Vec::new();

    let conn = Connection::open("etoh.sqlite3")?;
    let mut stmt = conn.prepare(
        "SELECT tb.badge_id, t.name, t.difficulty
         FROM towers t
         JOIN tower_badges tb ON t.name = tb.name
         WHERE t.found_in = (
             SELECT a.name
             FROM areas a
             WHERE a.acronym = 'Z10'
         )
         ORDER BY t.difficulty;",
    )?;

    let rows = stmt.query_map([], |row| {
        // println!("Row: {:?}", row);

        Ok((
            row.get(0)?, // badge_id
            row.get(1)?, // name
            row.get(2)?, // difficulty
        ))
    })?;

    data.extend(rows.filter_map(Result::ok));

    // println!("{:?}", data);
    let mut file = std::fs::File::create("output.json").unwrap();
    let mut tower_vec: Vec<(String, Tower)> = data
        .iter()
        .map(|(badge_id, name, difficulty)| row_to_json(*badge_id, name.clone(), *difficulty))
        // .sorted_by(|a, b| a.1.order(&b.1))
        .collect();

    // tower_vec.sort_by(|a, b| a.1.order(&b.1));
    tower_vec.sort();

    let tower_map: HashMap<String, Tower> = tower_vec.into_iter().collect();

    println!("Tower Map: {:?}", tower_map);

    let json_string = serde_json::to_string_pretty(&tower_map).unwrap();
    file.write_all(json_string.as_bytes()).unwrap();

    Ok(())
}
