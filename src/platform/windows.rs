// ================================================================================================
// src/platform/windows.rs - ローカルscriptsディレクトリ対応
// ================================================================================================

use crate::{BrowserInfoError, BrowserType};
use active_win_pos_rs::ActiveWindow;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

/// Windows環境でのURL・タイトル抽出。
/// Windowsではウィンドウ名がそのままタブタイトルとして信頼できるので、
/// ここでは `title: None` を返し、呼び出し側にウィンドウ名を使わせる。
pub fn extract_page(
    window: &ActiveWindow,
    browser_type: &BrowserType,
) -> Result<crate::url_extraction::ExtractedPage, BrowserInfoError> {
    extract_url(window, browser_type)
        .map(|url| crate::url_extraction::ExtractedPage { url, title: None })
}

/// Windows環境でのURL抽出メイン関数
pub fn extract_url(
    window: &ActiveWindow,
    _browser_type: &BrowserType,
) -> Result<String, BrowserInfoError> {
    println!(
        "🔍 Windows URL extraction for: {app_name}",
        app_name = window.app_name
    );

    // ローカルPowerShellスクリプトを実行
    if let Ok(url) = try_local_powershell_script() {
        println!("✅ Local PowerShell script succeeded: {url}");
        return Ok(url);
    }

    // フォールバック: 内蔵スクリプト
    if let Ok(url) = try_embedded_powershell_script() {
        println!("✅ Embedded PowerShell script succeeded: {url}");
        return Ok(url);
    }

    // 最終フォールバック: タイトルベース
    println!("⚠️  PowerShell extraction failed, using title fallback");
    extract_url_from_title(&window.title)
}

/// ローカルPowerShellスクリプトを実行
fn try_local_powershell_script() -> Result<String, BrowserInfoError> {
    // ローカルスクリプトパスの候補
    let script_paths = [
        // メインの場所
        "src/platform/scripts/windows_get_url.ps1",
        // 開発時の相対パス
        "platform/scripts/windows_get_url.ps1",
        "scripts/windows_get_url.ps1",
        // 実行時の相対パス（targetディレクトリから）
        "../src/platform/scripts/windows_get_url.ps1",
        "../../src/platform/scripts/windows_get_url.ps1",
        "../../../src/platform/scripts/windows_get_url.ps1",
    ];

    for script_path in &script_paths {
        if Path::new(script_path).exists() {
            println!("📁 Found PowerShell script at: {script_path}");
            return execute_powershell_file(script_path);
        }
    }

    Err(BrowserInfoError::PlatformError(
        "PowerShell script not found. Expected at: src/platform/scripts/windows_get_url.ps1"
            .to_string(),
    ))
}

/// PowerShellファイルを実行
fn execute_powershell_file(script_path: &str) -> Result<String, BrowserInfoError> {
    let start_time = Instant::now();
    let timeout = Duration::from_secs(10);

    println!("🔧 Executing PowerShell file: {script_path}");

    let output = Command::new("powershell")
        .args([
            "-ExecutionPolicy",
            "Bypass",
            "-NoProfile",
            "-File",
            script_path,
        ])
        .output()
        .map_err(|e| {
            BrowserInfoError::PlatformError(format!("PowerShell file execution error: {e}"))
        })?;

    if start_time.elapsed() > timeout {
        return Err(BrowserInfoError::Timeout);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        println!("⚠️ PowerShell stderr: {stderr}");
    }

    if !output.status.success() {
        return Err(BrowserInfoError::PlatformError(format!(
            "PowerShell script failed with exit code: {}",
            output.status
        )));
    }

    let stdout = String::from_utf8(output.stdout).map_err(|e| {
        BrowserInfoError::PlatformError(format!("PowerShell output parsing error: {e}"))
    })?;

    parse_atode_powershell_output(&stdout)
}

/// 内蔵PowerShellスクリプト（フォールバック）
fn try_embedded_powershell_script() -> Result<String, BrowserInfoError> {
    println!("🔧 Falling back to embedded PowerShell script...");

    let script = r#"
        [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
        Add-Type -AssemblyName System.Windows.Forms
        
        Add-Type -TypeDefinition @"
            using System;
            using System.Runtime.InteropServices;
            public class BrowserAPI {
                [DllImport("user32.dll")] public static extern void keybd_event(byte bVk, byte bScan, int dwFlags, int dwExtraInfo);
                public const int KEYEVENTF_KEYUP = 0x0002;
                public const byte VK_CONTROL = 0x11;
                public const byte VK_L = 0x4C;
                public const byte VK_C = 0x43;
                public const byte VK_ESCAPE = 0x1B;
            }
"@
        
        try {
            $originalClipboard = ""
            try { $originalClipboard = [System.Windows.Forms.Clipboard]::GetText() } catch {}
            
            # Ctrl+L -> Ctrl+C
            [BrowserAPI]::keybd_event([BrowserAPI]::VK_CONTROL, 0, 0, 0)
            [BrowserAPI]::keybd_event([BrowserAPI]::VK_L, 0, 0, 0)
            Start-Sleep -Milliseconds 50
            [BrowserAPI]::keybd_event([BrowserAPI]::VK_C, 0, 0, 0)
            [BrowserAPI]::keybd_event([BrowserAPI]::VK_L, 0, [BrowserAPI]::KEYEVENTF_KEYUP, 0)
            [BrowserAPI]::keybd_event([BrowserAPI]::VK_C, 0, [BrowserAPI]::KEYEVENTF_KEYUP, 0)
            [BrowserAPI]::keybd_event([BrowserAPI]::VK_CONTROL, 0, [BrowserAPI]::KEYEVENTF_KEYUP, 0)
            Start-Sleep -Milliseconds 100
            
            $url = [System.Windows.Forms.Clipboard]::GetText().Trim()
            
            # Clear selection
            [BrowserAPI]::keybd_event([BrowserAPI]::VK_ESCAPE, 0, 0, 0)
            [BrowserAPI]::keybd_event([BrowserAPI]::VK_ESCAPE, 0, [BrowserAPI]::KEYEVENTF_KEYUP, 0)
            
            # Restore clipboard
            try { if ($originalClipboard) { [System.Windows.Forms.Clipboard]::SetText($originalClipboard) } } catch {}
            
            if ($url -and (($url -match '^https?://') -or ($url -match '^file://'))) {
                Write-Output "SUCCESS|$url|embedded"
            } else {
                Write-Output "FAILED|Invalid URL format: $url|embedded"
            }
        } catch {
            Write-Output "ERROR|$($_.Exception.Message)|embedded"
        }
    "#;

    execute_embedded_powershell_script(script)
}

/// 内蔵PowerShellスクリプト実行
fn execute_embedded_powershell_script(script: &str) -> Result<String, BrowserInfoError> {
    let start_time = Instant::now();
    let timeout = Duration::from_secs(5);

    let output = Command::new("powershell")
        .args([
            "-ExecutionPolicy",
            "Bypass",
            "-NoProfile",
            "-Command",
            script,
        ])
        .output()
        .map_err(|e| {
            BrowserInfoError::PlatformError(format!("Embedded PowerShell execution error: {e}"))
        })?;

    if start_time.elapsed() > timeout {
        return Err(BrowserInfoError::Timeout);
    }

    if !output.status.success() {
        return Err(BrowserInfoError::PlatformError(
            "Embedded PowerShell script failed".to_string(),
        ));
    }

    let stdout = String::from_utf8(output.stdout).map_err(|e| {
        BrowserInfoError::PlatformError(format!("Embedded script output parsing error: {e}"))
    })?;

    parse_simple_powershell_output(&stdout)
}

/// AtodeスタイルのPowerShell出力解析
fn parse_atode_powershell_output(output: &str) -> Result<String, BrowserInfoError> {
    println!("🔍 Parsing Atode-style PowerShell output...");

    let lines: Vec<&str> = output.lines().collect();

    // Atodeの出力形式: "URL|Title|ProcessName"
    let result_line = lines
        .iter()
        .rev()
        .find(|line| line.contains("|") && !line.trim().is_empty())
        .unwrap_or(&"")
        .trim();

    if result_line.is_empty() {
        return Err(BrowserInfoError::UrlExtractionFailed(
            "No valid output from Atode PowerShell script".to_string(),
        ));
    }

    println!("📤 PowerShell result line: {result_line}");

    let parts: Vec<&str> = result_line.split('|').collect();

    if !parts.is_empty() {
        let url = parts[0].trim();

        // エラーチェック
        if url.starts_with("ERROR") {
            let error_msg = parts.get(1).unwrap_or(&"Unknown error").trim();
            return Err(BrowserInfoError::PlatformError(error_msg.to_string()));
        }

        if url.starts_with("NOT_BROWSER") {
            return Err(BrowserInfoError::NotABrowser);
        }

        // 正常なURL
        if url.starts_with("http") || url.starts_with("file://") {
            let title = parts.get(1).unwrap_or(&"").trim();
            let process = parts.get(2).unwrap_or(&"").trim();

            println!("✅ Parsed - URL: {url}, Title: {title}, Process: {process}",);
            Ok(url.to_string())
        } else {
            Err(BrowserInfoError::InvalidUrl(format!(
                "Invalid URL format from script: {url}",
            )))
        }
    } else {
        Err(BrowserInfoError::UrlExtractionFailed(
            "Invalid Atode PowerShell output format".to_string(),
        ))
    }
}

/// 簡単なPowerShell出力解析
fn parse_simple_powershell_output(output: &str) -> Result<String, BrowserInfoError> {
    let lines: Vec<&str> = output.lines().collect();
    let result_line = lines
        .iter()
        .rev()
        .find(|line| line.contains("|"))
        .unwrap_or(&"")
        .trim();

    if result_line.is_empty() {
        return Err(BrowserInfoError::UrlExtractionFailed(
            "No output from embedded PowerShell script".to_string(),
        ));
    }

    let parts: Vec<&str> = result_line.split('|').collect();

    if parts.len() >= 2 {
        match parts[0] {
            "SUCCESS" => {
                let url = parts[1].trim();
                if url.starts_with("http") || url.starts_with("file://") {
                    Ok(url.to_string())
                } else {
                    Err(BrowserInfoError::InvalidUrl(url.to_string()))
                }
            }
            "FAILED" => Err(BrowserInfoError::UrlExtractionFailed(parts[1].to_string())),
            "ERROR" => Err(BrowserInfoError::PlatformError(parts[1].to_string())),
            _ => Err(BrowserInfoError::UrlExtractionFailed(
                "Unknown embedded script output format".to_string(),
            )),
        }
    } else {
        Err(BrowserInfoError::UrlExtractionFailed(
            "Invalid embedded PowerShell output".to_string(),
        ))
    }
}

/// タイトルからのURL推測（最終フォールバック）
fn extract_url_from_title(title: &str) -> Result<String, BrowserInfoError> {
    println!("🔍 Final fallback: extracting URL from title: {title}");

    let title_lower = title.to_lowercase();

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
    } else {
        Err(BrowserInfoError::UrlExtractionFailed(format!(
            "Cannot determine URL from title: {title}",
        )))
    }
}
