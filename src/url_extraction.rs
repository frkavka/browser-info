use crate::{BrowserType, BrowserInfoError};
use active_win_pos_rs::ActiveWindow;

/// Extract URL from the active browser window
pub fn extract_url(
    window: &ActiveWindow, 
    browser_type: &BrowserType
) -> Result<String, BrowserInfoError> {
    
    #[cfg(target_os = "windows")]
    {
        crate::platform::windows::extract_url(window, browser_type)
    }
    
    #[cfg(target_os = "macos")]
    {
        // TODO: Implement macOS URL extraction
        Err(BrowserInfoError::PlatformError("macOS not yet implemented".to_string()))
    }
    
    #[cfg(target_os = "linux")]
    {
        // TODO: Implement Linux URL extraction  
        Err(BrowserInfoError::PlatformError("Linux not yet implemented".to_string()))
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err(BrowserInfoError::PlatformError("Unsupported platform".to_string()))
    }
}