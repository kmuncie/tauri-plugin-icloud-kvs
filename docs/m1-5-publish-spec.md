# M1.5 Spec — Polish + publish-ready

Design agreed 2026-07-14 (Kevin + Claude). This spec covers making
tauri-plugin-icloud-kvs 0.1.0 fully publish-ready. **The actual
crates.io/npm publish and announcements are gated** on Team Times
observing real-device sync (via a path or git dependency on this repo)
and are executed later via the runbook this effort produces — they are
explicitly out of scope for the implementation plan.

## Scope decisions

- **Publish gate:** The milestones' "sync observed on real devices
  before 0.1.0 publish" holds. Team Times integration is the vehicle:
  it consumes the plugin as a path/git dependency first, verifies
  cross-device sync and change events on real hardware, and only then
  does 0.1.0 ship — after which Team Times switches to the published
  version. The milestones' definition of done is amended to match
  (previously it required Team Times to consume the *published*
  plugin, which was circular with the verification gate).
- Everything else lands now so the publish itself is a runbook
  execution, not a work session.

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

### 4. CHANGELOG.md + API-surface audit

- `CHANGELOG.md` at the repo root with a 0.1.0 entry (Keep a Changelog
  format; initial release notes summarizing the full feature set).
- A final read-through audit of `guest-js/src/index.ts` and the public
  Rust API for naming/doc-comment consistency. Expected to produce at
  most trivial doc fixes; any behavioral change it surfaces is out of
  scope and becomes an issue instead.

### 5. Publish runbook (DEVELOPERS.md)

New "Publishing a release" section:

1. Gate checklist: Team Times real-device sync + change events
   observed (per the cross-device protocol); all CI checks green;
   `CHANGELOG.md` updated.
2. `cargo publish` (with `--dry-run` rehearsal first).
3. `cd guest-js && npm publish` (prepack copies docs/licenses
   automatically; `npm pack --dry-run` rehearsal first).
4. `git tag v0.1.0 && git push --tags`.
5. Announce: paste the prepared drafts (Discord, awesome-tauri).
6. Switch Team Times to the published versions.

### 6. Announce drafts (docs/announcements/)

- `discord-0.1.0.md`: short Tauri Discord #plugins post (what it does,
  platforms, links, quota caveat).
- `awesome-tauri-pr.md`: the one-line entry + PR description for the
  awesome-tauri list.

### 7. Milestones amendment (docs/milestones.md)

- Definition of done reworded per the scope decision above.
- M1.5 entry split into "publish-ready" (this effort's deliverables,
  checked off when they land) and the gated publish/announce itself
  (checked off after the runbook is executed post-verification).

## Out of scope

- Running `cargo publish` / `npm publish` for real.
- Posting announcements.
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
- Nothing has been published.
