#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(feature = "devtools")]
pub mod chrome_devtools;

// 将来の拡張用
// pub mod firefox_remote;
