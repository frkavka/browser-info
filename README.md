# browser-info

🚀 Cross-platform library for retrieving active browser URL and detailed information.

Fast, reliable, and easy-to-use browser information extraction with multiple strategies.

## ✨ Features

- ⚡ **Ultra Fast**: PowerShell-based extraction (sub-millisecond performance)
- 🔧 **DevTools Support**: Chrome DevTools Protocol for advanced scenarios
- 🌐 **Multi-Browser**: Chrome, Firefox, Edge, Safari, Brave, Opera, Vivaldi
- 🎛️ **Multiple Strategies**: Choose between speed, compatibility, or detailed info
- 🔄 **Auto Fallback**: Intelligent method selection with graceful fallbacks
- 🖥️ **Windows Ready**: Native Windows support (macOS/Linux planned)

## 🚀 Quick Start

### Basic Usage

```rust
use browser_info::get_browser_info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let info = get_browser_info().await?;
    
    println!("📖 Title: {}", info.title);
    println!("🔗 URL: {}", info.url);
    println!("🌐 Browser: {}", info.browser_name);
    println!("📍 Position: ({}, {})", info.window_position.x, info.window_position.y);
    
    Ok(())
}
```

### Method Selection

```rust
use browser_info::{get_browser_info_safe, get_browser_info_detailed, ExtractionMethod, get_browser_info_with_method};

// Fast & Compatible (PowerShell - Recommended)
let info = get_browser_info_safe()?;

// Detailed Info (Chrome DevTools - Requires debug mode)
let info = get_browser_info_detailed().await?;

// Explicit Method Selection
let info = get_browser_info_with_method(ExtractionMethod::PowerShell).await?;
```

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
browser-info = "0.1"
tokio = { version = "1.0", features = ["full"] }
```

## 🎛️ Extraction Methods

| Method | Speed | Setup Required | Best For |
|--------|-------|----------------|----------|
| **Auto** | ⚡ Fast | None | General use (recommended) |
| **PowerShell** | ⚡ Ultra Fast | None | Production, reliability |
| **DevTools** | 🔧 Moderate | Chrome debug mode | Advanced info, no UI interference |

### Chrome DevTools Setup (Optional)

For enhanced functionality, start Chrome with debug mode:

```bash
chrome.exe --remote-debugging-port=9222 --user-data-dir=temp
```

## 📊 Performance

Based on our benchmarks:

- **PowerShell**: ~0.4ms (sub-millisecond)
- **DevTools**: ~300ms (network overhead)
- **Auto**: Uses fastest available method

## 🔍 Examples

Check out `/examples` for more usage patterns:

```bash
cargo run --example basic_usage
```

## 🧪 Benchmarking

Run performance tests:

```bash
cargo bench
```

View detailed HTML reports in `target/criterion/`.

## 🤝 Contributing

Contributions welcome! Please feel free to submit issues or pull requests.

## 📄 License

Licensed under MIT License. See [LICENSE](LICENSE) for details.

---