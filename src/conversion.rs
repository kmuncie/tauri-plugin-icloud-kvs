//! JSON ↔ property-list conversion for the macOS implementation.
//!
//! Storable values map 1:1 onto plist types (NSString, NSNumber,
//! NSArray, NSDictionary). `null` is not storable. Raw `NSData` written
//! by other native code is returned as a base64 string (documented
//! edge case; there is no bytes API in v1).

use objc2::encode::Encoding;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_foundation::{
   NSArray, NSData, NSDataBase64EncodingOptions, NSDictionary, NSNumber, NSString,
};
use serde_json::{Map, Number, Value};

use crate::error::{Error, Result};

#[allow(dead_code)]
pub(crate) fn json_to_plist(value: &Value) -> Result<Retained<AnyObject>> {
   match value {
      Value::Null => Err(Error::Serialization(
         "null is not a storable value; use remove() to delete a key".into(),
      )),
      Value::Bool(b) => Ok(NSNumber::new_bool(*b).into()),
      Value::Number(n) => number_to_plist(n),
      Value::String(s) => Ok(NSString::from_str(s).into()),
      Value::Array(items) => {
         let converted = items
            .iter()
            .map(json_to_plist)
            .collect::<Result<Vec<_>>>()?;

         Ok(NSArray::from_retained_slice(&converted).into())
      }
      Value::Object(map) => {
         let keys: Vec<Retained<NSString>> = map.keys().map(|k| NSString::from_str(k)).collect();
         let key_refs: Vec<&NSString> = keys.iter().map(|k| &**k).collect();
         let values = map
            .values()
            .map(json_to_plist)
            .collect::<Result<Vec<_>>>()?;

         Ok(NSDictionary::from_retained_objects(&key_refs, &values).into())
      }
   }
}

fn number_to_plist(n: &Number) -> Result<Retained<AnyObject>> {
   if let Some(i) = n.as_i64() {
      Ok(NSNumber::new_i64(i).into())
   } else if let Some(u) = n.as_u64() {
      Ok(NSNumber::new_u64(u).into())
   } else if let Some(f) = n.as_f64() {
      Ok(NSNumber::new_f64(f).into())
   } else {
      Err(Error::Serialization(format!(
         "unrepresentable JSON number: {n}"
      )))
   }
}

#[allow(dead_code)]
pub(crate) fn plist_to_json(obj: &AnyObject) -> Result<Value> {
   if let Some(s) = obj.downcast_ref::<NSString>() {
      return Ok(Value::String(s.to_string()));
   }

   if let Some(n) = obj.downcast_ref::<NSNumber>() {
      return number_to_json(n);
   }

   if let Some(array) = obj.downcast_ref::<NSArray>() {
      let items = array
         .to_vec()
         .iter()
         .map(|item| plist_to_json(item))
         .collect::<Result<Vec<_>>>()?;

      return Ok(Value::Array(items));
   }

   if let Some(dict) = obj.downcast_ref::<NSDictionary>() {
      let (keys, values) = dict.to_vecs();
      let mut map = Map::with_capacity(keys.len());

      for (key, value) in keys.into_iter().zip(values) {
         let key_string = key
            .downcast_ref::<NSString>()
            .ok_or_else(|| Error::Serialization("non-string dictionary key".into()))?
            .to_string();

         map.insert(key_string, plist_to_json(&value)?);
      }

      return Ok(Value::Object(map));
   }

   if let Some(data) = obj.downcast_ref::<NSData>() {
      let base64 = data.base64EncodedStringWithOptions(NSDataBase64EncodingOptions::empty());

      return Ok(Value::String(base64.to_string()));
   }

   Err(Error::Serialization(format!(
      "unsupported plist type: {:?}",
      obj.class()
   )))
}

fn number_to_json(n: &NSNumber) -> Result<Value> {
   match n.encoding() {
      // CFBoolean reports 'c' (Char); C99 _Bool reports 'B'. An NSNumber
      // wrapping a genuine i8 also reports 'c' and will read back as a
      // boolean — acceptable: this plugin never writes i8, and JSON has
      // no i8 type to preserve.
      Encoding::Char | Encoding::Bool => Ok(Value::Bool(n.as_bool())),
      Encoding::Float | Encoding::Double => Number::from_f64(n.as_f64())
         .map(Value::Number)
         .ok_or_else(|| Error::Serialization("non-finite float in store".into())),
      Encoding::ULongLong => Ok(Value::Number(Number::from(n.as_u64()))),
      _ => Ok(Value::Number(Number::from(n.as_i64()))),
   }
}

#[cfg(test)]
mod tests {
   use objc2_foundation::NSData;
   use serde_json::json;

   use super::*;
   use crate::error::Error;

   #[test]
   fn round_trips_every_json_shape() {
      let value = json!({
         "string": "hello",
         "int": 42,
         "negative": -7,
         "float": 1.5,
         "boolTrue": true,
         "boolFalse": false,
         "list": [1, "two", false],
         "nested": { "inner": [true, 2.25] }
      });

      let plist = json_to_plist(&value).unwrap();

      assert_eq!(plist_to_json(&plist).unwrap(), value);
   }

   #[test]
   fn booleans_stay_booleans_not_numbers() {
      let plist = json_to_plist(&json!(true)).unwrap();

      assert_eq!(plist_to_json(&plist).unwrap(), json!(true));
   }

   #[test]
   fn null_is_rejected_everywhere() {
      assert!(matches!(
         json_to_plist(&json!(null)),
         Err(Error::Serialization(_))
      ));
      assert!(matches!(
         json_to_plist(&json!({ "a": null })),
         Err(Error::Serialization(_))
      ));
      assert!(matches!(
         json_to_plist(&json!([1, null])),
         Err(Error::Serialization(_))
      ));
   }

   #[test]
   fn foreign_nsdata_reads_back_as_base64_string() {
      let data = NSData::with_bytes(&[1, 2, 3]);

      assert_eq!(plist_to_json(&data).unwrap(), json!("AQID"));
   }
}
