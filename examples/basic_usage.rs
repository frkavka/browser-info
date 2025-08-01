use browser_info::{
    ExtractionMethod, get_browser_info, get_browser_info_safe, get_browser_info_with_method,
};
use browser_info::{get_active_browser_info, get_active_browser_url, is_browser_active};

#[cfg(all(feature = "devtools", target_os = "windows"))]
use browser_info::get_browser_info_fast;

use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Browser Info Library - Basic Usage Demo");
    println!("==========================================");

    // Step 1: 操作用の時間
    println!("\n📋 Instructions:");
    println!("1. Open a browser (Chrome, Firefox, Edge, etc.)");
    println!("2. Navigate to any website");
    println!("3. Wait for the countdown to finish");
    println!("4. When it says 'NOW!', quickly click on the browser window");
    println!("\n⏰ Starting in 5 seconds...");

    // カウントダウン
    for i in (1..=5).rev() {
        println!("   {i} seconds...");
        thread::sleep(Duration::from_secs(1));
    }

    println!("\n🚀 NOW! Quickly click on your browser window!");
    thread::sleep(Duration::from_millis(2000)); // 2秒の猶予

    // Step 2: ブラウザチェック
    println!("\n🔍 Checking for active browser...");
    if !is_browser_active() {
        println!("❌ No browser window detected as active");
        println!("\n🔄 Let's try a different approach...");

        // リトライループ
        println!("📝 Please click on your browser window, then press Enter here...");
        println!("   (You have 10 seconds to switch)");

        // 10秒間、1秒おきにチェック
        for i in (1..=10).rev() {
            thread::sleep(Duration::from_secs(1));
            if is_browser_active() {
                println!("✅ Browser detected!");
                break;
            }
            if i > 1 {
                println!("   Checking... {} seconds left", i - 1);
            }
        }

        // 最終チェック
        if !is_browser_active() {
            println!("❌ Still no browser detected");
            println!("💡 Try running this command while keeping a browser open:");
            println!("   cargo run --example basic_usage");
            return Ok(());
        }
    } else {
        println!("✅ Browser detected immediately!");
    }

    // Step 3: URL取得テスト
    println!("\n🔗 Testing URL extraction...");
    match get_active_browser_url() {
        Ok(url) => {
            println!("✅ URL extracted: {url}");
        }
        Err(e) => {
            println!("⚠️  URL extraction failed: {e}");
            println!("   (This is expected with current dummy implementation)");
        }
    }

    // Step 4: 完全な情報取得
    println!("\n📋 Testing full browser info extraction...");
    match get_active_browser_info() {
        Ok(info) => {
            println!("✅ Full information extracted:");
            println!("   🔗 URL: {url}", url = info.url);
            println!("   📝 Title: {title}", title = info.title);
            println!(
                "   🌐 Browser: {} ({:?})",
                info.browser_name, info.browser_type
            );
            println!(
                "   🆔 Process ID: {process_id}",
                process_id = info.process_id
            );
            println!(
                "   📐 Position: ({:.0}, {:.0})",
                info.window_position.x, info.window_position.y
            );
            println!(
                "   📏 Size: {:.0}x{:.0}",
                info.window_position.width, info.window_position.height
            );

            if info.is_incognito {
                println!("   🔒 Private browsing: Yes");
            }
        }
        Err(e) => {
            println!("❌ Full info extraction failed: {e}");
        }
    }

    // 新しいテスト群
    println!("\n🎛️ Testing different extraction methods...");

    // 1. 自動選択
    println!("\n1️⃣ Auto method:");
    match get_browser_info().await {
        Ok(info) => println!(
            "   ✅ Auto: {browser_name} - {title}",
            browser_name = info.browser_name,
            title = info.title
        ),
        Err(e) => println!("   ❌ Auto failed: {e}"),
    }

    // 2. 高速モード (Windows only)
    #[cfg(all(feature = "devtools", target_os = "windows"))]
    {
        println!("\n2️⃣ Fast method (DevTools - Windows only):");
        match get_browser_info_fast().await {
            Ok(info) => println!(
                "   ✅ Fast: {browser_name} - {title}",
                browser_name = info.browser_name,
                title = info.title
            ),
            Err(e) => println!("   ❌ Fast failed: {e}"),
        }
    }

    #[cfg(not(all(feature = "devtools", target_os = "windows")))]
    {
        println!("\n2️⃣ Fast method: Not available on this platform (Windows only)");
    }

    // 3. 安全モード (Cross-platform)
    println!("\n3️⃣ Safe method (Cross-platform):");
    match get_browser_info_safe() {
        Ok(info) => println!(
            "   ✅ Safe: {browser_name} - {title}",
            browser_name = info.browser_name,
            title = info.title
        ),
        Err(e) => println!("   ❌ Safe failed: {e}"),
    }

    // 4. 明示的指定
    println!("\n4️⃣ Explicit method specification:");
    for method in [
        ExtractionMethod::Auto,
        ExtractionMethod::DevTools,
        ExtractionMethod::PowerShell,
    ] {
        match get_browser_info_with_method(method).await {
            Ok(info) => println!(
                "   ✅ {method:?}: {browser_name} - {title}",
                browser_name = info.browser_name,
                title = info.title
            ),
            Err(e) => println!("   ❌ {method:?} failed: {e}"),
        }
    }

    println!("\n🎯 Test completed!");
    println!("💡 Notes:");
    println!("   • DevTools methods require Chrome with --remote-debugging-port=9222");
    println!("   • DevTools and some methods are Windows-only");
    println!("   • macOS uses AppleScript, Linux support is planned");

    Ok(())
}
