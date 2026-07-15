# M1.5 Spec — Polish + rc publish

Design agreed 2026-07-14 (Kevin + Claude; revised same day from
"git-dep gating" to an rc pre-release flow). This spec covers making
tauri-plugin-icloud-kvs fully publish-ready and ends with a **guided
publish of `0.1.0-rc.1`** to crates.io and npm (Kevin executes the
publish commands). The **stable `0.1.0` publish and all announcements
remain gated** on Team Times observing real-device sync, and are
executed later via the runbook this effort produces.

## Scope decisions

- **Pre-release flow:** `0.1.0-rc.1` goes to both registries now.
  Cargo never resolves a pre-release unless a consumer pins it
  explicitly, and npm publishes it under the `rc` dist-tag (no
  `latest` tag exists yet), so nothing can pick it up by accident.
  Team Times consumes the published rc from the real registries —
  this dissolves the old circularity (the milestones' definition of
  done required Team Times to consume the *published* plugin, while
  the publish gate required Team Times verification first) and
  end-to-end tests the publish pipeline itself.
- **Publish gate (unchanged for stable):** The milestones' "sync
  observed on real devices before 0.1.0 publish" holds for the stable
  release. After Team Times verifies sync + change events on real
  hardware against the rc, the runbook publishes `0.1.0`, tags it
  `latest`, announces, and Team Times bumps to the stable version.
- crates.io versions are permanent (yank-only); the rc staying in the
  index forever is normal and accepted.
- rc naming (not beta): the API is feature-complete; this exact code
  is expected to become 0.1.0 barring a verification blocker.

## Deliverables

### 1. Consumer docs (README.md)

Replace the "guide coming with the first release" placeholder with an
end-to-end entitlement guide, and add the sections the design spec
promised:

- **Entitlement setup:** `com.apple.developer.ubiquity-kvstore-identifier`
  = `$(TeamIdentifierPrefix)$(CFBundleIdentifier)`; enabling iCloud
  Key-value storage in Xcode / the developer portal; macOS dev-signed
  builds; the sandboxed Mac App Store case; the iOS
  provisioning-profile wrinkle. Content aligns with (and links to) the
  demo app README's signing walkthrough rather than duplicating its
  step-by-step detail.
- **Quota section:** the 1 MB / 1024-key / 64-byte-key limits;
  quota exhaustion is notification-only — with example
  `onExternalChange` code detecting `reason === 'quotaViolation'`.
- **API reference:** short table/section covering all eight guest
  functions (`get`, `set`, `remove`, `keys`, `getAll`, `synchronize`,
  `accountStatus`, `onExternalChange`) and the `ChangeEvent` payload.
- **Behavioral gotchas:** `synchronize()` is flush-only; OS write
  throttling (debounce guidance); silent local-only mode when signed
  out (detect via `accountStatus()`); account-switch store
  replacement; foreign `NSData`/`NSDate` read-back mappings.

### 2. npm packaging (M1.1-review carryover)

In `guest-js/package.json`:

- `prepack` script copying `README.md`, `LICENSE-MIT`, and
  `LICENSE-APACHE` from the repo root into `guest-js/` so the tarball
  ships license texts and the npm package page renders the README.
- The copies added to `"files"` (npm auto-includes them, but listing
  them keeps intent explicit) and to `.gitignore` (they are build
  artifacts in `guest-js/`, not committed).
- `keywords`: `tauri`, `tauri-plugin`, `icloud`, `key-value`, `sync`,
  `apple`.

Verified by `npm pack --dry-run` listing `dist/`, `README.md`, and
both license files.

### 3. crates.io packaging

In `Cargo.toml`:

- `keywords` (crates.io max 5): `tauri-plugin`, `icloud`, `key-value`,
  `sync`, `apple`.
- `categories` (official slugs): `os::macos-apis`, `api-bindings`.
- `[package.metadata.docs.rs]` with `default-target =
  "aarch64-apple-darwin"` and `targets` covering
  `aarch64-apple-darwin`, `aarch64-apple-ios`, and
  `x86_64-unknown-linux-gnu`, so docs.rs renders the real Apple API
  surface (not just the stub) while still proving the stub documents
  cleanly.

Verified by `cargo publish --dry-run` and a clean `cargo doc
--no-deps`.

### 4. Version bump, CHANGELOG.md + API-surface audit

- Version set to `0.1.0-rc.1` in both `Cargo.toml` and
  `guest-js/package.json` (the demo app's path dependency is
  unaffected).
- `CHANGELOG.md` at the repo root with a 0.1.0-rc.1 entry (Keep a
  Changelog format; initial release notes summarizing the full
  feature set; the 0.1.0 stable entry is added at stable-publish
  time per the runbook).
- A final read-through audit of `guest-js/src/index.ts` and the public
  Rust API for naming/doc-comment consistency. Expected to produce at
  most trivial doc fixes; any behavioral change it surfaces is out of
  scope and becomes an issue instead.

### 5. Publish runbook (DEVELOPERS.md)

New "Publishing a release" section covering both stages:

**Stage 1 — rc (executed at the end of this effort, guided):**

1. Checklist: all CI checks green; `cargo publish --dry-run` and
   `npm pack --dry-run` rehearsals pass; version is `0.1.0-rc.1`.
2. `cargo publish`.
3. `cd guest-js && npm publish --tag rc` (prepack copies
   docs/licenses automatically). **Never publish the npm package
   without a `--tag` until 0.1.0 — the first untagged publish
   becomes `latest`.**
4. `git tag v0.1.0-rc.1 && git push --tags`.
5. Hand the rc coordinates to Team Times for real-device
   verification. No announcements.

**Stage 2 — stable (gated on Team Times verification):**

1. Gate checklist: Team Times real-device sync + change events
   observed (per the cross-device protocol); no open rc-found issues;
   `CHANGELOG.md` gains the 0.1.0 entry.
2. Bump both versions to `0.1.0`, commit, re-run rehearsals.
3. `cargo publish`, then `cd guest-js && npm publish` (untagged —
   this sets `latest`).
4. `git tag v0.1.0 && git push --tags`.
5. Announce: paste the prepared drafts (Discord, awesome-tauri).
6. Bump Team Times to the stable versions.

### 6. Announce drafts (docs/announcements/)

- `discord-0.1.0.md`: short Tauri Discord #plugins post (what it does,
  platforms, links, quota caveat).
- `awesome-tauri-pr.md`: the one-line entry + PR description for the
  awesome-tauri list.

### 7. Milestones amendment (docs/milestones.md)

- Definition of done reworded per the scope decisions above (Team
  Times verifies against the published rc, then consumes the
  published stable).
- M1.5 entry split into "publish-ready + rc published" (this
  effort's deliverables) and the gated stable publish/announce
  (checked off after runbook Stage 2 runs post-verification).

## Out of scope

- The stable `0.1.0` publish (runbook Stage 2).
- Posting announcements (stable-release time only).
- Team Times integration work itself.
- Any new plugin features or API changes.

## Acceptance criteria

- `cargo publish --dry-run` succeeds.
- `npm pack --dry-run` (after a build) lists `dist/`, `README.md`,
  `LICENSE-MIT`, `LICENSE-APACHE`.
- `cargo doc --no-deps` builds without warnings.
- Existing checks stay green: `cargo test`, `cargo lint-clippy`,
  `cargo lint-fmt`, guest-js and demo-app builds.
- README contains the entitlement guide, quota section, API reference,
  and gotchas — no "coming with the first release" placeholders left.
- Runbook, changelog, and both announce drafts exist.
- `0.1.0-rc.1` is live on crates.io and on npm under the `rc`
  dist-tag with no `latest` tag set (guided publish, executed by
  Kevin at the end of the effort).
- `0.1.0` stable has NOT been published; nothing announced.
