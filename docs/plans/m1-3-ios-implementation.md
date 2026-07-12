# tauri-plugin-icloud-kvs M1.3 (iOS Implementation) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The full command set working on iOS by sharing the existing macOS `objc2` implementation, with an iOS cross-compile check in CI and the design spec corrected.

**Architecture:** The design spec assumed Tauri 2 mobile plugins require a Swift half; they don't — Swift is only needed for lifecycle hooks or APIs Rust can't reach. `objc2-foundation` compiles for iOS and exposes the identical `NSUbiquitousKeyValueStore` API, so M1.3 is: rename `src/desktop.rs` → `src/store.rs`, widen every `#[cfg(target_os = "macos")]` gate to `#[cfg(any(target_os = "macos", target_os = "ios"))]`, add the iOS dependency target + toolchain targets, and prove it with `cargo clippy --target aarch64-apple-ios` locally and in CI. No `ios/` Swift package, no `src/mobile.rs`, no duplicated logic; M1.4's notification observer will also be written once. (Decision made 2026-07-12 with Kevin; the spec's "~100-line Swift duplication" trade-off is amended away in Task 4.)

**Tech Stack:** Rust (objc2 0.6.4, objc2-foundation 0.3.2, Tauri 2 plugin API), GitHub Actions (macOS runner).

**Spec:** `docs/design-spec.md` (amended by Task 4). Milestone definition: `docs/milestones.md` (M1.3 entry).

## Status (paused 2026-07-12)

Tasks 1–4 and Task 5 Step 1 are **done and pushed**; CI run 29213159307 is
green including the new "iOS cross-compile check" step. Remaining: Task 5
Steps 2–4 (simulator checkpoint + milestone close-out), **blocked on Kevin
installing full Xcode** (his machine has only the Command Line Tools, which
lack the iOS SDK — see `DEVELOPERS.md` "iOS verification").

Deviations from the plan as written:

- Task 2 Step 1/5: the `cargo tree … | grep objc2` check needs `--depth 1`
  — tauri itself pulls objc2 crates transitively on iOS, so the plan's
  "no output before the change" expectation only holds for direct deps.
- Task 2 Step 5 / Task 3 Step 2: `cargo clippy --target aarch64-apple-ios`
  cannot run locally without full Xcode (dependency build scripts compile C
  against the iOS SDK). Verification was done by CI instead (the Step 5
  contingency), and the Xcode requirement is documented in `DEVELOPERS.md`.
- `Cargo.lock` is gitignored in this repo, so it was not committed.

Pickup: install Xcode → run Task 5 Step 2's protocol (in `DEVELOPERS.md`)
→ Steps 3–4. Optional idea discussed with Kevin: pull the M1.5 example app
(`examples/demo-app`) forward and use it as the checkpoint host app instead
of a throwaway scratch app.

## Global Constraints

- Rust 1.89.0, edition 2024; 3-space indent; `rustfmt.toml` / `.cargo/config.toml` already in repo — do not modify
- All dependencies stay pinned to exact semver versions; this milestone adds **no new dependencies** (it only widens the existing `objc2`/`objc2-foundation` target predicate)
- Renames/moves in separate commits from content changes (import/`mod` updates may ride along with the move)
- Every commit passes `cargo test && cargo lint-clippy && cargo lint-fmt` on the host (macOS)
- Commit messages: Conventional Commits, imperative, ≤72-char subject
- Testing policy (`docs/milestones.md`, memory): contributors never need entitled hardware; required checks must run on a plain macOS machine. The iOS "test" for this milestone is the cross-compile clippy check; simulator verification is a manual human checkpoint
- Cross-device sync verification (Mac ↔ iOS, same container) stays deferred to Team Times integration (~M1.5) — do not add it as a required step here
- M1.2-review carryover about foreign `NSDate` in `plist_to_json` belongs to **M1.4**, not this plan — leave it alone

---

### Task 1: Rename `src/desktop.rs` → `src/store.rs`

The module is about to serve iOS too, so "desktop" becomes a misnomer. Pure rename commit (content changes come in Task 2), per the standards rule that moves are separate from edits; the `mod`/`use` updates in `lib.rs` ride along.

**Files:**
- Move: `src/desktop.rs` → `src/store.rs`
- Modify: `src/lib.rs` (lines 17 and 28)

**Interfaces:**
- Consumes: existing `desktop::{account_status, get, get_all, keys, remove, set, synchronize}`.
- Produces: the same seven functions re-exported from `store::…`; the crate-root `pub use` (what `commands.rs` and `tests/kvs_roundtrip.rs` call) is unchanged, so nothing else moves.

- [x] **Step 1: Verify green baseline**

Run: `cargo test && cargo lint-clippy && cargo lint-fmt`
Expected: PASS (this is the pre-change baseline; if it fails, stop and report).

- [x] **Step 2: Move the file**

```bash
git mv src/desktop.rs src/store.rs
```

- [x] **Step 3: Update `src/lib.rs`**

Change line 17:

```rust
mod store;
```

(was `mod desktop;`)

Change line 28:

```rust
pub use store::{account_status, get, get_all, keys, remove, set, synchronize};
```

(was `pub use desktop::{…};` — same list.)

- [x] **Step 4: Run everything to verify the rename is complete**

Run: `cargo test && cargo lint-clippy && cargo lint-fmt`
Expected: PASS, identical test counts to Step 1.

- [x] **Step 5: Commit**

```bash
git add src/store.rs src/lib.rs
git commit -m "refactor: Rename desktop module to store"
```

(`git add src/store.rs` picks up the staged rename; `git status` should show `renamed: src/desktop.rs -> src/store.rs`.)

---

### Task 2: Enable the shared objc2 implementation on iOS

Widen the dependency target and every cfg gate from macOS-only to macOS+iOS, and add the iOS rustup targets so contributors and CI can cross-compile.

**Files:**
- Modify: `Cargo.toml` (line 21), `rust-toolchain.toml`, `src/lib.rs` (line 15), `src/store.rs` (module doc + both cfg gates), `src/conversion.rs` (module doc, line 1)

**Interfaces:**
- Consumes: `store.rs` / `conversion.rs` from Task 1 (unchanged logic).
- Produces: on `target_os = "ios"` the real `NSUbiquitousKeyValueStore` implementation compiles instead of the `UnsupportedPlatform` fallback. Task 3's CI step and Task 5's simulator checkpoint rely on this. Public API signatures unchanged.

- [x] **Step 1: Capture the failing "test" — iOS currently gets the stub**

Run: `cargo tree --target aarch64-apple-ios -p tauri-plugin-icloud-kvs | grep objc2`
Expected: **no output** (exit code 1) — proving the objc2 implementation is not part of the iOS build today. (`cargo tree` works without the target toolchain installed.)

- [x] **Step 2: Install the iOS targets and declare them in `rust-toolchain.toml`**

```bash
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
```

Append to `rust-toolchain.toml` (below `components`):

```toml
targets = [
   "aarch64-apple-ios",
   "aarch64-apple-ios-sim"
]
```

(Declaring them means fresh `rustup toolchain install` runs — including CI's — pull them automatically.)

- [x] **Step 3: Widen the dependency target in `Cargo.toml`**

Change line 21 from:

```toml
[target.'cfg(target_os = "macos")'.dependencies]
```

to:

```toml
[target.'cfg(any(target_os = "macos", target_os = "ios"))'.dependencies]
```

(The `objc2` / `objc2-foundation` entries below it are unchanged — same exact pins, same feature list; every enabled Foundation API is available on iOS: `NSUbiquitousKeyValueStore` since iOS 5, `ubiquityIdentityToken` since iOS 6.)

- [x] **Step 4: Widen the cfg gates**

`src/lib.rs` line 15, change:

```rust
#[cfg(any(target_os = "macos", target_os = "ios"))]
mod conversion;
```

`src/store.rs` — replace the module doc (lines 1–3) with:

```rust
//! Store operations. Apple platforms (macOS and iOS) talk to
//! `NSUbiquitousKeyValueStore` via objc2-foundation; all other
//! platforms return `Error::UnsupportedPlatform` from every operation.
```

and change the two gates:

```rust
#[cfg(any(target_os = "macos", target_os = "ios"))]
mod imp {
```

(was `#[cfg(target_os = "macos")]`), and:

```rust
#[cfg(not(any(target_os = "macos", target_os = "ios")))]
mod imp {
```

(was `#[cfg(not(target_os = "macos"))]`).

`src/conversion.rs` line 1, change:

```rust
//! JSON ↔ property-list conversion for the Apple-platform implementation.
```

(was `…for the macOS implementation.`)

- [x] **Step 5: Verify the iOS build now contains the real implementation**

Run: `cargo tree --target aarch64-apple-ios -p tauri-plugin-icloud-kvs | grep objc2`
Expected: `objc2` and `objc2-foundation` lines appear.

Run: `cargo clippy --target aarch64-apple-ios -- -D warnings`
Expected: compiles clean (lib only — deliberately not `--all-targets`; tests never run on the iOS target). First run is slow (tauri cross-compiles).

Contingency: if a transitive dependency's build script demands the iOS SDK, confirm Xcode provides it (`xcodebuild -showsdks | grep iphoneos`) — the plan assumes a full Xcode install, which CI's macOS runners have.

- [x] **Step 6: Verify the host build is untouched**

Run: `cargo test && cargo lint-clippy && cargo lint-fmt`
Expected: PASS, same test counts as Task 1.

- [x] **Step 7: Commit**

```bash
git add Cargo.toml rust-toolchain.toml src/lib.rs src/store.rs src/conversion.rs
git commit -m "feat: Enable the shared objc2 KVS implementation on iOS"
```

---

### Task 3: CI iOS cross-compile check

**Files:**
- Modify: `.github/workflows/ci.yml` (rust job)

**Interfaces:**
- Consumes: rustup targets from Task 2 (`rust-toolchain.toml` makes CI's `rustup toolchain install` fetch them).
- Produces: a required CI step that fails the build if the plugin stops compiling for iOS. (Push + watch happens in Task 5 so CI is verified once, after the docs land.)

- [x] **Step 1: Add the step to the rust job**

In `.github/workflows/ci.yml`, after the `Lint` step and before `Test`, insert:

```yaml
         - name: iOS cross-compile check
           run: cargo clippy --target aarch64-apple-ios -- -D warnings
```

(3-space YAML indent matching the file; runner already has Xcode + the toolchain step installs the targets declared in `rust-toolchain.toml`.)

- [x] **Step 2: Verify the exact command locally**

Run: `cargo clippy --target aarch64-apple-ios -- -D warnings`
Expected: clean (cached from Task 2).

- [x] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: Add iOS cross-compile clippy check"
```

---

### Task 4: Documentation — README, DEVELOPERS.md, design-spec amendment

**Files:**
- Modify: `README.md` (platform table + usage heading), `DEVELOPERS.md` (new iOS section), `docs/design-spec.md` (decision bullet, repo layout, architecture bullets)

**Interfaces:**
- Consumes: everything above.
- Produces: consumer- and contributor-facing docs matching reality; the spec no longer claims a Swift iOS half.

- [x] **Step 1: Update `README.md`**

Platform table (line 21), change:

```markdown
| iOS      | ✅ Supported (same pure-Rust implementation as macOS) |
```

(was `| iOS      | Planned (Swift) |`.)

Usage heading (line 24), change:

```markdown
## Usage (pre-release)
```

(was `## Usage (macOS, pre-release)`; the body already applies to both platforms.)

- [x] **Step 2: Add an iOS section to `DEVELOPERS.md`**

Insert after the "Integration test: `tests/kvs_roundtrip.rs`" section (before "## Planning"):

```markdown
## iOS verification (manual, simulator)

The iOS build shares the macOS objc2 implementation; CI proves it
compiles (`cargo clippy --target aarch64-apple-ios`). Exercising the
commands needs a host app in the simulator:

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
```

- [x] **Step 3: Amend `docs/design-spec.md`**

**(a)** In "Decisions (settled during brainstorming)", replace the bullet:

```markdown
- **macOS implementation: pure Rust via `objc2-foundation`** (which ships
  `NSUbiquitousKeyValueStore` bindings). No Xcode/Swift toolchain required
  for desktop consumers. The iOS half is Swift, as Tauri 2 mobile plugins
  require. The shallow ~100-line duplication is accepted.
```

with:

```markdown
- **Implementation: pure Rust via `objc2-foundation`** (which ships
  `NSUbiquitousKeyValueStore` bindings) **on both macOS and iOS**. No
  Xcode/Swift toolchain required. *Correction (found during M1.3):*
  the brainstorm assumed Tauri 2 mobile plugins require a Swift half;
  they don't — Swift is only needed for lifecycle hooks or APIs Rust
  cannot reach. The single objc2 implementation compiles for iOS, so
  the accepted ~100-line Swift duplication never materialized.
```

**(b)** In "Repo layout", replace the two lines:

```text
│   ├── desktop.rs        # macOS via objc2-foundation; non-Apple → error
│   ├── mobile.rs         # bridges to Swift via tauri::plugin::PluginHandle
```

with:

```text
│   ├── store.rs          # macOS+iOS via objc2-foundation; non-Apple → error
```

and delete the line:

```text
├── ios/                  # Swift package: NSUbiquitousKeyValueStore impl
```

**(c)** In "Architecture & data flow", change the first bullet's lead-in from `**macOS (`desktop.rs`):**` to `**macOS & iOS (`store.rs`):**`, and delete the entire iOS bullet:

```markdown
- **iOS (`ios/`, Swift):** identical logic in Swift via Tauri 2's mobile
  plugin system; events surface through the plugin `trigger` mechanism
  under the same event name and payload shape, so frontend code is
  platform-agnostic.
```

(Leave "Alternatives considered" untouched — it is a historical record.)

- [x] **Step 4: Full local verification**

Run: `cargo test && cargo lint-clippy && cargo lint-fmt`
Expected: green (docs-only change; guards against stray edits).

- [x] **Step 5: Commit**

```bash
git add README.md DEVELOPERS.md docs/design-spec.md
git commit -m "docs: Document shared pure-Rust iOS support"
```

---

### Task 5: CI verification, simulator checkpoint, milestone close-out

**Files:**
- Modify: `docs/milestones.md` (M1.3 entry), `docs/design-spec.md` (Milestones checklist)

**Interfaces:**
- Consumes: all prior tasks; CI workflow from Task 3.

- [x] **Step 1: Push and watch CI**

```bash
git push
gh run watch --repo kmuncie/tauri-plugin-icloud-kvs --exit-status
```

Expected: latest run `success` on both jobs, including the new "iOS cross-compile check" step. If only that step fails, fix the compile error it reports (the host build being green means it's an iOS-only API difference — check the objc2-foundation docs for the symbol's availability) rather than deleting the step.

- [ ] **Step 2: HUMAN CHECKPOINT — simulator verification**

Follow the `DEVELOPERS.md` "iOS verification (manual, simulator)" protocol from Task 4. This needs Kevin's Xcode, simulator, and Apple ID and cannot be done by an agent. **Stop here and report status to Kevin if executing autonomously.** The bar is: commands round-trip in an entitled simulator app; cross-device sync stays deferred.

- [ ] **Step 3: Check off the milestone**

After simulator verification passes, in `docs/milestones.md` replace the M1.3 entry with:

```markdown
- [x] **M1.3 — iOS implementation.** Same API on iOS by sharing the
  macOS objc2 implementation — the spec's premise that Tauri 2 requires
  a Swift half was wrong and is corrected in the design spec. CI
  cross-compiles with clippy for `aarch64-apple-ios`. Command
  round-trips verified manually in an entitled simulator app;
  cross-device sync against the Mac build stays deferred to Team Times
  integration (~M1.5).
```

and in `docs/design-spec.md`'s Milestones list, change the M1.3 line to `- [x]`.

- [ ] **Step 4: Commit and push**

```bash
git add docs/milestones.md docs/design-spec.md
git commit -m "docs: Check off M1.3 in plugin milestones"
git push
```
