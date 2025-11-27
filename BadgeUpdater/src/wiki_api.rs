use futures::future;
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest::Response;
use reqwest_middleware::ClientWithMiddleware;
use url::Url;

/// Custom struct as a wrapper for custom functions
pub struct RustClient(ClientWithMiddleware);
/// Custom error to include all potential reqwest related errors.
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
        let responses = request_urls(&self.0, urls).await;
        future::join_all(responses.into_iter().map(|res| async move {
            match res {
                Ok(resp) => resp.text().await.map_err(RustError::Underly),
                Err(e) => Err(RustError::MiddleWare(e)),
            }
        }))
        .await
    }
}

/// Internal function to request urls with.
async fn request_urls(
    client: &ClientWithMiddleware,
    urls: Vec<Url>,
) -> Vec<Result<Response, reqwest_middleware::Error>> {
    future::join_all(urls.into_iter().map(|url| {
        let client = client.clone();
        async move { client.get(url).send().await }
    }))
    .await
}
