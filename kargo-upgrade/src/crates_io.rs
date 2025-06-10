//! Crates.io API client for querying the latest versions of crates

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use reqwest::Client;
use serde_json::Value;

/// Shared HTTP client for crates.io API requests
static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .user_agent("krater/version-up2date")
        .build()
        .unwrap_or_else(|e| {
            log::error!("Failed to create HTTP client: {}", e);
            panic!("Critical error: Failed to create HTTP client: {}", e);
        })
});

/// Get the latest version of a crate from crates.io
/// Returns a Future that resolves to the latest version
pub async fn get_latest_version(crate_name: &str) -> Result<Option<String>> {
    let future = VersionFuture {
        crate_name: crate_name.to_string(),
    };
    future.fetch().await
}

/// Domain-specific type for fetching a crate version
pub struct VersionFuture {
    crate_name: String,
}

impl VersionFuture {
    /// Internal method that performs the actual async work
    pub fn fetch(self) -> impl std::future::Future<Output = Result<Option<String>>> + Send {
        async move {
            // Query crates.io API
            let url = format!("https://crates.io/api/v1/crates/{}", self.crate_name);

            match CLIENT.get(&url).send().await {
                Ok(response) => {
                    if !response.status().is_success() {
                        return Ok(None);
                    }

                    match response.json::<Value>().await {
                        Ok(data) => {
                            // Extract the latest version
                            let version = data
                                .get("crate")
                                .and_then(|c| c.get("max_version"))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());

                            Ok(version)
                        }
                        Err(_) => Ok(None),
                    }
                }
                Err(e) => Err(anyhow!("Failed to query crates.io: {}", e)),
            }
        }
    }
}
