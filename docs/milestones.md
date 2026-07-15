# Project 1: `tauri-plugin-icloud-kvs`

> **Portability note:** Lives here; originated in the team-times roadmap,
> which retains the surrounding multi-platform context.

## Purpose

A Tauri 2 plugin exposing Apple's iCloud Key-Value Store
(`NSUbiquitousKeyValueStore`) to Tauri apps on **macOS and iOS**. Lets a
Tauri app sync small data (≤1 MB total, ≤1024 keys) across a user's Apple
devices with no server, no accounts, and no CloudKit schema.

Built by the makers of [Team Times](https://kmuncie.com/team-times/)
(the README credits the app).

## Why it needs to exist

As of July 2026 there is no production-quality option in the ecosystem:

- `justinwaltrip/tauri-plugin-icloud` — iOS-only, file-oriented (iCloud
  Drive), self-described not production ready, AGPL-3.0.
- `TensorBinge/tauri-plugin-icloud-container` — iOS-only ubiquity-container
  file sync, pre-1.0, no desktop support.

Neither covers `NSUbiquitousKeyValueStore`, and neither supports macOS.

## Scope

**In scope:**

- `get(key)` / `set(key, value)` / `remove(key)` / `keys()` / `synchronize()`
- Change notifications (`NSUbiquitousKeyValueStoreDidChangeExternallyNotification`)
  surfaced as a Tauri event, including the change reason (server change,
  initial sync, quota violation, account change)
- macOS implementation (Rust via `objc2` bindings, or a small Swift/ObjC shim)
- iOS implementation (Swift, via Tauri 2's mobile plugin system)
- TypeScript guest bindings with typed API
- Permissions/capabilities definitions per Tauri 2 plugin conventions
- Documentation for the required entitlement
  (`com.apple.developer.ubiquity-kvstore-identifier`) and provisioning setup,
  including the sandboxed Mac App Store case
- Example app demonstrating two-device sync
- Published to crates.io + npm; MIT/Apache-2.0 dual license (community
  standard; explicitly not AGPL)

**Out of scope:**

- iCloud Drive / ubiquity container file sync
- CloudKit databases
- Android/Windows no-op shims (callers gate on platform; a graceful
  "unsupported platform" error is enough)
- Conflict resolution beyond what KVS provides (last-writer-wins)

## Design questions to settle in the brainstorm/spec phase

- API value type: strings only (caller serializes) vs JSON values vs bytes
- Event payload shape and whether changed keys are enumerated per event;
  must expose the change reason (server change, initial sync, quota
  violation, account change) — quota violations are only observable via
  the notification, never as a `set` error
- Whether `set` auto-calls `synchronize()` or leaves flushing to the caller
  (note: `synchronize()` only flushes locally and requests upload; it does
  not force a server round-trip — document this so callers don't build
  "sync now" UX on it)
- Whether the plugin offers account-status introspection (signed out of
  iCloud silently degrades to local-only; callers need a way to detect it)
- Crate/package naming and org placement (personal account vs new org)
- macOS approach: pure-Rust `objc2` vs sharing the Swift implementation

## Milestones

- [x] **M1.1 — Repo scaffold.** New repo from the Tauri 2 plugin template
  (Swift mobile + Rust desktop layout), license, CI (build + clippy + fmt per
  Rust standards), README skeleton, this document moved in as the plan.
- [x] **M1.2 — macOS implementation.** get/set/remove/keys/synchronize
  working on macOS. Unit tests for the Rust layer; store round-trip test
  gated behind `KVS_INTEGRATION=1` (unentitled processes get an inert
  store — see `DEVELOPERS.md`). Real-device sync verification is
  **deferred to Team Times integration** (~M1.5): contributors never need
  entitled hardware, but sync must be observed working on real devices
  once before the 0.1.0 publish/announce. M1.1-review carryovers
  (permissions/ dir timing, error-test backfill, `Error::Platform`
  rename) all landed during implementation.
- [x] **M1.3 — iOS implementation.** Same API on iOS by sharing the
  macOS objc2 implementation — the spec's premise that Tauri 2 requires
  a Swift half was wrong and is corrected in the design spec. CI
  cross-compiles with clippy for `aarch64-apple-ios`. Command
  round-trips verified 2026-07-13 in the iOS simulator via
  `examples/demo-app` (entitled build; functional store, `noAccount`
  local-only mode confirmed working). The demo app (an M1.5
  deliverable, pulled forward) also caught and fixed a latent crash:
  `keys()`/`get_all()` panicked on the NULL `dictionaryRepresentation`
  every unentitled process gets. Cross-device sync against the Mac
  build stays deferred to Team Times integration (~M1.5).
- [x] **M1.4 — Change events.** External-change notifications emitted as
  Tauri events (`icloud-kvs://external-change`) on both platforms: an
  always-on `NSNotificationCenter` observer registered at plugin setup,
  with pure, unit-tested notification parsing (unknown future reason
  codes drop the event rather than mis-label it). `onExternalChange()`
  added to the guest bindings, and the demo app's placeholder pane
  replaced with a live event log that also refreshes the KV table. The
  M1.2-review carryover landed too: foreign `NSDate` values now read
  back as ISO-8601 UTC strings instead of failing all of `get_all`.
  Live two-device event delivery follows the protocol in
  `DEVELOPERS.md` (step 5) and stays deferred to Team Times
  integration (~M1.5), per the testing policy.
- [ ] **M1.5 — Polish + publish.** TypeScript bindings finalized, example
  app, entitlement/provisioning docs, publish to crates.io + npm, announce
  (Tauri Discord, awesome-tauri PR). Carried over from the M1.1 review: the
  npm package's `"files": ["dist"]` won't include the repo-root license
  texts — copy `LICENSE-*` into `guest-js/` at publish time (e.g. a
  `prepack` script) so the tarball ships license texts.

## Definition of done

A third-party Tauri developer can add the plugin from crates.io/npm, follow
the README to configure entitlements, and have working cross-device KV sync
without reading the plugin source. Team Times project 2 consumes the
published plugin (not a path dependency).
