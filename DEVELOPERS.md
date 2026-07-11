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

## Cross-device sync verification

Manual protocol (two devices, one Apple ID) — to be documented with M1.4
change events. CI cannot exercise real iCloud sync.

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

## Planning

Design spec and milestone plans live in `docs/`.
