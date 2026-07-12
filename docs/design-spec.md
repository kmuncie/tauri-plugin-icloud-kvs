# Design: `tauri-plugin-icloud-kvs`

A Tauri 2 plugin exposing Apple's iCloud Key-Value Store
(`NSUbiquitousKeyValueStore`) on macOS and iOS.

> **Portability note:** Lives here; originated in the team-times roadmap,
> which retains the surrounding multi-platform context.

## Status

| Date       | Status   | Author                      |
|------------|----------|-----------------------------|
| 2026-07-09 | Accepted | Kevin Muncie (with Claude)  |

## Purpose

Let Tauri apps sync small data (≤1 MB total, ≤1024 keys) across a user's
Apple devices with no server, no user accounts, and no CloudKit schema.
Built as a standalone open-source project by the makers of
[Team Times](https://kmuncie.com/team-times/); the README credits the
app (marketing angle). No production-quality alternative exists as of July
2026 — the two existing Tauri iCloud plugins are iOS-only, file-oriented
(iCloud Drive / ubiquity containers), and pre-production.

## Decisions (settled during brainstorming)

- **Value model: JSON values**, mapped to KVS's native plist types. No
  bytes API in v1 (YAGNI; raw `Data` written by other native code is
  returned as base64 — documented edge case).
- **Implementation: pure Rust via `objc2-foundation`** (which ships
  `NSUbiquitousKeyValueStore` bindings) **on both macOS and iOS**. No
  Xcode/Swift toolchain required. *Correction (found during M1.3):*
  the brainstorm assumed Tauri 2 mobile plugins require a Swift half;
  they don't — Swift is only needed for lifecycle hooks or APIs Rust
  cannot reach. The single objc2 implementation compiles for iOS, so
  the accepted ~100-line Swift duplication never materialized.
- **Home & names:** GitHub `kmuncie/tauri-plugin-icloud-kvs`, crate
  `tauri-plugin-icloud-kvs`, npm `tauri-plugin-icloud-kvs-api`.
- **License: MIT/Apache-2.0 dual** (community standard; explicitly not
  AGPL).
- **`set` requests upload immediately;** write debouncing is the caller's
  responsibility (documented — the OS throttles/coalesces frequent writes).
- **Non-Apple platforms return a typed `UnsupportedPlatform` error** from
  every command; no no-op shims. Callers gate on platform.

## Repo layout

Standard Tauri 2 plugin structure (from the official plugin template):

```text
kmuncie/tauri-plugin-icloud-kvs
├── src/                  # Rust: plugin init, desktop impl, command defs
│   ├── lib.rs            # plugin builder, #[cfg] platform dispatch
│   ├── store.rs          # macOS+iOS via objc2-foundation; non-Apple → error
│   ├── commands.rs       # #[tauri::command] wrappers
│   ├── models.rs         # serde types (ChangeEvent, AccountStatus, …)
│   └── error.rs          # thiserror-based Error enum
├── guest-js/             # TypeScript API → npm tauri-plugin-icloud-kvs-api
├── permissions/          # Tauri capability definitions per command
├── examples/demo-app/    # KV editor + live change-event log
├── rustfmt.toml
├── rust-toolchain.toml
├── .cargo/config.toml    # lint/fix aliases per Silvermine Rust standards
├── README.md
├── DEVELOPERS.md         # build, test, manual sync-verification protocol
├── LICENSE-MIT
└── LICENSE-APACHE
```

CI (GitHub Actions, macOS runner): `cargo lint-clippy && cargo lint-fmt`,
`cargo test`, iOS target compile check, TypeScript build.

## API surface

TypeScript guest API; Rust commands mirror it 1:1.

```ts
type KvsValue =
   | string
   | number
   | boolean
   | KvsValue[]
   | { [key: string]: KvsValue };

get(key: string): Promise<KvsValue | null>;
set(key: string, value: KvsValue): Promise<void>;
remove(key: string): Promise<void>;
keys(): Promise<string[]>;
getAll(): Promise<Record<string, KvsValue>>;

// Flush-only: writes to local disk and *requests* upload. Does NOT force a
// server round-trip or pull fresh data. Documented prominently so callers
// don't build "sync now" UX on it.
synchronize(): Promise<boolean>;

// Via FileManager.ubiquityIdentityToken. Signed-out iCloud silently
// degrades KVS to local-only; this is the only way callers can detect it.
accountStatus(): Promise<'available' | 'noAccount'>;

onExternalChange(handler: (event: ChangeEvent) => void): Promise<UnlistenFn>;

interface ChangeEvent {
   reason: 'serverChange' | 'initialSync' | 'quotaViolation' | 'accountChange';
   changedKeys: string[];
}
```

Value mapping: JSON ↔ plist both directions (string ↔ NSString, number ↔
NSNumber, boolean ↔ NSNumber(bool), array ↔ NSArray, object ↔
NSDictionary). `null` is not a storable value — use `remove`.

## Architecture & data flow

- **macOS & iOS (`store.rs`):** `NSUbiquitousKeyValueStore::defaultStore()` via
  `objc2-foundation`, main-thread dispatch where required. An
  `NSNotificationCenter` observer for
  `NSUbiquitousKeyValueStoreDidChangeExternallyNotification` converts the
  notification's userInfo (change-reason code + changed-key list) into the
  `ChangeEvent` payload and emits it as the Tauri event
  `icloud-kvs://external-change`.
- **Other platforms:** every command returns `Error::UnsupportedPlatform`.
- The plugin is stateless beyond the notification observer; KVS itself is
  the store. No caching layer.

## Error handling

`thiserror`-based enum:

| Variant | When |
|---------|------|
| `UnsupportedPlatform` | Any command on non-Apple platforms |
| `InvalidKey` | Empty key, or key >64 bytes UTF-8 (pre-checked; KVS misbehaves silently otherwise) |
| `ValueTooLarge` | Serialized value >1 MB (pre-checked; KVS never errors on quota) |
| `Serialization(String)` | JSON ↔ plist conversion failure |
| `PlatformError(String)` | Unexpected native-layer failure |

Quota exhaustion is **not** an error return — the OS reports it only via
the external-change notification (`reason: 'quotaViolation'`). The README
has a dedicated "Quota" section with example detection code.

## Testing

- **Rust unit tests** (in-file, per Rust standards) for JSON↔plist
  conversion and key/size validation. Conversion and validation logic is
  kept pure and separate from the `objc2` calls so these run on any
  platform.
- **macOS integration test:** real KVS round-trip (get/set/remove/keys).
  *Correction (found during M1.2):* `NSUbiquitousKeyValueStore` has no
  functional backing store — not even the local-only, no-account
  fallback — unless the process is code-signed with the
  `ubiquity-kvstore-identifier` entitlement. A bare `cargo test` binary
  can never carry that entitlement, so the round-trip test is gated
  behind `KVS_INTEGRATION=1` and must be exercised from a signed,
  entitled host (e.g. the example app's dev build). CI runs only the
  non-store-dependent tests; see `DEVELOPERS.md`.
- **Cross-device sync and change events:** manual verification protocol in
  `DEVELOPERS.md` (two devices, one Apple ID); genuinely not automatable.
- **Example app** doubles as the manual test rig: key/value editor pane
  plus live event-log pane.
- Doc examples annotated `no_run` per Rust standards.

## Entitlements & documentation

README documents end-to-end setup:

- Entitlement `com.apple.developer.ubiquity-kvstore-identifier` =
  `$(TeamIdentifierPrefix)$(CFBundleIdentifier)`; enabling the iCloud
  capability in the developer portal / Xcode
- The sandboxed Mac App Store case and the iOS provisioning-profile wrinkle
- Behavioral gotchas: 1 MB / 1024-key / 64-byte-key quotas; async-only
  quota-violation reporting; `synchronize()` semantics; silent local-only
  mode when signed out; account-switch store replacement; OS write
  throttling (debounce guidance)

Root README targets consumers; `DEVELOPERS.md` targets contributors (per
Rust documentation standards, the README is not inlined into rustdoc).

## Milestones

- [ ] **M1.1 — Repo scaffold.** Plugin template, licenses, CI, README
  skeleton, this spec moved in.
- [ ] **M1.2 — macOS implementation.** Full command set working via
  `objc2`; unit + integration tests green.
- [ ] **M1.3 — iOS implementation.** Same API via Swift; verified in
  simulator/device against the same iCloud container as the Mac build.
- [ ] **M1.4 — Change events.** External-change notifications on both
  platforms; two-device live-update demo works.
- [ ] **M1.5 — Polish + publish.** TS bindings finalized, example app,
  entitlement docs, publish to crates.io + npm, announce (Tauri Discord,
  awesome-tauri PR).

## Definition of done

A third-party Tauri developer can add the plugin from crates.io/npm, follow
the README to configure entitlements, and have working cross-device KV sync
without reading the plugin source. Team Times (roadmap project 2) consumes
the published plugin, not a path dependency.

## Alternatives considered

- **Strings-only or strings+bytes API** — less mapping code, but worse
  ergonomics for the community's dominant small-preferences use case;
  base64-wrapped binary also wastes ~33% of the hard 1 MB quota. Rejected
  for JSON values.
- **Shared Swift core for both platforms** — single source of truth, but
  forces the Swift toolchain onto desktop consumers and CI, hurting
  adoption. Rejected for pure-Rust `objc2` on macOS.
- **Extending an existing plugin** — both existing iCloud plugins target
  file sync, not KVS; one is AGPL. Rejected; clean-room KVS plugin.
- **GitHub org home** — more "official" look, but org overhead for a
  single plugin and a weaker tie to the Team Times brand. Rejected for the
  personal account.
