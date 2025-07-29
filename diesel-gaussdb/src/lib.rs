//! # Diesel-GaussDB
//!
//! A GaussDB backend for the Diesel ORM framework.
//!
//! This crate provides GaussDB database support for Diesel, enabling Rust applications
//! to work with GaussDB and OpenGauss databases using Diesel's type-safe query builder.
//!
//! ## Features
//!
//! - **GaussDB Compatibility**: Full support for GaussDB and OpenGauss databases
//! - **PostgreSQL Protocol**: Leverages PostgreSQL compatibility for maximum feature support
//! - **Type Safety**: Compile-time verified queries with Diesel's type system
//! - **Authentication**: Support for GaussDB's SHA256 and MD5_SHA256 authentication methods
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use diesel::prelude::*;
//! use diesel_gaussdb::GaussDBConnection;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let database_url = "gaussdb://username:password@localhost:5432/database_name";
//!     let mut connection = GaussDBConnection::establish(&database_url)?;
//!
//!     // Use Diesel as normal
//!     // ...
//!
//!     Ok(())
//! }
//! ```

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod backend;
pub mod connection;
pub mod metadata_lookup;
pub mod query_builder;
pub mod types;
pub mod gaussdb_extensions;
pub mod expression;
pub mod serialize;
pub mod value;

// Re-export core types
pub use backend::GaussDB;
pub use connection::GaussDBConnection;
pub use query_builder::GaussDBQueryBuilder;

// Re-export commonly used types from diesel
pub use diesel::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder_advanced_features() {
        use crate::query_builder::{on_constraint, ConflictTarget};
        use crate::query_builder::distinct_on::DistinctOnClause;
        use crate::query_builder::limit_offset::LimitOffsetClause;

        // Test ON CONSTRAINT functionality
        let constraint = on_constraint("users_email_unique");
        assert_eq!(constraint.constraint_name(), "users_email_unique");

        let target = ConflictTarget::new(constraint);
        assert_eq!(target.0.constraint_name(), "users_email_unique");

        // Test DISTINCT ON clause
        let distinct_clause = DistinctOnClause::new("user_id");
        // Test that the clause was created successfully
        assert!(format!("{:?}", distinct_clause).contains("user_id"));

        // Test LIMIT OFFSET clause
        let limit_offset = LimitOffsetClause::limit_and_offset(10i64, 20i64);
        // Test that the clause was created successfully
        assert!(format!("{:?}", limit_offset).contains("10"));
        assert!(format!("{:?}", limit_offset).contains("20"));
    }

    #[test]
    fn test_sql_generation_advanced() {
        use crate::query_builder::GaussDBQueryBuilder;
        use diesel::query_builder::QueryBuilder;

        let mut builder = GaussDBQueryBuilder::new();

        // Test complex query building
        builder.push_sql("SELECT DISTINCT ON (");
        builder.push_identifier("user_id").unwrap();
        builder.push_sql(") ");
        builder.push_identifier("user_id").unwrap();
        builder.push_sql(", ");
        builder.push_identifier("name").unwrap();
        builder.push_sql(" FROM ");
        builder.push_identifier("users").unwrap();
        builder.push_sql(" WHERE ");
        builder.push_identifier("active").unwrap();
        builder.push_sql(" = ");
        builder.push_bind_param();
        builder.push_sql(" ORDER BY ");
        builder.push_identifier("user_id").unwrap();
        builder.push_sql(", ");
        builder.push_identifier("created_at").unwrap();
        builder.push_sql(" DESC LIMIT ");
        builder.push_bind_param();
        builder.push_sql(" OFFSET ");
        builder.push_bind_param();

        let expected = "SELECT DISTINCT ON (\"user_id\") \"user_id\", \"name\" FROM \"users\" WHERE \"active\" = $1 ORDER BY \"user_id\", \"created_at\" DESC LIMIT $2 OFFSET $3";
        assert_eq!(builder.sql(), expected);
        assert_eq!(builder.bind_idx(), 3);
    }

    #[test]
    fn test_type_metadata() {
        use crate::metadata_lookup::{GaussDBMetadataCache, GaussDBMetadataCacheKey};
        use std::borrow::Cow;

        let mut cache = GaussDBMetadataCache::new();
        let key = GaussDBMetadataCacheKey::new(None, Cow::Borrowed("text"));

        // Test that we can store and retrieve metadata
        assert!(cache.lookup_type(&key).is_none());

        // Store some metadata
        cache.store_type(key.clone(), (25, 1009));

        // Verify we can retrieve it
        let metadata = cache.lookup_type(&key);
        assert!(metadata.is_some());
    }
}
