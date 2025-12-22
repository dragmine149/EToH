use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest_middleware::ClientWithMiddleware;

/// Custom struct as a wrapper for custom functions
#[derive(Debug, Clone)]
pub struct RustClient(pub ClientWithMiddleware);
/// Custom error to include all potential reqwest related errors.
#[derive(Debug)]
#[allow(dead_code, reason = "I use this for debugging...")]
pub enum RustError {
    MiddleWare(reqwest_middleware::Error),
    Underly(reqwest::Error),
}

impl RustClient {
    /// Create a new client with middleware which auto caches based on HTTP headers
    ///
    /// # Arguments
    /// - cache_path -> The path to store the cache. Defaults to `./.cache`
    /// - user_agent -> Custom user agent to tell the server. Defaults to `Some program written in rust...`
    ///
    /// # Returns
    /// - a new client object to use.
    pub fn new(cache_path: Option<&str>, user_agent: Option<&str>) -> Self {
        let client = reqwest_middleware::ClientBuilder::new(
            reqwest::ClientBuilder::new()
                .user_agent(user_agent.unwrap_or("Some program written in rust..."))
                .build()
                .unwrap(),
        )
        .with(Cache(HttpCache {
            mode: CacheMode::ForceCache,
            manager: CACacheManager::new(cache_path.unwrap_or("./.cache").into(), true),
            options: HttpCacheOptions::default(),
        }))
        .build();
        Self(client)
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
