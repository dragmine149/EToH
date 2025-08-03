use std::f64;

use crate::definitions::{AreaInformation, AreaRequirements, TowerDifficulties, TowerType};
use pyo3::{
    Bound, PyAny, PyResult, Python,
    ffi::c_str,
    types::{PyAnyMethods, PyDict, PyModule},
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

// Yes, two different parse functions because the wiki is annoying...

fn get_raw<'a>(item: &'a Bound<'a, PyAny>, name: &'a str) -> Bound<'a, PyAny> {
    println!("Getting arg for {:?}", name);
    let result = item.call_method1("get_arg", (name,));
    if let Some(arg) = result.ok() {
        if !arg.is_none() {
            println!("Normal");
            return arg;
        }
    }
    let result = item.call_method1("get_arg", (name.to_owned() + "1",));
    if let Some(arg) = result.ok() {
        if !arg.is_none() {
            println!("+1");
            return arg;
        }
    }
    let result = item.call_method1("get_arg", (name.to_owned() + "<!--1-->",));
    if let Some(arg) = result.ok() {
        if !arg.is_none() {
            println!("+comment");
            return arg;
        }
    }

    println!("Uncaught case!");
    panic!("An uncaught case somehow");
}

fn parse_infobox(
    templates: &Bound<'_, PyAny>,
    index: &u8,
    wtp: &Bound<'_, PyModule>,
) -> Result<WIkiTower, Box<dyn std::error::Error>> {
    let item = templates.get_item(index)?;
    // println!("{:?}", item);
    let raw_difficulty = get_raw(&item, "difficulty")
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
    // println!("{:?}", tower_difficulty);

    let raw_location = get_raw(&item, "found_in")
        .getattr("value")?
        .extract::<String>()?;
    // println!("{:?}", raw_location);
    let tower_location = wtp
        .call_method1("parse", (raw_location,))?
        .call_method0("plain_text")?
        .extract::<String>()?
        .lines()
        .next()
        .unwrap()
        .trim()
        .to_string();
    // println!("{:?}", tower_location);

    let raw_type = get_raw(&item, "type_of_tower")
        .getattr("value")?
        .extract::<String>()?;
    // println!("{:?}", raw_type);
    let tower_type = wtp
        .call_method1("parse", (raw_type,))?
        .call_method0("plain_text")?
        .extract::<String>()?
        .trim()
        .to_string();
    // println!("{:?}", tower_type);

    Ok(WIkiTower {
        difficulty: tower_difficulty,
        location: tower_location,
        tower_type: tower_type.into(),
        ..Default::default()
    })
}

fn parse_area(
    templates: &Bound<'_, PyAny>,
    index: &u8,
    wtp: &Bound<'_, PyModule>,
) -> Result<AreaInformation, Box<dyn std::error::Error>> {
    let item = templates.get_item(index)?;
    println!("{:?}", item);

    let raw_subarea = match item.call_method1("get_arg", ("realm",)) {
        Ok(raw) => {
            println!("{:?}", raw);
            if raw.is_none() {
                None
            } else {
                Some(raw.getattr("value")?.extract::<String>()?)
            }
        }
        Err(_) => None,
    };
    println!("{:?}", raw_subarea);
    let sub_area = match raw_subarea {
        Some(area) => Some(
            wtp.call_method1("parse", (area,))?
                .call_method0("plain_text")?
                .extract::<String>()?
                .trim()
                .to_string(),
        ),
        None => None,
    };
    println!("sub: {:?}", sub_area);

    let raw_requirements;
    let mut requirements = AreaRequirements::default();
    if sub_area.is_none() {
        raw_requirements = item
            .call_method1("get_arg", ("towers_required",))?
            .getattr("value")?
            .extract::<String>()?;
        println!("{:?}", raw_requirements);
        let items = wtp
            .call_method1("parse", (raw_requirements,))?
            .call_method0("get_lists")?
            .get_item(0)?
            .getattr("items")?;
        println!("{:?}", items);
        let globals = PyDict::new(wtp.py());
        globals.set_item("items", &items)?;
        println!(
            "{:?}",
            wtp.py().eval(c_str!("len(items)"), Some(&globals), None)
        );
        for i in 0..wtp
            .py()
            .eval(c_str!("len(items)"), Some(&globals), None)?
            .extract::<u8>()?
        {
            println!("i: {:?}", i);
            let raw_item = items.get_item(i)?;
            let req = wtp.call_method1("parse", (&raw_item,))?;
            let plain = req.call_method0("plain_text")?.extract::<String>()?;
            println!("{:?}", plain);
            if plain
                .replace("*", "")
                .trim()
                .to_lowercase()
                .starts_with("none")
            {
                break;
            }
            // if !plain.starts_with("*") {
            //     continue;
            // }
            if plain.to_lowercase().contains("tower points") {
                let mut iter = plain.splitn(3, " ");
                iter.next();
                requirements.points = iter.next().unwrap().parse().unwrap();
                println!("{:?}", requirements.points);
                continue;
            }

            // println!("{:?}", &req.extract::<String>());
            // println!("{:?}", items.get_item(i)?);
            let diff = TowerDifficulties::find_type(&raw_item.extract::<String>()?.to_lowercase())
                .unwrap();
            println!("{:?}", diff);
            // println!(
            //     "{:?}",
            //     plain
            //         .to_lowercase()
            //         .split_once("beat")
            //         .unwrap()
            //         .1
            //         .trim()
            //         .split_once(" ")
            //         .unwrap()
            // );
            let num = plain
                .to_lowercase()
                .split_once("beat")
                .unwrap()
                .1
                .trim()
                .split_once(" ")
                .unwrap()
                .0
                .parse::<u64>()
                .unwrap();
            println!("{:?}", num);

            // let difficulty = plain
            //     .replace("Beat", "")
            //     .split_once("+")
            //     .unwrap()
            //     .0
            //     .split_once(" ")
            //     .map(|s| (s.0.to_string(), s.1.to_string()))
            //     .unwrap();
            // let num = difficulty.0.parse::<u64>().unwrap();
            requirements.difficulties.from_difficulty(&diff, num);
        }
    }

    Ok(AreaInformation {
        requirements,
        sub_area,
        ..Default::default()
    })
}

pub fn parse_wiki_text_area(wikitext: &str) -> Option<AreaInformation> {
    let mut area = AreaInformation::default();

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
        let mut v2 = false;
        for template in 0..len {
            let name = templates
                .get_item(template)?
                .getattr("name")?
                .extract::<String>()?;
            // println!("{:?}", name);
            if name.trim().to_lowercase().starts_with("ringinfobox") {
                index = template;
                v2 = name.trim().to_lowercase() == "ringinfobox-2panels";
                break;
            }
        }

        if index == u8::MAX {
            eprintln!("Somehow index not set! area edition");
            return Ok(());
        }

        println!("Version: {:?}", v2);
        area = parse_area(&templates, &index, &wtp)
            .ok()
            .unwrap_or_default();

        // println!("{:?}", index);

        Ok(())
    })
    .unwrap();

    Some(area)
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
        let mut v2 = false;
        for template in 0..len {
            let name = templates
                .get_item(template)?
                .getattr("name")?
                .extract::<String>()?;
            // println!("{:?}", name);
            if name.trim().to_lowercase().starts_with("towerinfobox") {
                index = template;
                v2 = name.trim().to_lowercase() == "towerinfobox-2panels";
                break;
            }
        }

        if index == u8::MAX {
            eprintln!("Somehow index not set!");
            return Ok(());
        }

        println!("Version: {:?}", v2);
        tower = parse_infobox(&templates, &index, &wtp).ok().unwrap();

        // println!("{:?}", index);

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
