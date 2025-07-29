// ================================================================================================
// src/platform/macos.rs
// ================================================================================================

use crate::{BrowserInfoError, BrowserType};
use active_win_pos_rs::ActiveWindow;
use std::process::Command;

pub fn extract_url(
    window: &ActiveWindow,
    browser_type: &BrowserType,
) -> Result<String, BrowserInfoError> {
    // 1. AppleScript
    if let Ok(url) = try_applescript_extraction(browser_type) {
        return Ok(url);
    }

    // 2.キーボードシミュレーション（win版と同じアプローチ）
    if let Ok(url) = try_keyboard_extraction() {
        return Ok(url);
    }

    // 3. タイトル推測 (最終手段)
    extract_url_from_title(&window.title)
}

fn try_applescript_extraction(browser_type: &BrowserType) -> Result<String, BrowserInfoError> {
    println!(
        "🔧 Attempting AppleScript extraction for {:?}",
        browser_type
    );

    // まず外部スクリプトファイルを試行
    if let Ok(url) = try_external_applescript_file() {
        return Ok(url);
    }

    // フォールバック: インライン AppleScript
    println!("⚠️ External script failed, trying inline AppleScript...");

    let script = match browser_type {
        BrowserType::Chrome => {
            r#"tell application "Google Chrome"
                if (count of windows) > 0 then
                    get URL of active tab of front window
                else
                    error "No Chrome windows open"
                end if
            end tell"#
        }
        BrowserType::Safari => {
            r#"tell application "Safari"
                if (count of windows) > 0 then
                    get URL of front document
                else
                    error "No Safari windows open"
                end if
            end tell"#
        }
        BrowserType::Edge => {
            r#"tell application "Microsoft Edge"
                if (count of windows) > 0 then
                    get URL of active tab of front window
                else
                    error "No Edge windows open"
                end if
            end tell"#
        }
        BrowserType::Brave => {
            r#"tell application "Brave Browser"
                if (count of windows) > 0 then
                    get URL of active tab of front window
                else
                    error "No Brave windows open"
                end if
            end tell"#
        }
        BrowserType::Firefox => {
            // FirefoxはAppleScript未対応なのでキーボード方式にフォールバック
            return Err(BrowserInfoError::PlatformError(
                "Firefox does not support AppleScript, trying keyboard method".to_string(),
            ));
        }
        _ => {
            return Err(BrowserInfoError::PlatformError(format!(
                "Unsupported browser for AppleScript: {:?}",
                browser_type
            )));
        }
    };

    execute_inline_applescript(script)
}

/// 外部AppleScriptファイルを実行
fn try_external_applescript_file() -> Result<String, BrowserInfoError> {
    let script_paths = [
        // メインの場所
        "src/platform/scripts/macos_get_url.scpt",
        // 開発時の相対パス
        "platform/scripts/macos_get_url.scpt",
        "scripts/macos_get_url.scpt",
        // 実行時の相対パス
        "../src/platform/scripts/macos_get_url.scpt",
        "../../src/platform/scripts/macos_get_url.scpt",
    ];

    for script_path in &script_paths {
        if std::path::Path::new(script_path).exists() {
            println!("📁 Found AppleScript file at: {}", script_path);
            return execute_external_applescript_file(script_path);
        }
    }

    Err(BrowserInfoError::PlatformError(
        "AppleScript file not found. Expected at: src/platform/scripts/macos_get_url.scpt"
            .to_string(),
    ))
}

/// 外部AppleScriptファイルを実行
fn execute_external_applescript_file(script_path: &str) -> Result<String, BrowserInfoError> {
    use std::time::{Duration, Instant};

    let start_time = Instant::now();
    let timeout = Duration::from_secs(5);

    println!("🔧 Executing external AppleScript file: {}", script_path);

    let output = Command::new("osascript")
        .arg(script_path)
        .output()
        .map_err(|e| {
            BrowserInfoError::PlatformError(format!("AppleScript file execution error: {}", e))
        })?;

    if start_time.elapsed() > timeout {
        return Err(BrowserInfoError::Timeout);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        println!("⚠️ AppleScript stderr: {}", stderr);
    }

    if !output.status.success() {
        return Err(BrowserInfoError::PlatformError(format!(
            "AppleScript file failed with exit code: {}",
            output.status
        )));
    }

    let stdout = String::from_utf8(output.stdout).map_err(|e| {
        BrowserInfoError::PlatformError(format!("AppleScript output parsing error: {}", e))
    })?;

    parse_applescript_output(&stdout)
}

/// インライン AppleScript を実行
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
            BrowserInfoError::PlatformError(format!("AppleScript execution error: {}", e))
        })?;

    if start_time.elapsed() > timeout {
        return Err(BrowserInfoError::Timeout);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        println!("⚠️ AppleScript stderr: {}", stderr);
    }

    if !output.status.success() {
        return Err(BrowserInfoError::PlatformError(format!(
            "AppleScript failed with exit code: {}",
            output.status
        )));
    }

    let stdout = String::from_utf8(output.stdout).map_err(|e| {
        BrowserInfoError::PlatformError(format!("AppleScript output parsing error: {}", e))
    })?;

    let url = stdout.trim().to_string();

    if url.starts_with("http") || url.starts_with("file://") {
        Ok(url)
    } else {
        Err(BrowserInfoError::InvalidUrl(format!(
            "Invalid URL format from AppleScript: {}",
            url
        )))
    }
}

/// AppleScript出力を解析
fn parse_applescript_output(output: &str) -> Result<String, BrowserInfoError> {
    println!("🔍 Parsing AppleScript output...");

    let lines: Vec<&str> = output.lines().collect();

    // 外部スクリプトの出力形式: "SUCCESS|URL|method" または "ERROR|message|method"
    let result_line = lines
        .iter()
        .rev()
        .find(|line| line.contains("|") && !line.trim().is_empty())
        .unwrap_or(&"")
        .trim();

    if result_line.is_empty() {
        return Err(BrowserInfoError::UrlExtractionFailed(
            "No valid output from AppleScript".to_string(),
        ));
    }

    println!("📤 AppleScript result line: {}", result_line);

    let parts: Vec<&str> = result_line.split('|').collect();

    if parts.len() >= 2 {
        match parts[0] {
            "SUCCESS" => {
                let url = parts[1].trim();
                if url.starts_with("http") || url.starts_with("file://") {
                    println!("✅ AppleScript extraction successful: {}", url);
                    Ok(url.to_string())
                } else {
                    Err(BrowserInfoError::InvalidUrl(format!(
                        "Invalid URL from AppleScript: {}",
                        url
                    )))
                }
            }
            "ERROR" => {
                let error_msg = parts[1].trim();
                Err(BrowserInfoError::PlatformError(format!(
                    "AppleScript error: {}",
                    error_msg
                )))
            }
            _ => {
                // 単純な URL の場合（互換性のため）
                let url = parts[0].trim();
                if url.starts_with("http") || url.starts_with("file://") {
                    Ok(url.to_string())
                } else {
                    Err(BrowserInfoError::UrlExtractionFailed(
                        "Unknown AppleScript output format".to_string(),
                    ))
                }
            }
        }
    } else {
        Err(BrowserInfoError::UrlExtractionFailed(
            "Invalid AppleScript output format".to_string(),
        ))
    }
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
    println!("🔍 macOS fallback: extracting URL from title: {}", title);

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
            "Cannot determine URL from macOS title: {}",
            title
        )))
    }
}

// 将来のキーボードシミュレーション実装用（現在は未使用）
#[allow(dead_code)]
fn get_clipboard_content() -> Result<String, BrowserInfoError> {
    // clipboard crateを使用した実装（実機でテスト必要）
    use clipboard::ClipboardProvider;
    let mut ctx = clipboard::ClipboardContext::new()
        .map_err(|e| BrowserInfoError::PlatformError(format!("Clipboard context error: {}", e)))?;

    ctx.get_contents()
        .map_err(|e| BrowserInfoError::PlatformError(format!("Clipboard read error: {}", e)))
}

#[allow(dead_code)]
fn set_clipboard_content(content: &str) -> Result<(), BrowserInfoError> {
    use clipboard::ClipboardProvider;
    let mut ctx = clipboard::ClipboardContext::new()
        .map_err(|e| BrowserInfoError::PlatformError(format!("Clipboard context error: {}", e)))?;

    ctx.set_contents(content.to_string())
        .map_err(|e| BrowserInfoError::PlatformError(format!("Clipboard write error: {}", e)))
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
