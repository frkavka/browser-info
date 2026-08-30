# browser-info

[![Rust CI](https://github.com/frkavka/browser-info/actions/workflows/rust.yml/badge.svg)](https://github.com/frkavka/browser-info/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/browser-info.svg)](https://crates.io/crates/browser-info)
[![Documentation](https://docs.rs/browser-info/badge.svg)](https://docs.rs/browser-info)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 🏆 Used by
- **[Atode-GUI(mine)](https://github.com/frkavka/Atode-GUI)** - Read-later article management tool with quick save functionality

*If you use browser-info in your project, let me know and I'll add you to the list!*

🚀 Cross-platform library for retrieving active browser URL and detailed information.

Fast, reliable, and easy-to-use browser information extraction with multiple strategies.

## ✨ Features

- ⚡ **Ultra Fast**: PowerShell-based extraction (sub-millisecond performance)
- 🔧 **DevTools Support**: Chrome DevTools Protocol for advanced scenarios (Windows only)
- 🌐 **Multi-Browser**: Chrome, Firefox, Edge, Safari, Brave, Opera, Vivaldi
- 🎛️ **Multiple Strategies**: Choose between speed, compatibility, or detailed info
- 🔄 **Auto Fallback**: Intelligent method selection with graceful fallbacks
- 🖥️ **Cross-Platform**: Windows (full support), macOS (partial), Linux (planned)
- 🔒 **Security-First**: No vulnerable dependencies, regular security audits

## 🚀 Quick Start

### Basic Usage

```rust
use browser_info::get_active_browser_info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if a browser is currently active
    if !browser_info::is_browser_active() {
        println!("No browser is currently active");
        return Ok(());
    }
    
    // Get comprehensive browser information
    let info = get_active_browser_info()?;
    
    println!("📖 Title: {}", info.title);
    println!("🔗 URL: {}", info.url);
    println!("🌐 Browser: {:?}", info.browser_type);
    println!("📍 Position: ({}, {})", info.window_position.x, info.window_position.y);
    println!("🔒 Incognito: {}", info.is_incognito);
    
    Ok(())
}
```

### Async API (with DevTools support)

```rust
use browser_info::{get_browser_info, ExtractionMethod};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Auto-detection with fallback (PowerShell → DevTools)
    let info = get_browser_info().await?;
    println!("URL: {}", info.url);
    
    Ok(())
}
```

### Method Selection

```rust
use browser_info::{get_browser_info_safe, ExtractionMethod, get_browser_info_with_method};

// Fast & Compatible (PowerShell - Recommended)
let info = get_browser_info_safe()?;

// Just get the URL (lightweight)
let url = browser_info::get_active_browser_url()?;

// Explicit Method Selection
let info = get_browser_info_with_method(ExtractionMethod::PowerShell).await?;

// DevTools method (Windows only, requires debug mode)
#[cfg(all(feature = "devtools", target_os = "windows"))]
let info = browser_info::get_browser_info_detailed().await?;
```

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
browser-info = "0.2"

# Optional: for async API and DevTools support
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
```

### Features

- `default = ["devtools"]`: Includes DevTools support (Windows only)
- `devtools`: Chrome DevTools Protocol support (requires `reqwest` and `tokio`)

## 🎛️ Extraction Methods

| Method | Speed | Setup Required | Platform | Best For |
|--------|-------|----------------|----------|----------|
| **Auto** | ⚡ Fast | None | Windows, macOS | General use (recommended) |
| **PowerShell** | ⚡ Ultra Fast | None | Windows | Production, reliability |
| **DevTools** | 🔧 Moderate | Chrome debug mode | Windows only | Advanced info, no UI interference |
| **AppleScript** | 🍎 Fast | None | macOS only | Native macOS support |

### Platform Support

| Platform | Status | Methods Available |
|----------|--------|-------------------|
| **Windows** | ✅ Full | PowerShell, DevTools, Auto |
| **macOS** | 🚧 Partial | AppleScript, Auto |
| **Linux** | ⏳ Planned | Coming soon |

### Chrome DevTools Setup (Optional)

For DevTools method on Windows, start Chrome with debug mode:

```bash
chrome.exe --remote-debugging-port=9222 --user-data-dir=temp
```

## 📊 Performance

Based on our benchmarks:

- **PowerShell**: ~0.4ms (sub-millisecond)
- **AppleScript**: ~50ms (native macOS)
- **DevTools**: ~300ms (network overhead)
- **Auto**: Uses fastest available method per platform

## 🔧 Development

### Building

```bash
# Build with all features
cargo build --all-features

# Build without DevTools (faster compilation)
cargo build --no-default-features

# Platform-specific builds
cargo build --target x86_64-pc-windows-msvc
cargo build --target x86_64-apple-darwin
```

### Testing

```bash
# Run all tests
cargo test --all-features

# Run platform-specific tests
cargo test --features devtools  # Windows only
```

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

## 🛡️ Security

This library prioritizes security:

- ✅ **No vulnerable dependencies** - Regular security audits with `cargo audit`
- ✅ **Safe Rust** - No unsafe code blocks
- ✅ **Input validation** - All external data is validated
- ✅ **CI/CD security** - Automated security scanning in GitHub Actions

## 🐛 Troubleshooting

### Common Issues

**Windows**: "PowerShell execution policy"
```bash
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

**macOS**: `title` is empty
- Since 1.1.0 the tab title comes from the same AppleScript call as the URL, so it no longer depends on the window name (`kCGWindowName`, which macOS returns as an empty string to apps without Screen Recording permission — no error is raised).
- If the URL itself fails with "Permission denied", grant the calling app Automation access to the browser in System Settings > Privacy & Security > Automation.

**DevTools**: "Connection refused"
- Ensure Chrome is running with `--remote-debugging-port=9222`
- Check if port 9222 is not blocked by firewall

### Debug Mode

Enable debug logging:
```rust
env_logger::init();
let info = get_active_browser_info()?;
```

## 🤝 Contributing

Contributions welcome! Please see our [contributing guidelines](CONTRIBUTING.md).

### Development Setup

1. Clone the repository
2. Install Rust (1.87+ required)
3. Run tests: `cargo test --all-features`
4. Submit a pull request

### Roadmap

- [ ] Linux support (X11 and Wayland)
- [ ] Firefox DevTools Protocol support
- [ ] Browser extension API
- [ ] WebDriver integration

## 📄 License

Licensed under MIT License. See [LICENSE](LICENSE) for details.

---

<div align="center">
  <sub>Built with ❤️ by <a href="https://github.com/frkavka">Katy</a></sub>
</div>