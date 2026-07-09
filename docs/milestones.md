# Project 1: `tauri-plugin-icloud-kvs` (standalone open source)

> **Portability note:** Lives here; originated in the team-times roadmap,
> which retains the surrounding multi-platform context.

## Purpose

A Tauri 2 plugin exposing Apple's iCloud Key-Value Store
(`NSUbiquitousKeyValueStore`) to Tauri apps on **macOS and iOS**. Lets a
Tauri app sync small data (≤1 MB total, ≤1024 keys) across a user's Apple
devices with no server, no accounts, and no CloudKit schema.

Built by the makers of [Team Times](https://apps.apple.com/app/team-times)
(mention in README — this project doubles as advertising for the app).

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

- [ ] **M1.1 — Repo scaffold.** New repo from the Tauri 2 plugin template
  (Swift mobile + Rust desktop layout), license, CI (build + clippy + fmt per
  Rust standards), README skeleton, this document moved in as the plan.
- [ ] **M1.2 — macOS implementation.** get/set/remove/keys/synchronize
  working on macOS with the entitlement, verified manually between two Macs
  (or Mac + iCloud web-observable behavior). Unit tests for the Rust layer.
- [ ] **M1.3 — iOS implementation.** Same API via Swift mobile plugin;
  verified in the iOS simulator/device against the same iCloud container as
  the Mac build.
- [ ] **M1.4 — Change events.** External-change notifications emitted as
  Tauri events on both platforms; two-device live-update demo works.
- [ ] **M1.5 — Polish + publish.** TypeScript bindings finalized, example
  app, entitlement/provisioning docs, publish to crates.io + npm, announce
  (Tauri Discord, awesome-tauri PR).

## Definition of done

A third-party Tauri developer can add the plugin from crates.io/npm, follow
the README to configure entitlements, and have working cross-device KV sync
without reading the plugin source. Team Times project 2 consumes the
published plugin (not a path dependency).
