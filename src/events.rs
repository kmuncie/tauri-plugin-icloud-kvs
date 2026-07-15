//! External-change notifications: an `NSNotificationCenter` observer
//! (registered once at plugin setup) converts
//! `NSUbiquitousKeyValueStoreDidChangeExternallyNotification` into the
//! Tauri event [`EXTERNAL_CHANGE_EVENT`] with a
//! [`ChangeEvent`](crate::models::ChangeEvent) payload.

use std::ptr::NonNull;

use block2::RcBlock;
use objc2_foundation::{
   NSArray, NSDictionary, NSNotification, NSNotificationCenter, NSNumber, NSOperationQueue,
   NSString, NSUbiquitousKeyValueStore, NSUbiquitousKeyValueStoreAccountChange,
   NSUbiquitousKeyValueStoreChangeReasonKey, NSUbiquitousKeyValueStoreChangedKeysKey,
   NSUbiquitousKeyValueStoreDidChangeExternallyNotification,
   NSUbiquitousKeyValueStoreInitialSyncChange, NSUbiquitousKeyValueStoreQuotaViolationChange,
   NSUbiquitousKeyValueStoreServerChange,
};
use tauri::{AppHandle, Emitter, Runtime};

use crate::models::{ChangeEvent, ChangeReason};

/// Tauri event emitted for every external change to the store.
pub const EXTERNAL_CHANGE_EVENT: &str = "icloud-kvs://external-change";

// The generated constants are NSInteger (isize); widen once so the pure
// parser below can take a plain i64 (what NSNumber::as_i64 returns).
const SERVER_CHANGE: i64 = NSUbiquitousKeyValueStoreServerChange as i64;
const INITIAL_SYNC: i64 = NSUbiquitousKeyValueStoreInitialSyncChange as i64;
const QUOTA_VIOLATION: i64 = NSUbiquitousKeyValueStoreQuotaViolationChange as i64;
const ACCOUNT_CHANGE: i64 = NSUbiquitousKeyValueStoreAccountChange as i64;

/// Registers the external-change observer for the process lifetime and
/// primes notification delivery.
pub fn register<R: Runtime>(app: &AppHandle<R>) {
   // Apple requires one synchronize() after launch before the OS
   // delivers external-change notifications to this process.
   NSUbiquitousKeyValueStore::defaultStore().synchronize();

   let handle = app.clone();
   let block = RcBlock::new(move |notification: NonNull<NSNotification>| {
      // SAFETY: NSNotificationCenter passes a valid notification for
      // the duration of the block invocation.
      let notification = unsafe { notification.as_ref() };

      if let Some(event) = parse_notification(notification) {
         // Failure here means no webview exists yet; nothing to do.
         let _ = handle.emit(EXTERNAL_CHANGE_EVENT, event);
      }
   });

   // SAFETY: name is a valid notification name; the block is sendable
   // (it captures only an AppHandle, which is Send + Sync) and runs on
   // the main queue.
   let token = unsafe {
      NSNotificationCenter::defaultCenter().addObserverForName_object_queue_usingBlock(
         Some(NSUbiquitousKeyValueStoreDidChangeExternallyNotification),
         None,
         Some(&NSOperationQueue::mainQueue()),
         &block,
      )
   };

   // The observer must outlive every window; leak the token so it is
   // never unregistered (the plugin lives as long as the process).
   std::mem::forget(token);
}

fn parse_notification(notification: &NSNotification) -> Option<ChangeEvent> {
   let user_info = notification.userInfo()?;
   // SAFETY: Foundation's userInfo-key statics are immutable NSString
   // constants; reading them is always sound.
   let reason_key = unsafe { NSUbiquitousKeyValueStoreChangeReasonKey };
   let reason_code = user_info
      .objectForKey(reason_key)?
      .downcast_ref::<NSNumber>()?
      .as_i64();

   change_event_from_parts(reason_code, changed_keys(&user_info))
}

fn changed_keys(user_info: &NSDictionary) -> Vec<String> {
   // SAFETY: Foundation's userInfo-key statics are immutable NSString
   // constants; reading them is always sound.
   let keys_key = unsafe { NSUbiquitousKeyValueStoreChangedKeysKey };
   let Some(list) = user_info.objectForKey(keys_key) else {
      return Vec::new();
   };
   let Some(array) = list.downcast_ref::<NSArray>() else {
      return Vec::new();
   };

   array
      .to_vec()
      .iter()
      .filter_map(|item| item.downcast_ref::<NSString>().map(|s| s.to_string()))
      .collect()
}

/// Pure mapping from the notification's userInfo parts to the event
/// payload. Unknown (future) reason codes drop the notification rather
/// than mis-label it.
fn change_event_from_parts(reason_code: i64, changed_keys: Vec<String>) -> Option<ChangeEvent> {
   let reason = match reason_code {
      SERVER_CHANGE => ChangeReason::ServerChange,
      INITIAL_SYNC => ChangeReason::InitialSync,
      QUOTA_VIOLATION => ChangeReason::QuotaViolation,
      ACCOUNT_CHANGE => ChangeReason::AccountChange,
      _ => return None,
   };

   Some(ChangeEvent {
      reason,
      changed_keys,
   })
}

#[cfg(test)]
mod tests {
   use super::*;
   use crate::models::ChangeReason;

   #[test]
   fn maps_all_four_documented_reason_codes() {
      let cases = [
         (0, ChangeReason::ServerChange),
         (1, ChangeReason::InitialSync),
         (2, ChangeReason::QuotaViolation),
         (3, ChangeReason::AccountChange),
      ];

      for (code, expected) in cases {
         let event = change_event_from_parts(code, vec!["k".into()]).unwrap();

         assert_eq!(event.reason, expected);
         assert_eq!(event.changed_keys, vec!["k".to_string()]);
      }
   }

   #[test]
   fn unknown_reason_codes_produce_no_event() {
      assert!(change_event_from_parts(4, Vec::new()).is_none());
      assert!(change_event_from_parts(-1, Vec::new()).is_none());
   }

   #[test]
   fn missing_key_list_becomes_empty_vec() {
      let event = change_event_from_parts(0, Vec::new()).unwrap();

      assert!(event.changed_keys.is_empty());
   }
}
