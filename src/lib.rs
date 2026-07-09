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

use tauri::{
   Runtime,
   plugin::{Builder, TauriPlugin},
};

/// Initializes the iCloud Key-Value Store plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
   Builder::new("icloud-kvs").build()
}
