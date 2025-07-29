//! Type system for GaussDB
//!
//! This module provides type mappings and value handling for GaussDB,
//! which is largely PostgreSQL-compatible.

use std::fmt;

/// A raw value from GaussDB
///
/// This represents a value as received from the database before deserialization.
#[derive(Debug, Clone)]
pub struct GaussDBValue<'a> {
    raw_bytes: Option<&'a [u8]>,
    type_oid: u32,
}

impl<'a> GaussDBValue<'a> {
    /// Create a new GaussDBValue
    pub fn new(raw_bytes: Option<&'a [u8]>, type_oid: u32) -> Self {
        Self { raw_bytes, type_oid }
    }

    /// Get the raw bytes of this value
    pub fn as_bytes(&self) -> Option<&'a [u8]> {
        self.raw_bytes
    }

    /// Get the type OID of this value
    pub fn type_oid(&self) -> u32 {
        self.type_oid
    }

    /// Check if this value is NULL
    pub fn is_null(&self) -> bool {
        self.raw_bytes.is_none()
    }
}

impl<'a> fmt::Display for GaussDBValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.raw_bytes {
            Some(bytes) => {
                write!(f, "GaussDBValue(type_oid: {}, bytes: {:?})", self.type_oid, bytes)
            }
            None => {
                write!(f, "GaussDBValue(type_oid: {}, NULL)", self.type_oid)
            }
        }
    }
}

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
    fn test_gaussdb_value_display() {
        let bytes = b"test";
        let value = GaussDBValue::new(Some(bytes), 25);
        let display = format!("{}", value);
        
        assert!(display.contains("type_oid: 25"));
        assert!(display.contains("bytes:"));
    }

    #[test]
    fn test_gaussdb_value_display_null() {
        let value = GaussDBValue::new(None, 25);
        let display = format!("{}", value);
        
        assert!(display.contains("type_oid: 25"));
        assert!(display.contains("NULL"));
    }
}
