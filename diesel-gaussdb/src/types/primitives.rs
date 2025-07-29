//! Primitive type support for GaussDB
//!
//! This module provides support for basic PostgreSQL-compatible types
//! that are supported by GaussDB.

use crate::backend::GaussDB;
use crate::value::GaussDBValue;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::*;
use std::io::Write;

// Implement basic integer types
impl FromSql<SmallInt, GaussDB> for i16 {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("SmallInt value is null")?;
        if bytes.len() != 2 {
            return Err("Invalid SmallInt length".into());
        }
        Ok(i16::from_be_bytes([bytes[0], bytes[1]]))
    }
}

impl ToSql<SmallInt, GaussDB> for i16 {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_all(&self.to_be_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<Integer, GaussDB> for i32 {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("Integer value is null")?;
        if bytes.len() != 4 {
            return Err("Invalid Integer length".into());
        }
        Ok(i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }
}

impl ToSql<Integer, GaussDB> for i32 {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_all(&self.to_be_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<BigInt, GaussDB> for i64 {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("BigInt value is null")?;
        if bytes.len() != 8 {
            return Err("Invalid BigInt length".into());
        }
        let mut array = [0u8; 8];
        array.copy_from_slice(bytes);
        Ok(i64::from_be_bytes(array))
    }
}

impl ToSql<BigInt, GaussDB> for i64 {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_all(&self.to_be_bytes())?;
        Ok(IsNull::No)
    }
}

// Implement floating point types
impl FromSql<Float, GaussDB> for f32 {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("Float value is null")?;
        if bytes.len() != 4 {
            return Err("Invalid Float length".into());
        }
        let bits = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        Ok(f32::from_bits(bits))
    }
}

impl ToSql<Float, GaussDB> for f32 {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_all(&self.to_bits().to_be_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<Double, GaussDB> for f64 {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("Double value is null")?;
        if bytes.len() != 8 {
            return Err("Invalid Double length".into());
        }
        let mut array = [0u8; 8];
        array.copy_from_slice(bytes);
        let bits = u64::from_be_bytes(array);
        Ok(f64::from_bits(bits))
    }
}

impl ToSql<Double, GaussDB> for f64 {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_all(&self.to_bits().to_be_bytes())?;
        Ok(IsNull::No)
    }
}

// Implement text types
impl FromSql<Text, GaussDB> for String {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("Text value is null")?;
        String::from_utf8(bytes.to_vec()).map_err(|e| format!("Invalid UTF-8: {}", e).into())
    }
}

impl ToSql<Text, GaussDB> for String {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_all(self.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl ToSql<Text, GaussDB> for str {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_all(self.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl<'a> ToSql<Text, GaussDB> for &'a str {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_all(self.as_bytes())?;
        Ok(IsNull::No)
    }
}

// Implement boolean type
impl FromSql<Bool, GaussDB> for bool {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("Bool value is null")?;
        if bytes.len() != 1 {
            return Err("Invalid Bool length".into());
        }
        Ok(bytes[0] != 0)
    }
}

impl ToSql<Bool, GaussDB> for bool {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_all(&[if *self { 1 } else { 0 }])?;
        Ok(IsNull::No)
    }
}

// Implement binary data type
impl FromSql<Binary, GaussDB> for Vec<u8> {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("Binary value is null")?;
        Ok(bytes.to_vec())
    }
}

impl ToSql<Binary, GaussDB> for Vec<u8> {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_all(self)?;
        Ok(IsNull::No)
    }
}

impl ToSql<Binary, GaussDB> for [u8] {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_all(self)?;
        Ok(IsNull::No)
    }
}

impl<'a> ToSql<Binary, GaussDB> for &'a [u8] {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_all(self)?;
        Ok(IsNull::No)
    }
}

// Implement OID type
impl FromSql<diesel::sql_types::Oid, GaussDB> for u32 {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("OID value is null")?;
        if bytes.len() != 4 {
            return Err("Invalid OID length".into());
        }
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }
}

impl ToSql<diesel::sql_types::Oid, GaussDB> for u32 {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_all(&self.to_be_bytes())?;
        Ok(IsNull::No)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::GaussDBValue;
    use diesel::query_builder::bind_collector::ByteWrapper;
    use diesel::serialize::{Output, ToSql};
    use diesel::deserialize::FromSql;

    #[test]
    fn test_i16_roundtrip() {
        let original = 12345i16;
        let mut buffer = Vec::new();
        let mut output = Output::test(ByteWrapper(&mut buffer));
        
        original.to_sql(&mut output).unwrap();
        let value = GaussDBValue::new(Some(&buffer), 21); // smallint OID
        let result: i16 = FromSql::from_sql(value).unwrap();
        
        assert_eq!(original, result);
    }

    #[test]
    fn test_i32_roundtrip() {
        let original = 1234567890i32;
        let mut buffer = Vec::new();
        let mut output = Output::test(ByteWrapper(&mut buffer));
        
        original.to_sql(&mut output).unwrap();
        let value = GaussDBValue::new(Some(&buffer), 23); // int4 OID
        let result: i32 = FromSql::from_sql(value).unwrap();
        
        assert_eq!(original, result);
    }

    #[test]
    fn test_i64_roundtrip() {
        let original = 1234567890123456789i64;
        let mut buffer = Vec::new();
        let mut output = Output::test(ByteWrapper(&mut buffer));
        
        original.to_sql(&mut output).unwrap();
        let value = GaussDBValue::new(Some(&buffer), 20); // int8 OID
        let result: i64 = FromSql::from_sql(value).unwrap();
        
        assert_eq!(original, result);
    }

    #[test]
    fn test_f32_roundtrip() {
        let original = 3.14159f32;
        let mut buffer = Vec::new();
        let mut output = Output::test(ByteWrapper(&mut buffer));
        
        original.to_sql(&mut output).unwrap();
        let value = GaussDBValue::new(Some(&buffer), 700); // float4 OID
        let result: f32 = FromSql::from_sql(value).unwrap();
        
        assert!((original - result).abs() < f32::EPSILON);
    }

    #[test]
    fn test_f64_roundtrip() {
        let original = 3.141592653589793f64;
        let mut buffer = Vec::new();
        let mut output = Output::test(ByteWrapper(&mut buffer));
        
        original.to_sql(&mut output).unwrap();
        let value = GaussDBValue::new(Some(&buffer), 701); // float8 OID
        let result: f64 = FromSql::from_sql(value).unwrap();
        
        assert!((original - result).abs() < f64::EPSILON);
    }

    #[test]
    fn test_string_roundtrip() {
        let original = "Hello, GaussDB!".to_string();
        let mut buffer = Vec::new();
        let mut output = Output::test(ByteWrapper(&mut buffer));
        
        original.to_sql(&mut output).unwrap();
        let value = GaussDBValue::new(Some(&buffer), 25); // text OID
        let result: String = FromSql::from_sql(value).unwrap();
        
        assert_eq!(original, result);
    }

    #[test]
    fn test_str_to_sql() {
        let original = "Hello, GaussDB!";
        let mut buffer = Vec::new();
        let mut output = Output::test(ByteWrapper(&mut buffer));
        
        original.to_sql(&mut output).unwrap();
        assert_eq!(buffer, original.as_bytes());
    }

    #[test]
    fn test_bool_roundtrip() {
        for &original in &[true, false] {
            let mut buffer = Vec::new();
            let mut output = Output::test(ByteWrapper(&mut buffer));
            
            original.to_sql(&mut output).unwrap();
            let value = GaussDBValue::new(Some(&buffer), 16); // bool OID
            let result: bool = FromSql::from_sql(value).unwrap();
            
            assert_eq!(original, result);
        }
    }

    #[test]
    fn test_binary_roundtrip() {
        let original = vec![0x01, 0x02, 0x03, 0x04, 0xFF];
        let mut buffer = Vec::new();
        let mut output = Output::test(ByteWrapper(&mut buffer));
        
        original.to_sql(&mut output).unwrap();
        let value = GaussDBValue::new(Some(&buffer), 17); // bytea OID
        let result: Vec<u8> = FromSql::from_sql(value).unwrap();
        
        assert_eq!(original, result);
    }

    #[test]
    fn test_slice_to_sql() {
        let original = &[0x01, 0x02, 0x03, 0x04, 0xFF];
        let mut buffer = Vec::new();
        let mut output = Output::test(ByteWrapper(&mut buffer));
        
        original.to_sql(&mut output).unwrap();
        assert_eq!(buffer, original);
    }

    #[test]
    fn test_oid_roundtrip() {
        let original = 12345u32;
        let mut buffer = Vec::new();
        let mut output = Output::test(ByteWrapper(&mut buffer));
        
        original.to_sql(&mut output).unwrap();
        let value = GaussDBValue::new(Some(&buffer), 26); // oid OID
        let result: u32 = FromSql::from_sql(value).unwrap();
        
        assert_eq!(original, result);
    }

    #[test]
    fn test_null_values() {
        let value = GaussDBValue::new(None, 25);
        
        let result: Result<String, _> = FromSql::from_sql(value);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Text value is null");
    }

    #[test]
    fn test_invalid_length() {
        // Test with wrong length for i32
        let value = GaussDBValue::new(Some(&[0x01, 0x02]), 23); // Only 2 bytes for int4
        let result: Result<i32, _> = FromSql::from_sql(value);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Invalid Integer length");
    }

    #[test]
    fn test_invalid_utf8() {
        // Test with invalid UTF-8 bytes
        let invalid_utf8 = &[0xFF, 0xFE, 0xFD];
        let value = GaussDBValue::new(Some(invalid_utf8), 25);
        let result: Result<String, _> = FromSql::from_sql(value);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid UTF-8"));
    }
}
