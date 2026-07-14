# tauri-plugin-icloud-kvs

Sync small data across a user's Apple devices from a [Tauri 2](https://tauri.app)
app — no server, no user accounts, no CloudKit schema. This plugin exposes
Apple's iCloud Key-Value Store
([`NSUbiquitousKeyValueStore`](https://developer.apple.com/documentation/foundation/nsubiquitouskeyvaluestore))
on **macOS** and **iOS**.

> ⚠️ **Status: under development.** The API is not yet stable and the
> crate/npm packages are not yet published. Watch releases for 0.1.0.

Built by the maker of [Team Times](https://kmuncie.com/team-times/),
a menu-bar app for tracking distributed teams across time zones — this
plugin powers its cross-device config sync.

## Platform support

| Platform | Support |
|----------|---------|
| macOS    | ✅ Supported (pure Rust via `objc2`, no Swift toolchain needed) |
| iOS      | ✅ Supported (same pure-Rust implementation as macOS) |
| Others   | Commands return an `UnsupportedPlatform` error |

## Usage (pre-release)

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

This plugin requires the
`com.apple.developer.ubiquity-kvstore-identifier` entitlement (guide
coming with the first release). Without it, the backing store is
inert — writes silently no-op. With the entitlement but signed out of
iCloud, the store works locally but never syncs; use
`accountStatus()` to detect the signed-out case.

A runnable example lives in
[`examples/demo-app`](examples/demo-app/README.md) — a key-value editor
that doubles as the plugin's manual test rig.

## What iCloud KVS gives you (and its limits)

- 1 MB total per app, max 1024 keys, key names ≤ 64 bytes UTF-8
- Last-writer-wins conflict resolution; sync latency of seconds (no guarantees)
- Quota violations are reported **asynchronously** via change events, never
  as a call-site error
- Requires the `com.apple.developer.ubiquity-kvstore-identifier` entitlement
  (see "Usage" above; setup guide coming with the first release)

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at
your option.
