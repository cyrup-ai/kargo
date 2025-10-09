//! Crates.io API client for querying the latest versions of crates

use anyhow::{Context, Result, anyhow};
use reqwest::Client;
use serde_json::Value;

/// Get the latest version of a crate from crates.io (async)
/// Returns a Future that resolves to the latest version
pub async fn get_latest_version(crate_name: &str, allow_prerelease: bool) -> Result<Option<String>> {
    let future = VersionFuture {
        crate_name: crate_name.to_string(),
        allow_prerelease,
    };
    future.fetch().await
}

/// Blocking variant for use inside spawn_blocking
pub fn get_latest_version_blocking(crate_name: &str, allow_prerelease: bool) -> Result<Option<String>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("krater/version-up2date")
        .build()
        .context("Failed to create HTTP client")?;

    let url = format!("https://crates.io/api/v1/crates/{}", crate_name);

    match client.get(&url).send() {
        Ok(resp) => {
            if !resp.status().is_success() {
                return Ok(None);
            }
            match resp.json::<Value>() {
                Ok(data) => {
                    let version_field = if allow_prerelease { "max_version" } else { "max_stable_version" };
                    let version = data
                        .get("crate")
                        .and_then(|c| c.get(version_field))
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

/// Domain-specific type for fetching a crate version
pub struct VersionFuture {
    crate_name: String,
    allow_prerelease: bool,
}

impl VersionFuture {
    /// Internal method that performs the actual async work
    pub async fn fetch(self) -> Result<Option<String>> {
        // Create HTTP client with user agent
        let client = Client::builder()
            .user_agent("krater/version-up2date")
            .build()
            .context("Failed to create HTTP client")?;

        // Query crates.io API
        let url = format!("https://crates.io/api/v1/crates/{}", self.crate_name);

        match client.get(&url).send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    return Ok(None);
                }

                match response.json::<Value>().await {
                    Ok(data) => {
                        // Choose version field based on preference
                        let version_field = if self.allow_prerelease {
                            "max_version"
                        } else {
                            "max_stable_version"
                        };

                        // Extract the latest version
                        let version = data
                            .get("crate")
                            .and_then(|c| c.get(version_field))
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
