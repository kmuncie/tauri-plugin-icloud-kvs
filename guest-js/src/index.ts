/**
 * TypeScript bindings for tauri-plugin-icloud-kvs.
 *
 * `onExternalChange` (change events) is added in a later milestone;
 * everything else in the API surface is available here.
 */

import { invoke } from '@tauri-apps/api/core';

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

/**
 * Whether the device is signed in to iCloud. When signed out, KVS
 * silently degrades to local-only storage; this is the only way to
 * detect that condition.
 */
export type AccountStatus = 'available' | 'noAccount';

/**
 * Returns the value for `key`, or `null` if the key is not present.
 */
export async function get(key: string): Promise<KvsValue | null> {
   return await invoke<KvsValue | null>('plugin:icloud-kvs|get', { key });
}

/**
 * Stores `value` under `key` and requests upload to iCloud. The OS
 * throttles/coalesces frequent writes — debounce rapid updates in the
 * caller.
 */
export async function set(key: string, value: KvsValue): Promise<void> {
   await invoke('plugin:icloud-kvs|set', { key, value });
}

/**
 * Deletes `key` from the store. Deleting a missing key is not an error.
 */
export async function remove(key: string): Promise<void> {
   await invoke('plugin:icloud-kvs|remove', { key });
}

/**
 * Lists every key currently in the store.
 */
export async function keys(): Promise<string[]> {
   return await invoke<string[]>('plugin:icloud-kvs|keys');
}

/**
 * Returns the entire store as a plain object.
 */
export async function getAll(): Promise<Record<string, KvsValue>> {
   return await invoke<Record<string, KvsValue>>('plugin:icloud-kvs|get_all');
}

/**
 * Flush-only: writes pending changes to local disk and *requests*
 * upload. Does NOT force a server round-trip or pull fresh data —
 * do not build "sync now" UX on this.
 */
export async function synchronize(): Promise<boolean> {
   return await invoke<boolean>('plugin:icloud-kvs|synchronize');
}

/**
 * Reports whether the device is signed in to iCloud
 * (via `FileManager.ubiquityIdentityToken`).
 */
export async function accountStatus(): Promise<AccountStatus> {
   return await invoke<AccountStatus>('plugin:icloud-kvs|account_status');
}
