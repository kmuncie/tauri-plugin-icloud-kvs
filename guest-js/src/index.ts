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
