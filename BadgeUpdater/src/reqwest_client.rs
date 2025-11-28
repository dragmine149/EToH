use futures::future;
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest::Response;
use reqwest_middleware::ClientWithMiddleware;
use url::Url;

/// Custom struct as a wrapper for custom functions
#[derive(Debug, Clone)]
pub struct RustClient(pub ClientWithMiddleware);
/// Custom error to include all potential reqwest related errors.
#[derive(Debug)]
pub enum RustError {
    MiddleWare(reqwest_middleware::Error),
    Underly(reqwest::Error),
}
pub struct RustURL(Url, Response);

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
            mode: CacheMode::Default,
            manager: CACacheManager::new(cache_path.unwrap_or("./.cache").into(), true),
            options: HttpCacheOptions::default(),
        }))
        .build();
        Self(client)
    }

    /// Pass in a bunch of urls to get the raw text output of all the urls.
    ///
    /// # Arguments
    /// - urls -> A vector of urls to parse.
    ///
    /// # Returns
    /// - Vec
    /// 	- OK(String) -> The raw text of the page
    /// 	- Err(RustError) -> Custom error enum handing all possible errors.
    pub async fn get_text(&self, urls: Vec<Url>) -> Vec<Result<std::string::String, RustError>> {
        let responses = self.request_urls(urls).await;
        future::join_all(responses.into_iter().map(|res| async move {
            match res {
                Ok(resp) => resp.1.text().await.map_err(RustError::Underly),
                Err(e) => Err(RustError::MiddleWare(e)),
            }
        }))
        .await
    }

    pub async fn request_urls(
        &self,
        urls: Vec<Url>,
    ) -> Vec<Result<RustURL, reqwest_middleware::Error>> {
        future::join_all(urls.into_iter().map(|url| {
            let client = self.0.clone();
            async move {
                let response = client.get(url.clone()).send().await?;
                Ok(RustURL(url, response))
            }
        }))
        .await
    }

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
