# tauri-plugin-icloud-kvs M1.5 (Polish + rc publish) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the plugin fully publish-ready (docs, packaging, metadata, changelog, runbook, announce drafts) and finish with a guided publish of `0.1.0-rc.1` to crates.io and npm (`rc` dist-tag), leaving stable `0.1.0` gated on Team Times real-device verification.

**Architecture:** Pure packaging/documentation work — no plugin code changes. Registry metadata and version bumps land first (they gate the dry-run rehearsals), then the README consumer guide, changelog, runbook, and announce drafts. The final task is an interactive checkpoint: Kevin executes the actual `cargo publish` / `npm publish --tag rc` commands after rehearsals pass.

**Tech Stack:** cargo publish/doc tooling, npm pack/publish lifecycle (`prepack`), Keep a Changelog format.

**Spec:** `docs/m1-5-publish-spec.md`. Milestone definition: `docs/milestones.md` (M1.5 entry).

## Global Constraints

- No plugin feature or API changes; any behavioral issue found becomes a GitHub issue, not a fix in this plan
- Version everywhere is exactly `0.1.0-rc.1` (Cargo.toml and guest-js/package.json); the demo app's path/file dependencies are version-less and stay untouched
- npm must NEVER be published without `--tag rc` until stable 0.1.0 (the first untagged publish becomes `latest`)
- Announcements are drafts only — nothing gets posted in this effort
- Every commit passes `cargo test && cargo lint-clippy && cargo lint-fmt` on the host (macOS); `Cargo.lock` is gitignored — never commit it
- Commit messages: Conventional Commits, imperative, ≤72-char subject
- Docs style: existing README voice; 3-space indent in code snippets; Rust doc examples stay `no_run`
- crates.io keyword rules: max 5, ≤20 chars each; category slugs must be from the official list (`os::macos-apis`, `api-bindings` are valid)

---

### Task 1: crates.io metadata + version bump

**Files:**
- Modify: `Cargo.toml`

**Interfaces:**
- Consumes: nothing
- Produces: crate version `0.1.0-rc.1` (Tasks 3, 4, 6, 8 reference this string); `cargo publish --dry-run` green (Task 8 rehearses it again)

- [ ] **Step 1: Update `[package]`**

In `Cargo.toml`, change the version line and add keywords/categories/exclude after the existing `repository` line:

```toml
version = "0.1.0-rc.1"
```

```toml
repository = "https://github.com/kmuncie/tauri-plugin-icloud-kvs"
keywords = ["tauri-plugin", "icloud", "key-value", "sync", "apple"]
categories = ["os::macos-apis", "api-bindings"]
exclude = ["docs/", "examples/", ".github/"]
```

- [ ] **Step 2: Add the docs.rs metadata block**

Append at the end of `Cargo.toml`:

```toml
[package.metadata.docs.rs]
default-target = "aarch64-apple-darwin"
targets = [
   "aarch64-apple-darwin",
   "aarch64-apple-ios",
   "x86_64-unknown-linux-gnu",
]
```

(docs.rs builds docs for Apple targets from Linux hosts; the objc2 crates support metadata-only doc builds cross-target. The Linux target proves the `UnsupportedPlatform` stub documents cleanly.)

- [ ] **Step 3: Verify**

Run: `cargo test && cargo lint-clippy && cargo lint-fmt`
Expected: green.

Run: `cargo doc --no-deps 2>&1 | tee /dev/stderr | grep -ci warning`
Expected: `0` warnings (command prints the doc output; the count must be 0).

Run: `cargo publish --dry-run --allow-dirty 2>&1 | tail -5`
Expected: ends with `warning: aborting upload due to dry run` (build + verify succeeded). If it errors on missing fields or packaging problems, fix them now.

Sanity-check the package listing excludes docs/examples:

Run: `cargo package --list --allow-dirty | head -30`
Expected: `src/`, `permissions/`, `build.rs`, `Cargo.toml`, `README.md`, `LICENSE-*` present; nothing under `docs/`, `examples/`, or `.github/`.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml
git commit -m "build: Add crates.io metadata and bump to 0.1.0-rc.1"
```

---

### Task 2: npm packaging (license carryover + rc version)

**Files:**
- Modify: `guest-js/package.json`
- Modify: `.gitignore`

**Interfaces:**
- Consumes: nothing
- Produces: npm version `0.1.0-rc.1`; a `prepack` that builds and copies `README.md`, `LICENSE-MIT`, `LICENSE-APACHE` into `guest-js/` (Task 8 relies on this during `npm publish`)

- [ ] **Step 1: Update `guest-js/package.json`**

Change `version`, add `keywords`, extend `files`, and add `prepack`:

```json
{
   "name": "tauri-plugin-icloud-kvs-api",
   "version": "0.1.0-rc.1",
   "description": "TypeScript bindings for tauri-plugin-icloud-kvs (Apple iCloud Key-Value Store for Tauri 2)",
   "license": "MIT OR Apache-2.0",
   "repository": "github:kmuncie/tauri-plugin-icloud-kvs",
   "keywords": ["tauri", "tauri-plugin", "icloud", "key-value", "sync", "apple"],
   "type": "module",
   "main": "dist/index.js",
   "types": "dist/index.d.ts",
   "files": [
      "dist",
      "README.md",
      "LICENSE-MIT",
      "LICENSE-APACHE"
   ],
   "engines": {
      "node": ">=24"
   },
   "scripts": {
      "build": "tsc",
      "prepack": "npm run build && cp ../README.md ../LICENSE-MIT ../LICENSE-APACHE ."
   },
   "devDependencies": {
      "typescript": "7.0.2"
   },
   "dependencies": {
      "@tauri-apps/api": "2.11.1"
   }
}
```

- [ ] **Step 2: Ignore the prepack copies**

Append to `.gitignore`:

```
guest-js/README.md
guest-js/LICENSE-MIT
guest-js/LICENSE-APACHE
```

- [ ] **Step 3: Verify the tarball contents**

Run: `cd guest-js && npm pack --dry-run 2>&1 | grep -E "README|LICENSE|dist/|name:|version:"`
Expected: listing shows `dist/index.js`, `dist/index.d.ts`, `README.md`, `LICENSE-MIT`, `LICENSE-APACHE`; `version: 0.1.0-rc.1`. (npm ≥7 runs `prepack` even for `--dry-run`; if the copies are missing, run plain `npm pack`, inspect the same way, and delete the generated `.tgz`.)

Then confirm the copies are ignored: `git status --short` must show only `package.json` and `.gitignore` as modified — no untracked `guest-js/README.md` or `guest-js/LICENSE-*`.

- [ ] **Step 4: Commit**

```bash
git add guest-js/package.json .gitignore
git commit -m "build: Ship licenses and README in the npm tarball"
```

Body:

```
M1.1-review carryover: "files": ["dist"] omitted the repo-root license
texts, so published tarballs carried no license files. prepack now
builds and copies README + both licenses into guest-js/ (gitignored
there). Also bumps to 0.1.0-rc.1 and adds npm keywords.
```

---

### Task 3: README consumer guide

Replaces the "guide coming with the first release" placeholders with the real entitlement guide, quota section, API reference, and gotchas. This README ships inside both packages (crate root + npm prepack copy), so it must stand alone for a consumer who never opens the repo.

**Files:**
- Modify: `README.md`

**Interfaces:**
- Consumes: version string `0.1.0-rc.1` (Task 1)
- Produces: final consumer README (Tasks 6's announce drafts link to its sections by name: "Setup", "Quota")

- [ ] **Step 1: Update the status note**

Replace the current status blockquote (lines 9–10):

```markdown
> 🧪 **Status: release candidate.** `0.1.0-rc.1` is published for
> early testing — pin it explicitly (pre-releases are never resolved
> by default). Stable 0.1.0 ships once cross-device sync has been
> verified on real hardware.
```

- [ ] **Step 2: Rewrite "Usage (pre-release)" as "Setup"**

Replace the whole `## Usage (pre-release)` section (heading through the paragraph ending "detect the signed-out case.") with:

````markdown
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
[`examples/demo-app`](examples/demo-app/README.md).

### 4. Detect the signed-out case

With the entitlement but no iCloud account signed in, the store works
**locally but never syncs** — and nothing errors. Surface it:

```ts
import { accountStatus } from 'tauri-plugin-icloud-kvs-api';

if (await accountStatus() === 'noAccount') {
   // Warn: data stays on this device until iCloud sign-in.
}
```
````

- [ ] **Step 3: Add the API reference**

Insert a new section between "Setup" and "What iCloud KVS gives you":

````markdown
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
````

- [ ] **Step 4: Expand the limits section with quota + gotchas**

Replace the `## What iCloud KVS gives you (and its limits)` section body:

````markdown
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
````

- [ ] **Step 5: Verify and commit**

Check no placeholder text remains: `grep -n "coming with the first release\|not yet published" README.md`
Expected: no matches.

Confirm every claim against the actual API (`guest-js/src/index.ts`): eight functions, exact return types, event name `icloud-kvs://external-change`.

```bash
git add README.md
git commit -m "docs: Write consumer setup guide, API reference, and gotchas"
```

---

### Task 4: CHANGELOG + API-surface audit

**Files:**
- Create: `CHANGELOG.md`
- Possibly modify (doc comments only): `guest-js/src/index.ts`, `src/lib.rs`, `src/models.rs`

**Interfaces:**
- Consumes: version `0.1.0-rc.1`
- Produces: `CHANGELOG.md` (Task 5's runbook tells Stage 2 to add the 0.1.0 entry)

- [ ] **Step 1: Create `CHANGELOG.md`**

```markdown
# Changelog

All notable changes to this project will be documented in this file.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versioning: [SemVer](https://semver.org/).

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
```

Use today's actual date if it is no longer 2026-07-14.

- [ ] **Step 2: API-surface audit**

Read-through checklist (fix trivial doc issues inline; anything
behavioral becomes a GitHub issue instead):

1. `guest-js/src/index.ts`: every export has a doc comment; names and
   types match the README API table exactly.
2. `src/lib.rs`: crate doc comment current (mentions change events);
   every `pub use` item documented.
3. `src/models.rs` / `src/error.rs`: public types documented;
   serialized names match the TS types (`camelCase`).
4. `cargo doc --no-deps --open` — skim the rendered front page once.

Expected: no changes, or doc-comment tweaks only.

- [ ] **Step 3: Verify and commit**

Run: `cargo test && cargo lint-clippy && cargo lint-fmt && (cd guest-js && npm run build)`
Expected: green.

```bash
git add CHANGELOG.md guest-js/src/index.ts src/lib.rs src/models.rs
git commit -m "docs: Add changelog for 0.1.0-rc.1"
```

(Only add the source files if the audit actually touched them.)

---

### Task 5: Publish runbook in DEVELOPERS.md

**Files:**
- Modify: `DEVELOPERS.md` (new section before "Planning")

**Interfaces:**
- Consumes: rc flow decisions (spec); Task 2's prepack behavior
- Produces: the runbook Task 8 executes (Stage 1) and Kevin executes later (Stage 2)

- [ ] **Step 1: Add the section**

Insert before the `## Planning` heading:

````markdown
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
   ⚠️ **Never untagged before stable** — the first untagged publish
   becomes `latest`.
5. `git tag v0.1.0-rc.1 && git push --tags`.
6. Hand the rc coordinates to the consuming app (Team Times) for the
   real-device protocol above. No announcements at rc.

### Stage 2 — stable 0.1.0 (gated)

1. Gate checklist: cross-device sync **and** change events observed
   on real hardware via the protocol above; no open rc-found issues;
   `CHANGELOG.md` gains the `0.1.0` entry.
2. Bump both versions to `0.1.0`, update the README status note and
   install snippets (drop `@rc` / `@0.1.0-rc.1` pins), commit,
   re-run the rehearsals.
3. `cargo publish`, then `cd guest-js && npm publish` (untagged —
   this sets `latest`).
4. `git tag v0.1.0 && git push --tags`.
5. Announce: post the drafts in `docs/announcements/` (Tauri Discord
   #plugins, awesome-tauri PR).
6. Bump Team Times to the stable versions.
````

- [ ] **Step 2: Commit**

```bash
git add DEVELOPERS.md
git commit -m "docs: Add two-stage publish runbook"
```

---

### Task 6: Announce drafts

**Files:**
- Create: `docs/announcements/discord-0.1.0.md`
- Create: `docs/announcements/awesome-tauri-pr.md`

**Interfaces:**
- Consumes: README section names ("Setup", "Quota") from Task 3
- Produces: paste-ready drafts used in runbook Stage 2 step 5

- [ ] **Step 1: Write `docs/announcements/discord-0.1.0.md`**

```markdown
# Tauri Discord post (#plugins) — post at STABLE 0.1.0, not rc

---

**tauri-plugin-icloud-kvs 0.1.0** 🍎☁️

Sync small data across a user's Apple devices from a Tauri 2 app — no
server, no accounts, no CloudKit schema. Wraps Apple's iCloud
Key-Value Store (`NSUbiquitousKeyValueStore`) on **macOS** and
**iOS**.

- JSON values: `get` / `set` / `remove` / `keys` / `getAll`
- External changes arrive as Tauri events (`onExternalChange`) —
  another device writes, your UI updates
- `accountStatus()` detects the signed-out (local-only) case
- Pure Rust via objc2 — no Swift toolchain needed
- Demo app included (KV editor + live event log)

Good fit for: settings/preferences sync, "continue on your other
device" state. Not for big data — Apple caps the store at 1 MB /
1024 keys, and it needs the iCloud KVS entitlement (README has a
step-by-step signing guide).

📦 crates.io: <https://crates.io/crates/tauri-plugin-icloud-kvs>
📦 npm: <https://www.npmjs.com/package/tauri-plugin-icloud-kvs-api>
🔗 GitHub: <https://github.com/kmuncie/tauri-plugin-icloud-kvs>

Built for [Team Times](https://kmuncie.com/team-times/), which uses it
in production for cross-device config sync. Feedback welcome!
```

- [ ] **Step 2: Write `docs/announcements/awesome-tauri-pr.md`**

````markdown
# awesome-tauri PR — open at STABLE 0.1.0, not rc

Repo: <https://github.com/tauri-apps/awesome-tauri> (check
CONTRIBUTING.md for current format/ordering rules before opening).

## List entry (Plugins section, alphabetical)

```
- [icloud-kvs](https://github.com/kmuncie/tauri-plugin-icloud-kvs) - Sync small data across a user's Apple devices via iCloud Key-Value Store (macOS, iOS).
```

## PR title

```
Add tauri-plugin-icloud-kvs to Plugins
```

## PR description

```
Adds tauri-plugin-icloud-kvs: exposes Apple's iCloud Key-Value Store
(NSUbiquitousKeyValueStore) to Tauri 2 apps on macOS and iOS —
serverless key-value sync across a user's devices, with external
changes delivered as Tauri events.

- Published on crates.io + npm (0.1.0)
- MIT OR Apache-2.0
- Docs cover the required entitlement/signing setup end to end
- Includes a demo app used as the manual test rig
```
````

- [ ] **Step 3: Commit**

```bash
git add docs/announcements/
git commit -m "docs: Draft stable-release announcements"
```

---

### Task 7: Milestones amendment

**Files:**
- Modify: `docs/milestones.md` (M1.5 entry + "Definition of done")

**Interfaces:** none (docs only).

- [ ] **Step 1: Split the M1.5 entry**

Replace the current `- [ ] **M1.5 — Polish + publish.** …` entry with two entries (keep the carried-over npm-license note inside M1.5a's text as resolved):

```markdown
- [ ] **M1.5a — Publish-ready + rc.** Consumer docs finalized
  (entitlement/signing guide, API reference, quota + gotchas),
  crates.io/docs.rs metadata, changelog, two-stage publish runbook in
  `DEVELOPERS.md`, announce drafts in `docs/announcements/`, and
  `0.1.0-rc.1` published to crates.io and npm (under the `rc`
  dist-tag; no `latest` tag exists). The M1.1-review carryover landed:
  `prepack` now ships license texts + README in the npm tarball.
- [ ] **M1.5b — Stable publish + announce (gated).** After Team Times
  observes cross-device sync and change events on real hardware
  against the published rc: publish `0.1.0` (npm untagged → sets
  `latest`), tag, announce (Tauri Discord, awesome-tauri PR), and bump
  Team Times to stable. Runbook: `DEVELOPERS.md` "Publishing a
  release", Stage 2.
```

(M1.5a's checkbox gets ticked in Task 8 after the rc publish succeeds.)

- [ ] **Step 2: Amend the definition of done**

Replace the "Definition of done" paragraph's last sentence
(`Team Times project 2 consumes the published plugin (not a path
dependency).`) with:

```markdown
Team Times project 2 verifies real-device sync against the published
`0.1.0-rc.1`, then consumes the published stable `0.1.0` (never a
path dependency).
```

- [ ] **Step 3: Commit**

```bash
git add docs/milestones.md
git commit -m "docs: Split M1.5 into rc and gated stable milestones"
```

---

### Task 8: Guided rc publish (interactive checkpoint with Kevin)

This task is executed together with Kevin — the publishes are his to
run (accounts, tokens, 2FA). Do not run `cargo publish` or
`npm publish` autonomously.

**Files:** none (registry state + git tag; one checkbox edit in `docs/milestones.md`)

**Interfaces:**
- Consumes: everything above; runbook Stage 1 (Task 5)
- Produces: `tauri-plugin-icloud-kvs 0.1.0-rc.1` on crates.io; `tauri-plugin-icloud-kvs-api@0.1.0-rc.1` on npm under `rc`; git tag `v0.1.0-rc.1`

- [ ] **Step 1: Push and confirm CI**

```bash
git push
gh run watch  # or: gh run list --limit 1
```

Expected: CI green on `main` with all tasks' commits included.

- [ ] **Step 2: Final rehearsals**

```bash
cargo publish --dry-run
cd guest-js && npm pack --dry-run && cd ..
```

Expected: dry run aborts with the standard "aborting upload due to dry
run" warning; tarball lists `dist/`, `README.md`, `LICENSE-MIT`,
`LICENSE-APACHE`, version `0.1.0-rc.1`.

- [ ] **Step 3: Kevin publishes the crate**

Kevin runs (needs a crates.io API token with publish scope):

```bash
cargo login   # once per machine
cargo publish
```

Verify: <https://crates.io/crates/tauri-plugin-icloud-kvs> shows
`0.1.0-rc.1`.

- [ ] **Step 4: Kevin publishes the npm package**

Kevin runs (needs npm account; 2FA prompt likely):

```bash
cd guest-js
npm login     # once per machine
npm publish --tag rc
```

⚠️ `--tag rc` is mandatory (Global Constraints).

Verify: `npm dist-tag ls tauri-plugin-icloud-kvs-api` prints only
`rc: 0.1.0-rc.1` (no `latest`).

- [ ] **Step 5: Tag and verify installability**

```bash
git tag v0.1.0-rc.1
git push --tags
```

Smoke-test resolution from a temp dir (scratchpad, not the repo):

```bash
npm view tauri-plugin-icloud-kvs-api@rc version   # → 0.1.0-rc.1
cargo search tauri-plugin-icloud-kvs --limit 1    # → 0.1.0-rc.1 listed
```

- [ ] **Step 6: Check off M1.5a and commit**

In `docs/milestones.md`, change `- [ ] **M1.5a` to `- [x] **M1.5a`.

```bash
git add docs/milestones.md
git commit -m "docs: Check off M1.5a after 0.1.0-rc.1 publish"
git push
```

Then hand the rc coordinates to Team Times (runbook Stage 1, step 6).
