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
