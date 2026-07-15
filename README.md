# tauri-plugin-icloud-kvs

Sync small data across a user's Apple devices from a [Tauri 2](https://tauri.app)
app — no server, no user accounts, no CloudKit schema. This plugin exposes
Apple's iCloud Key-Value Store
([`NSUbiquitousKeyValueStore`](https://developer.apple.com/documentation/foundation/nsubiquitouskeyvaluestore))
on **macOS** and **iOS**.

> 🧪 **Status: release candidate.** `0.1.0-rc.1` is published for
> early testing — pin it explicitly (pre-releases are never resolved
> by default). Stable 0.1.0 ships once cross-device sync has been
> verified on real hardware.

Built by the maker of [Team Times](https://kmuncie.com/team-times/),
a menu-bar app for tracking distributed teams across time zones — this
plugin powers its cross-device config sync.

## Platform support

| Platform | Support |
|----------|---------|
| macOS    | ✅ Supported (pure Rust via `objc2`, no Swift toolchain needed) |
| iOS      | ✅ Supported (same pure-Rust implementation as macOS) |
| Others   | Commands return an `UnsupportedPlatform` error |

## Setup

### 1. Install

```sh
cargo add tauri-plugin-icloud-kvs@0.1.0-rc.1
npm install tauri-plugin-icloud-kvs-api@rc
```

### 2. Register the plugin and permission

```rust
tauri::Builder::default()
   .plugin(tauri_plugin_icloud_kvs::init())
```

In your capability file (e.g. `src-tauri/capabilities/default.json`):

```json
{ "permissions": ["icloud-kvs:default"] }
```

```ts
import { set, get } from 'tauri-plugin-icloud-kvs-api';

await set('theme', { mode: 'dark', accent: 'teal' });
const theme = await get('theme');
```

### 3. Entitlement & signing (required)

iCloud KVS only works in a **code-signed** app carrying the
`com.apple.developer.ubiquity-kvstore-identifier` entitlement, set to
`$(TeamIdentifierPrefix)$(CFBundleIdentifier)` (your Team ID prefix +
bundle identifier). Without it the backing store is inert: every write
silently no-ops. You need a paid Apple Developer Program membership.

**macOS (direct distribution):**

1. Create `src-tauri/entitlements.plist`:

   ```xml
   <?xml version="1.0" encoding="UTF-8"?>
   <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
      "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
   <plist version="1.0">
   <dict>
      <key>com.apple.developer.ubiquity-kvstore-identifier</key>
      <string>TEAMID1234.com.example.yourapp</string>
   </dict>
   </plist>
   ```

   Use your **literal** values — Tauri signs with `codesign`, which
   does not substitute `$(...)` Xcode variables.

2. Point Tauri at it in `tauri.conf.json`:

   ```json
   { "bundle": { "macOS": { "entitlements": "./entitlements.plist" } } }
   ```

3. Build signed:
   `APPLE_SIGNING_IDENTITY="Apple Development: You (TEAMID1234)" npm run tauri build`

**Mac App Store:** the sandbox is mandatory there, so your
entitlements file also needs `com.apple.security.app-sandbox`, and the
KVS entitlement must be embedded in your provisioning profile — enable
the iCloud capability (Key-value storage) for the app's identifier in
the developer portal and regenerate the profile before signing.

**iOS:** after `tauri ios init`, open the generated Xcode project,
select your team under Signing & Capabilities, and add the **iCloud**
capability with **Key-value storage** checked. Xcode manages the
entitlement and provisioning profile from there. If you manage
profiles manually in the developer portal, enable iCloud on the App ID
and regenerate the profile — the entitlement rides in the profile on
iOS.

A worked example (including the simulator flow) lives in
[`examples/demo-app`](examples/demo-app/README.md) — a key-value
editor that doubles as the plugin's manual test rig.

### 4. Detect the signed-out case

With the entitlement but no iCloud account signed in, the store works
**locally but never syncs** — and nothing errors. Surface it:

```ts
import { accountStatus } from 'tauri-plugin-icloud-kvs-api';

if (await accountStatus() === 'noAccount') {
   // Warn: data stays on this device until iCloud sign-in.
}
```

## API

```ts
import * as kvs from 'tauri-plugin-icloud-kvs-api';
```

| Function | Returns | Notes |
|----------|---------|-------|
| `get(key)` | `Promise<KvsValue \| null>` | `null` when absent |
| `set(key, value)` | `Promise<void>` | JSON value; requests upload immediately. Debounce rapid writes — the OS throttles |
| `remove(key)` | `Promise<void>` | Removing a missing key is not an error |
| `keys()` | `Promise<string[]>` | |
| `getAll()` | `Promise<Record<string, KvsValue>>` | |
| `synchronize()` | `Promise<boolean>` | Flush-only; does **not** force a server round-trip — don't build "sync now" UX on it |
| `accountStatus()` | `Promise<'available' \| 'noAccount'>` | |
| `onExternalChange(handler)` | `Promise<UnlistenFn>` | See below |

Constraints: keys ≤ 64 bytes UTF-8 and non-empty; serialized values
≤ 1 MB; `null` is not storable (use `remove`). Violations reject with
a descriptive error string.

### Change events

```ts
const unlisten = await kvs.onExternalChange((event) => {
   // event.reason: 'serverChange' | 'initialSync'
   //             | 'quotaViolation' | 'accountChange'
   // event.changedKeys: string[] (may be empty)
});
```

Fired when another device changes a value (`serverChange`), the first
iCloud download lands (`initialSync`), the store exceeds quota
(`quotaViolation`), or the signed-in iCloud account changes
(`accountChange` — the local store is replaced with the new account's
data). Rust-side consumers can listen for the same Tauri event,
`icloud-kvs://external-change`.

## What iCloud KVS gives you (and its limits)

- 1 MB total per app, max 1024 keys, key names ≤ 64 bytes UTF-8
- Last-writer-wins conflict resolution; sync latency of seconds to
  minutes (no guarantees)
- No offline queue to manage — the OS persists locally and syncs when
  it can

### Quota violations are asynchronous

The OS **never** reports quota exhaustion at the call site — `set`
succeeds, the write is dropped later, and a `quotaViolation` change
event fires. Handle it:

```ts
import { onExternalChange } from 'tauri-plugin-icloud-kvs-api';

await onExternalChange((event) => {
   if (event.reason === 'quotaViolation') {
      // Over quota: recent writes were rejected by the OS.
      // Shrink stored data, then re-set the keys you need.
   }
});
```

### Other gotchas

- **Write throttling:** the OS coalesces frequent `set` calls;
  debounce bursts (e.g. slider input) in your app.
- **`synchronize()` is flush-only:** it writes pending changes to
  disk and *requests* upload. It does not pull fresh data.
- **Signed out = silent local-only mode:** see "Setup" step 4.
- **Account switch replaces the store:** on `accountChange` the local
  data is swapped for the new account's — re-read anything you cache.
- **Foreign native values:** if other native code shares the store,
  raw `NSData` reads back as a base64 string and `NSDate` as an
  ISO-8601 UTC string (one-way mappings; this plugin never writes
  those types).

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at
your option.
