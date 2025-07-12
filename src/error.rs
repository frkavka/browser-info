// ================================================================================================
// Error type Definition  - エラー型定義
// ================================================================================================

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BrowserInfoError {
    /// No active window found
    #[error("No active window found")]
    WindowNotFound,
    
    /// Active window is not a browser
    #[error("Active window is not a browser")]
    NotABrowser,
    
    /// Failed to extract URL from browser
    #[error("Failed to extract URL from browser: {0}")]
    UrlExtractionFailed(String),
    
    /// Browser detection failed
    #[error("Browser detection failed: {0}")]
    BrowserDetectionFailed(String),
    
    /// Platform-specific error
    #[error("Platform-specific error: {0}")]
    PlatformError(String),
    
    /// Invalid URL format
    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),
    
    /// Timeout during operation
    #[error("Timeout during operation")]
    Timeout,
    
    /// Permission denied (e.g., accessibility permissions on macOS)
    #[error("Permission denied")]
    PermissionDenied,
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("JSON parse error: {0}")]
    ParseError(String),
    
    #[error("No active tabs found")]
    NoActiveTabs,
    
    #[error("Chrome DevTools not available")]
    ChromeDevToolsNotAvailable,
    
    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

pub type BrowserError = BrowserInfoError;