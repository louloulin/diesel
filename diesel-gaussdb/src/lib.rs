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
pub mod query_builder;
pub mod types;
pub mod expression;
pub mod serialize;

// Re-export core types
pub use backend::GaussDB;
pub use connection::GaussDBConnection;
pub use query_builder::GaussDBQueryBuilder;

// Re-export commonly used types from diesel
pub use diesel::prelude::*;
