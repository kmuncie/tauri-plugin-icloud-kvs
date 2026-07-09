use serde::{Serialize, Serializer};
use thiserror::Error;

/// Convenience result type used across the plugin.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by plugin commands.
///
/// Note: iCloud KVS quota exhaustion is NOT an error variant — the OS
/// reports it only asynchronously via the external-change notification
/// (`reason: quotaViolation`), never as a call-site failure.
#[derive(Debug, Error)]
pub enum Error {
   #[error("iCloud Key-Value Store is only available on macOS and iOS")]
   UnsupportedPlatform,

   #[error("invalid key: {0}")]
   InvalidKey(String),

   #[error("value too large: {size} bytes exceeds the 1 MB iCloud KVS limit")]
   ValueTooLarge { size: usize },

   #[error("serialization error: {0}")]
   Serialization(String),

   #[error("platform error: {0}")]
   PlatformError(String),
}

impl Serialize for Error {
   fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
      serializer.serialize_str(&self.to_string())
   }
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn display_messages_are_actionable() {
      assert_eq!(
         Error::UnsupportedPlatform.to_string(),
         "iCloud Key-Value Store is only available on macOS and iOS"
      );
      assert_eq!(
         Error::InvalidKey("key exceeds 64 bytes".into()).to_string(),
         "invalid key: key exceeds 64 bytes"
      );
      assert_eq!(
         Error::ValueTooLarge { size: 2_000_000 }.to_string(),
         "value too large: 2000000 bytes exceeds the 1 MB iCloud KVS limit"
      );
   }

   #[test]
   fn serializes_as_display_string() {
      let json = serde_json::to_string(&Error::UnsupportedPlatform).unwrap();

      assert_eq!(
         json,
         "\"iCloud Key-Value Store is only available on macOS and iOS\""
      );
   }
}
