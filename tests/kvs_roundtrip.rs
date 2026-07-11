//! Real NSUbiquitousKeyValueStore round-trip.
//!
//! `NSUbiquitousKeyValueStore` only has a functional backing store (local
//! or synced) inside a process that is code-signed with the
//! `com.apple.developer.ubiquity-kvstore-identifier` entitlement. A plain
//! `cargo test` binary is ad-hoc signed with no entitlements and no
//! `Info.plist`, so `-setObject:forKey:` / `-synchronize` silently no-op
//! and every read returns nil — this is true on every machine, not just
//! CI runners. The store-touching test below is gated behind
//! `KVS_INTEGRATION` and must be run from a signed, entitled host (e.g. a
//! Tauri dev build) to exercise the real store. See `DEVELOPERS.md`.

#![cfg(target_os = "macos")]

use serde_json::json;
use tauri_plugin_icloud_kvs as kvs;

#[test]
fn round_trips_set_get_keys_get_all_remove() {
   if std::env::var("KVS_INTEGRATION").is_err() {
      eprintln!(
         "skipping: requires KVS_INTEGRATION=1 and a code-signed host with \
          the ubiquity-kvstore-identifier entitlement (see DEVELOPERS.md)"
      );
      return;
   }

   let key = "m12-roundtrip-test";
   let value = json!({
      "string": "hello",
      "int": 42,
      "float": 1.5,
      "bool": true,
      "list": [1, "two", false],
      "nested": { "a": [true] }
   });

   kvs::set(key, &value).unwrap();

   assert_eq!(kvs::get(key).unwrap(), Some(value));
   assert!(kvs::keys().unwrap().contains(&key.to_string()));
   assert!(kvs::get_all().unwrap().contains_key(key));

   kvs::remove(key).unwrap();

   assert_eq!(kvs::get(key).unwrap(), None);
}

#[test]
fn missing_key_is_none_not_error() {
   assert_eq!(kvs::get("m12-never-written").unwrap(), None);
}

#[test]
fn synchronize_and_account_status_do_not_error() {
   // Value is environment-dependent (signed-in state); just must not error.
   let _flushed: bool = kvs::synchronize().unwrap();
   let _status: kvs::AccountStatus = kvs::account_status().unwrap();
}

#[test]
fn validation_errors_surface_through_the_public_api() {
   assert!(matches!(
      kvs::set("", &json!(1)),
      Err(kvs::Error::InvalidKey(_))
   ));
   assert!(matches!(
      kvs::set(&"k".repeat(65), &json!(1)),
      Err(kvs::Error::InvalidKey(_))
   ));
   assert!(matches!(kvs::get(""), Err(kvs::Error::InvalidKey(_))));
}
