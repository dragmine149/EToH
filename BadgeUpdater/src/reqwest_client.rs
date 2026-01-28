//! An extension to the default reqwest client but with caching support.
//!
//! It's done like this just to make it easier to make new versions / have a centrialised location for the client.

use reqwest::{Client, Response};
use serde::Deserialize;
use tokio::time::sleep;

use crate::fmt_secs;
use std::{
    fmt::Debug,
    fs::{self, create_dir_all},
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
    time::{self, Duration, SystemTime},
};

/// Custom struct as a wrapper for custom functions
///
/// # Usage
/// ```
/// RustClient::new(Some("a path".into()), None)
/// ```
/// You could make it from the struct but it's recommended to use [RustClient::new] that way. If you want to make it from a struct...
#[derive(Debug, Clone)]
pub struct RustClient(
    /// The [reqwest::Client] we expand and rely upon.
    pub Client,
    /// The cache path location.
    PathBuf,
    /// How long to wait after each request.
    ///
    /// This is not the best solution, but a solution to avoid spamming servers which might return errors from too much spam...
    Duration,
);
/// Custom error to include all potential reqwest related errors.
#[derive(Debug)]
#[allow(dead_code, reason = "I use this for debugging...")]
pub enum RustError {
    /// Something happened within the reqwest layer, or the reqwest itself.
    Underly(reqwest::Error),
    /// Something happened with saving/loading the data.
    File(std::io::Error),
    /// Something happened with parsing the data.
    IOParse(std::str::Utf8Error),
    /// Something happened which caused a reqwst::error_from_status_code and we stored it.
    CacheError(String),
}

impl From<reqwest::Error> for RustError {
    fn from(value: reqwest::Error) -> Self {
        Self::Underly(value)
    }
}
impl From<std::io::Error> for RustError {
    fn from(value: std::io::Error) -> Self {
        Self::File(value)
    }
}
impl From<std::str::Utf8Error> for RustError {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::IOParse(value)
    }
}

impl RustClient {
    /// Create a new client, cache is forced and only cleared after every day.
    ///
    /// # Arguments
    /// * cache_path -> The path to store the cache. Defaults to `./.cache`
    /// * user_agent -> Custom user agent to tell the server. Defaults to `Some program written in rust...`
    /// * wait_time -> How long to wait after the request. This does mean everything will be slowed down but should reduce getting blocked due to mass requests.
    ///
    /// # Returns
    /// - a new client object to use.
    pub fn new(
        cache_path: Option<&str>,
        user_agent: Option<&str>,
        wait_time: Option<Duration>,
    ) -> Self {
        let cache = PathBuf::from(cache_path.unwrap_or("./.cache"));
        let client = reqwest::ClientBuilder::new()
            .user_agent(user_agent.unwrap_or("Some program written in rust..."))
            .build()
            .unwrap();
        let c = Self(client, cache, wait_time.unwrap_or_default());
        c.clear_cache();
        c
    }

    /// Clear the cache.
    ///
    /// Only clears cache if:
    /// - we can get metadata
    /// - we can get created date
    /// - created data is > 1 day ago
    /// - we have permission to delete folder (and everything inside)
    fn clear_cache(&self) {
        let meta = self.1.metadata();
        if let Err(e) = meta {
            log::error!("Failed to get metadata for cache: {:?}", e);
            return;
        }

        let created = meta.unwrap().created();
        if let Err(e) = created {
            log::error!("Failed to get created data for cache: {:?}", e);
            return;
        }

        let duration = SystemTime::now().duration_since(created.unwrap());
        if let Err(e) = duration {
            log::error!("Failed to compare duration times (backwards?): {:?}", e);
            return;
        }

        let age = duration.unwrap().as_secs();
        let comp = age > 86400;
        if !comp {
            log::info!(
                "Not deleting cache dir due to being < 1d ({:?}s, aka {:?})",
                age,
                fmt_secs(age)
            );
            return;
        }

        if let Err(e) = fs::remove_dir_all(&self.1) {
            log::error!("Failed to remove cache dir {:?}", e);
            return;
        }
        log::warn!("Deleted cache dir, might take a bit longer to process");
    }

    /// Sends a reqwest using reqwest as well as cache the reqwest.
    ///
    /// Instead of returning the builder, the resulting bytes is returned.
    /// As much as we loose out on detail of the response, i don't think we really need half the response.
    /// If we do, then we'll figure that out then.
    pub async fn get<U>(&self, url: U) -> Result<ResponseBytes, RustError>
    where
        U: reqwest::IntoUrl,
    {
        // Make a new unique hash for each item. (not worth building on itself as threads)
        let mut hasher = DefaultHasher::new();
        url.as_str().hash(&mut hasher);
        let hashrl = hasher.finish();
        // make our cache.
        let cache_path = self.1.join(hashrl.to_string());

        // check our cache for previous entries
        log::debug!("Checking cache: {:?}", cache_path);
        let bytes = ResponseBytes::read_from_file(&cache_path);
        if let Ok(b) = bytes {
            if b.0 == b"error" {
                // special case for error entries
                return Err(RustError::CacheError("Status error from api...".into()));
            }

            log::debug!("Cache hit, returning cache!");
            // log::error!("{:?} already exists...", cache_path);
            return Ok(b);
        }

        // send a network response
        log::debug!("Cache miss, sending API reqwest.");
        let response = self.0.get(url).send().await?.error_for_status();
        if let Err(err) = response {
            // special network error entry
            ResponseBytes::write_error(&cache_path)?;
            return Err(err.into());
        }
        // process it and return the result after saving.
        let response = response.unwrap();
        let bytes = ResponseBytes::from_response(response).await?;
        bytes.write_to_file(&cache_path)?;

        sleep(self.2).await;
        Ok(bytes)
    }
}

/// Custom storage of the bytes from the response.
pub struct ResponseBytes(Vec<u8>);
impl ResponseBytes {
    /// Turns a reqwest::Response into bytes we can use.
    ///
    /// NOTE: If the response fails, this will still save bytes no matter what. Unless there is no bytes.
    pub async fn from_response(response: Response) -> Result<ResponseBytes, reqwest::Error> {
        Ok(Self(response.bytes().await?.to_vec()))
    }

    /// Return a new copy of ourself by reading a file.
    pub fn read_from_file(path: &PathBuf) -> Result<Self, std::io::Error> {
        Ok(Self(fs::read(path)?))
    }
    /// Store ourselves to the provided path
    ///
    /// # Panics!
    /// Panics if path doesn't have a parent dir. *we need the file somewhere...*
    pub fn write_to_file(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        // if we don't have a path parent, make a temp dir...
        create_dir_all(path.parent().expect("Expected path to have a parent dir"))?;
        fs::write(path, self.0.clone())
    }

    /// Return the provided json for ourselves
    ///
    /// aka a wrapper of [serde_json::from_slice]
    pub fn json<'de, T>(&'de self) -> Result<T, serde_json::Error>
    where
        T: Deserialize<'de>,
    {
        serde_json::from_slice::<T>(&self.0)
    }
    /// Return the provided text for ourselves
    pub fn text(&self) -> Result<&str, std::str::Utf8Error> {
        str::from_utf8(&self.0)
    }
    /// Write an error to the file to tell future stuff not to worry about this.
    pub fn write_error(path: &PathBuf) -> Result<(), std::io::Error> {
        fs::write(path, b"error")
    }
}

impl Debug for ResponseBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "ResponseBytes({:?})", self.0)
        } else {
            write!(
                f,
                "ResponseBytes({})",
                match self.0.is_empty() {
                    true => "Empty",
                    false => "Has Data",
                }
            )
        }
    }
}
