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
