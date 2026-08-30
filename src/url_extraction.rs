use crate::{BrowserInfoError, BrowserType};
use active_win_pos_rs::ActiveWindow;

/// URL抽出と同じ経路で取れたページ情報。
///
/// `title` はプラットフォーム側が**信頼できる出所**から取れたときだけ `Some`。
/// `None` は「取得手段が無かった」であって「タイトルが空」ではない
/// （空文字のタイトルも `None` に正規化する）。呼び出し側はこのとき
/// ウィンドウ名など別の出所に当たる。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedPage {
    pub url: String,
    pub title: Option<String>,
}

/// Extract URL from the active browser window
pub fn extract_url(
    window: &ActiveWindow,
    browser_type: &BrowserType,
) -> Result<String, BrowserInfoError> {
    extract_page(window, browser_type).map(|page| page.url)
}

/// Extract URL and (where the platform can provide it) the page title
/// from the active browser window
pub fn extract_page(
    window: &ActiveWindow,
    browser_type: &BrowserType,
) -> Result<ExtractedPage, BrowserInfoError> {
    #[cfg(target_os = "windows")]
    {
        crate::platform::windows::extract_page(window, browser_type)
    }

    #[cfg(target_os = "macos")]
    {
        crate::platform::macos::extract_page(window, browser_type)
    }

    #[cfg(target_os = "linux")]
    {
        let _ = (window, browser_type); // Suppress unused variable warnings
        // TODO: Implement Linux URL extraction
        Err(BrowserInfoError::PlatformError(
            "Linux not yet implemented".to_string(),
        ))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        let _ = (window, browser_type); // Suppress unused variable warnings
        Err(BrowserInfoError::PlatformError(
            "Unsupported platform".to_string(),
        ))
    }
}
