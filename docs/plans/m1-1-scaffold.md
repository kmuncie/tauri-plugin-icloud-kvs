# tauri-plugin-icloud-kvs M1.1 (Repo Scaffold) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create the `kmuncie/tauri-plugin-icloud-kvs` public repo with a compiling Tauri 2 plugin skeleton, tested error module, TypeScript guest package, green CI, and the planning docs moved in.

**Architecture:** Standard Tauri 2 plugin layout: Rust crate at the root (plugin builder + error types for now; KVS logic lands in M1.2), `guest-js/` TypeScript package publishing as `tauri-plugin-icloud-kvs-api`, GitHub Actions CI on a macOS runner. This milestone deliberately contains no `objc2`/Swift code.

**Tech Stack:** Rust (Tauri 2 plugin API, thiserror, serde), TypeScript (tsc only), GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-07-09-icloud-kvs-plugin-design.md` (moves into the new repo in Task 6).

## Global Constraints

- New repo path: `/Users/kmuncie/Code/PERSONAL/tauri-plugin-icloud-kvs`; GitHub: `kmuncie/tauri-plugin-icloud-kvs` (public)
- Crate name `tauri-plugin-icloud-kvs`; npm package `tauri-plugin-icloud-kvs-api`
- License: `MIT OR Apache-2.0` (dual, both full texts in repo)
- Rust 1.89.0, edition 2024, per Silvermine Rust standards: 3-space indent, `rustfmt.toml` + `.cargo/config.toml` aliases exactly as given in Task 2
- All Cargo dependencies pinned to exact semver versions (no caret/tilde) — Task 2 Step 6 defines the pinning procedure
- Errors via `thiserror`; doc examples annotated ` ```no_run `; unit tests live in the same file as the code
- Commit messages: Conventional Commits, imperative, ≤72-char subject
- Every commit in the new repo must pass `cargo lint-clippy && cargo lint-fmt && cargo test`

---

### Task 1: Repo creation, licenses, gitignore

**Files:**
- Create: `/Users/kmuncie/Code/PERSONAL/tauri-plugin-icloud-kvs/.gitignore`
- Create: `LICENSE-MIT`, `LICENSE-APACHE` (repo root)

**Interfaces:**
- Produces: initialized git repo with `main` branch that all later tasks commit into.

- [ ] **Step 1: Create local repo**

```bash
mkdir -p /Users/kmuncie/Code/PERSONAL/tauri-plugin-icloud-kvs
cd /Users/kmuncie/Code/PERSONAL/tauri-plugin-icloud-kvs
git init -b main
```

- [ ] **Step 2: Write `.gitignore`**

```gitignore
/target/
Cargo.lock
node_modules/
guest-js/dist/
.DS_Store
```

Note: `Cargo.lock` is ignored because this is a library crate (Rust convention).

- [ ] **Step 3: Fetch standard license texts**

```bash
curl -sSf https://raw.githubusercontent.com/rust-lang/rust/master/LICENSE-MIT -o LICENSE-MIT
curl -sSf https://raw.githubusercontent.com/rust-lang/rust/master/LICENSE-APACHE -o LICENSE-APACHE
```

Then edit `LICENSE-MIT`: replace the copyright line with
`Copyright (c) 2026 Kevin Muncie`. (`LICENSE-APACHE` needs no edit; Apache-2.0 text is used verbatim.)

- [ ] **Step 4: Verify**

Run: `head -3 LICENSE-MIT LICENSE-APACHE && git status --short`
Expected: MIT text begins with the copyright/permission lines; Apache text begins "Apache License"; three untracked files.

- [ ] **Step 5: Commit**

```bash
git add .gitignore LICENSE-MIT LICENSE-APACHE
git commit -m "chore: Initialize repo with dual MIT/Apache-2.0 license"
```

- [ ] **Step 6: Create GitHub repo and push**

```bash
gh repo create kmuncie/tauri-plugin-icloud-kvs --public \
   --description "Tauri 2 plugin for Apple's iCloud Key-Value Store (NSUbiquitousKeyValueStore) on macOS and iOS" \
   --source . --push
```

Expected: repo visible at github.com/kmuncie/tauri-plugin-icloud-kvs.

---

### Task 2: Rust crate scaffold with standards tooling

**Files:**
- Create: `Cargo.toml`, `build.rs`, `rust-toolchain.toml`, `rustfmt.toml`, `.cargo/config.toml`, `src/lib.rs`

**Interfaces:**
- Produces: `pub fn init<R: Runtime>() -> TauriPlugin<R>` in `src/lib.rs` (the plugin entry point every Tauri consumer calls); `cargo lint-clippy` / `lint-fmt` / `fix-clippy` / `fix-fmt` aliases used by CI (Task 5).

- [ ] **Step 1: Write `rust-toolchain.toml`**

```toml
[toolchain]
channel = "1.89.0"
components = [
   "rustfmt",
   "clippy"
]
```

- [ ] **Step 2: Write `rustfmt.toml`**

```toml
tab_spaces = 3
hard_tabs = false
newline_style = "Unix"
remove_nested_parens = false
use_field_init_shorthand = true
use_try_shorthand = true
```

- [ ] **Step 3: Write `.cargo/config.toml`**

```toml
[alias]
lint-clippy = "clippy --all-targets --all-features -- -D warnings"
fix-clippy = "clippy --fix"
lint-fmt = "fmt -- --check"
fix-fmt = "fmt"
```

- [ ] **Step 4: Write `Cargo.toml`**

Versions below are placeholders for the resolution procedure in Step 6 — do not skip Step 6.

```toml
[package]
name = "tauri-plugin-icloud-kvs"
version = "0.1.0"
description = "Tauri 2 plugin for Apple's iCloud Key-Value Store (NSUbiquitousKeyValueStore) on macOS and iOS"
authors = ["Kevin Muncie <kevin@kmuncie.com>"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/kmuncie/tauri-plugin-icloud-kvs"
edition = "2024"
rust-version = "1.89.0"
links = "tauri-plugin-icloud-kvs"

[dependencies]
tauri = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"

[build-dependencies]
tauri-plugin = { version = "2", features = ["build"] }
```

- [ ] **Step 5: Write `build.rs` and `src/lib.rs`**

`build.rs` (commands list is empty until M1.2 adds real commands):

```rust
const COMMANDS: &[&str] = &[];

fn main() {
   tauri_plugin::Builder::new(COMMANDS).build();
}
```

`src/lib.rs`:

```rust
//! Tauri 2 plugin exposing Apple's iCloud Key-Value Store
//! (`NSUbiquitousKeyValueStore`) on macOS and iOS.
//!
//! Lets a Tauri app sync small data (1 MB total, 1024 keys) across a
//! user's Apple devices with no server and no user accounts.
//!
//! # Examples
//!
//! ```no_run
//! tauri::Builder::default()
//!    .plugin(tauri_plugin_icloud_kvs::init());
//! ```

use tauri::{
   plugin::{Builder, TauriPlugin},
   Runtime,
};

/// Initializes the iCloud Key-Value Store plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
   Builder::new("icloud-kvs").build()
}
```

- [ ] **Step 6: Pin exact dependency versions**

```bash
cargo update
grep -A2 'name = "tauri"' Cargo.lock | head -3
```

Copy each top-level dependency's resolved version from `Cargo.lock` into `Cargo.toml` as an exact pin (e.g. if the lock resolved tauri 2.9.4, write `tauri = "2.9.4"`). Do this for `tauri`, `serde`, `serde_json`, `thiserror`, and `tauri-plugin`. Then re-run `cargo check` to confirm nothing shifted.

- [ ] **Step 7: Verify build and lints**

Run: `cargo check && cargo lint-clippy && cargo lint-fmt`
Expected: all pass with no warnings. (If `tauri_plugin::Builder` demands a `permissions/` dir even with zero commands, create empty `permissions/default.toml` containing `[default]\ndescription = "Default permissions for the plugin"\npermissions = []` and re-run.)

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml build.rs rust-toolchain.toml rustfmt.toml .cargo/config.toml src/lib.rs permissions/ 2>/dev/null || git add Cargo.toml build.rs rust-toolchain.toml rustfmt.toml .cargo/config.toml src/lib.rs
git commit -m "feat: Scaffold Tauri 2 plugin crate with standards tooling"
```

---

### Task 3: Error module (TDD)

**Files:**
- Create: `src/error.rs`
- Modify: `src/lib.rs` (add `mod error;` + re-exports)

**Interfaces:**
- Produces: `Error` enum with variants `UnsupportedPlatform`, `InvalidKey(String)`, `ValueTooLarge { size: usize }`, `Serialization(String)`, `PlatformError(String)`; `pub type Result<T> = std::result::Result<T, Error>`; `Error` implements `serde::Serialize` (as its Display string) so commands can return it to the frontend. M1.2 consumes all of this.

- [ ] **Step 1: Write `src/error.rs` with failing tests first**

Write the file with ONLY the test module (the enum doesn't exist yet):

```rust
#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn display_messages_are_actionable() {
      assert_eq!(
         Error::UnsupportedPlatform.to_string(),
         "iCloud Key-Value Store is only available on macOS and iOS"
      );
      assert_eq!(
         Error::InvalidKey("key exceeds 64 bytes".into()).to_string(),
         "invalid key: key exceeds 64 bytes"
      );
      assert_eq!(
         Error::ValueTooLarge { size: 2_000_000 }.to_string(),
         "value too large: 2000000 bytes exceeds the 1 MB iCloud KVS limit"
      );
   }

   #[test]
   fn serializes_as_display_string() {
      let json = serde_json::to_string(&Error::UnsupportedPlatform).unwrap();

      assert_eq!(
         json,
         "\"iCloud Key-Value Store is only available on macOS and iOS\""
      );
   }
}
```

Add to `src/lib.rs` below the doc comment:

```rust
mod error;

pub use error::{Error, Result};
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test`
Expected: FAIL to compile — `Error` not found.

- [ ] **Step 3: Implement the error enum above the test module**

```rust
use serde::{Serialize, Serializer};
use thiserror::Error;

/// Convenience result type used across the plugin.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by plugin commands.
///
/// Note: iCloud KVS quota exhaustion is NOT an error variant — the OS
/// reports it only asynchronously via the external-change notification
/// (`reason: quotaViolation`), never as a call-site failure.
#[derive(Debug, Error)]
pub enum Error {
   #[error("iCloud Key-Value Store is only available on macOS and iOS")]
   UnsupportedPlatform,

   #[error("invalid key: {0}")]
   InvalidKey(String),

   #[error("value too large: {size} bytes exceeds the 1 MB iCloud KVS limit")]
   ValueTooLarge { size: usize },

   #[error("serialization error: {0}")]
   Serialization(String),

   #[error("platform error: {0}")]
   PlatformError(String),
}

impl Serialize for Error {
   fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
      serializer.serialize_str(&self.to_string())
   }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test && cargo lint-clippy && cargo lint-fmt`
Expected: 2 tests PASS, lints clean.

- [ ] **Step 5: Commit**

```bash
git add src/error.rs src/lib.rs
git commit -m "feat: Add thiserror-based error types with frontend serialization"
```

---

### Task 4: TypeScript guest package scaffold

**Files:**
- Create: `guest-js/package.json`, `guest-js/tsconfig.json`, `guest-js/src/index.ts`

**Interfaces:**
- Produces: npm package `tauri-plugin-icloud-kvs-api` exporting the `KvsValue` type (the value model from the spec; M1.2 adds the functions). `npm run build` inside `guest-js/` is the CI contract.

- [ ] **Step 1: Write `guest-js/package.json`**

```json
{
   "name": "tauri-plugin-icloud-kvs-api",
   "version": "0.1.0",
   "description": "TypeScript bindings for tauri-plugin-icloud-kvs (Apple iCloud Key-Value Store for Tauri 2)",
   "license": "MIT OR Apache-2.0",
   "repository": "github:kmuncie/tauri-plugin-icloud-kvs",
   "type": "module",
   "main": "dist/index.js",
   "types": "dist/index.d.ts",
   "files": ["dist"],
   "scripts": {
      "build": "tsc"
   },
   "devDependencies": {
      "typescript": "5.8.3"
   }
}
```

(Pin `typescript` to the latest exact version at implementation time: `npm view typescript version`.)

- [ ] **Step 2: Write `guest-js/tsconfig.json`**

```json
{
   "compilerOptions": {
      "target": "ES2021",
      "module": "ESNext",
      "moduleResolution": "bundler",
      "strict": true,
      "declaration": true,
      "outDir": "dist",
      "rootDir": "src"
   },
   "include": ["src"]
}
```

- [ ] **Step 3: Write `guest-js/src/index.ts`**

```ts
/**
 * TypeScript bindings for tauri-plugin-icloud-kvs.
 *
 * Functions (get/set/remove/keys/getAll/synchronize/accountStatus and
 * onExternalChange) are added alongside their Rust commands in later
 * milestones; this module currently exports the value model only.
 */

/**
 * A JSON value storable in iCloud KVS. Mapped to native property-list
 * types (NSString, NSNumber, NSArray, NSDictionary). `null` is not
 * storable — use `remove` to delete a key.
 */
export type KvsValue =
   | string
   | number
   | boolean
   | KvsValue[]
   | { [key: string]: KvsValue };
```

- [ ] **Step 4: Verify build**

Run: `cd guest-js && npm install && npm run build && cd ..`
Expected: `guest-js/dist/index.js` and `index.d.ts` produced, no errors.

- [ ] **Step 5: Commit**

```bash
git add guest-js/package.json guest-js/tsconfig.json guest-js/src/index.ts guest-js/package-lock.json
git commit -m "feat: Scaffold TypeScript guest package with KvsValue model"
```

---

### Task 5: CI workflow

**Files:**
- Create: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: cargo aliases from Task 2, `npm run build` contract from Task 4.
- Produces: required CI check for all future PRs. (iOS compile check is added in M1.3 when the iOS target exists.)

- [ ] **Step 1: Write `.github/workflows/ci.yml`**

```yaml
name: CI

on:
   push:
      branches: [main]
   pull_request:

jobs:
   rust:
      runs-on: macos-14
      steps:
         - uses: actions/checkout@v4
         - name: Install toolchain from rust-toolchain.toml
           run: rustup show active-toolchain || rustup toolchain install
         - uses: Swatinem/rust-cache@v2
         - name: Lint
           run: cargo lint-clippy && cargo lint-fmt
         - name: Test
           run: cargo test

   typescript:
      runs-on: macos-14
      steps:
         - uses: actions/checkout@v4
         - uses: actions/setup-node@v4
           with:
              node-version: 22
         - name: Build guest bindings
           run: npm ci && npm run build
           working-directory: guest-js
```

- [ ] **Step 2: Commit and push**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: Add lint, test, and TypeScript build workflow"
git push
```

- [ ] **Step 3: Verify CI is green**

Run: `gh run watch --repo kmuncie/tauri-plugin-icloud-kvs --exit-status` (or `gh run list --limit 1` and check status)
Expected: latest run concludes `success` on both jobs. If a job fails, fix the workflow (common issue: rustup toolchain install syntax) and push again before proceeding.

---

### Task 6: Documentation and planning-doc migration

**Files:**
- Create: `README.md`, `DEVELOPERS.md`, `docs/` (in the plugin repo)
- Modify (team-times repo): `docs/roadmap/index.md`, `docs/roadmap/01-icloud-kvs-plugin.md`

**Interfaces:**
- Consumes: spec at `team-times/docs/superpowers/specs/2026-07-09-icloud-kvs-plugin-design.md`, roadmap detail at `team-times/docs/roadmap/01-icloud-kvs-plugin.md`, this plan.
- Produces: the plugin repo becomes the planning home for project 1; team-times roadmap links point at GitHub.

- [ ] **Step 1: Write `README.md`** (consumer-facing skeleton; full API docs land with the features)

```markdown
# tauri-plugin-icloud-kvs

Sync small data across a user's Apple devices from a [Tauri 2](https://tauri.app)
app — no server, no user accounts, no CloudKit schema. This plugin exposes
Apple's iCloud Key-Value Store
([`NSUbiquitousKeyValueStore`](https://developer.apple.com/documentation/foundation/nsubiquitouskeyvaluestore))
on **macOS** and **iOS**.

> ⚠️ **Status: under development.** The API is not yet stable and the
> crate/npm packages are not yet published. Watch releases for 0.1.0.

Built by the maker of [Team Times](https://apps.apple.com/app/team-times),
a menu-bar app for tracking distributed teams across time zones — this
plugin powers its cross-device config sync.

## Platform support

| Platform | Support |
|----------|---------|
| macOS    | Planned (pure Rust via `objc2`) |
| iOS      | Planned (Swift) |
| Others   | Commands return an `UnsupportedPlatform` error |

## What iCloud KVS gives you (and its limits)

- 1 MB total per app, max 1024 keys, key names ≤ 64 bytes UTF-8
- Last-writer-wins conflict resolution; sync latency of seconds (no guarantees)
- Quota violations are reported **asynchronously** via change events, never
  as a call-site error
- Requires the `com.apple.developer.ubiquity-kvstore-identifier` entitlement
  (setup guide coming with the first release)

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at
your option.
```

- [ ] **Step 2: Write `DEVELOPERS.md`**

```markdown
# Developing tauri-plugin-icloud-kvs

## Prerequisites

- Rust (version pinned in `rust-toolchain.toml`; rustup installs it automatically)
- Node.js 22+ (for the TypeScript guest bindings)
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

## Planning

Design spec and milestone plans live in `docs/`.
```

- [ ] **Step 3: Move planning docs into the plugin repo**

```bash
cd /Users/kmuncie/Code/PERSONAL/tauri-plugin-icloud-kvs
mkdir -p docs/plans
cp /Users/kmuncie/Code/PERSONAL/team-times/docs/superpowers/specs/2026-07-09-icloud-kvs-plugin-design.md docs/design-spec.md
cp /Users/kmuncie/Code/PERSONAL/team-times/docs/roadmap/01-icloud-kvs-plugin.md docs/milestones.md
cp /Users/kmuncie/Code/PERSONAL/team-times/docs/superpowers/plans/2026-07-09-icloud-kvs-plugin-m1-1-scaffold.md docs/plans/m1-1-scaffold.md
```

Then edit the two copied files' portability notes: in `docs/design-spec.md` and `docs/milestones.md`, replace the "moves to the plugin repo" note with "Lives here; originated in the team-times roadmap, which retains the surrounding multi-platform context."

- [ ] **Step 4: Commit and push the plugin repo**

```bash
git add README.md DEVELOPERS.md docs/
git commit -m "docs: Add README, developer guide, and imported planning docs"
git push
```

- [ ] **Step 5: Update team-times to point at the new repo**

In `/Users/kmuncie/Code/PERSONAL/team-times`:

- `docs/roadmap/index.md`: in the Projects table, change project 1's Detail link to `https://github.com/kmuncie/tauri-plugin-icloud-kvs/blob/main/docs/milestones.md`; update its Status to "In progress (M1.1 done)"; delete the portability paragraph under "Working Model".
- Replace the body of `docs/roadmap/01-icloud-kvs-plugin.md` with a stub:

```markdown
# Project 1: `tauri-plugin-icloud-kvs`

Moved. Planning now lives in the plugin repo:
<https://github.com/kmuncie/tauri-plugin-icloud-kvs/tree/main/docs>
```

- [ ] **Step 6: Commit team-times**

```bash
cd /Users/kmuncie/Code/PERSONAL/team-times
git add docs/roadmap/index.md docs/roadmap/01-icloud-kvs-plugin.md
git commit -m "docs: Point roadmap project 1 at the new plugin repo"
```

- [ ] **Step 7: Verify milestone complete**

Run in the plugin repo: `cargo lint-clippy && cargo lint-fmt && cargo test && (cd guest-js && npm run build)` and `gh run list --limit 1`
Expected: all green locally, latest CI run `success`. M1.1 is done — check it off in `docs/milestones.md` (plugin repo) in a final `docs:` commit.
