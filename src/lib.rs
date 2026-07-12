//! Tauri 2 plugin exposing Apple's iCloud Key-Value Store
//! (`NSUbiquitousKeyValueStore`) on macOS and iOS.
//!
//! Lets a Tauri app sync small data (1 MB total, 1024 keys) across a
//! user's Apple devices with no server and no user accounts.
//!
//! # Examples
//!
//! ```no_run
//! tauri::Builder::default()
//!    .plugin(tauri_plugin_icloud_kvs::init());
//! ```

mod commands;
#[cfg(target_os = "macos")]
mod conversion;
mod error;
mod models;
mod store;
mod validation;

pub use error::{Error, Result};
pub use models::AccountStatus;

/// The plugin's public Rust API. App frontends running in the webview
/// should use the guest bindings (TypeScript) instead; this API is for
/// Rust code running in the Tauri host process.
pub use store::{account_status, get, get_all, keys, remove, set, synchronize};

use tauri::{
   Runtime,
   plugin::{Builder, TauriPlugin},
};

/// Initializes the iCloud Key-Value Store plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
   Builder::new("icloud-kvs")
      .invoke_handler(tauri::generate_handler![
         commands::get,
         commands::set,
         commands::remove,
         commands::keys,
         commands::get_all,
         commands::synchronize,
         commands::account_status
      ])
      .build()
}
