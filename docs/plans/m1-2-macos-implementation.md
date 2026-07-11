# tauri-plugin-icloud-kvs M1.2 (macOS Implementation) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The full command set (`get`/`set`/`remove`/`keys`/`get_all`/`synchronize`/`account_status`) working on macOS against the real `NSUbiquitousKeyValueStore`, with green unit + integration tests, Tauri commands + permissions registered, and TypeScript bindings for every command.

**Architecture:** Pure-Rust macOS implementation via `objc2-foundation` (no Swift/Xcode for desktop). Pure, cross-platform validation logic (`src/validation.rs`) is separated from the `objc2` calls; JSON↔plist conversion lives in its own macOS-only module (`src/conversion.rs`) so `src/desktop.rs` stays focused on store operations (small addition to the spec's file layout). Non-macOS builds get stubs returning `Error::UnsupportedPlatform`. The plugin stays stateless — every call fetches `defaultStore()` fresh; change-event observers arrive in M1.4.

**Tech Stack:** Rust (objc2 0.6.x, objc2-foundation 0.3.x, Tauri 2 plugin API, serde_json, thiserror), TypeScript (`@tauri-apps/api`).

**Spec:** `docs/design-spec.md`. Milestone definition + carryovers: `docs/milestones.md` (M1.2 entry).

## Global Constraints

- Rust 1.89.0, edition 2024; 3-space indent; `rustfmt.toml` / `.cargo/config.toml` already in repo — do not modify
- All new Cargo/npm dependencies pinned to exact semver versions (resolve via `cargo update` + copy from `Cargo.lock`, or `npm view <pkg> version`)
- Errors via the existing `thiserror` enum in `src/error.rs`; doc examples annotated ` ```no_run `
- Unit tests in the same file as the code; integration tests in `tests/`
- Every commit passes `cargo lint-clippy && cargo lint-fmt && cargo test` (and `npm run build` in `guest-js/` when TS changes)
- Commit messages: Conventional Commits, imperative, ≤72-char subject
- Value model: JSON values mapped to plist types; `null` is never storable; raw `NSData` written by other native code reads back as a base64 string (documented edge case)
- Quota exhaustion is NOT a call-site error (notification-only, M1.4); `set` calls `synchronize()` after writing (spec decision: "set requests upload immediately")
- Limits pre-checked in Rust: key non-empty and ≤64 bytes UTF-8; serialized value ≤1 MiB (1,048,576 bytes)
- Carryovers from the M1.1 review (all handled in this plan): rename `Error::PlatformError` → `Error::Platform` before anything constructs it (Task 1); backfill Display/serialization tests for `Serialization` and `Platform` (Task 1); create `permissions/` in the same commit as the first `COMMANDS` entry (Task 6)

---

### Task 1: Error-module carryovers (rename + backfilled tests)

**Files:**
- Modify: `src/error.rs`

**Interfaces:**
- Consumes: existing `Error` enum from M1.1.
- Produces: `Error::Platform(String)` (renamed from `PlatformError`); all five variants covered by Display tests. Tasks 2–5 construct `InvalidKey`, `ValueTooLarge`, `Serialization`, `Platform`, `UnsupportedPlatform`.

- [ ] **Step 1: Extend the tests (failing) in `src/error.rs`**

In the existing `tests` module, add to `display_messages_are_actionable`:

```rust
      assert_eq!(
         Error::Serialization("null is not a storable value".into()).to_string(),
         "serialization error: null is not a storable value"
      );
      assert_eq!(
         Error::Platform("NSUbiquitousKeyValueStore unavailable".into()).to_string(),
         "platform error: NSUbiquitousKeyValueStore unavailable"
      );
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test`
Expected: FAIL to compile — no variant `Platform` on `Error`.

- [ ] **Step 3: Rename the variant**

In the enum, change:

```rust
   #[error("platform error: {0}")]
   Platform(String),
```

(i.e. `PlatformError(String)` → `Platform(String)`; the `#[error]` message is unchanged.)

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test && cargo lint-clippy && cargo lint-fmt`
Expected: all tests PASS, lints clean.

- [ ] **Step 5: Commit**

```bash
git add src/error.rs
git commit -m "refactor: Rename Error::PlatformError to Error::Platform"
```

---

### Task 2: Pure validation module (key + value limits)

**Files:**
- Create: `src/validation.rs`
- Modify: `src/lib.rs` (add `mod validation;`)

**Interfaces:**
- Consumes: `Error`, `Result` from Task 1.
- Produces: `pub(crate) fn validate_key(key: &str) -> Result<()>` and `pub(crate) fn validate_value(value: &serde_json::Value) -> Result<()>`; constants `MAX_KEY_BYTES: usize = 64`, `MAX_VALUE_BYTES: usize = 1_048_576`. Task 5 calls both before touching the store. Pure Rust — compiles and tests on any platform.

- [ ] **Step 1: Write `src/validation.rs` with only the failing tests**

```rust
#[cfg(test)]
mod tests {
   use serde_json::json;

   use super::*;
   use crate::error::Error;

   #[test]
   fn accepts_valid_keys() {
      assert!(validate_key("theme").is_ok());
      assert!(validate_key(&"k".repeat(64)).is_ok());
   }

   #[test]
   fn rejects_empty_key() {
      assert!(matches!(validate_key(""), Err(Error::InvalidKey(_))));
   }

   #[test]
   fn rejects_key_over_64_utf8_bytes() {
      // 65 ASCII bytes
      assert!(matches!(validate_key(&"k".repeat(65)), Err(Error::InvalidKey(_))));
      // 22 chars × 3 bytes = 66 UTF-8 bytes — byte length is what counts
      assert!(matches!(validate_key(&"€".repeat(22)), Err(Error::InvalidKey(_))));
   }

   #[test]
   fn accepts_small_values() {
      assert!(validate_value(&json!({ "a": [1, 2.5, true, "x"] })).is_ok());
   }

   #[test]
   fn rejects_value_over_one_mebibyte() {
      let big = json!("x".repeat(MAX_VALUE_BYTES + 1));

      assert!(matches!(
         validate_value(&big),
         Err(Error::ValueTooLarge { size }) if size > MAX_VALUE_BYTES
      ));
   }
}
```

Add to `src/lib.rs` below `mod error;`:

```rust
mod validation;
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test`
Expected: FAIL to compile — `validate_key` / `validate_value` / `MAX_VALUE_BYTES` not found.

- [ ] **Step 3: Implement above the test module**

```rust
//! Pre-checks for iCloud KVS limits. KVS silently misbehaves instead of
//! erroring, so limits are enforced here before any native call.

use serde_json::Value;

use crate::error::{Error, Result};

/// KVS key names are limited to 64 bytes of UTF-8.
pub(crate) const MAX_KEY_BYTES: usize = 64;

/// The whole store is capped at 1 MB; a single value can never exceed it.
pub(crate) const MAX_VALUE_BYTES: usize = 1_048_576;

pub(crate) fn validate_key(key: &str) -> Result<()> {
   if key.is_empty() {
      return Err(Error::InvalidKey("key must not be empty".into()));
   }

   if key.len() > MAX_KEY_BYTES {
      return Err(Error::InvalidKey(format!(
         "key exceeds {MAX_KEY_BYTES} bytes UTF-8 (got {} bytes)",
         key.len()
      )));
   }

   Ok(())
}

pub(crate) fn validate_value(value: &Value) -> Result<()> {
   let size = serde_json::to_vec(value)
      .map_err(|e| Error::Serialization(e.to_string()))?
      .len();

   if size > MAX_VALUE_BYTES {
      return Err(Error::ValueTooLarge { size });
   }

   Ok(())
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test && cargo lint-clippy && cargo lint-fmt`
Expected: 5 new tests PASS, lints clean. (Clippy may flag `MAX_KEY_BYTES` as unused outside tests until Task 5 — if so, temporarily allow with `#[allow(dead_code)]` on the items and remove the allows in Task 5.)

- [ ] **Step 5: Commit**

```bash
git add src/validation.rs src/lib.rs
git commit -m "feat: Add pure key and value-size validation for KVS limits"
```

---

### Task 3: Models module (`AccountStatus`)

**Files:**
- Create: `src/models.rs`
- Modify: `src/lib.rs` (add `mod models;` + re-export)

**Interfaces:**
- Produces: `pub enum AccountStatus { Available, NoAccount }`, serde-serialized as `"available"` / `"noAccount"` (matches the TS union `'available' | 'noAccount'`). Task 5's `account_status()` returns it; Task 6's command serializes it to the frontend.

- [ ] **Step 1: Write `src/models.rs` with only the failing tests**

```rust
#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn account_status_serializes_as_camel_case() {
      assert_eq!(
         serde_json::to_string(&AccountStatus::Available).unwrap(),
         "\"available\""
      );
      assert_eq!(
         serde_json::to_string(&AccountStatus::NoAccount).unwrap(),
         "\"noAccount\""
      );
   }
}
```

Add to `src/lib.rs` below `mod error;`:

```rust
mod models;
```

and extend the re-exports:

```rust
pub use error::{Error, Result};
pub use models::AccountStatus;
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test`
Expected: FAIL to compile — `AccountStatus` not found.

- [ ] **Step 3: Implement above the test module**

```rust
//! Serde types shared between the Rust commands and the TypeScript API.

use serde::{Deserialize, Serialize};

/// Whether the device is signed in to iCloud.
///
/// A signed-out device silently degrades KVS to local-only storage; this
/// status (via `FileManager.ubiquityIdentityToken`) is the only way
/// callers can detect that condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AccountStatus {
   Available,
   NoAccount,
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test && cargo lint-clippy && cargo lint-fmt`
Expected: PASS, lints clean.

- [ ] **Step 5: Commit**

```bash
git add src/models.rs src/lib.rs
git commit -m "feat: Add AccountStatus model for iCloud sign-in detection"
```

---

### Task 4: objc2 dependencies + JSON↔plist conversion (macOS-only)

**Files:**
- Modify: `Cargo.toml` (macOS-target objc2 dependencies)
- Create: `src/conversion.rs`
- Modify: `src/lib.rs` (add cfg-gated `mod conversion;`)

**Interfaces:**
- Consumes: `Error`, `Result` (Task 1).
- Produces (macOS only): `pub(crate) fn json_to_plist(value: &serde_json::Value) -> Result<Retained<AnyObject>>` and `pub(crate) fn plist_to_json(obj: &AnyObject) -> Result<serde_json::Value>`. Task 5 uses both.
- API facts verified against objc2-foundation 0.3.2 docs: `NSNumber::new_bool/new_i64/new_f64` and `.encoding()`; `AnyObject::downcast_ref::<T>()`; `Retained<T>` upcasts to `Retained<AnyObject>` via `.into()`; `NSArray::from_retained_slice`; `NSDictionary::from_retained_objects(&[&K], &[Retained<V>])`; `NSData::base64EncodedStringWithOptions` (feature `NSString`). If a signature drifted in the pinned patch release, adjust the call — the tests define the contract.

- [ ] **Step 1: Add macOS-target dependencies to `Cargo.toml`**

Append (versions are placeholders — Step 2 pins them):

```toml
[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.6"
objc2-foundation = { version = "0.3", features = [
   "NSArray",
   "NSData",
   "NSDictionary",
   "NSEnumerator",
   "NSFileManager",
   "NSObject",
   "NSString",
   "NSUbiquitousKeyValueStore",
   "NSValue",
] }
```

(`NSValue` provides `NSNumber`; `NSEnumerator` provides dictionary/array iteration; `NSFileManager` + `NSObject` are for Task 5's `account_status`.)

- [ ] **Step 2: Pin exact versions**

```bash
cargo update
grep -A1 'name = "objc2"' Cargo.lock | head -2
grep -A1 'name = "objc2-foundation"' Cargo.lock | head -2
```

Copy the resolved versions into `Cargo.toml` as exact pins (e.g. `objc2 = "0.6.3"`, `objc2-foundation = { version = "0.3.2", ... }`). Re-run `cargo check` to confirm nothing shifted.

- [ ] **Step 3: Write `src/conversion.rs` with only the failing tests**

```rust
#[cfg(test)]
mod tests {
   use objc2_foundation::NSData;
   use serde_json::json;

   use super::*;
   use crate::error::Error;

   #[test]
   fn round_trips_every_json_shape() {
      let value = json!({
         "string": "hello",
         "int": 42,
         "negative": -7,
         "float": 1.5,
         "boolTrue": true,
         "boolFalse": false,
         "list": [1, "two", false],
         "nested": { "inner": [true, 2.25] }
      });

      let plist = json_to_plist(&value).unwrap();

      assert_eq!(plist_to_json(&plist).unwrap(), value);
   }

   #[test]
   fn booleans_stay_booleans_not_numbers() {
      let plist = json_to_plist(&json!(true)).unwrap();

      assert_eq!(plist_to_json(&plist).unwrap(), json!(true));
   }

   #[test]
   fn null_is_rejected_everywhere() {
      assert!(matches!(json_to_plist(&json!(null)), Err(Error::Serialization(_))));
      assert!(matches!(
         json_to_plist(&json!({ "a": null })),
         Err(Error::Serialization(_))
      ));
      assert!(matches!(
         json_to_plist(&json!([1, null])),
         Err(Error::Serialization(_))
      ));
   }

   #[test]
   fn foreign_nsdata_reads_back_as_base64_string() {
      let data = NSData::with_bytes(&[1, 2, 3]);

      assert_eq!(plist_to_json(&data).unwrap(), json!("AQID"));
   }
}
```

Add to `src/lib.rs` below `mod models;`:

```rust
#[cfg(target_os = "macos")]
mod conversion;
```

- [ ] **Step 4: Run tests to verify they fail**

Run: `cargo test`
Expected: FAIL to compile — `json_to_plist` / `plist_to_json` not found.

- [ ] **Step 5: Implement above the test module**

```rust
//! JSON ↔ property-list conversion for the macOS implementation.
//!
//! Storable values map 1:1 onto plist types (NSString, NSNumber,
//! NSArray, NSDictionary). `null` is not storable. Raw `NSData` written
//! by other native code is returned as a base64 string (documented
//! edge case; there is no bytes API in v1).

use objc2::encode::Encoding;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_foundation::{
   NSArray, NSData, NSDataBase64EncodingOptions, NSDictionary, NSNumber, NSString,
};
use serde_json::{Map, Number, Value};

use crate::error::{Error, Result};

pub(crate) fn json_to_plist(value: &Value) -> Result<Retained<AnyObject>> {
   match value {
      Value::Null => Err(Error::Serialization(
         "null is not a storable value; use remove() to delete a key".into(),
      )),
      Value::Bool(b) => Ok(NSNumber::new_bool(*b).into()),
      Value::Number(n) => number_to_plist(n),
      Value::String(s) => Ok(NSString::from_str(s).into()),
      Value::Array(items) => {
         let converted = items.iter().map(json_to_plist).collect::<Result<Vec<_>>>()?;

         Ok(NSArray::from_retained_slice(&converted).into())
      },
      Value::Object(map) => {
         let keys: Vec<Retained<NSString>> = map.keys().map(|k| NSString::from_str(k)).collect();
         let key_refs: Vec<&NSString> = keys.iter().map(|k| &**k).collect();
         let values = map.values().map(json_to_plist).collect::<Result<Vec<_>>>()?;

         Ok(NSDictionary::from_retained_objects(&key_refs, &values).into())
      },
   }
}

fn number_to_plist(n: &Number) -> Result<Retained<AnyObject>> {
   if let Some(i) = n.as_i64() {
      Ok(NSNumber::new_i64(i).into())
   } else if let Some(u) = n.as_u64() {
      Ok(NSNumber::new_u64(u).into())
   } else if let Some(f) = n.as_f64() {
      Ok(NSNumber::new_f64(f).into())
   } else {
      Err(Error::Serialization(format!("unrepresentable JSON number: {n}")))
   }
}

pub(crate) fn plist_to_json(obj: &AnyObject) -> Result<Value> {
   if let Some(s) = obj.downcast_ref::<NSString>() {
      return Ok(Value::String(s.to_string()));
   }

   if let Some(n) = obj.downcast_ref::<NSNumber>() {
      return number_to_json(n);
   }

   if let Some(array) = obj.downcast_ref::<NSArray>() {
      let items = array
         .to_vec()
         .iter()
         .map(|item| plist_to_json(item))
         .collect::<Result<Vec<_>>>()?;

      return Ok(Value::Array(items));
   }

   if let Some(dict) = obj.downcast_ref::<NSDictionary>() {
      let (keys, values) = dict.to_vecs();
      let mut map = Map::with_capacity(keys.len());

      for (key, value) in keys.into_iter().zip(values) {
         let key_string = key
            .downcast_ref::<NSString>()
            .ok_or_else(|| Error::Serialization("non-string dictionary key".into()))?
            .to_string();

         map.insert(key_string, plist_to_json(&value)?);
      }

      return Ok(Value::Object(map));
   }

   if let Some(data) = obj.downcast_ref::<NSData>() {
      let base64 = data.base64EncodedStringWithOptions(NSDataBase64EncodingOptions::empty());

      return Ok(Value::String(base64.to_string()));
   }

   Err(Error::Serialization(format!(
      "unsupported plist type: {:?}",
      obj.class()
   )))
}

fn number_to_json(n: &NSNumber) -> Result<Value> {
   match n.encoding() {
      // CFBoolean reports 'c' (Char); C99 _Bool reports 'B'. An NSNumber
      // wrapping a genuine i8 also reports 'c' and will read back as a
      // boolean — acceptable: this plugin never writes i8, and JSON has
      // no i8 type to preserve.
      Encoding::Char | Encoding::Bool => Ok(Value::Bool(n.as_bool())),
      Encoding::Float | Encoding::Double => Number::from_f64(n.as_f64())
         .map(Value::Number)
         .ok_or_else(|| Error::Serialization("non-finite float in store".into())),
      Encoding::ULongLong => Ok(Value::Number(Number::from(n.as_u64()))),
      _ => Ok(Value::Number(Number::from(n.as_i64()))),
   }
}
```

Notes for the implementer:
- The `.into()` upcasts rely on `impl From<Retained<T>> for Retained<AnyObject>` (T: ClassType + 'static) — verified in objc2 docs. If type inference balks on a chained call, bind to a `let x: Retained<AnyObject>` first.
- `downcast_ref::<NSArray>()` / `::<NSDictionary>()` target the default `AnyObject` generic parameters — this is the supported downcast form for generic Foundation classes.
- If `NSDictionary::to_vecs` or `NSArray::to_vec` need the `NSEnumerator` feature and the compiler says so, it is already enabled in Step 1.

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test && cargo lint-clippy && cargo lint-fmt`
Expected: 4 new conversion tests PASS (they run because the dev machine is macOS), lints clean.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock src/conversion.rs src/lib.rs
git commit -m "feat: Add JSON-to-plist conversion via objc2-foundation"
```

(If `Cargo.lock` is gitignored in this repo, drop it from the `git add`.)

---

### Task 5: Store operations (`desktop.rs`) + public Rust API + integration test

**Files:**
- Create: `src/desktop.rs`
- Modify: `src/lib.rs` (add `mod desktop;` + public re-exports)
- Test: `tests/kvs_roundtrip.rs`

**Interfaces:**
- Consumes: `validate_key` / `validate_value` (Task 2), `AccountStatus` (Task 3), `json_to_plist` / `plist_to_json` (Task 4).
- Produces the crate's public Rust API, re-exported from the root (Task 6's commands and the integration test both call these exact signatures):
  - `pub fn get(key: &str) -> Result<Option<serde_json::Value>>`
  - `pub fn set(key: &str, value: &serde_json::Value) -> Result<()>` — also requests upload (calls `synchronize()` internally per spec)
  - `pub fn remove(key: &str) -> Result<()>`
  - `pub fn keys() -> Result<Vec<String>>`
  - `pub fn get_all() -> Result<serde_json::Map<String, serde_json::Value>>`
  - `pub fn synchronize() -> Result<bool>`
  - `pub fn account_status() -> Result<AccountStatus>`
  - On non-macOS targets every function returns `Err(Error::UnsupportedPlatform)`.

- [ ] **Step 1: Write the failing integration test `tests/kvs_roundtrip.rs`**

```rust
//! Real NSUbiquitousKeyValueStore round-trip. The local store works
//! without an iCloud account or entitlement (it just never syncs), so
//! this runs on GitHub's macOS runners.

#![cfg(target_os = "macos")]

use serde_json::json;
use tauri_plugin_icloud_kvs as kvs;

#[test]
fn round_trips_set_get_keys_get_all_remove() {
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
   assert!(matches!(kvs::set("", &json!(1)), Err(kvs::Error::InvalidKey(_))));
   assert!(matches!(
      kvs::set(&"k".repeat(65), &json!(1)),
      Err(kvs::Error::InvalidKey(_))
   ));
   assert!(matches!(kvs::get(""), Err(kvs::Error::InvalidKey(_))));
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test --test kvs_roundtrip`
Expected: FAIL to compile — `kvs::set` etc. not found in the crate root.

- [ ] **Step 3: Write `src/desktop.rs`**

```rust
//! Desktop implementation. macOS talks to `NSUbiquitousKeyValueStore`
//! via objc2-foundation; other desktop platforms return
//! `Error::UnsupportedPlatform` from every operation.

#[cfg(target_os = "macos")]
mod imp {
   use objc2_foundation::{NSFileManager, NSString, NSUbiquitousKeyValueStore};
   use serde_json::{Map, Value};

   use crate::conversion::{json_to_plist, plist_to_json};
   use crate::error::Result;
   use crate::models::AccountStatus;
   use crate::validation::{validate_key, validate_value};

   pub fn get(key: &str) -> Result<Option<Value>> {
      validate_key(key)?;

      let store = NSUbiquitousKeyValueStore::defaultStore();

      match store.objectForKey(&NSString::from_str(key)) {
         Some(obj) => Ok(Some(plist_to_json(&obj)?)),
         None => Ok(None),
      }
   }

   pub fn set(key: &str, value: &Value) -> Result<()> {
      validate_key(key)?;
      validate_value(value)?;

      let plist = json_to_plist(value)?;
      let store = NSUbiquitousKeyValueStore::defaultStore();

      // SAFETY: plist is one of NSString/NSNumber/NSArray/NSDictionary,
      // all valid KVS value types, produced by json_to_plist.
      unsafe { store.setObject_forKey(Some(&plist), &NSString::from_str(key)) };

      // Spec decision: `set` requests upload immediately; debouncing is
      // the caller's responsibility (the OS coalesces frequent writes).
      store.synchronize();

      Ok(())
   }

   pub fn remove(key: &str) -> Result<()> {
      validate_key(key)?;

      let store = NSUbiquitousKeyValueStore::defaultStore();

      store.removeObjectForKey(&NSString::from_str(key));
      store.synchronize();

      Ok(())
   }

   pub fn keys() -> Result<Vec<String>> {
      let store = NSUbiquitousKeyValueStore::defaultStore();
      let dict = store.dictionaryRepresentation();

      Ok(dict.keys().map(|k| k.to_string()).collect())
   }

   pub fn get_all() -> Result<Map<String, Value>> {
      let store = NSUbiquitousKeyValueStore::defaultStore();
      let dict = store.dictionaryRepresentation();
      let (dict_keys, dict_values) = dict.to_vecs();
      let mut map = Map::with_capacity(dict_keys.len());

      for (key, value) in dict_keys.into_iter().zip(dict_values) {
         map.insert(key.to_string(), plist_to_json(&value)?);
      }

      Ok(map)
   }

   pub fn synchronize() -> Result<bool> {
      Ok(NSUbiquitousKeyValueStore::defaultStore().synchronize())
   }

   pub fn account_status() -> Result<AccountStatus> {
      let token = NSFileManager::defaultManager().ubiquityIdentityToken();

      Ok(match token {
         Some(_) => AccountStatus::Available,
         None => AccountStatus::NoAccount,
      })
   }
}

#[cfg(not(target_os = "macos"))]
mod imp {
   use serde_json::{Map, Value};

   use crate::error::{Error, Result};
   use crate::models::AccountStatus;

   pub fn get(_key: &str) -> Result<Option<Value>> {
      Err(Error::UnsupportedPlatform)
   }

   pub fn set(_key: &str, _value: &Value) -> Result<()> {
      Err(Error::UnsupportedPlatform)
   }

   pub fn remove(_key: &str) -> Result<()> {
      Err(Error::UnsupportedPlatform)
   }

   pub fn keys() -> Result<Vec<String>> {
      Err(Error::UnsupportedPlatform)
   }

   pub fn get_all() -> Result<Map<String, Value>> {
      Err(Error::UnsupportedPlatform)
   }

   pub fn synchronize() -> Result<bool> {
      Err(Error::UnsupportedPlatform)
   }

   pub fn account_status() -> Result<AccountStatus> {
      Err(Error::UnsupportedPlatform)
   }
}

pub use imp::{account_status, get, get_all, keys, remove, set, synchronize};
```

In `src/lib.rs`, add below the other mods and re-export the API from the crate root (rustdoc note that this is the Rust-side API; app frontends use the guest bindings):

```rust
mod desktop;

pub use desktop::{account_status, get, get_all, keys, remove, set, synchronize};
```

- [ ] **Step 4: Run all tests to verify they pass**

Run: `cargo test && cargo lint-clippy && cargo lint-fmt`
Expected: unit tests + all 4 integration tests PASS, lints clean.

Contingency: if the integration test fails on a **CI runner** because the runner's environment gives KVS a non-functional local store (writes silently dropped), gate the two store-touching tests behind `std::env::var("KVS_INTEGRATION").is_ok()`, run them locally, and record the CI limitation in `DEVELOPERS.md` (Task 8). Do not weaken the assertions.

- [ ] **Step 5: Commit**

```bash
git add src/desktop.rs src/lib.rs tests/kvs_roundtrip.rs
git commit -m "feat: Add macOS KVS store operations with integration tests"
```

---

### Task 6: Tauri commands, build-script registration, permissions

**Files:**
- Create: `src/commands.rs`, `permissions/default.toml` (plus build-generated `permissions/autogenerated/`, `permissions/schemas/`)
- Modify: `build.rs`, `src/lib.rs`

**Interfaces:**
- Consumes: crate-root API from Task 5, `AccountStatus` from Task 3.
- Produces: Tauri commands `get`, `set`, `remove`, `keys`, `get_all`, `synchronize`, `account_status` reachable as `plugin:icloud-kvs|<name>` (Task 7's TS bindings invoke these exact names); a `default` permission granting all seven.

**Carryover honored here:** `permissions/default.toml` is created in the same commit that adds the first `COMMANDS` entry to `build.rs`.

- [ ] **Step 1: Write `src/commands.rs`**

```rust
//! `#[tauri::command]` wrappers around the crate-root API. Kept thin:
//! all validation and platform logic lives behind the public functions.

use serde_json::{Map, Value};

use crate::error::Result;
use crate::models::AccountStatus;

#[tauri::command]
pub(crate) fn get(key: String) -> Result<Option<Value>> {
   crate::get(&key)
}

#[tauri::command]
pub(crate) fn set(key: String, value: Value) -> Result<()> {
   crate::set(&key, &value)
}

#[tauri::command]
pub(crate) fn remove(key: String) -> Result<()> {
   crate::remove(&key)
}

#[tauri::command]
pub(crate) fn keys() -> Result<Vec<String>> {
   crate::keys()
}

#[tauri::command]
pub(crate) fn get_all() -> Result<Map<String, Value>> {
   crate::get_all()
}

#[tauri::command]
pub(crate) fn synchronize() -> Result<bool> {
   crate::synchronize()
}

#[tauri::command]
pub(crate) fn account_status() -> Result<AccountStatus> {
   crate::account_status()
}
```

- [ ] **Step 2: Register commands in `build.rs` and `src/lib.rs`**

`build.rs`:

```rust
const COMMANDS: &[&str] = &[
   "get",
   "set",
   "remove",
   "keys",
   "get_all",
   "synchronize",
   "account_status",
];

fn main() {
   tauri_plugin::Builder::new(COMMANDS).build();
}
```

`src/lib.rs` — add `mod commands;` and wire the handler:

```rust
mod commands;
```

```rust
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
```

- [ ] **Step 3: Write `permissions/default.toml`**

```toml
"$schema" = "schemas/schema.json"

[default]
description = "Default permissions for the icloud-kvs plugin: allows every command."
permissions = [
   "allow-get",
   "allow-set",
   "allow-remove",
   "allow-keys",
   "allow-get-all",
   "allow-synchronize",
   "allow-account-status",
]
```

- [ ] **Step 4: Build to generate permission files, then run everything**

Run: `cargo build && cargo test && cargo lint-clippy && cargo lint-fmt`
Expected: build generates `permissions/autogenerated/` and `permissions/schemas/`; all tests pass; lints clean. (If the build errors that `default.toml`'s `$schema` path doesn't exist yet, delete the `"$schema"` line, build once to generate `permissions/schemas/schema.json`, then restore the line and rebuild.)

- [ ] **Step 5: Commit (permissions dir + first COMMANDS entry together)**

```bash
git add build.rs src/commands.rs src/lib.rs permissions/
git commit -m "feat: Register Tauri commands with default permission set"
```

---

### Task 7: TypeScript guest bindings

**Files:**
- Modify: `guest-js/package.json`, `guest-js/src/index.ts`

**Interfaces:**
- Consumes: command names from Task 6 (`plugin:icloud-kvs|get` … `plugin:icloud-kvs|account_status`), `AccountStatus` wire values from Task 3 (`"available"` / `"noAccount"`).
- Produces: npm API `get`, `set`, `remove`, `keys`, `getAll`, `synchronize`, `accountStatus` plus types `KvsValue`, `AccountStatus`. (`onExternalChange` and `ChangeEvent` arrive in M1.4.)

- [ ] **Step 1: Add the Tauri API dependency**

```bash
cd guest-js
npm view @tauri-apps/api version   # note the exact latest 2.x version
npm install --save-exact @tauri-apps/api@<that version>
```

- [ ] **Step 2: Rewrite `guest-js/src/index.ts`**

```ts
/**
 * TypeScript bindings for tauri-plugin-icloud-kvs.
 *
 * `onExternalChange` (change events) is added in a later milestone;
 * everything else in the API surface is available here.
 */

import { invoke } from '@tauri-apps/api/core';

/**
 * A JSON value storable in iCloud KVS. Mapped to native property-list
 * types (NSString, NSNumber, NSArray, NSDictionary). `null` is not
 * storable — use `remove` to delete a key.
 */
export type KvsValue =
   | string
   | number
   | boolean
   | KvsValue[]
   | { [key: string]: KvsValue };

/**
 * Whether the device is signed in to iCloud. When signed out, KVS
 * silently degrades to local-only storage; this is the only way to
 * detect that condition.
 */
export type AccountStatus = 'available' | 'noAccount';

/**
 * Returns the value for `key`, or `null` if the key is not present.
 */
export async function get(key: string): Promise<KvsValue | null> {
   return await invoke<KvsValue | null>('plugin:icloud-kvs|get', { key });
}

/**
 * Stores `value` under `key` and requests upload to iCloud. The OS
 * throttles/coalesces frequent writes — debounce rapid updates in the
 * caller.
 */
export async function set(key: string, value: KvsValue): Promise<void> {
   await invoke('plugin:icloud-kvs|set', { key, value });
}

/**
 * Deletes `key` from the store. Deleting a missing key is not an error.
 */
export async function remove(key: string): Promise<void> {
   await invoke('plugin:icloud-kvs|remove', { key });
}

/**
 * Lists every key currently in the store.
 */
export async function keys(): Promise<string[]> {
   return await invoke<string[]>('plugin:icloud-kvs|keys');
}

/**
 * Returns the entire store as a plain object.
 */
export async function getAll(): Promise<Record<string, KvsValue>> {
   return await invoke<Record<string, KvsValue>>('plugin:icloud-kvs|get_all');
}

/**
 * Flush-only: writes pending changes to local disk and *requests*
 * upload. Does NOT force a server round-trip or pull fresh data —
 * do not build "sync now" UX on this.
 */
export async function synchronize(): Promise<boolean> {
   return await invoke<boolean>('plugin:icloud-kvs|synchronize');
}

/**
 * Reports whether the device is signed in to iCloud
 * (via `FileManager.ubiquityIdentityToken`).
 */
export async function accountStatus(): Promise<AccountStatus> {
   return await invoke<AccountStatus>('plugin:icloud-kvs|account_status');
}
```

- [ ] **Step 3: Verify build**

Run: `cd guest-js && npm run build && cd ..`
Expected: compiles clean; `dist/index.js` + `dist/index.d.ts` regenerated.

- [ ] **Step 4: Commit**

```bash
git add guest-js/package.json guest-js/package-lock.json guest-js/src/index.ts
git commit -m "feat: Add TypeScript bindings for all KVS commands"
```

---

### Task 8: Documentation, CI verification, milestone close-out

**Files:**
- Modify: `README.md` (platform table + usage), `DEVELOPERS.md` (manual verification protocol), `docs/milestones.md` (check off M1.2)

**Interfaces:**
- Consumes: everything above; CI workflow from M1.1.

- [ ] **Step 1: Update `README.md`**

In the platform-support table, change the macOS row to:

```markdown
| macOS    | ✅ Supported (pure Rust via `objc2`, no Swift toolchain needed) |
```

Below the table, add a short usage section (full entitlement guide still lands in M1.5 — keep the status warning in place):

````markdown
## Usage (macOS, pre-release)

Register the plugin and allow its commands in your capability file:

```rust
tauri::Builder::default()
   .plugin(tauri_plugin_icloud_kvs::init())
```

```json
{ "permissions": ["icloud-kvs:default"] }
```

```ts
import { set, get } from 'tauri-plugin-icloud-kvs-api';

await set('theme', { mode: 'dark', accent: 'teal' });
const theme = await get('theme');
```

Cross-device sync requires the
`com.apple.developer.ubiquity-kvstore-identifier` entitlement (guide
coming with the first release). Without it — or when signed out of
iCloud — the store still works locally but never syncs; use
`accountStatus()` to detect the signed-out case.
````

- [ ] **Step 2: Update `DEVELOPERS.md`**

Replace the "Cross-device sync verification" placeholder section with:

```markdown
## Cross-device sync verification (manual)

CI exercises the local KVS store only; real iCloud sync needs signed,
entitled app bundles on two devices with the same Apple ID. Protocol:

1. Build a scratch Tauri app (or, once it exists, `examples/demo-app`)
   that registers this plugin, with the
   `com.apple.developer.ubiquity-kvstore-identifier` entitlement set to
   `$(TeamIdentifierPrefix)$(CFBundleIdentifier)` and codesigned with a
   Development certificate on both Macs.
2. On Mac A: `set('sync-check', <timestamp>)`.
3. On Mac B: poll `get('sync-check')` (KVS latency is seconds to
   minutes; no guarantees). The value arriving proves upload + download.
4. Repeat in the reverse direction.

`accountStatus()` must report `available` on both machines first.
```

- [ ] **Step 3: Full local verification**

Run: `cargo test && cargo lint-clippy && cargo lint-fmt && (cd guest-js && npm run build)`
Expected: everything green.

- [ ] **Step 4: Commit docs and push**

```bash
git add README.md DEVELOPERS.md
git commit -m "docs: Document macOS support and manual sync verification"
git push
```

- [ ] **Step 5: Verify CI**

Run: `gh run watch --repo kmuncie/tauri-plugin-icloud-kvs --exit-status`
Expected: latest run `success` on both jobs. If the Rust job fails only in the KVS integration test, apply the Task 5 contingency (env-gate + document) rather than weakening assertions.

- [ ] **Step 6: HUMAN CHECKPOINT — manual two-Mac verification**

The milestone text requires manual verification between two Macs (or Mac + observable iCloud behavior). This needs Kevin's hardware and Apple ID and cannot be done by an agent. Follow the `DEVELOPERS.md` protocol from Step 2. **Stop here and report status to Kevin if executing autonomously.**

- [ ] **Step 7: Check off the milestone**

After manual verification passes, in `docs/milestones.md` change the M1.2 entry to `- [x]` (and tick M1.2 in `docs/design-spec.md`'s Milestones list), then:

```bash
git add docs/milestones.md docs/design-spec.md
git commit -m "docs: Check off M1.2 in plugin milestones"
git push
```
