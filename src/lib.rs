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

use active_win_pos_rs::get_active_window;
use serde::{Deserialize, Serialize};

pub mod browser_detection;
pub mod error;
pub mod url_extraction;

pub mod platform;

pub use error::BrowserInfoError;

#[cfg(any(
    all(feature = "devtools", target_os = "windows"),
    all(doc, feature = "devtools")
))]
pub use platform::chrome_devtools::ChromeDevToolsExtractor;

//================================================================================================
// Data Types & Module Variables
//================================================================================================

#[derive(Debug, Clone, Copy)]
pub enum ExtractionMethod {
    /// Auto decision (PowerShell優先 - 推奨)
    Auto,
    /// Chrome DevTools Protocol (詳細情報取得 - デバッグモード必要)
    DevTools,
    /// PowerShell (高速・互換性重視)
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

/// 高速・互換性重視（PowerShell方式）
pub fn get_browser_info_safe() -> Result<BrowserInfo, BrowserInfoError> {
    get_active_browser_info()
}

/// 詳細情報重視（Chrome DevTools - デバッグモード必要）
#[cfg(any(
    all(feature = "devtools", target_os = "windows"),
    all(doc, feature = "devtools")
))]
pub async fn get_browser_info_detailed() -> Result<BrowserInfo, BrowserInfoError> {
    ChromeDevToolsExtractor::extract_browser_info().await
}

/// 後方互換性のためのエイリアス
#[cfg(any(
    all(feature = "devtools", target_os = "windows"),
    all(doc, feature = "devtools")
))]
pub async fn get_browser_info_fast() -> Result<BrowserInfo, BrowserInfoError> {
    get_browser_info_detailed().await
}

/// デフォルト（自動判定・推奨）- PowerShell優先
pub async fn get_browser_info() -> Result<BrowserInfo, BrowserInfoError> {
    // 1. PowerShell方式を最優先（高速・確実）
    match get_browser_info_safe() {
        Ok(info) => {
            println!("✅ Using PowerShell method (fastest)");
            return Ok(info);
        }
        Err(e) => {
            println!("⚠️ PowerShell failed: {e}, trying DevTools...");
        }
    }

    // 2. PowerShell失敗時のみDevTools
    #[cfg(all(feature = "devtools", target_os = "windows"))]
    if ChromeDevToolsExtractor::is_available().await {
        println!("🔄 Fallback to Chrome DevTools Protocol");
        return ChromeDevToolsExtractor::extract_browser_info().await;
    }

    Err(BrowserInfoError::Other(
        "All extraction methods failed".to_string(),
    ))
}

/// 明示的な方法指定
pub async fn get_browser_info_with_method(
    method: ExtractionMethod,
) -> Result<BrowserInfo, BrowserInfoError> {
    match method {
        ExtractionMethod::Auto => get_browser_info().await,
        #[cfg(any(
            all(feature = "devtools", target_os = "windows"),
            all(doc, feature = "devtools")
        ))]
        ExtractionMethod::DevTools => get_browser_info_detailed().await,
        #[cfg(not(any(
            all(feature = "devtools", target_os = "windows"),
            all(doc, feature = "devtools")
        )))]
        ExtractionMethod::DevTools => Err(BrowserInfoError::Other(
            "DevTools feature not available on this platform".to_string(),
        )),
        ExtractionMethod::PowerShell => get_browser_info_safe(),
    }
}
