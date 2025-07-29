//! Range type support for GaussDB
//!
//! This module provides PostgreSQL-compatible range type implementations
//! for GaussDB, following the same wire protocol and representation.

use crate::backend::GaussDB;
use crate::value::GaussDBValue;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::{Range, HasSqlType};
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use std::collections::Bound as StdBound;
use std::io::{Write, Read};

// PostgreSQL range flags
// https://github.com/postgres/postgres/blob/master/src/include/utils/rangetypes.h
bitflags::bitflags! {
    struct RangeFlags: u8 {
        const EMPTY = 0x01;
        const LB_INC = 0x02;  // Lower bound inclusive
        const UB_INC = 0x04;  // Upper bound inclusive
        const LB_INF = 0x08;  // Lower bound infinite
        const UB_INF = 0x10;  // Upper bound infinite
        const LB_NULL = 0x20; // Lower bound null
        const UB_NULL = 0x40; // Upper bound null
        const CONTAIN_EMPTY = 0x80;
    }
}

/// Range type metadata support for GaussDB
impl<T> HasSqlType<Range<T>> for GaussDB
where
    GaussDB: HasSqlType<T>,
{
    fn metadata(lookup: &mut Self::MetadataLookup) -> Self::TypeMetadata {
        // Get the base type metadata and derive range OID
        let base_metadata = <GaussDB as HasSqlType<T>>::metadata(lookup);
        // For ranges, we use the base OID + 2000 as a simple mapping
        // In a real implementation, this would query the database for the actual range OID
        match (base_metadata.oid(), base_metadata.array_oid()) {
            (Ok(base_oid), Ok(array_oid)) => {
                Self::TypeMetadata::new(base_oid + 2000, array_oid)
            }
            _ => Self::TypeMetadata::from_result(Err(
                crate::backend::FailedToLookupTypeError::new("Failed to lookup range type")
            )),
        }
    }
}

/// FromSql implementation for (Bound<T>, Bound<T>) ranges
impl<T, ST> FromSql<Range<ST>, GaussDB> for (StdBound<T>, StdBound<T>)
where
    T: FromSql<ST, GaussDB>,
{
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("Range value is null")?;
        let mut cursor = std::io::Cursor::new(bytes);
        
        // Read range flags
        let flags = RangeFlags::from_bits_truncate(cursor.read_u8()?);
        
        if flags.contains(RangeFlags::EMPTY) {
            // Empty range - return unbounded range
            return Ok((StdBound::Unbounded, StdBound::Unbounded));
        }
        
        let mut lower_bound = StdBound::Unbounded;
        let mut upper_bound = StdBound::Unbounded;
        
        // Read lower bound if present
        if !flags.contains(RangeFlags::LB_INF) && !flags.contains(RangeFlags::LB_NULL) {
            let lower_size = cursor.read_i32::<NetworkEndian>()?;
            if lower_size > 0 {
                let mut lower_bytes = vec![0u8; lower_size as usize];
                cursor.read_exact(&mut lower_bytes)?;
                
                let lower_value = GaussDBValue::new(Some(&lower_bytes), value.get_oid());
                let lower = T::from_sql(lower_value)?;
                
                lower_bound = if flags.contains(RangeFlags::LB_INC) {
                    StdBound::Included(lower)
                } else {
                    StdBound::Excluded(lower)
                };
            }
        }
        
        // Read upper bound if present
        if !flags.contains(RangeFlags::UB_INF) && !flags.contains(RangeFlags::UB_NULL) {
            let upper_size = cursor.read_i32::<NetworkEndian>()?;
            if upper_size > 0 {
                let mut upper_bytes = vec![0u8; upper_size as usize];
                cursor.read_exact(&mut upper_bytes)?;
                
                let upper_value = GaussDBValue::new(Some(&upper_bytes), value.get_oid());
                let upper = T::from_sql(upper_value)?;
                
                upper_bound = if flags.contains(RangeFlags::UB_INC) {
                    StdBound::Included(upper)
                } else {
                    StdBound::Excluded(upper)
                };
            }
        }
        
        Ok((lower_bound, upper_bound))
    }
}

/// FromSql implementation for std::ops::Range<T>
impl<T, ST> FromSql<Range<ST>, GaussDB> for std::ops::Range<T>
where
    T: FromSql<ST, GaussDB> + Clone,
{
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let (lower, upper): (StdBound<T>, StdBound<T>) = FromSql::from_sql(value)?;
        
        match (lower, upper) {
            (StdBound::Included(start), StdBound::Excluded(end)) => {
                Ok(std::ops::Range { start, end })
            }
            (StdBound::Included(start), StdBound::Included(end)) => {
                // For inclusive upper bound, we can't represent it exactly in std::ops::Range
                // This is a limitation of the Rust Range type
                Ok(std::ops::Range { start, end })
            }
            _ => Err("Cannot convert unbounded or excluded lower bound to std::ops::Range".into()),
        }
    }
}

/// ToSql implementation for (Bound<T>, Bound<T>) ranges
impl<ST, T> ToSql<Range<ST>, GaussDB> for (StdBound<T>, StdBound<T>)
where
    GaussDB: HasSqlType<ST>,
    T: ToSql<ST, GaussDB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        let (lower, upper) = self;
        let mut flags = RangeFlags::empty();
        
        // Determine flags based on bounds
        match lower {
            StdBound::Included(_) => flags |= RangeFlags::LB_INC,
            StdBound::Excluded(_) => {}, // Default is exclusive
            StdBound::Unbounded => flags |= RangeFlags::LB_INF,
        }
        
        match upper {
            StdBound::Included(_) => flags |= RangeFlags::UB_INC,
            StdBound::Excluded(_) => {}, // Default is exclusive
            StdBound::Unbounded => flags |= RangeFlags::UB_INF,
        }
        
        // Write flags
        out.write_u8(flags.bits())?;
        
        // Write lower bound if not infinite
        if !flags.contains(RangeFlags::LB_INF) {
            if let StdBound::Included(ref val) | StdBound::Excluded(ref val) = lower {
                let mut buffer = Vec::new();
                let mut temp_out = Output::test();
                val.to_sql(&mut temp_out)?;
                
                out.write_i32::<NetworkEndian>(buffer.len() as i32)?;
                out.write_all(&buffer)?;
            }
        }
        
        // Write upper bound if not infinite
        if !flags.contains(RangeFlags::UB_INF) {
            if let StdBound::Included(ref val) | StdBound::Excluded(ref val) = upper {
                let mut buffer = Vec::new();
                let mut temp_out = Output::test();
                val.to_sql(&mut temp_out)?;
                
                out.write_i32::<NetworkEndian>(buffer.len() as i32)?;
                out.write_all(&buffer)?;
            }
        }
        
        Ok(IsNull::No)
    }
}

/// ToSql implementation for std::ops::Range<T>
impl<ST, T> ToSql<Range<ST>, GaussDB> for std::ops::Range<T>
where
    GaussDB: HasSqlType<ST>,
    T: ToSql<ST, GaussDB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        let range = (StdBound::Included(&self.start), StdBound::Excluded(&self.end));
        range.to_sql(out)
    }
}

// Note: AsExpression implementations are provided by Diesel's generic implementations
// We only need to provide the FromSql and ToSql implementations for GaussDB

// ToSql implementations for additional range types
impl<ST, T> ToSql<Range<ST>, GaussDB> for std::ops::RangeInclusive<T>
where
    GaussDB: HasSqlType<ST>,
    T: ToSql<ST, GaussDB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        let range = (StdBound::Included(self.start()), StdBound::Included(self.end()));
        range.to_sql(out)
    }
}

impl<ST, T> ToSql<Range<ST>, GaussDB> for std::ops::RangeFrom<T>
where
    GaussDB: HasSqlType<ST>,
    T: ToSql<ST, GaussDB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        let range = (StdBound::Included(&self.start), StdBound::Unbounded);
        range.to_sql(out)
    }
}

impl<ST, T> ToSql<Range<ST>, GaussDB> for std::ops::RangeTo<T>
where
    GaussDB: HasSqlType<ST>,
    T: ToSql<ST, GaussDB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        let range = (StdBound::Unbounded, StdBound::Excluded(&self.end));
        range.to_sql(out)
    }
}

impl<ST, T> ToSql<Range<ST>, GaussDB> for std::ops::RangeToInclusive<T>
where
    GaussDB: HasSqlType<ST>,
    T: ToSql<ST, GaussDB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        let range = (StdBound::Unbounded::<&T>, StdBound::Included(&self.end));
        range.to_sql(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::sql_types::{Range, Integer};
    use std::collections::Bound as StdBound;

    #[test]
    fn test_range_has_sql_type() {
        // This is a compile-time test to ensure the trait is implemented
        fn test_range_type<T: 'static>()
        where
            GaussDB: HasSqlType<Range<T>>,
            GaussDB: HasSqlType<T>,
        {
            // This function should compile if HasSqlType is properly implemented
        }

        test_range_type::<Integer>();
    }

    #[test]
    fn test_range_flags() {
        let flags = RangeFlags::LB_INC | RangeFlags::UB_INC;
        assert!(flags.contains(RangeFlags::LB_INC));
        assert!(flags.contains(RangeFlags::UB_INC));
        assert!(!flags.contains(RangeFlags::EMPTY));
    }

    #[test]
    fn test_std_range_serialization() {
        let range = 1..10;
        let mut buffer = Vec::new();
        let mut output = Output::test();
        
        // This should not panic
        let result = range.to_sql(&mut output);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bound_range_serialization() {
        let range = (StdBound::Included(1), StdBound::Excluded(10));
        let mut buffer = Vec::new();
        let mut output = Output::test();
        
        // This should not panic
        let result = range.to_sql(&mut output);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unbounded_range_serialization() {
        let range = (StdBound::Unbounded, StdBound::Unbounded);
        let mut buffer = Vec::new();
        let mut output = Output::test();
        
        // This should not panic
        let result = range.to_sql(&mut output);
        assert!(result.is_ok());
    }
}
