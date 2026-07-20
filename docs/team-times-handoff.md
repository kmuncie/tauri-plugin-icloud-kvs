# Handoff: tauri-plugin-icloud-kvs 0.1.0-rc.1 → Team Times

Date: 2026-07-14. Audience: whoever (human or agent) integrates
iCloud KVS sync into Team Times project 2.

> **Update (2026-07-20):** All four checks below passed on real
> hardware (iPhone + MacBook) — see `docs/milestones.md` M1.5b.
> Stable `0.1.0` is publishing now; once it's live, bump Team Times
> to `tauri-plugin-icloud-kvs = "0.1.0"` /
> `tauri-plugin-icloud-kvs-api@0.1.0` and drop the `@rc` pin, per
> "Reporting back" below.

## TL;DR

The plugin is published as a release candidate and ready to consume
from the real registries. Team Times is the verification vehicle: it
must observe cross-device sync and change events on real hardware
before the plugin ships as stable 0.1.0.

## Coordinates

- crates.io: `tauri-plugin-icloud-kvs = "0.1.0-rc.1"`
  (exact pin required — cargo never resolves pre-releases implicitly)
- npm: `npm install tauri-plugin-icloud-kvs-api@0.1.0-rc.1`
  (or `@rc`; note `latest` currently also points at the rc — a known,
  accepted deviation until stable ships)
- Repo/docs: <https://github.com/kmuncie/tauri-plugin-icloud-kvs>
  (tag `v0.1.0-rc.1`)

Do **not** use a path or git dependency — consuming the published rc
is part of what's being verified.

## Integration steps

1. Add both dependencies (exact pins above).
2. Register the plugin and permission:
   - `tauri::Builder::default().plugin(tauri_plugin_icloud_kvs::init())`
   - capability file: `{ "permissions": ["icloud-kvs:default"] }`
3. Entitlement + signing: follow the README "Setup" section step 3
   (literal `TEAMID.bundleid` in `entitlements.plist`; codesign does
   not expand Xcode variables). Signed builds only — unsigned dev
   builds have an inert store that silently no-ops writes.
4. Gate sync UX on `accountStatus()` (`noAccount` = local-only mode).
5. Subscribe with `onExternalChange()` for live updates. Payload:
   `{ reason: 'serverChange' | 'initialSync' | 'quotaViolation' |
   'accountChange', changedKeys: string[] }`.

API surface and gotchas (quota is async-only, `synchronize()` is
flush-only, OS write throttling): README "API" and "What iCloud KVS
gives you" sections.

## What Team Times must verify (the stable-release gate)

On two real devices signed into the same Apple ID (e.g. Mac +
iPhone), with signed, entitled Team Times builds:

1. `accountStatus()` reports `available` on both.
2. A value `set` on device A arrives on device B (poll `get` or watch
   the UI; latency is seconds to minutes, no guarantees).
3. Device B receives a `serverChange` event via `onExternalChange`
   listing the changed key — without user action.
4. Repeat in the reverse direction.

This mirrors `DEVELOPERS.md` → "Cross-device sync verification
(manual)" in the plugin repo (steps 2–5 there use the demo app; Team
Times itself is the better rig).

## Reporting back

- ✅ All four checks pass → tell the plugin repo; stable `0.1.0`
  ships via `DEVELOPERS.md` → "Publishing a release" Stage 2, then
  bump Team Times to `0.1.0` / drop the `@rc` pin.
- ❌ Anything fails or surprises (sync never arrives, events missing,
  serialization issues, crashes) → file an issue at
  <https://github.com/kmuncie/tauri-plugin-icloud-kvs/issues> with
  device/OS versions and reproduction notes. Fixes go out as
  `0.1.0-rc.2` (npm publishes default to the `rc` tag via
  `publishConfig` — leave that mechanism alone).

## Known state / caveats for the integrator

- Real-device cross-device sync has **never** been observed yet —
  that is the point of this exercise. Simulator round-trips, macOS
  signed-build round-trips, and `noAccount` local-only mode were all
  verified during development (see `docs/milestones.md`).
- npm `latest` currently equals the rc (untagged first publish);
  harmless, self-corrects at stable.
- Store limits: 1 MB total / 1024 keys / 64-byte key names. Team
  Times config sync should stay far below all three.
- The iCloud KVS entitlement requires a paid Apple Developer team on
  every build that should sync.
