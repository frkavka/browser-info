// ================================================================================================
// src/platform/macos.rs
// ================================================================================================

use crate::url_extraction::ExtractedPage;
use crate::{BrowserInfoError, BrowserType};
use active_win_pos_rs::ActiveWindow;
use std::process::Command;

/// AppleScriptがURLとタブタイトルの間に置く区切り文字（ASCII Record Separator）。
/// URLにもタブのタイトルにも現れない文字を選んでいる。
/// AppleScript側は `(character id 30)` で同じ文字を出力する。
const FIELD_SEPARATOR: char = '\u{1e}';

/// URLだけを返す互換API。`extract_page` を参照。
pub fn extract_url(
    window: &ActiveWindow,
    browser_type: &BrowserType,
) -> Result<String, BrowserInfoError> {
    extract_page(window, browser_type).map(|page| page.url)
}

/// アクティブなブラウザのURLとタブタイトルを取得する。
///
/// タイトルをここで取る理由: macOSでは `ActiveWindow::title`（`kCGWindowName`）が
/// 画面収録の権限を持たないプロセスに対して**エラーではなく空文字**を返すため、
/// ウィンドウ名をタイトルの出所にできない。AppleScriptはURL取得ですでに
/// Automation権限を通しているので、同じ呼び出しでタブタイトルも一緒に取る。
pub fn extract_page(
    window: &ActiveWindow,
    browser_type: &BrowserType,
) -> Result<ExtractedPage, BrowserInfoError> {
    // 1. AppleScript（URLとタイトルを同時に取得）
    let applescript_err = match try_applescript_extraction(browser_type) {
        Ok(page) => return Ok(page),
        Err(e) => e,
    };

    // 2.キーボードシミュレーション（win版と同じアプローチ）
    if let Ok(url) = try_keyboard_extraction() {
        return Ok(ExtractedPage { url, title: None });
    }

    // 3. タイトル推測 (最終手段)
    if let Ok(url) = extract_url_from_title(&window.title) {
        return Ok(ExtractedPage { url, title: None });
    }

    // 全滅。タイトル推測失敗の汎用メッセージより、AppleScriptの実際の失敗理由
    // （権限拒否など）の方が診断に役立つのでそちらを返す
    Err(applescript_err)
}

fn try_applescript_extraction(
    browser_type: &BrowserType,
) -> Result<ExtractedPage, BrowserInfoError> {
    println!("🔧 Attempting AppleScript extraction for {browser_type:?}");
    let script = applescript_for(browser_type)?;
    let stdout = execute_inline_applescript(&script)?;
    parse_page_output(&stdout)
}

/// ブラウザ種別ごとの AppleScript を組み立てる。
/// 出力形式は `URL & (character id 30) & title` の1行。
fn applescript_for(browser_type: &BrowserType) -> Result<String, BrowserInfoError> {
    // Chromium系はスクリプティング辞書が共通（active tab of front window）
    let chromium = |app_name: &str| {
        format!(
            r#"tell application "{app_name}"
    if (count of windows) > 0 then
        set t to active tab of front window
        return (URL of t) & (character id 30) & (title of t)
    else
        error "No {app_name} windows open"
    end if
end tell"#
        )
    };

    match browser_type {
        BrowserType::Chrome => Ok(chromium("Google Chrome")),
        BrowserType::Edge => Ok(chromium("Microsoft Edge")),
        BrowserType::Brave => Ok(chromium("Brave Browser")),
        BrowserType::Vivaldi => Ok(chromium("Vivaldi")),
        BrowserType::Opera => Ok(chromium("Opera")),
        BrowserType::Safari => Ok(r#"tell application "Safari"
    if (count of windows) > 0 then
        set d to front document
        return (URL of d) & (character id 30) & (name of d)
    else
        error "No Safari windows open"
    end if
end tell"#
            .to_string()),
        BrowserType::Firefox => {
            // FirefoxはAppleScript未対応なのでキーボード方式にフォールバック
            Err(BrowserInfoError::PlatformError(
                "Firefox does not support AppleScript, trying keyboard method".to_string(),
            ))
        }
        _ => Err(BrowserInfoError::PlatformError(format!(
            "Unsupported browser for AppleScript: {browser_type:?}"
        ))),
    }
}

/// osascriptの失敗をエラー種別に分類する。
/// Automation権限が未許可の場合、stderrに"Not authorized"やエラーコード-1743が
/// 含まれるため、これを`PermissionDenied`として区別し、呼び出し元が
/// 「システム設定でオートメーション権限を確認してください」のような
/// 具体的な案内を出せるようにする。
fn osascript_error(stderr: &str, exit_status: String) -> BrowserInfoError {
    let stderr_lower = stderr.to_lowercase();
    if stderr_lower.contains("not authorized") || stderr.contains("-1743") {
        BrowserInfoError::PermissionDenied
    } else if stderr.trim().is_empty() {
        BrowserInfoError::PlatformError(format!("AppleScript failed with exit code: {exit_status}"))
    } else {
        BrowserInfoError::PlatformError(format!(
            "AppleScript failed with exit code {exit_status}: {}",
            stderr.trim()
        ))
    }
}

/// インライン AppleScript を実行し、標準出力を返す
fn execute_inline_applescript(script: &str) -> Result<String, BrowserInfoError> {
    use std::time::{Duration, Instant};

    let start_time = Instant::now();
    let timeout = Duration::from_secs(5);

    println!("🔧 Executing inline AppleScript...");

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| {
            BrowserInfoError::PlatformError(format!("AppleScript execution error: {e}"))
        })?;

    if start_time.elapsed() > timeout {
        return Err(BrowserInfoError::Timeout);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        println!("⚠️ AppleScript stderr: {stderr}");
    }

    if !output.status.success() {
        return Err(osascript_error(&stderr, output.status.to_string()));
    }

    String::from_utf8(output.stdout).map_err(|e| {
        BrowserInfoError::PlatformError(format!("AppleScript output parsing error: {e}"))
    })
}

/// `URL␞title` 形式の AppleScript 出力を解析する。
///
/// 区切り文字が無い出力は「タイトル無し」ではなく書式不一致として失敗させる
/// （このモジュールが組み立てたスクリプトは必ず区切り文字を出力する）。
/// タイトルが空文字のときだけ `title: None` にする。
fn parse_page_output(stdout: &str) -> Result<ExtractedPage, BrowserInfoError> {
    let line = stdout.trim_end_matches(['\n', '\r']);

    let Some((url, title)) = line.split_once(FIELD_SEPARATOR) else {
        return Err(BrowserInfoError::UrlExtractionFailed(format!(
            "AppleScript output lacks the URL/title separator: {line}"
        )));
    };

    let url = url.trim();
    if !(url.starts_with("http") || url.starts_with("file://")) {
        return Err(BrowserInfoError::InvalidUrl(format!(
            "Invalid URL format from AppleScript: {url}"
        )));
    }

    let title = title.trim();
    let title = if title.is_empty() {
        println!("⚠️ AppleScript returned an empty tab title for {url}");
        None
    } else {
        Some(title.to_string())
    };

    Ok(ExtractedPage {
        url: url.to_string(),
        title,
    })
}

fn try_keyboard_extraction() -> Result<String, BrowserInfoError> {
    // TODO: macOS版キーボードシミュレーション（実機テスト後に実装）
    // 現在はAppleScript優先のため、フォールバックとして実装予定
    println!("⚠️ Keyboard simulation fallback - not yet implemented for macOS");
    Err(BrowserInfoError::PlatformError(
        "Keyboard extraction not implemented - AppleScript method preferred".to_string(),
    ))
}

/// タイトルからのURL推測（最終フォールバック）
fn extract_url_from_title(title: &str) -> Result<String, BrowserInfoError> {
    println!("🔍 macOS fallback: extracting URL from title: {title}");

    let title_lower = title.to_lowercase();

    // 一般的なサイトのURL推測（Windows版と同様）
    if title_lower.contains("claude") {
        Ok("https://claude.ai/chat".to_string())
    } else if title_lower.contains("github") {
        Ok("https://github.com".to_string())
    } else if title_lower.contains("google") {
        Ok("https://www.google.com".to_string())
    } else if title_lower.contains("youtube") {
        Ok("https://www.youtube.com".to_string())
    } else if title_lower.contains("stackoverflow") {
        Ok("https://stackoverflow.com".to_string())
    } else if title_lower.contains("twitter") || title_lower.contains("x.com") {
        Ok("https://x.com".to_string())
    } else if title_lower.contains("reddit") {
        Ok("https://www.reddit.com".to_string())
    } else {
        Err(BrowserInfoError::UrlExtractionFailed(format!(
            "Cannot determine URL from macOS title: {title}"
        )))
    }
}

// Clipboard functionality removed due to security vulnerability RUSTSEC-2020-0097
// TODO: Implement clipboard functionality using native macOS APIs if needed

// 将来のキーボードシミュレーション実装用（現在は未使用）
#[allow(dead_code)]
fn get_clipboard_content() -> Result<String, BrowserInfoError> {
    // TODO: Implement using native macOS Pasteboard APIs
    Err(BrowserInfoError::PlatformError(
        "Clipboard functionality not implemented".to_string(),
    ))
}

#[allow(dead_code)]
fn set_clipboard_content(_content: &str) -> Result<(), BrowserInfoError> {
    // TODO: Implement using native macOS Pasteboard APIs
    Err(BrowserInfoError::PlatformError(
        "Clipboard functionality not implemented".to_string(),
    ))
}

#[allow(dead_code)]
fn simulate_key_combination(_keys: &[u32]) -> Result<(), BrowserInfoError> {
    // TODO: Core Graphics実装（実機テスト必要）
    Err(BrowserInfoError::PlatformError(
        "Key simulation not implemented".to_string(),
    ))
}

#[allow(dead_code)]
fn simulate_key_press(_key: u32) -> Result<(), BrowserInfoError> {
    // TODO: Core Graphics実装（実機テスト必要）
    Err(BrowserInfoError::PlatformError(
        "Key simulation not implemented".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_url_and_title() {
        let page = parse_page_output("https://example.com/a?b=1\u{1e}Example — Site\n").unwrap();
        assert_eq!(page.url, "https://example.com/a?b=1");
        assert_eq!(page.title.as_deref(), Some("Example — Site"));
    }

    #[test]
    fn empty_title_becomes_none() {
        let page = parse_page_output("https://example.com\u{1e}\n").unwrap();
        assert_eq!(page.url, "https://example.com");
        assert_eq!(page.title, None);
    }

    #[test]
    fn missing_separator_is_a_format_error() {
        // 旧形式（URLのみ）はタイトル無しではなく書式不一致として落とす
        let err = parse_page_output("https://example.com\n").unwrap_err();
        assert!(
            matches!(err, BrowserInfoError::UrlExtractionFailed(_)),
            "{err:?}"
        );
    }

    #[test]
    fn non_url_is_rejected() {
        let err = parse_page_output("about:blank\u{1e}New Tab\n").unwrap_err();
        assert!(matches!(err, BrowserInfoError::InvalidUrl(_)), "{err:?}");
    }

    #[test]
    fn every_scriptable_browser_emits_the_separator() {
        for browser in [
            BrowserType::Chrome,
            BrowserType::Edge,
            BrowserType::Brave,
            BrowserType::Vivaldi,
            BrowserType::Opera,
            BrowserType::Safari,
        ] {
            let script = applescript_for(&browser).unwrap();
            assert!(
                script.contains("(character id 30)"),
                "{browser:?}: {script}"
            );
        }
        assert!(applescript_for(&BrowserType::Firefox).is_err());
    }
}
