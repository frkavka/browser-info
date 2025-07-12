// ================================================================================================
// Logic to detect the active browser - ブラウザ検出ロジック
// ================================================================================================

use crate::{BrowserType, BrowserInfoError};
use active_win_pos_rs::ActiveWindow;

/// Browser metadata extracted from the window
pub struct BrowserMetadata {
    pub version: Option<String>,
    pub tabs_count: Option<u32>,
    pub is_incognito: bool,
}

/// Classify the browser type from window information
pub fn classify_browser(window: &ActiveWindow) -> Result<BrowserType, BrowserInfoError> {

    let app_name = window.app_name.to_lowercase();
    
    let process_path = window.process_path
        .to_str()
        .unwrap_or("")
        .to_lowercase();
    
    // Detailed browser classification
    if app_name.contains("chrome") && !app_name.contains("edge") {
        Ok(BrowserType::Chrome)
    } else if app_name.contains("firefox") {
        Ok(BrowserType::Firefox)
    } else if app_name.contains("msedge") || app_name.contains("edge") {
        Ok(BrowserType::Edge)
    } else if app_name.contains("safari") {
        Ok(BrowserType::Safari)
    } else if app_name.contains("brave") {
        Ok(BrowserType::Brave)
    } else if app_name.contains("opera") {
        Ok(BrowserType::Opera)
    } else if app_name.contains("vivaldi") {
        Ok(BrowserType::Vivaldi)
    } else if is_browser_by_path(&process_path) {
        // Fallback: check by process path
        detect_browser_from_path(&process_path)
    } else {
        Err(BrowserInfoError::NotABrowser)
    }
    
}

/// Get additional browser metadata
pub fn get_browser_metadata(
    window: &ActiveWindow, 
    browser_type: &BrowserType
) -> Result<BrowserMetadata, BrowserInfoError> {
    Ok(BrowserMetadata {
        version: get_browser_version(window, browser_type),
        tabs_count: count_tabs(window, browser_type),
        is_incognito: detect_incognito_mode(window, browser_type),
    })
}

fn is_browser_by_path(path: &str) -> bool {
    let browser_indicators = [
        "chrome", "firefox", "edge", "safari", "brave", "opera", "vivaldi"
    ];
    browser_indicators.iter().any(|&indicator| path.contains(indicator))
}

fn detect_browser_from_path(path: &str) -> Result<BrowserType, BrowserInfoError> {
    if path.contains("chrome") { Ok(BrowserType::Chrome) }
    else if path.contains("firefox") { Ok(BrowserType::Firefox) }
    else if path.contains("edge") { Ok(BrowserType::Edge) }
    else { Ok(BrowserType::Unknown("detected_from_path".to_string())) }
}

fn get_browser_version(_window: &ActiveWindow, _browser_type: &BrowserType) -> Option<String> {
    // TODO: Implement version detection(Not Essential)
    None
}

fn count_tabs(_window: &ActiveWindow, _browser_type: &BrowserType) -> Option<u32> {
    // TODO: Implement tab counting(Not Essential)
    None
}

fn detect_incognito_mode(window: &ActiveWindow, _browser_type: &BrowserType) -> bool {
    // Basic incognito detection from window title
    let title = window.title.to_lowercase();
    title.contains("incognito") || 
    title.contains("private") || 
    title.contains("inprivate")
}