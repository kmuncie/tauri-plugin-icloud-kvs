# examples/demo-app Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A vanilla-TS Tauri 2 demo app (`examples/demo-app`) — KV editor + placeholder event-log pane — that serves as the M1.3 simulator checkpoint host, the permanent manual test rig, and the M1.5 example deliverable.

**Architecture:** Standard Tauri 2 app layout, decoupled from the plugin crate (own Cargo/npm trees, path/`file:` dependencies back to the repo root). One HTML page, one `main.ts`, one CSS file — no framework. Rust side is the stock builder registering `tauri_plugin_icloud_kvs::init()`. CI gets a light `demo-app` job (frontend build + `cargo check`), no `tauri build`.

**Tech Stack:** TypeScript + Vite (frontend), Tauri 2.11.5 / tauri-build 2.6.3 (Rust), `tauri-plugin-icloud-kvs-api` via `file:../../guest-js`.

**Spec:** `docs/demo-app-spec.md`.

## Global Constraints

- Rust 1.89.0, edition 2024, 3-space indent everywhere (Rust, TS, JSON, YAML, HTML, CSS)
- All new dependencies pinned to exact semver versions (resolve with `npm view <pkg> version` before writing them; placeholders below are marked `<PIN>`)
- `@tauri-apps/api` must match guest-js's pin: **2.11.1**; Rust `tauri = "2.11.5"`, `tauri-build = "2.6.3"`, `serde_json = "1.0.150"` (same pins as the plugin)
- Plugin consumed by path only: Rust `{ path = "../.." }`, npm `"file:../../guest-js"`
- Signing identity/team is never committed; entitlements file is committed
- Demo app is NOT a cargo workspace member of the plugin — its `src-tauri` is a standalone crate (the plugin repo has no workspace, so no `exclude` needed; just never add one)
- Every commit: plugin-root `cargo test && cargo lint-clippy && cargo lint-fmt` stays green (the demo app doesn't affect it, but guards stray edits); demo-app checks per task
- Commit messages: Conventional Commits, imperative, ≤72-char subject
- Node 24 (root `.nvmrc` applies; run `nvm use` if needed)

---

### Task 1: App scaffold — Rust side + Tauri config

**Files:**
- Create: `examples/demo-app/src-tauri/Cargo.toml`, `examples/demo-app/src-tauri/build.rs`, `examples/demo-app/src-tauri/tauri.conf.json`, `examples/demo-app/src-tauri/entitlements.plist`, `examples/demo-app/src-tauri/capabilities/default.json`, `examples/demo-app/src-tauri/src/main.rs`, `examples/demo-app/src-tauri/src/lib.rs`, `examples/demo-app/.gitignore`

**Interfaces:**
- Consumes: the plugin crate at repo root (`tauri_plugin_icloud_kvs::init()`).
- Produces: a compiling Tauri app crate named `demo-app` (lib `demo_app_lib`) whose window is labeled `main` and whose webview has the `icloud-kvs:default` + `core:default` permissions. Task 2's frontend is served from `../dist` / dev port 1420.

- [ ] **Step 1: Create `examples/demo-app/.gitignore`**

```gitignore
node_modules
dist
src-tauri/target
src-tauri/gen
```

- [ ] **Step 2: Create `examples/demo-app/src-tauri/Cargo.toml`**

```toml
[package]
name = "demo-app"
version = "0.1.0"
description = "Demo app and manual test rig for tauri-plugin-icloud-kvs"
edition = "2024"
rust-version = "1.89.0"
publish = false

[lib]
name = "demo_app_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2.6.3", features = [] }

[dependencies]
tauri = { version = "2.11.5", features = [] }
tauri-plugin-icloud-kvs = { path = "../.." }
```

- [ ] **Step 3: Create `examples/demo-app/src-tauri/build.rs`**

```rust
fn main() {
   tauri_build::build();
}
```

- [ ] **Step 4: Create `examples/demo-app/src-tauri/src/lib.rs`**

```rust
//! Demo app for tauri-plugin-icloud-kvs. All app logic lives in the
//! frontend (`../src/main.ts`); this crate only registers the plugin.

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
   tauri::Builder::default()
      .plugin(tauri_plugin_icloud_kvs::init())
      .run(tauri::generate_context!())
      .expect("error while running demo app");
}
```

- [ ] **Step 5: Create `examples/demo-app/src-tauri/src/main.rs`**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
   demo_app_lib::run();
}
```

- [ ] **Step 6: Create `examples/demo-app/src-tauri/tauri.conf.json`**

```json
{
   "$schema": "https://schema.tauri.app/config/2",
   "productName": "icloud-kvs-demo",
   "version": "0.1.0",
   "identifier": "com.kmuncie.icloud-kvs-demo",
   "build": {
      "beforeDevCommand": "npm run dev",
      "devUrl": "http://localhost:1420",
      "beforeBuildCommand": "npm run build",
      "frontendDist": "../dist"
   },
   "app": {
      "windows": [
         {
            "title": "iCloud KVS Demo",
            "width": 960,
            "height": 680
         }
      ],
      "security": {
         "csp": null
      }
   },
   "bundle": {
      "active": true,
      "targets": ["app"],
      "macOS": {
         "entitlements": "./entitlements.plist"
      }
   }
}
```

- [ ] **Step 7: Create `examples/demo-app/src-tauri/entitlements.plist`**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
   <key>com.apple.developer.ubiquity-kvstore-identifier</key>
   <string>$(TeamIdentifierPrefix)$(CFBundleIdentifier)</string>
</dict>
</plist>
```

(The `$(…)` variables are substituted by Xcode-driven signing. Tauri's own
`codesign` path does NOT substitute them — the app README written in Task 3
documents replacing them with a literal `TEAMID.bundle.id` for signed macOS
bundles. Unsigned dev builds ignore entitlements entirely.)

- [ ] **Step 8: Create `examples/demo-app/src-tauri/capabilities/default.json`**

```json
{
   "$schema": "../gen/schemas/desktop-schema.json",
   "identifier": "default",
   "description": "Demo-app window permissions",
   "windows": ["main"],
   "permissions": ["core:default", "icloud-kvs:default"]
}
```

- [ ] **Step 9: Verify the crate compiles**

Run: `cd examples/demo-app/src-tauri && cargo check 2>&1 | tail -3`
Expected: `Finished` with no errors. (First run compiles the whole tauri stack — several minutes. The schema reference in `capabilities/default.json` points at `gen/`, which tauri-build generates during this check.)

- [ ] **Step 10: Commit**

```bash
cd ../../..   # repo root
git add examples/demo-app/.gitignore examples/demo-app/src-tauri
git commit -m "feat: Scaffold demo app Rust side with plugin registered"
```

---

### Task 2: Frontend — KV editor + placeholder event-log pane

**Files:**
- Create: `examples/demo-app/package.json`, `examples/demo-app/tsconfig.json`, `examples/demo-app/vite.config.ts`, `examples/demo-app/index.html`, `examples/demo-app/src/main.ts`, `examples/demo-app/src/styles.css`

**Interfaces:**
- Consumes: `tauri-plugin-icloud-kvs-api` (from `file:../../guest-js`): `get`, `set`, `remove`, `getAll`, `synchronize`, `accountStatus`, type `KvsValue`. Task 1's window label `main` and dev port 1420.
- Produces: `npm run build` emitting `dist/`; DOM ids used below are internal to this task.

- [ ] **Step 1: Resolve exact dependency versions**

```bash
npm view vite version && npm view typescript version && npm view @tauri-apps/cli version
```

Note the outputs; use them for the `<PIN>` placeholders in Step 2. `typescript` should match guest-js's `7.0.2` unless a newer patch exists.

- [ ] **Step 2: Create `examples/demo-app/package.json`** (replace `<PIN>`s)

```json
{
   "name": "icloud-kvs-demo",
   "private": true,
   "version": "0.1.0",
   "type": "module",
   "engines": {
      "node": ">=24"
   },
   "scripts": {
      "dev": "vite",
      "build": "tsc && vite build",
      "tauri": "tauri"
   },
   "dependencies": {
      "tauri-plugin-icloud-kvs-api": "file:../../guest-js"
   },
   "devDependencies": {
      "@tauri-apps/cli": "<PIN>",
      "typescript": "<PIN>",
      "vite": "<PIN>"
   }
}
```

(`@tauri-apps/api` comes transitively through the guest-js package; the demo app imports only the plugin API.)

- [ ] **Step 3: Create `examples/demo-app/tsconfig.json`**

```json
{
   "compilerOptions": {
      "target": "ES2022",
      "module": "ESNext",
      "moduleResolution": "Bundler",
      "lib": ["ES2022", "DOM", "DOM.Iterable"],
      "strict": true,
      "noEmit": true,
      "skipLibCheck": true
   },
   "include": ["src"]
}
```

- [ ] **Step 4: Create `examples/demo-app/vite.config.ts`**

```ts
import { defineConfig } from 'vite';

export default defineConfig({
   clearScreen: false,
   server: {
      port: 1420,
      strictPort: true,
   },
});
```

- [ ] **Step 5: Create `examples/demo-app/index.html`**

```html
<!doctype html>
<html lang="en">
   <head>
      <meta charset="UTF-8" />
      <meta name="viewport" content="width=device-width, initial-scale=1.0" />
      <title>iCloud KVS Demo</title>
      <link rel="stylesheet" href="/src/styles.css" />
   </head>
   <body>
      <header>
         <h1>iCloud KVS Demo</h1>
         <span id="account-status" class="badge">checking…</span>
         <button id="synchronize">Synchronize</button>
         <span id="sync-result"></span>
      </header>

      <div id="error-bar" hidden></div>

      <main>
         <section id="editor-pane">
            <h2>Key-Value Store</h2>
            <form id="kv-form">
               <input id="kv-key" placeholder="key" autocomplete="off" />
               <textarea id="kv-value" rows="3"
                  placeholder='value — parsed as JSON, else stored as a string'></textarea>
               <div class="form-actions">
                  <button type="submit">Set</button>
                  <button type="button" id="refresh">Refresh</button>
               </div>
            </form>
            <table id="kv-table">
               <thead>
                  <tr><th>Key</th><th>Value</th><th></th></tr>
               </thead>
               <tbody></tbody>
            </table>
         </section>

         <section id="event-pane">
            <h2>External Changes</h2>
            <p class="placeholder">
               Live external-change events land in M1.4. This pane will list
               each change with its reason (serverChange, initialSync,
               quotaViolation, accountChange) and the affected keys.
            </p>
         </section>
      </main>

      <script type="module" src="/src/main.ts"></script>
   </body>
</html>
```

- [ ] **Step 6: Create `examples/demo-app/src/styles.css`**

```css
:root {
   font-family: system-ui, sans-serif;
   color-scheme: light dark;
}

body {
   margin: 0;
   padding: 1rem;
}

header {
   display: flex;
   align-items: center;
   gap: 0.75rem;
}

header h1 {
   font-size: 1.25rem;
   margin: 0 auto 0 0;
}

.badge {
   padding: 0.15rem 0.6rem;
   border-radius: 999px;
   background: #8884;
   font-size: 0.85rem;
}

.badge.available {
   background: #2a24;
}

.badge.no-account {
   background: #a224;
}

#error-bar {
   margin-top: 0.75rem;
   padding: 0.5rem 0.75rem;
   border-radius: 6px;
   background: #a224;
   font-family: ui-monospace, monospace;
   font-size: 0.85rem;
}

main {
   display: grid;
   grid-template-columns: 3fr 2fr;
   gap: 1.5rem;
   margin-top: 1rem;
}

#kv-form {
   display: grid;
   gap: 0.5rem;
   margin-bottom: 1rem;
}

#kv-form input,
#kv-form textarea {
   font-family: ui-monospace, monospace;
   padding: 0.4rem;
}

.form-actions {
   display: flex;
   gap: 0.5rem;
}

#kv-table {
   width: 100%;
   border-collapse: collapse;
}

#kv-table th,
#kv-table td {
   text-align: left;
   padding: 0.35rem 0.5rem;
   border-bottom: 1px solid #8883;
   vertical-align: top;
}

#kv-table td.value {
   font-family: ui-monospace, monospace;
   white-space: pre-wrap;
   word-break: break-word;
}

#kv-table tbody tr {
   cursor: pointer;
}

.placeholder {
   color: #888;
   font-style: italic;
}
```

- [ ] **Step 7: Create `examples/demo-app/src/main.ts`**

```ts
/**
 * Demo app UI: a key-value editor over tauri-plugin-icloud-kvs.
 * Doubles as the plugin's manual test rig — errors are surfaced
 * verbatim in the error bar rather than swallowed.
 */

import {
   accountStatus,
   getAll,
   remove,
   set,
   synchronize,
   type KvsValue,
} from 'tauri-plugin-icloud-kvs-api';

const statusBadge = document.querySelector<HTMLSpanElement>('#account-status')!,
      syncButton = document.querySelector<HTMLButtonElement>('#synchronize')!,
      syncResult = document.querySelector<HTMLSpanElement>('#sync-result')!,
      errorBar = document.querySelector<HTMLDivElement>('#error-bar')!,
      form = document.querySelector<HTMLFormElement>('#kv-form')!,
      keyInput = document.querySelector<HTMLInputElement>('#kv-key')!,
      valueInput = document.querySelector<HTMLTextAreaElement>('#kv-value')!,
      refreshButton = document.querySelector<HTMLButtonElement>('#refresh')!,
      tableBody = document.querySelector<HTMLTableSectionElement>('#kv-table tbody')!;

function showError(err: unknown): void {
   errorBar.textContent = String(err);
   errorBar.hidden = false;
}

function clearError(): void {
   errorBar.hidden = true;
}

/**
 * Parses the value textarea: JSON when possible, otherwise the raw text
 * as a string (convenience for quick manual tests).
 */
function parseValue(raw: string): KvsValue {
   try {
      return JSON.parse(raw) as KvsValue;
   } catch {
      return raw;
   }
}

async function updateAccountStatus(): Promise<void> {
   try {
      const status = await accountStatus();

      statusBadge.textContent = status;
      statusBadge.className = 'badge ' + (status === 'available' ? 'available' : 'no-account');
   } catch (err) {
      statusBadge.textContent = 'unknown';
      showError(err);
   }
}

async function refresh(): Promise<void> {
   try {
      const all = await getAll();

      tableBody.replaceChildren();

      for (const [key, value] of Object.entries(all)) {
         const row = document.createElement('tr'),
               keyCell = document.createElement('td'),
               valueCell = document.createElement('td'),
               actionCell = document.createElement('td'),
               deleteButton = document.createElement('button');

         keyCell.textContent = key;
         valueCell.textContent = JSON.stringify(value, null, 2);
         valueCell.className = 'value';
         deleteButton.textContent = 'Delete';
         deleteButton.addEventListener('click', async (event) => {
            event.stopPropagation();
            clearError();
            try {
               await remove(key);
               await refresh();
            } catch (err) {
               showError(err);
            }
         });
         row.addEventListener('click', () => {
            keyInput.value = key;
            valueInput.value = JSON.stringify(value, null, 2);
         });

         actionCell.appendChild(deleteButton);
         row.append(keyCell, valueCell, actionCell);
         tableBody.appendChild(row);
      }
   } catch (err) {
      showError(err);
   }
}

form.addEventListener('submit', async (event) => {
   event.preventDefault();
   clearError();
   try {
      await set(keyInput.value, parseValue(valueInput.value));
      await refresh();
   } catch (err) {
      showError(err);
   }
});

refreshButton.addEventListener('click', () => {
   clearError();
   void refresh();
});

syncButton.addEventListener('click', async () => {
   clearError();
   try {
      const flushed = await synchronize();

      syncResult.textContent = `synchronize() → ${String(flushed)}`;
   } catch (err) {
      showError(err);
   }
});

window.addEventListener('focus', () => void updateAccountStatus());

void updateAccountStatus();
void refresh();
```

- [ ] **Step 8: Install and build**

```bash
cd guest-js && npm ci && npm run build && cd ..
cd examples/demo-app && npm install && npm run build
```

Expected: `tsc` clean, `vite build` emits `dist/`. (`npm install` — not `ci` — on first run to generate `package-lock.json`; the `file:` dep needs guest-js `dist/` built first.)

- [ ] **Step 9: Smoke-test on macOS**

Run: `cd examples/demo-app && npm run tauri dev`
Expected: window opens; account badge populates; Set/Refresh/Delete work. **Note:** an unsigned dev build has an inert store — writes silently no-op and the table stays empty. Seeing the UI operate without errors is the pass bar here; the badge and `synchronize() → false` are expected. Close the window to exit.

- [ ] **Step 10: Commit**

```bash
git add examples/demo-app/package.json examples/demo-app/package-lock.json \
   examples/demo-app/tsconfig.json examples/demo-app/vite.config.ts \
   examples/demo-app/index.html examples/demo-app/src
git commit -m "feat: Add demo app KV editor frontend"
```

---

### Task 3: App README + DEVELOPERS.md updates

**Files:**
- Create: `examples/demo-app/README.md`
- Modify: `DEVELOPERS.md` (two protocol sections reference "a scratch Tauri app"), `README.md` (mention the example)

**Interfaces:**
- Consumes: everything above.

- [ ] **Step 1: Create `examples/demo-app/README.md`**

````markdown
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

Open the generated project in Xcode, select your team under
Signing & Capabilities, add the **iCloud** capability with
**Key-value storage** checked, then:

```sh
npm run tauri ios dev
```

Sign in to an Apple ID in the simulator (Settings) for
`accountStatus()` to report `available`.
````

- [ ] **Step 2: Update `DEVELOPERS.md`**

In the "Cross-device sync verification (manual)" section, replace:

```markdown
1. Build a scratch Tauri app (or, once it exists, `examples/demo-app`)
   that registers this plugin, with the
```

with:

```markdown
1. Build `examples/demo-app` (see its README for signing) — it registers
   this plugin, with the
```

In the "iOS verification (manual, simulator)" section, replace:

```markdown
1. Create a scratch Tauri app, register this plugin as a path
   dependency, and run `tauri ios init`.
```

with:

```markdown
1. In `examples/demo-app`, run `npm run tauri ios init` (see its
   README).
```

- [ ] **Step 3: Update root `README.md`**

After the Usage section's closing paragraph (ends "…detect the signed-out
case."), add:

```markdown
A runnable example lives in
[`examples/demo-app`](examples/demo-app/README.md) — a key-value editor
that doubles as the plugin's manual test rig.
```

- [ ] **Step 4: Verify plugin checks still green**

Run: `cargo test && cargo lint-clippy && cargo lint-fmt`
Expected: PASS (docs-only change).

- [ ] **Step 5: Commit**

```bash
git add examples/demo-app/README.md DEVELOPERS.md README.md
git commit -m "docs: Document demo app usage and signing setup"
```

---

### Task 4: CI job + push

**Files:**
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: Tasks 1–2 build commands.
- Produces: a `demo-app` CI job failing on plugin↔app API drift.

- [ ] **Step 1: Append the job to `.github/workflows/ci.yml`**

```yaml
   demo-app:
      runs-on: macos-14
      steps:
         - uses: actions/checkout@v5
         - name: Install toolchain from rust-toolchain.toml
           run: rustup show active-toolchain || rustup toolchain install
         - uses: Swatinem/rust-cache@v2
           with:
              workspaces: examples/demo-app/src-tauri
         - uses: actions/setup-node@v5
           with:
              node-version-file: .nvmrc
         - name: Build guest bindings
           run: npm ci && npm run build
           working-directory: guest-js
         - name: Build demo frontend
           run: npm ci && npm run build
           working-directory: examples/demo-app
         - name: Check demo Rust crate
           run: cargo check
           working-directory: examples/demo-app/src-tauri
```

(3-space YAML indent; job key at the same level as `rust:` / `typescript:`.)

- [ ] **Step 2: Commit and push**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: Add demo-app build checks"
git push
```

- [ ] **Step 3: Watch CI**

```bash
RUN_ID=$(gh run list --repo kmuncie/tauri-plugin-icloud-kvs --branch main --limit 1 --json databaseId --jq '.[0].databaseId')
gh run watch "$RUN_ID" --repo kmuncie/tauri-plugin-icloud-kvs --exit-status --compact
```

Expected: all three jobs green. The `demo-app` job's first run is slow (cold cargo cache); subsequent runs are cached.

---

### Task 5: Hand back to M1.3 close-out

Not a build task — after this plan completes, resume
`docs/plans/m1-3-ios-implementation.md` Task 5 Steps 2–4 using this app as
the simulator host (`npm run tauri ios init` → Xcode capability → `npm run
tauri ios dev`), then check off M1.3. The demo app itself stays listed as
an M1.5 deliverable (event-log pane still pending M1.4).
