//! Type system for GaussDB
//!
//! This module provides type mappings and value handling for GaussDB,
//! which is largely PostgreSQL-compatible.

// For now, we'll focus on basic type support without complex array/json implementations
// mod array;
// #[cfg(feature = "serde_json")]
// mod json;
// mod primitives;
// mod custom;

pub mod sql_types;

// Re-export GaussDBValue from the value module
pub use crate::value::GaussDBValue;

// Re-export commonly used types from diesel
pub use diesel::sql_types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaussdb_value_creation() {
        let bytes = b"test";
        let value = GaussDBValue::new(Some(bytes), 25);
        
        assert_eq!(value.as_bytes(), Some(bytes.as_slice()));
        assert_eq!(value.type_oid(), 25);
        assert!(!value.is_null());
    }

    #[test]
    fn test_gaussdb_value_null() {
        let value = GaussDBValue::new(None, 25);
        
        assert_eq!(value.as_bytes(), None);
        assert_eq!(value.type_oid(), 25);
        assert!(value.is_null());
    }

    #[test]
    fn test_gaussdb_value_debug() {
        let bytes = b"test";
        let value = GaussDBValue::new(Some(bytes), 25);
        let debug = format!("{:?}", value);

        assert!(debug.contains("type_oid: 25"));
        assert!(debug.contains("4 bytes"));
    }

    #[test]
    fn test_gaussdb_value_debug_null() {
        let value = GaussDBValue::new(None, 25);
        let debug = format!("{:?}", value);

        assert!(debug.contains("type_oid: 25"));
        assert!(debug.contains("None"));
    }
}
