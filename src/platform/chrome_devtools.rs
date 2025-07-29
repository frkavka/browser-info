//! Chrome DevTools Protocol integration for detailed browser information extraction.
//!
//! This module is only available on Windows with the `devtools` feature enabled.

use crate::{BrowserInfo, BrowserInfoError, BrowserType};
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct ChromeTab {
    #[allow(dead_code)]
    id: String,
    title: String,
    url: String,
    #[serde(rename = "type")]
    tab_type: String,
}

pub struct ChromeDevToolsExtractor;

impl ChromeDevToolsExtractor {
    const DEFAULT_PORT: u16 = 9222;
    const TIMEOUT_SECS: u64 = 3;

    pub async fn is_available() -> bool {
        Self::test_connection(Self::DEFAULT_PORT).await
    }

    async fn test_connection(port: u16) -> bool {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(Self::TIMEOUT_SECS))
            .build()
            .unwrap();

        let url = format!("http://localhost:{port}/json/version");
        client.get(&url).send().await.is_ok()
    }

    pub async fn extract_browser_info() -> Result<BrowserInfo, BrowserInfoError> {
        let tabs = Self::get_tabs(Self::DEFAULT_PORT).await?;

        // 最初に見つかったページタブを返す
        let active_tab = tabs
            .into_iter()
            .find(|tab| tab.tab_type == "page")
            .ok_or(BrowserInfoError::Other("No active tabs found".to_string()))?;

        Ok(BrowserInfo {
            url: active_tab.url,
            title: active_tab.title,
            browser_name: "Chrome".to_string(),
            browser_type: BrowserType::Chrome,
            version: None,       // DevTools APIからは簡単には取得できない
            tabs_count: None,    // 今回は簡略化
            is_incognito: false, // 今回は簡略化
            process_id: 0,       // DevTools APIからは取得できない
            window_position: Default::default(), // Default trait使用
        })
    }

    async fn get_tabs(port: u16) -> Result<Vec<ChromeTab>, BrowserInfoError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(Self::TIMEOUT_SECS))
            .build()
            .map_err(|e| BrowserInfoError::Other(format!("Network error: {e}")))?;

        let url = format!("http://localhost:{port}/json");
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| BrowserInfoError::Other(format!("Network error: {e}")))?;

        let tabs: Vec<ChromeTab> = response
            .json()
            .await
            .map_err(|e| BrowserInfoError::Other(format!("Parse error: {e}")))?;

        Ok(tabs)
    }
}
