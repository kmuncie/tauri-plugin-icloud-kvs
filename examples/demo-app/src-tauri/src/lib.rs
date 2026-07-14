//! Demo app for tauri-plugin-icloud-kvs. All app logic lives in the
//! frontend (`../src/main.ts`); this crate only registers the plugin.

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
   tauri::Builder::default()
      .plugin(tauri_plugin_icloud_kvs::init())
      .run(tauri::generate_context!())
      .expect("error while running demo app");
}
