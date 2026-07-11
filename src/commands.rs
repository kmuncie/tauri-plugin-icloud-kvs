//! `#[tauri::command]` wrappers around the crate-root API. Kept thin:
//! all validation and platform logic lives behind the public functions.

use serde_json::{Map, Value};

use crate::error::Result;
use crate::models::AccountStatus;

#[tauri::command]
pub(crate) fn get(key: String) -> Result<Option<Value>> {
   crate::get(&key)
}

#[tauri::command]
pub(crate) fn set(key: String, value: Value) -> Result<()> {
   crate::set(&key, &value)
}

#[tauri::command]
pub(crate) fn remove(key: String) -> Result<()> {
   crate::remove(&key)
}

#[tauri::command]
pub(crate) fn keys() -> Result<Vec<String>> {
   crate::keys()
}

#[tauri::command]
pub(crate) fn get_all() -> Result<Map<String, Value>> {
   crate::get_all()
}

#[tauri::command]
pub(crate) fn synchronize() -> Result<bool> {
   crate::synchronize()
}

#[tauri::command]
pub(crate) fn account_status() -> Result<AccountStatus> {
   crate::account_status()
}
