//! Pre-checks for iCloud KVS limits. KVS silently misbehaves instead of
//! erroring, so limits are enforced here before any native call.

use serde_json::Value;

use crate::error::{Error, Result};

/// KVS key names are limited to 64 bytes of UTF-8.
#[allow(dead_code)]
pub(crate) const MAX_KEY_BYTES: usize = 64;

/// The whole store is capped at 1 MB; a single value can never exceed it.
#[allow(dead_code)]
pub(crate) const MAX_VALUE_BYTES: usize = 1_048_576;

#[allow(dead_code)]
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

#[allow(dead_code)]
pub(crate) fn validate_value(value: &Value) -> Result<()> {
   let size = serde_json::to_vec(value)
      .map_err(|e| Error::Serialization(e.to_string()))?
      .len();

   if size > MAX_VALUE_BYTES {
      return Err(Error::ValueTooLarge { size });
   }

   Ok(())
}

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
      assert!(matches!(
         validate_key(&"k".repeat(65)),
         Err(Error::InvalidKey(_))
      ));
      // 22 chars × 3 bytes = 66 UTF-8 bytes — byte length is what counts
      assert!(matches!(
         validate_key(&"€".repeat(22)),
         Err(Error::InvalidKey(_))
      ));
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
