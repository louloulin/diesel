//! Array type support for GaussDB
//!
//! This module provides support for PostgreSQL-style array types,
//! which are also supported by GaussDB.

use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use std::fmt;
use std::io::Write;

use crate::backend::{GaussDB, GaussDBTypeMetadata};
use crate::value::GaussDBValue;
use diesel::deserialize::{self, FromSql};
use diesel::query_builder::bind_collector::ByteWrapper;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::{Array, HasSqlType, Nullable};

/// Implement HasSqlType for Array types
impl<T> HasSqlType<Array<T>> for GaussDB
where
    GaussDB: HasSqlType<T>,
{
    fn metadata(lookup: &mut Self::MetadataLookup) -> GaussDBTypeMetadata {
        // Get the base type metadata and use its array OID
        let base_metadata = <GaussDB as HasSqlType<T>>::metadata(lookup);
        match base_metadata.array_oid() {
            Ok(array_oid) => GaussDBTypeMetadata::new(array_oid, 0),
            Err(_) => GaussDBTypeMetadata::from_result(Err(
                crate::backend::FailedToLookupTypeError::new("Failed to lookup array type")
            )),
        }
    }
}

/// Implement FromSql for Vec<T> from Array<ST>
impl<T, ST> FromSql<Array<ST>, GaussDB> for Vec<T>
where
    T: FromSql<ST, GaussDB>,
{
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("Array value is null")?;
        let mut bytes = bytes;
        
        let num_dimensions = bytes.read_i32::<NetworkEndian>()?;
        let has_null = bytes.read_i32::<NetworkEndian>()? != 0;
        let _oid = bytes.read_i32::<NetworkEndian>()?;

        if num_dimensions == 0 {
            return Ok(Vec::new());
        }

        let num_elements = bytes.read_i32::<NetworkEndian>()?;
        let _lower_bound = bytes.read_i32::<NetworkEndian>()?;

        if num_dimensions != 1 {
            return Err("multi-dimensional arrays are not supported".into());
        }

        (0..num_elements)
            .map(|_| {
                let elem_size = bytes.read_i32::<NetworkEndian>()?;
                if has_null && elem_size == -1 {
                    T::from_nullable_sql(None)
                } else {
                    let (elem_bytes, new_bytes) = bytes.split_at(elem_size.try_into()?);
                    bytes = new_bytes;
                    T::from_sql(GaussDBValue::new(Some(elem_bytes), value.type_oid()))
                }
            })
            .collect()
    }
}

use diesel::expression::bound::Bound;
use diesel::expression::AsExpression;

macro_rules! array_as_expression {
    ($ty:ty, $sql_type:ty) => {
        // this simplifies the macro implementation
        // as some macro calls use this lifetime
        #[allow(clippy::extra_unused_lifetimes)]
        impl<'a, 'b, ST: 'static, T> AsExpression<$sql_type> for $ty {
            type Expression = Bound<$sql_type, Self>;

            fn as_expression(self) -> Self::Expression {
                Bound::new(self)
            }
        }
    };
}

array_as_expression!(&'a [T], Array<ST>);
array_as_expression!(&'a [T], Nullable<Array<ST>>);
array_as_expression!(&'a &'b [T], Array<ST>);
array_as_expression!(&'a &'b [T], Nullable<Array<ST>>);
array_as_expression!(Vec<T>, Array<ST>);
array_as_expression!(Vec<T>, Nullable<Array<ST>>);
array_as_expression!(&'a Vec<T>, Array<ST>);
array_as_expression!(&'a Vec<T>, Nullable<Array<ST>>);
array_as_expression!(&'a &'b Vec<T>, Array<ST>);
array_as_expression!(&'a &'b Vec<T>, Nullable<Array<ST>>);

/// Implement ToSql for slice types to Array<ST>
impl<ST, T> ToSql<Array<ST>, GaussDB> for [T]
where
    GaussDB: HasSqlType<ST>,
    T: ToSql<ST, GaussDB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        let num_dimensions = 1;
        out.write_i32::<NetworkEndian>(num_dimensions)?;
        let flags = 0;
        out.write_i32::<NetworkEndian>(flags)?;
        
        // Write element type OID (we'll use a placeholder for now)
        let element_type_oid = 0u32; // This should be the actual element type OID
        out.write_i32::<NetworkEndian>(element_type_oid as i32)?;
        
        // Write array dimensions
        out.write_i32::<NetworkEndian>(self.len() as i32)?;
        out.write_i32::<NetworkEndian>(1)?; // lower bound
        
        // Write elements
        for element in self {
            let mut element_bytes = Vec::new();
            let mut element_out = Output::test(ByteWrapper(&mut element_bytes));
            element.to_sql(&mut element_out)?;
            
            out.write_i32::<NetworkEndian>(element_bytes.len() as i32)?;
            out.write_all(&element_bytes)?;
        }
        
        Ok(IsNull::No)
    }
}

/// Implement ToSql for Vec<T> to Array<ST>
impl<ST, T> ToSql<Array<ST>, GaussDB> for Vec<T>
where
    GaussDB: HasSqlType<ST>,
    T: ToSql<ST, GaussDB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        self.as_slice().to_sql(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::sql_types::{Array, Integer, Text};

    #[test]
    fn test_array_has_sql_type() {
        // This is a compile-time test to ensure the trait is implemented
        fn test_array_type<T>() 
        where 
            GaussDB: HasSqlType<Array<T>>,
            GaussDB: HasSqlType<T>,
        {
            // This function should compile if HasSqlType is properly implemented
        }
        
        test_array_type::<Integer>();
        test_array_type::<Text>();
    }

    #[test]
    fn test_array_serialization_empty() {
        let empty_vec: Vec<i32> = Vec::new();
        let mut buffer = Vec::new();
        let mut output = Output::test(ByteWrapper(&mut buffer));
        
        // This should not panic
        let result = empty_vec.to_sql(&mut output);
        assert!(result.is_ok());
    }

    #[test]
    fn test_array_serialization_with_elements() {
        let vec = vec![1i32, 2i32, 3i32];
        let mut buffer = Vec::new();
        let mut output = Output::test(ByteWrapper(&mut buffer));
        
        // This should not panic
        let result = vec.to_sql(&mut output);
        assert!(result.is_ok());
        
        // Buffer should contain some data
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_slice_serialization() {
        let slice = &[1i32, 2i32, 3i32];
        let mut buffer = Vec::new();
        let mut output = Output::test(ByteWrapper(&mut buffer));
        
        // This should not panic
        let result = slice.to_sql(&mut output);
        assert!(result.is_ok());
        
        // Buffer should contain some data
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_array_deserialization_empty() {
        // Create empty array bytes (PostgreSQL format)
        let mut bytes = Vec::new();
        bytes.write_i32::<NetworkEndian>(0).unwrap(); // num_dimensions = 0
        bytes.write_i32::<NetworkEndian>(0).unwrap(); // flags
        bytes.write_i32::<NetworkEndian>(23).unwrap(); // element type OID (int4)
        
        let value = GaussDBValue::new(Some(&bytes), 1007); // int4 array OID
        let result: Result<Vec<i32>, _> = Vec::from_sql(value);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Vec::<i32>::new());
    }

    #[test]
    fn test_array_deserialization_error_multidimensional() {
        // Create multi-dimensional array bytes
        let mut bytes = Vec::new();
        bytes.write_i32::<NetworkEndian>(2).unwrap(); // num_dimensions = 2
        bytes.write_i32::<NetworkEndian>(0).unwrap(); // flags
        bytes.write_i32::<NetworkEndian>(23).unwrap(); // element type OID
        bytes.write_i32::<NetworkEndian>(4).unwrap(); // num_elements
        bytes.write_i32::<NetworkEndian>(1).unwrap(); // lower_bound
        
        let value = GaussDBValue::new(Some(&bytes), 1007);
        let result: Result<Vec<i32>, _> = Vec::from_sql(value);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("multi-dimensional"));
    }
}
