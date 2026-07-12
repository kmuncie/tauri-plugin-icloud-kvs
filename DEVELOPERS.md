# Developing tauri-plugin-icloud-kvs

## Prerequisites

- Rust (version pinned in `rust-toolchain.toml`; rustup installs it automatically)
- Node.js 24 (pinned in `.nvmrc`; run `nvm use`) — for the TypeScript guest bindings
- macOS (the plugin targets Apple platforms; tests require macOS)

## Commands

| Task | Command |
|------|---------|
| Lint | `cargo lint-clippy && cargo lint-fmt` |
| Auto-fix lints | `cargo fix-clippy && cargo fix-fmt` |
| Test | `cargo test` |
| Build TS bindings | `cd guest-js && npm install && npm run build` |

## Standards

3-space indentation, exact-pinned dependencies, `thiserror` for errors,
unit tests in-file, doc examples annotated `no_run`. See `rustfmt.toml`
and `.cargo/config.toml`.

## Cross-device sync verification (manual)

CI only runs the non-store-dependent unit/integration tests (see
"Integration test" below); real iCloud sync needs signed, entitled app
bundles on two devices with the same Apple ID. Protocol:

1. Build a scratch Tauri app (or, once it exists, `examples/demo-app`)
   that registers this plugin, with the
   `com.apple.developer.ubiquity-kvstore-identifier` entitlement set to
   `$(TeamIdentifierPrefix)$(CFBundleIdentifier)` and codesigned with a
   Development certificate on both Macs.
2. On Mac A: `set('sync-check', <timestamp>)`.
3. On Mac B: poll `get('sync-check')` (KVS latency is seconds to
   minutes; no guarantees). The value arriving proves upload + download.
4. Repeat in the reverse direction.

`accountStatus()` must report `available` on both machines first.

## Integration test: `tests/kvs_roundtrip.rs`

`NSUbiquitousKeyValueStore` only has a functional backing store (even the
local-only, no-account fallback) inside a process code-signed with the
`com.apple.developer.ubiquity-kvstore-identifier` entitlement. A plain
`cargo test` binary is ad-hoc signed with no entitlements and no
`Info.plist`, so every write/read silently no-ops and `-synchronize`
returns `false` — this is true on any machine, not just CI runners.

`round_trips_set_get_keys_get_all_remove` is gated behind
`KVS_INTEGRATION=1` for this reason; the other three tests in that file
don't depend on a functional store and always run. To exercise the real
round-trip, run the assertions from a signed, entitled host (e.g. inside
a Tauri dev build of the example app) rather than via `cargo test`.

## iOS verification (manual, simulator)

The iOS build shares the macOS objc2 implementation; CI proves it
compiles (`cargo clippy --target aarch64-apple-ios`). Running that
check locally requires **full Xcode** (the Command Line Tools ship
only the macOS SDK, and dependency build scripts compile C against
the iOS SDK) — without it, rely on CI. Exercising the commands needs
a host app in the simulator:

1. Create a scratch Tauri app, register this plugin as a path
   dependency, and run `tauri ios init`.
2. In the generated Xcode project, add the iCloud capability with
   "Key-value storage" checked (this sets the
   `com.apple.developer.ubiquity-kvstore-identifier` entitlement).
3. Run `tauri ios dev` into a simulator signed in to an Apple ID
   (Settings → sign in), confirm `accountStatus()` reports
   `available`, then exercise set/get/keys/getAll/remove round-trips
   from the webview.

Cross-device sync against the Mac build (same container) is deferred
to the Team Times integration (~M1.5), per the testing policy:
contributors never need entitled hardware.

## Planning

Design spec and milestone plans live in `docs/`.
