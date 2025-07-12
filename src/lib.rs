//! # browser-info
//! 
//! Cross-platform library for retrieving active browser URL and detailed information.
//! 
//! Built on top of `active-win-pos-rs` for reliable window detection, with specialized
//! browser information extraction capabilities.
//! 
//! ## Quick Start
//! 
//! ```rust
//! use browser_info::get_active_browser_info;
//! 
//! match get_active_browser_info() {
//!     Ok(info) => {
//!         println!("Current URL: {}", info.url);
//!         println!("Browser: {}", info.browser_name);
//!         println!("Title: {}", info.title);
//!     }
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! ```

//================================================================================================
// Import Section
//================================================================================================

use serde::{Deserialize, Serialize};
use active_win_pos_rs::get_active_window;

pub mod error;
pub mod browser_detection;
pub mod url_extraction;

#[cfg(target_os = "windows")]
pub mod platform;

pub use error::BrowserInfoError;

#[cfg(feature = "devtools")]
pub use platform::chrome_devtools::ChromeDevToolsExtractor;

//================================================================================================
// Data Types & Module Variables
//================================================================================================

#[derive(Debug, Clone, Copy)]
pub enum ExtractionMethod {
    /// auto decision（recomment）
    Auto,
    /// Chrome DevTools Protocol（high speed but debug mode is needed）
    DevTools,
    /// PowerShell（depends on environment）
    PowerShell,
}

/// [derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BrowserInfo {
    /// Current URL displayed in the browser
    pub url: String,
    pub title: String,
    pub browser_name: String,
    pub browser_type: BrowserType,
    pub version: Option<String>,
    pub tabs_count: Option<u32>,
    pub is_incognito: bool,
    /// Process ID
    pub process_id: u64,
    /// Window position and size
    pub window_position: WindowPosition,
}

/// Browser type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BrowserType {
    Chrome,
    Firefox,
    Edge,
    Safari,
    Brave,
    Opera,
    Vivaldi,
    Unknown(String),
}

/// Window position and dimensions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct WindowPosition {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

//================================================================================================
// procedure
//================================================================================================

/// Retrieve information about the currently active browser
/// 
/// This function combines window detection (via `active-win-pos-rs`) with 
/// specialized browser information extraction.
/// 
/// # Examples
/// 
/// ```rust
/// use browser_info::get_active_browser_info;
/// 
/// match get_active_browser_info() {
///     Ok(info) => {
///         println!("URL: {}", info.url);
///         println!("Browser: {:?}", info.browser_type);
///     }
///     Err(e) => eprintln!("Failed to get browser info: {}", e),
/// }
/// ```

pub fn get_active_browser_info() -> Result<BrowserInfo, BrowserInfoError> {
    // Step 0: Check if the active window is browser
    if !is_browser_active() {
        return Err(BrowserInfoError::NotABrowser);
    }

    // Step 1: Definitely browser. Get active window using active-win-pos-rs
    let window = get_active_window().map_err(|_| BrowserInfoError::WindowNotFound)?;
    
    // Step 2: Verify it's a browser window
    let browser_type = browser_detection::classify_browser(&window)?;
    
    // Step 3: Extract URL using platform-specific methods
    let url = url_extraction::extract_url(&window, &browser_type)?;
    
    // Step 4: Get additional browser metadata
    let metadata = browser_detection::get_browser_metadata(&window, &browser_type)?;
    
    Ok(BrowserInfo {
        url,
        title: window.title,
        browser_name: window.app_name,
        browser_type,
        version: metadata.version,
        tabs_count: metadata.tabs_count,
        is_incognito: metadata.is_incognito,
        process_id: window.process_id,
        window_position: WindowPosition {
            x: window.position.x,
            y: window.position.y,
            width: window.position.width,
            height: window.position.height,
        },
    })
}

/// Get only the URL from the active browser (lightweight version)
pub fn get_active_browser_url() -> Result<String, BrowserInfoError> {
    // Step 0: 高速事前チェック
    if !is_browser_active() {
        return Err(BrowserInfoError::NotABrowser);
    }
    
    let window = get_active_window().map_err(|_| BrowserInfoError::WindowNotFound)?;
    
    let browser_type = browser_detection::classify_browser(&window)?;
    url_extraction::extract_url(&window, &browser_type)
}

/// Check if the currently active window is a browser
pub fn is_browser_active() -> bool {
    if let Ok(window) = get_active_window() {
        browser_detection::classify_browser(&window).is_ok()
    } else {
        false
    }
}

/// 非同期版：自動判定でブラウザ情報を取得
#[cfg(feature = "devtools")]
pub async fn get_active_browser_info_async() -> Result<BrowserInfo, BrowserInfoError> {
    // Chrome DevToolsを試行
    if ChromeDevToolsExtractor::is_available().await {
        println!("🚀 Using Chrome DevTools Protocol");
        return ChromeDevToolsExtractor::extract_browser_info().await;
    }
    
    println!("⚠️ Chrome DevTools not available, falling back to sync method");
    // フォールバックとして既存の同期版を使用
    get_active_browser_info()
}

#[cfg(feature = "devtools")]
pub async fn get_browser_info_fast() -> Result<BrowserInfo, BrowserInfoError> {
    ChromeDevToolsExtractor::extract_browser_info().await
}

/// 互換性・安全性重視（通常ブラウザで動作）
pub fn get_browser_info_safe() -> Result<BrowserInfo, BrowserInfoError> {
    // PowerShell方式（同期）
    get_active_browser_info()
}

/// デフォルト（自動判定・推奨）
pub async fn get_browser_info() -> Result<BrowserInfo, BrowserInfoError> {
    get_active_browser_info_async().await
}

/// 明示的な方法指定
pub async fn get_browser_info_with_method(method: ExtractionMethod) -> Result<BrowserInfo, BrowserInfoError> {
    match method {
        ExtractionMethod::Auto => get_browser_info().await,
        #[cfg(feature = "devtools")]
        ExtractionMethod::DevTools => get_browser_info_fast().await,
        #[cfg(not(feature = "devtools"))]
        ExtractionMethod::DevTools => {
            Err(BrowserInfoError::Other("DevTools feature not enabled".to_string()))
        }
        ExtractionMethod::PowerShell => get_browser_info_safe(),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
