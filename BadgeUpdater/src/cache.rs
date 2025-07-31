use std::{
    fs,
    path::PathBuf,
    time::{Duration, SystemTime},
};

use url::Url;

fn make_path(url: &Url) -> PathBuf {
    let mut path = PathBuf::new();
    path.push(".cache");
    path.push(url.path().replace("/", ""));
    // println!("{path:?}, {:?}", fs::exists(&path));
    let exists = fs::exists(&path);
    if exists.is_err() || exists.unwrap() == false {
        // println!("No path, making one!");
        fs::create_dir_all(&path.parent().unwrap()).unwrap();
    }

    path
}

pub fn write_cache(url: &Url, data: &String) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(make_path(url), data)?;
    Ok(())
}

pub fn read_cache(url: &Url) -> Option<std::string::String> {
    let path = make_path(url);
    // Get file metadata to check modification time
    let metadata = match fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(_) => return None,
    };

    // Get the file's last modified time
    let modified_time = match metadata.modified() {
        Ok(time) => time,
        Err(_) => return None,
    };

    // Check if file is older than one day
    let now = SystemTime::now();
    let one_day = Duration::from_secs(24 * 60 * 60);

    if now.duration_since(modified_time).unwrap_or(Duration::ZERO) > one_day {
        return None;
    }

    // File is fresh, read and return contents
    match fs::read_to_string(&path) {
        Ok(contents) => Some(contents),
        Err(_) => None,
    }
}
