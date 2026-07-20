# Changelog

All notable changes to this project will be documented in this file.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versioning: [SemVer](https://semver.org/).

## [0.1.0] - 2026-07-20

First stable release. Cross-device sync (bidirectional edits, offline
catch-up) and external-change event delivery verified on real
hardware (iPhone + MacBook) against `0.1.0-rc.1`; no changes since
the rc.

## [0.1.0-rc.1] - 2026-07-14

Initial release candidate.

### Added

- `get` / `set` / `remove` / `keys` / `getAll` over Apple's iCloud
  Key-Value Store (`NSUbiquitousKeyValueStore`) on macOS and iOS,
  with JSON ↔ property-list value mapping
- `synchronize()` (flush-only) and `accountStatus()`
  (signed-out detection via `ubiquityIdentityToken`)
- External-change notifications emitted as the Tauri event
  `icloud-kvs://external-change`; `onExternalChange()` guest binding
  with typed `ChangeEvent` payloads (`serverChange`, `initialSync`,
  `quotaViolation`, `accountChange`)
- Key/value validation matching OS limits (64-byte keys, 1 MB values)
  with descriptive errors; quota violations surface via change events
  (OS behavior — never call-site errors)
- Foreign `NSData`/`NSDate` values read back as base64 / ISO-8601
  strings instead of erroring
- Pure-Rust implementation via `objc2` (no Swift toolchain);
  non-Apple platforms return `UnsupportedPlatform`
- Demo app (`examples/demo-app`): KV editor + live change-event log
