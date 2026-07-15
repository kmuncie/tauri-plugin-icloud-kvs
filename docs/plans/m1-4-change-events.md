# tauri-plugin-icloud-kvs M1.4 (Change Events) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** External-change notifications emitted as Tauri events on macOS and iOS (`icloud-kvs://external-change` with a `ChangeEvent` payload), an `onExternalChange()` guest binding, a live event-log pane in the demo app, and the M1.2-review carryover fixed (foreign `NSDate` values no longer poison `get_all`).

**Architecture:** A new Apple-only `src/events.rs` registers a block-based `NSNotificationCenter` observer for `NSUbiquitousKeyValueStoreDidChangeExternallyNotification` from the plugin's `setup` hook and broadcasts every parsed notification via `app.emit` (always-on, no listener refcounting — decision made 2026-07-14 with Kevin). Notification parsing is kept in pure functions so it unit-tests without notification machinery; unknown future reason codes drop the event rather than mis-label it. `plist_to_json` gains an `NSDate` → ISO-8601-string branch alongside the existing `NSData` → base64 one (decision 2026-07-14: string mapping, not skip-key or keep-erroring). Foreign `NSDate` values are read-only: a subsequent `set` of that key writes a string, which is fine since this plugin's API never writes dates.

**Tech Stack:** Rust (objc2 0.6.4, objc2-foundation 0.3.2, block2 0.6.2, Tauri 2 plugin API), TypeScript (`@tauri-apps/api` 2.11.1), vanilla-TS demo app.

**Spec:** `docs/design-spec.md` ("Architecture & data flow" already describes the observer). Milestone definition: `docs/milestones.md` (M1.4 entry, including the NSDate carryover).

## Global Constraints

- Rust 1.89.0, edition 2024; 3-space indent; `rustfmt.toml` / `.cargo/config.toml` already in repo — do not modify
- All dependencies stay pinned to exact semver versions. This milestone adds exactly one new direct dependency: `block2 = "0.6.2"` (already in the tree transitively via objc2; Kevin approved the M1.4 design that requires it)
- Every commit passes `cargo test && cargo lint-clippy && cargo lint-fmt` on the host (macOS). `Cargo.lock` is gitignored — never try to commit it
- Commit messages: Conventional Commits, imperative, ≤72-char subject, no issue numbers (this repo has none)
- Testing policy (`docs/milestones.md`, memory): contributors never need entitled hardware; required tests must run via plain `cargo test` on any Mac. Real event delivery (two devices) is a manual, documented protocol only — never a required test
- Doc examples in Rust doc comments are annotated `no_run`
- TypeScript/guest code: existing file style (3-space indent, combined `const` declarations)
- The Tauri event name is exactly `icloud-kvs://external-change` (fixed by the design spec)

---

### Task 1: NSDate → ISO-8601 string in `plist_to_json`

Fixes the M1.2-review carryover: today one foreign `NSDate` (written by e.g. a Swift app sharing the store) makes all of `get_all` fail with `Error::Serialization`. Map it to an ISO-8601 UTC string, exactly like the existing `NSData` → base64 branch maps bytes.

**Files:**
- Modify: `Cargo.toml` (objc2-foundation feature list, lines 23–33)
- Modify: `src/conversion.rs` (imports, module doc, new branch in `plist_to_json`, new test)

**Interfaces:**
- Consumes: existing `plist_to_json(obj: &AnyObject) -> Result<Value>`
- Produces: `plist_to_json` returns `Value::String("2025-07-14T13:33:20.500Z")`-style strings for `NSDate` inputs (no signature change; no other task depends on the specifics)

- [ ] **Step 1: Add the objc2-foundation features**

In `Cargo.toml`, extend the feature list (keep alphabetical order):

```toml
objc2-foundation = { version = "0.3.2", features = [
   "NSArray",
   "NSData",
   "NSDate",
   "NSDictionary",
   "NSEnumerator",
   "NSFileManager",
   "NSFormatter",
   "NSISO8601DateFormatter",
   "NSObject",
   "NSString",
   "NSUbiquitousKeyValueStore",
   "NSValue",
] }
```

(`NSISO8601DateFormatter` needs `NSFormatter`; `stringFromDate` needs `NSDate` + `NSString`. The formatter's default time zone is GMT, so no `NSTimeZone` feature is needed.)

- [ ] **Step 2: Write the failing test**

In the `tests` module of `src/conversion.rs`, add (note: `1752500000.5` epoch seconds = `2025-07-14T13:33:20.500Z` UTC):

```rust
   #[test]
   fn foreign_nsdate_reads_back_as_iso8601_string() {
      let date = NSDate::dateWithTimeIntervalSince1970(1_752_500_000.5);

      assert_eq!(
         plist_to_json(&date).unwrap(),
         json!("2025-07-14T13:33:20.500Z")
      );
   }
```

And add `NSDate` to the test module's imports:

```rust
   use objc2_foundation::{NSData, NSDate};
```

- [ ] **Step 3: Run the test to verify it fails**

Run: `cargo test foreign_nsdate_reads_back_as_iso8601_string`
Expected: FAIL — the current code hits the `unsupported plist type` fallback, so the test panics on `.unwrap()` with `Serialization("unsupported plist type: ...")`.

- [ ] **Step 4: Implement the NSDate branch**

In `src/conversion.rs`:

1. Extend the imports:

```rust
use objc2_foundation::{
   NSArray, NSData, NSDataBase64EncodingOptions, NSDate, NSDictionary, NSISO8601DateFormatOptions,
   NSISO8601DateFormatter, NSNumber, NSString,
};
```

2. In `plist_to_json`, add a branch after the `NSData` branch (before the final `Err`):

```rust
   if let Some(date) = obj.downcast_ref::<NSDate>() {
      let formatter = NSISO8601DateFormatter::new();

      formatter.setFormatOptions(
         NSISO8601DateFormatOptions::WithInternetDateTime
            | NSISO8601DateFormatOptions::WithFractionalSeconds,
      );

      return Ok(Value::String(formatter.stringFromDate(date).to_string()));
   }
```

3. Update the module doc comment (lines 4–6) to mention dates:

```rust
//! Storable values map 1:1 onto plist types (NSString, NSNumber,
//! NSArray, NSDictionary). `null` is not storable. Values written by
//! other native code in plist-only types are converted one-way: raw
//! `NSData` becomes a base64 string and `NSDate` becomes an ISO-8601
//! UTC string (documented edge cases; there is no bytes or date API
//! in v1).
```

- [ ] **Step 5: Run the full check**

Run: `cargo test && cargo lint-clippy && cargo lint-fmt`
Expected: all tests PASS, no clippy/fmt errors.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml src/conversion.rs
git commit -m "fix: Convert foreign NSDate values to ISO-8601 strings"
```

Body (explain the why):

```
A single NSDate written by other native code sharing the store
previously failed the whole get_all call with a Serialization error
(M1.2-review carryover). Dates now read back as ISO-8601 UTC strings
via NSISO8601DateFormatter, mirroring the existing NSData-to-base64
mapping. One-way by design: this plugin's API never writes dates.
```

---

### Task 2: `ChangeEvent` / `ChangeReason` models

**Files:**
- Modify: `src/models.rs` (new types + serialization tests)
- Modify: `src/lib.rs` (line 23: export the new types)

**Interfaces:**
- Consumes: nothing new
- Produces: `crate::models::ChangeReason` (enum: `ServerChange | InitialSync | QuotaViolation | AccountChange`) and `crate::models::ChangeEvent { pub reason: ChangeReason, pub changed_keys: Vec<String> }`, both `Serialize`/`Deserialize` camelCase, `ChangeEvent: Clone` (required by `tauri::Emitter::emit`). Task 3 constructs these; the wire shape consumed by Task 4 is `{"reason":"serverChange","changedKeys":["a"]}`.

- [ ] **Step 1: Write the failing test**

In the `tests` module of `src/models.rs`:

```rust
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
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test change_event change_reason`
Expected: FAIL to compile — `ChangeEvent`/`ChangeReason` not defined.

- [ ] **Step 3: Implement the types**

Append to `src/models.rs` (after `AccountStatus`, before the `tests` module):

```rust
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
```

In `src/lib.rs`, replace line 23:

```rust
pub use models::{AccountStatus, ChangeEvent, ChangeReason};
```

- [ ] **Step 4: Run the full check**

Run: `cargo test && cargo lint-clippy && cargo lint-fmt`
Expected: all tests PASS, no clippy/fmt errors.

- [ ] **Step 5: Commit**

```bash
git add src/models.rs src/lib.rs
git commit -m "feat: Add ChangeEvent and ChangeReason models"
```

---

### Task 3: Notification observer (`src/events.rs`) wired into plugin setup

The core of M1.4. An Apple-only module registers a block-based observer for `NSUbiquitousKeyValueStoreDidChangeExternallyNotification` at plugin setup and emits `icloud-kvs://external-change` for every parsed notification. Parsing is pure and unit-tested; the observer itself can only be exercised from an entitled host (manual protocol, Task 6).

**Files:**
- Modify: `Cargo.toml` (add `block2` dep + objc2-foundation features)
- Create: `src/events.rs`
- Modify: `src/lib.rs` (module decl + `setup` hook)

**Interfaces:**
- Consumes: `crate::models::{ChangeEvent, ChangeReason}` from Task 2 (exact shapes listed there)
- Produces: `events::register<R: Runtime>(app: &AppHandle<R>)` (called once from `lib.rs` setup) and `events::EXTERNAL_CHANGE_EVENT: &str = "icloud-kvs://external-change"`. Tasks 4–5 listen for that event name with the Task 2 wire shape.

- [ ] **Step 1: Add dependencies**

In `Cargo.toml`:

1. Add `block2` to the Apple target section (it is already in the dependency tree transitively — this only makes it a direct dependency):

```toml
[target.'cfg(any(target_os = "macos", target_os = "ios"))'.dependencies]
block2 = "0.6.2"
objc2 = "0.6.4"
```

2. Add three features to `objc2-foundation` (alphabetical; `NSNotification` for the center + notification types, `NSOperation` for `NSOperationQueue::mainQueue`, `block2` for the block-based observer method):

```toml
objc2-foundation = { version = "0.3.2", features = [
   "NSArray",
   "NSData",
   "NSDate",
   "NSDictionary",
   "NSEnumerator",
   "NSFileManager",
   "NSFormatter",
   "NSISO8601DateFormatter",
   "NSNotification",
   "NSObject",
   "NSOperation",
   "NSString",
   "NSUbiquitousKeyValueStore",
   "NSValue",
   "block2",
] }
```

Run: `cargo build` — expected: compiles clean (no code uses the new deps yet).

- [ ] **Step 2: Write the failing tests**

Create `src/events.rs` with only the doc comment and the tests (implementation comes next step — the file must exist for the tests to compile, so include stub-free tests referencing not-yet-written items and expect a compile failure):

```rust
//! External-change notifications: an `NSNotificationCenter` observer
//! (registered once at plugin setup) converts
//! `NSUbiquitousKeyValueStoreDidChangeExternallyNotification` into the
//! Tauri event [`EXTERNAL_CHANGE_EVENT`] with a
//! [`ChangeEvent`](crate::models::ChangeEvent) payload.

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
```

In `src/lib.rs`, declare the module below the `conversion` declaration (same cfg gate):

```rust
#[cfg(any(target_os = "macos", target_os = "ios"))]
mod events;
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test --lib events`
Expected: FAIL to compile — `change_event_from_parts` not defined.

- [ ] **Step 4: Implement `src/events.rs`**

Insert between the doc comment and the tests module:

```rust
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
   let reason_code = user_info
      .objectForKey(NSUbiquitousKeyValueStoreChangeReasonKey)?
      .downcast_ref::<NSNumber>()?
      .as_i64();

   change_event_from_parts(reason_code, changed_keys(&user_info))
}

fn changed_keys(user_info: &NSDictionary) -> Vec<String> {
   let Some(list) = user_info.objectForKey(NSUbiquitousKeyValueStoreChangedKeysKey) else {
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
```

Implementation notes for the engineer:

- `user_info.objectForKey(...)` accepts the `&'static NSString` key statics via deref coercion (`NSString` → … → `AnyObject`).
- `downcast_ref` on a `Retained<AnyObject>` works through deref; the temporaries live to the end of each statement, so the chained form in `parse_notification` borrows safely.
- If clippy flags `std::mem::forget`, the leak is intentional — silence with a `#[allow]` **only** if it actually fires, keeping the comment.

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test --lib events`
Expected: 3 tests PASS.

- [ ] **Step 6: Wire the setup hook in `src/lib.rs`**

Replace the `init` function body:

```rust
/// Initializes the iCloud Key-Value Store plugin.
///
/// On macOS and iOS this also registers an observer that emits the
/// `icloud-kvs://external-change` Tauri event whenever another device
/// (or the OS) changes the store; see `onExternalChange` in the guest
/// bindings.
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
      .setup(|_app, _api| {
         #[cfg(any(target_os = "macos", target_os = "ios"))]
         events::register(_app);

         Ok(())
      })
      .build()
}
```

(The parameter is named `_app` because it is unused on non-Apple targets; the cfg-gated call still receives it.)

- [ ] **Step 7: Run the full check, including the iOS cross-compile if available**

Run: `cargo test && cargo lint-clippy && cargo lint-fmt`
Expected: all tests PASS, no clippy/fmt errors.

If full Xcode is installed (see `DEVELOPERS.md`), also run: `cargo clippy --target aarch64-apple-ios -- -D warnings`
Expected: clean. If Xcode is not available, CI covers this — note it and move on.

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml src/events.rs src/lib.rs
git commit -m "feat: Emit Tauri events for external KVS changes"
```

Body:

```
Registers a block-based NSNotificationCenter observer for
NSUbiquitousKeyValueStoreDidChangeExternallyNotification at plugin
setup (always-on; frontends that never listen simply ignore the
broadcasts) and emits icloud-kvs://external-change with a ChangeEvent
payload. Setup also calls synchronize() once, which Apple requires
before the OS delivers these notifications. Parsing is pure and
unit-tested; unknown future reason codes drop the notification.
```

---

### Task 4: `onExternalChange()` guest binding

**Files:**
- Modify: `guest-js/src/index.ts`

**Interfaces:**
- Consumes: the Tauri event `icloud-kvs://external-change` with payload `{ reason, changedKeys }` (Tasks 2–3)
- Produces: `onExternalChange(handler: (event: ChangeEvent) => void): Promise<UnlistenFn>`, `type ChangeReason`, `interface ChangeEvent` — exported from `tauri-plugin-icloud-kvs-api`. Task 5 imports all three.

- [ ] **Step 1: Implement the binding**

(No test runner exists in `guest-js` — `npm run build` type-checking is the gate, per the existing setup.)

In `guest-js/src/index.ts`:

1. Replace the file-header comment (lines 1–6):

```ts
/**
 * TypeScript bindings for tauri-plugin-icloud-kvs.
 */
```

2. Add the event import after the existing `invoke` import:

```ts
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
```

3. Append at the end of the file:

```ts
/**
 * Why the OS reported an external change to the store.
 */
export type ChangeReason =
   | 'serverChange'
   | 'initialSync'
   | 'quotaViolation'
   | 'accountChange';

/**
 * Payload of an external-change notification.
 */
export interface ChangeEvent {
   reason: ChangeReason;
   /**
    * Keys whose values changed. Empty when the OS omits the key list
    * (it may for quota violations and account changes).
    */
   changedKeys: string[];
}

/**
 * Subscribes to external changes: another device changed a value, the
 * initial iCloud sync arrived, the store exceeded its quota, or the
 * iCloud account changed. Returns an unlisten function.
 */
export async function onExternalChange(
   handler: (event: ChangeEvent) => void
): Promise<UnlistenFn> {
   return await listen<ChangeEvent>('icloud-kvs://external-change', (event) => {
      handler(event.payload);
   });
}
```

- [ ] **Step 2: Verify it builds**

Run: `cd guest-js && npm install && npm run build`
Expected: `tsc` exits 0.

- [ ] **Step 3: Commit**

```bash
git add guest-js/src/index.ts
git commit -m "feat: Add onExternalChange guest binding"
```

---

### Task 5: Demo app live event log

Replaces the M1.4 placeholder pane with a live log fed by `onExternalChange`, and refreshes the KV table when a change arrives so the two panes stay consistent. This is the plugin's manual test rig for events.

**Files:**
- Modify: `examples/demo-app/index.html` (event pane, lines 39–46)
- Modify: `examples/demo-app/src/main.ts`
- Modify: `examples/demo-app/src/styles.css`

**Interfaces:**
- Consumes: `onExternalChange`, `type ChangeEvent` from `tauri-plugin-icloud-kvs-api` (Task 4)
- Produces: nothing consumed by later tasks

- [ ] **Step 1: Replace the placeholder pane in `index.html`**

Replace the `<section id="event-pane">` block:

```html
         <section id="event-pane">
            <h2>External Changes</h2>
            <div class="form-actions">
               <button type="button" id="clear-events">Clear</button>
            </div>
            <p id="event-empty" class="placeholder">
               No external changes yet. Change a value from another device
               (or another signed build of this app on the same Apple ID)
               and the event will appear here with its reason and keys.
            </p>
            <ul id="event-log"></ul>
         </section>
```

- [ ] **Step 2: Subscribe and render events in `main.ts`**

1. Extend the plugin import:

```ts
import {
   accountStatus,
   getAll,
   onExternalChange,
   remove,
   set,
   synchronize,
   type ChangeEvent,
   type KvsValue,
} from 'tauri-plugin-icloud-kvs-api';
```

2. Add the two element lookups to the combined `const` declaration at the top:

```ts
      eventLog = document.querySelector<HTMLUListElement>('#event-log')!,
      eventEmpty = document.querySelector<HTMLParagraphElement>('#event-empty')!,
      clearEventsButton = document.querySelector<HTMLButtonElement>('#clear-events')!,
```

3. Add the log renderer (after `refresh()`):

```ts
function logChangeEvent(event: ChangeEvent): void {
   const entry = document.createElement('li'),
         time = document.createElement('time'),
         reason = document.createElement('span'),
         keys = document.createElement('span');

   time.textContent = new Date().toLocaleTimeString();
   reason.textContent = event.reason;
   reason.className = `badge reason-${event.reason}`;
   keys.textContent = event.changedKeys.length > 0 ? event.changedKeys.join(', ') : '(no keys)';
   keys.className = 'value';

   entry.append(time, reason, keys);
   eventLog.prepend(entry);
   eventEmpty.hidden = true;
}
```

4. Wire the clear button and the subscription (before the trailing `void updateAccountStatus();`):

```ts
clearEventsButton.addEventListener('click', () => {
   eventLog.replaceChildren();
   eventEmpty.hidden = false;
});

void onExternalChange((event) => {
   logChangeEvent(event);
   // Keep the KV table in sync with what just changed remotely.
   void refresh();
}).catch(showError);
```

- [ ] **Step 3: Style the log**

Append to `examples/demo-app/src/styles.css` (match the existing file's conventions — check them before writing):

```css
#event-log {
   list-style: none;
   margin: 0;
   padding: 0;
}

#event-log li {
   display: flex;
   gap: 0.75rem;
   align-items: baseline;
   padding: 0.4rem 0;
   border-bottom: 1px solid #eee;
   font-size: 0.9rem;
}

#event-log time {
   color: #888;
   font-variant-numeric: tabular-nums;
}
```

- [ ] **Step 4: Verify the demo app builds**

Run: `cd examples/demo-app && npm install && npm run build`
Expected: exits 0 (this is the same check CI runs).

- [ ] **Step 5: Commit**

```bash
git add examples/demo-app/index.html examples/demo-app/src/main.ts examples/demo-app/src/styles.css
git commit -m "feat: Add live external-change event log to demo app"
```

---

### Task 6: Documentation + milestone check-off

**Files:**
- Modify: `README.md` (Usage section TS snippet)
- Modify: `DEVELOPERS.md` ("Cross-device sync verification" section)
- Modify: `docs/milestones.md` (M1.4 entry)
- Modify: `docs/design-spec.md` (M1.4 checkbox, ~line 187)

**Interfaces:** none (docs only).

- [ ] **Step 1: Document the event API in the README**

In `README.md`'s "Usage (pre-release)" TS snippet, append the event subscription so consumers see the full surface:

```ts
import { onExternalChange } from 'tauri-plugin-icloud-kvs-api';

const unlisten = await onExternalChange((event) => {
   // event.reason: 'serverChange' | 'initialSync' | 'quotaViolation' | 'accountChange'
   // event.changedKeys: string[] (may be empty)
});
```

Fit it into the existing snippet's shape — read the current block first and extend it rather than duplicating imports.

- [ ] **Step 2: Extend the manual verification protocol in `DEVELOPERS.md`**

In the "Cross-device sync verification (manual)" section, after step 4, add:

```markdown
5. Change events: with the demo app open on Mac B, run
   `set('sync-check', <new value>)` on Mac A. Mac B's "External
   Changes" pane must log a `serverChange` event listing `sync-check`
   within the same latency window, and its KV table must refresh to
   the new value without user action. Note: the OS only delivers these
   notifications to processes that called `synchronize()` once after
   launch — the plugin does this automatically at setup.
```

- [ ] **Step 3: Check off M1.4 in both docs**

In `docs/milestones.md`, change the M1.4 entry to `- [x]` and rewrite it past-tense, following the M1.2/M1.3 house style — state what landed (observer + Tauri event + `onExternalChange` + demo-app event log + NSDate carryover fix), that parsing is unit-tested, and that live two-device event delivery is verified per the `DEVELOPERS.md` protocol / deferred to Team Times integration (match the wording pattern M1.3 used for its deferred sync check).

In `docs/design-spec.md` (~line 187), change the M1.4 checkbox to `- [x]`.

- [ ] **Step 4: Run everything one last time**

Run: `cargo test && cargo lint-clippy && cargo lint-fmt && (cd guest-js && npm run build) && (cd examples/demo-app && npm run build)`
Expected: everything green.

- [ ] **Step 5: Commit**

```bash
git add README.md DEVELOPERS.md docs/milestones.md docs/design-spec.md
git commit -m "docs: Document change events and check off M1.4"
```
