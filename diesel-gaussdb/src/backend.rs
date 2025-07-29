//! The GaussDB backend implementation
//!
//! This module provides the core backend implementation for GaussDB,
//! which is PostgreSQL-compatible but with some GaussDB-specific features.

use diesel::backend::*;
use diesel::query_builder::bind_collector::RawBytesBindCollector;
use diesel::sql_types::{HasSqlType, TypeMetadata};

use crate::query_builder::GaussDBQueryBuilder;
use crate::types::GaussDBValue;

/// The GaussDB backend
///
/// This backend is PostgreSQL-compatible and supports GaussDB-specific
/// authentication methods and data types.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
pub struct GaussDB;

/// Type metadata for GaussDB types
///
/// Since GaussDB is PostgreSQL-compatible, we use similar metadata structure
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct GaussDBTypeMetadata {
    /// The OID of the type
    pub oid: u32,
    /// The OID of the array type
    pub array_oid: u32,
}

impl GaussDBTypeMetadata {
    /// Create new type metadata
    pub fn new(oid: u32, array_oid: u32) -> Self {
        Self { oid, array_oid }
    }

    /// Get the OID of this type
    pub fn oid(&self) -> u32 {
        self.oid
    }

    /// Get the array OID of this type
    pub fn array_oid(&self) -> u32 {
        self.array_oid
    }
}

impl From<(u32, u32)> for GaussDBTypeMetadata {
    fn from((oid, array_oid): (u32, u32)) -> Self {
        Self::new(oid, array_oid)
    }
}

impl Backend for GaussDB {
    type QueryBuilder = GaussDBQueryBuilder;
    type RawValue<'a> = GaussDBValue<'a>;
    type BindCollector<'a> = RawBytesBindCollector<GaussDB>;
}

impl TypeMetadata for GaussDB {
    type TypeMetadata = GaussDBTypeMetadata;
    type MetadataLookup = dyn GaussDBMetadataLookup;
}

/// Trait for looking up type metadata in GaussDB
pub trait GaussDBMetadataLookup {
    /// Look up metadata for a type
    fn lookup_type(&mut self, type_name: &str) -> Option<GaussDBTypeMetadata>;
}

impl SqlDialect for GaussDB {
    type ReturningClause = GaussDBReturningClause;
    type OnConflictClause = GaussDBOnConflictClause;
    type InsertWithDefaultKeyword = sql_dialect::default_keyword_for_insert::IsoSqlDefaultKeyword;
    type BatchInsertSupport = sql_dialect::batch_insert_support::PostgresLikeBatchInsertSupport;
    type ConcatClause = sql_dialect::concat_clause::ConcatWithPipesClause;
    type DefaultValueClauseForInsert = sql_dialect::default_value_clause::AnsiDefaultValueClause;
    type EmptyFromClauseSyntax = sql_dialect::from_clause_syntax::AnsiSqlFromClauseSyntax;
    type SelectStatementSyntax = sql_dialect::select_statement_syntax::AnsiSqlSelectStatement;
    type ExistsSyntax = sql_dialect::exists_syntax::AnsiSqlExistsSyntax;
    type ArrayComparison = sql_dialect::array_comparison::AnsiSqlArrayComparison;
    type AliasSyntax = sql_dialect::alias_syntax::AsAliasSyntax;
    type WindowFrameClauseGroupSupport = sql_dialect::window_frame_clause_group_support::IsoGroupWindowFrameUnit;
    type WindowFrameExclusionSupport = sql_dialect::window_frame_exclusion_support::FrameExclusionSupport;
    type AggregateFunctionExpressions = sql_dialect::aggregate_function_expressions::PostgresLikeAggregateFunctionExpressions;
    type BuiltInWindowFunctionRequireOrder = sql_dialect::built_in_window_function_require_order::NoOrderRequired;
}

// Basic SQL type support - we'll implement these similar to PostgreSQL
impl HasSqlType<diesel::sql_types::SmallInt> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(21, 1005) // smallint, _int2
    }
}

impl HasSqlType<diesel::sql_types::Integer> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(23, 1007) // int4, _int4
    }
}

impl HasSqlType<diesel::sql_types::BigInt> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(20, 1016) // int8, _int8
    }
}

impl HasSqlType<diesel::sql_types::Float> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(700, 1021) // float4, _float4
    }
}

impl HasSqlType<diesel::sql_types::Double> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(701, 1022) // float8, _float8
    }
}

impl HasSqlType<diesel::sql_types::Text> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(25, 1009) // text, _text
    }
}

impl HasSqlType<diesel::sql_types::Binary> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(17, 1001) // bytea, _bytea
    }
}

impl HasSqlType<diesel::sql_types::Date> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(1082, 1182) // date, _date
    }
}

impl HasSqlType<diesel::sql_types::Time> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(1083, 1183) // time, _time
    }
}

impl HasSqlType<diesel::sql_types::Timestamp> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(1114, 1115) // timestamp, _timestamp
    }
}

impl HasSqlType<diesel::sql_types::Bool> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(16, 1000) // bool, _bool
    }
}

impl DieselReserveSpecialization for GaussDB {}
impl TrustedBackend for GaussDB {}

// GaussDB-specific types for SQL dialect
/// GaussDB-specific ON CONFLICT clause support
#[derive(Debug, Copy, Clone)]
pub struct GaussDBOnConflictClause;

impl sql_dialect::on_conflict_clause::SupportsOnConflictClause for GaussDBOnConflictClause {}

/// GaussDB-specific RETURNING clause support
#[derive(Debug, Copy, Clone)]
pub struct GaussDBReturningClause;

impl sql_dialect::returning_clause::SupportsReturningClause for GaussDBReturningClause {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaussdb_backend_creation() {
        let backend = GaussDB::default();
        assert_eq!(backend, GaussDB);
    }

    #[test]
    fn test_type_metadata() {
        let metadata = GaussDBTypeMetadata::new(23, 1007);
        assert_eq!(metadata.oid(), 23);
        assert_eq!(metadata.array_oid(), 1007);
    }

    #[test]
    fn test_type_metadata_from_tuple() {
        let metadata: GaussDBTypeMetadata = (23, 1007).into();
        assert_eq!(metadata.oid(), 23);
        assert_eq!(metadata.array_oid(), 1007);
    }
}
