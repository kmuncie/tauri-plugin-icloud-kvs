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
