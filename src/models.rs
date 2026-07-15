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

/// Why the OS reported an external change to the store.
///
/// Reason codes come from `NSUbiquitousKeyValueStoreDidChangeExternallyNotification`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ChangeReason {
   /// Another device changed one or more values.
   ServerChange,
   /// First download of iCloud data after app launch/account setup.
   InitialSync,
   /// The app's key-value store exceeded its 1 MB / 1024-key quota.
   QuotaViolation,
   /// The user changed the signed-in iCloud account.
   AccountChange,
}

/// Payload of the `icloud-kvs://external-change` Tauri event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeEvent {
   pub reason: ChangeReason,
   /// Keys whose values changed. Empty when the OS omits the key list
   /// (it may for quota violations and account changes).
   pub changed_keys: Vec<String>,
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

   #[test]
   fn change_event_serializes_as_camel_case() {
      let event = ChangeEvent {
         reason: ChangeReason::ServerChange,
         changed_keys: vec!["theme".into()],
      };

      assert_eq!(
         serde_json::to_string(&event).unwrap(),
         r#"{"reason":"serverChange","changedKeys":["theme"]}"#
      );
   }

   #[test]
   fn change_reason_covers_all_documented_codes() {
      let reasons = [
         (ChangeReason::ServerChange, "\"serverChange\""),
         (ChangeReason::InitialSync, "\"initialSync\""),
         (ChangeReason::QuotaViolation, "\"quotaViolation\""),
         (ChangeReason::AccountChange, "\"accountChange\""),
      ];

      for (reason, expected) in reasons {
         assert_eq!(serde_json::to_string(&reason).unwrap(), expected);
      }
   }
}
