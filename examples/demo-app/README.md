# icloud-kvs-demo

Demo app and manual test rig for
[`tauri-plugin-icloud-kvs`](../../README.md): a key-value editor plus an
external-change event log (event log lands with plugin milestone M1.4).

## Run (macOS)

```sh
cd guest-js && npm install && npm run build && cd ..   # once, from repo root
cd examples/demo-app
npm install
npm run tauri dev
```

> ⚠️ **Unsigned dev builds have an inert store.** Without the iCloud KVS
> entitlement and code signature, every write silently no-ops (this is OS
> behavior — see the repo `DEVELOPERS.md`). The UI still runs; use it to
> check wiring and error surfaces. For a functional store you need a
> signed, entitled build (below).

## Signed macOS build (functional store)

1. `src-tauri/entitlements.plist` uses Xcode-style variables
   (`$(TeamIdentifierPrefix)$(CFBundleIdentifier)`). Tauri signs with
   `codesign`, which does **not** substitute them — replace the value
   with your literal prefix + bundle id, e.g.
   `AB12CD34EF.com.kmuncie.icloud-kvs-demo`. Do not commit that change.
2. Build and sign with your Developer ID / development certificate:
   `APPLE_SIGNING_IDENTITY="Apple Development: …" npm run tauri build`
3. Launch the app from `src-tauri/target/release/bundle/macos/` and
   verify the account badge shows `available` (requires being signed in
   to iCloud).

## iOS (simulator)

```sh
npm run tauri ios init   # once; generates src-tauri/gen/apple (gitignored)
```

Open `src-tauri/gen/apple/demo-app.xcodeproj` in Xcode, select your team
under Signing & Capabilities (target **demo-app_iOS**), add the
**iCloud** capability with **Key-value storage** checked, then:

> Prerequisites: full Xcode and CocoaPods (`brew install cocoapods`) —
> Tauri's iOS project generation requires the `pod` binary even though
> this app declares no pods. The iCloud capability requires a paid Apple
> Developer Program team.

```sh
npm run tauri ios dev
```

Sign in to an Apple ID in the simulator (Settings) for
`accountStatus()` to report `available`.
