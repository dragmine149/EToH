use std::{fs, path::PathBuf, time::SystemTime};

use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest_middleware::ClientWithMiddleware;

use crate::fmt_secs;

/// Custom struct as a wrapper for custom functions
#[derive(Debug, Clone)]
pub struct RustClient(pub ClientWithMiddleware, PathBuf);
/// Custom error to include all potential reqwest related errors.
#[derive(Debug)]
#[allow(dead_code, reason = "I use this for debugging...")]
pub enum RustError {
    MiddleWare(reqwest_middleware::Error),
    Underly(reqwest::Error),
}

impl RustClient {
    /// Create a new client with middleware, cache is forced and only cleared after every day.
    ///
    /// # Arguments
    /// - cache_path -> The path to store the cache. Defaults to `./.cache`
    /// - user_agent -> Custom user agent to tell the server. Defaults to `Some program written in rust...`
    ///
    /// # Returns
    /// - a new client object to use.
    pub fn new(cache_path: Option<&str>, user_agent: Option<&str>) -> Self {
        let cache = PathBuf::from(cache_path.unwrap_or("./.cache"));
        let client = reqwest_middleware::ClientBuilder::new(
            reqwest::ClientBuilder::new()
                .user_agent(user_agent.unwrap_or("Some program written in rust..."))
                .build()
                .unwrap(),
        )
        .with(Cache(HttpCache {
            mode: CacheMode::ForceCache,
            manager: CACacheManager::new(cache.clone(), true),
            options: HttpCacheOptions::default(),
        }))
        .build();
        let c = Self(client, cache);
        c.clear_cache();
        c
    }

    /// Clear the cache provided by the middleware.
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

        if let Err(e) = fs::remove_dir_all(self.1.to_owned()) {
            log::error!("Failed to remove cache dir {:?}", e);
            return;
        }
        log::warn!("Deleted cache dir, might take a bit longer to process");
    }

    /// Wrapper for [reqwest.get()].
    pub fn get<U>(&self, url: U) -> reqwest_middleware::RequestBuilder
    where
        U: reqwest::IntoUrl,
    {
        self.0.get(url)
    }
}

impl From<reqwest::Error> for RustError {
    fn from(value: reqwest::Error) -> Self {
        Self::Underly(value)
    }
}
impl From<reqwest_middleware::Error> for RustError {
    fn from(value: reqwest_middleware::Error) -> Self {
        Self::MiddleWare(value)
    }
}
