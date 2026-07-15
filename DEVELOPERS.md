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

1. Build `examples/demo-app` (see its README for signing) — it registers
   this plugin, with the
   `com.apple.developer.ubiquity-kvstore-identifier` entitlement set to
   `$(TeamIdentifierPrefix)$(CFBundleIdentifier)` and codesigned with a
   Development certificate on both Macs.
2. On Mac A: `set('sync-check', <timestamp>)`.
3. On Mac B: poll `get('sync-check')` (KVS latency is seconds to
   minutes; no guarantees). The value arriving proves upload + download.
4. Repeat in the reverse direction.
5. Change events: with the demo app open on Mac B, run
   `set('sync-check', <new value>)` on Mac A. Mac B's "External
   Changes" pane must log a `serverChange` event listing `sync-check`
   within the same latency window, and its KV table must refresh to
   the new value without user action. Note: the OS only delivers these
   notifications to processes that called `synchronize()` once after
   launch — the plugin does this automatically at setup.

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

1. In `examples/demo-app`, run `npm run tauri ios init` (see its
   README).
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

## Publishing a release

Two stages. Stage 1 (rc) happened with M1.5; Stage 2 (stable) is
gated on Team Times real-device verification.

### Stage 1 — release candidate

1. Checklist: CI green on `main`; versions are `0.1.0-rc.N` in
   `Cargo.toml` **and** `guest-js/package.json`; `CHANGELOG.md` has
   the rc entry.
2. Rehearse: `cargo publish --dry-run` and
   `cd guest-js && npm pack --dry-run` (tarball must list `dist/`,
   `README.md`, both `LICENSE-*` files).
3. `cargo login` (token from crates.io/settings/tokens), then
   `cargo publish`.
4. `cd guest-js && npm login && npm publish --tag rc`.
   ⚠️ **Never untagged before stable** — an untagged publish becomes
   `latest`. Safety net: `publishConfig.tag` in `guest-js/package.json`
   is set to `rc`, so even a bare `npm publish` tags correctly. (The
   0.1.0-rc.1 publish predated this guard and ran untagged; `latest`
   points at the rc until stable ships. Accepted deviation.)
5. `git tag v0.1.0-rc.1 && git push --tags`.
6. Hand the rc coordinates to the consuming app (Team Times) for the
   real-device protocol above. No announcements at rc.

### Stage 2 — stable 0.1.0 (gated)

1. Gate checklist: cross-device sync **and** change events observed
   on real hardware via the protocol above; no open rc-found issues;
   `CHANGELOG.md` gains the `0.1.0` entry.
2. Bump both versions to `0.1.0`, update the README status note and
   install snippets (drop `@rc` / `@0.1.0-rc.1` pins), **remove
   `publishConfig` from `guest-js/package.json`** (so the publish
   tags `latest`), commit, re-run the rehearsals.
3. `cargo publish`, then `cd guest-js && npm publish` (now sets
   `latest`).
4. `git tag v0.1.0 && git push --tags`.
5. Announce: post the drafts in `docs/announcements/` (Tauri Discord
   #plugins, awesome-tauri PR).
6. Bump Team Times to the stable versions.

## Planning

Design spec and milestone plans live in `docs/`.
