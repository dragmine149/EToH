use std::{f64, u8};

use crate::definitions::{AreaInformation, TowerType};
use pyo3::{
    PyResult, Python,
    ffi::c_str,
    types::{PyAnyMethods, PyDict},
};
use regex::Regex;

#[derive(Debug, Default, Clone)]
pub struct WIkiTower {
    pub tower_name: String,
    pub tower_type: TowerType,
    pub location: String,
    pub difficulty: f64,
}

// impl WIkiTower {
//     fn built(&self) -> bool {
//         self.tower_type.is_some() && self.location.is_some() && self.difficulty.is_some()
//     }
// }

fn generator(wikitext: &str) -> String {
    let regex = Regex::new(r"(?m)<!--[\w\s()]*-->").unwrap();
    let cleaned = regex.replace_all(wikitext, "");
    cleaned.to_string()
}

pub fn parse_wiki_text(wikitext: &str) -> Option<WIkiTower> {
    let mut tower = WIkiTower::default();
    let generator = generator(wikitext);
    let mut parser = generator.lines();
    parser.find(|x| *x == "{{TowerInfobox");

    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| -> PyResult<()> {
        let wtp = py.import("wikitextparser")?;
        let parsed = wtp.call_method1("parse", (wikitext,))?;
        let templates = parsed.getattr("templates")?;
        let globals = PyDict::new(py);
        globals.set_item("templates", &templates)?;
        let len = py
            .eval(c_str!("len(templates)"), Some(&globals), None)?
            .extract::<u8>()?;
        println!("{:?}", len);

        if len == 0 {
            // early break because i can't be bothered to deal with that.
            return Ok(());
        }

        let mut index = u8::MAX;
        for template in 0..len {
            if templates
                .get_item(template)?
                .getattr("name")?
                .extract::<String>()?
                .trim()
                .to_lowercase()
                == "towerinfobox"
            {
                index = template;
                break;
            }
        }

        if index == u8::MAX {
            eprintln!("WARNING: Skipping tower due to annoying wikitext!");
            return Ok(());
        }

        // println!("{:?}", index);
        let item = templates.get_item(index)?;
        println!("{:?}", item);
        let raw_difficulty = item
            .call_method1("get_arg", ("difficulty",))?
            .getattr("value")?
            .extract::<String>()?;
        println!("{:?}", raw_difficulty);
        let tower_difficulty = wtp
            .call_method1("parse", (raw_difficulty,))?
            .getattr("templates")?
            .get_item(0)?
            .getattr("arguments")?
            .get_item(0)?
            .getattr("value")?
            .extract::<String>()?
            .parse::<f64>()
            .unwrap_or(0 as f64);
        println!("{:?}", tower_difficulty);

        let raw_location = item
            .call_method1("get_arg", ("found_in",))?
            .getattr("value")?
            .extract::<String>()?;
        println!("{:?}", raw_location);
        let tower_location = wtp
            .call_method1("parse", (raw_location,))?
            .call_method0("plain_text")?
            .extract::<String>()?
            .lines()
            .next()
            .unwrap()
            .trim()
            .to_string();
        println!("{:?}", tower_location);

        let raw_type = item
            .call_method1("get_arg", ("type_of_tower",))?
            .getattr("value")?
            .extract::<String>()?;
        println!("{:?}", raw_type);
        let tower_type = wtp
            .call_method1("parse", (raw_type,))?
            .call_method0("plain_text")?
            .extract::<String>()?
            .trim()
            .to_string();
        println!("{:?}", tower_type);

        tower.difficulty = tower_difficulty;
        tower.location = tower_location;
        tower.tower_type = tower_type.into();

        Ok(())
    })
    .unwrap();

    Some(tower)
}

// fn parse_area(area_name: &str, wikitext: &str) -> AreaInformation {
//     let mut area_info = AreaInformation {
//         name: area_name.to_string(),
//         ..Default::default()
//     };

//     Python::with_gil(|py| {
//         let wtp = py.import("wikitextparser")?;
//         let parsed = wtp.call_method1("parse", (wikitext,))?;

//         Ok(())
//     });

//     area_info
// }
