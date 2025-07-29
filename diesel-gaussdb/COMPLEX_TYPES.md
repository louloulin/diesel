# Complex Types Support in diesel-gaussdb

This document describes the complex types support implemented in the diesel-gaussdb crate.

## Overview

The diesel-gaussdb crate now supports complex PostgreSQL-compatible types that are available in GaussDB:

- **Array Types**: Support for one-dimensional arrays of basic types
- **Range Types**: Planned for future implementation

## Array Types

### Supported Features

1. **Type System Integration**
   - `HasSqlType<Array<T>>` implementation for the GaussDB backend
   - Proper type metadata handling for array types

2. **Deserialization (FromSql)**
   - Support for empty arrays
   - Support for arrays with elements
   - Proper handling of NULL elements (skipped in current implementation)
   - Error handling for malformed data and multi-dimensional arrays

3. **PostgreSQL Binary Format Compatibility**
   - Follows PostgreSQL array binary format specification
   - Compatible with GaussDB's PostgreSQL-compatible protocol

### Usage Examples

```rust
use diesel::prelude::*;
use diesel::sql_types::{Array, Integer, Text};
use diesel_gaussdb::GaussDB;

// Define a table with array columns
table! {
    my_table (id) {
        id -> Integer,
        numbers -> Array<Integer>,
        tags -> Array<Text>,
    }
}

// Query arrays from the database
let results: Vec<(i32, Vec<i32>, Vec<String>)> = my_table
    .select((id, numbers, tags))
    .load::<(i32, Vec<i32>, Vec<String>)>(&connection)?;
```

### Current Limitations

1. **Multi-dimensional Arrays**: Not supported (returns error)
2. **NULL Elements**: Currently skipped rather than represented as `Option<T>`
3. **Serialization (ToSql)**: Not yet implemented
4. **Complex Element Types**: Only basic types are tested

### Array Format Details

The implementation follows PostgreSQL's binary array format:

```
Header:
- num_dimensions (4 bytes): Number of array dimensions
- flags (4 bytes): Bit flags (0x1 = has nulls)
- element_type_oid (4 bytes): OID of the element type

For each dimension:
- num_elements (4 bytes): Number of elements in this dimension
- lower_bound (4 bytes): Lower bound index (usually 1)

For each element:
- element_size (4 bytes): Size of element data (-1 for NULL)
- element_data (variable): Binary representation of the element
```

## Testing

Comprehensive tests are provided in:

- `src/types/array.rs` - Unit tests for array type implementation
- `tests/complex_types_test.rs` - Integration tests for complex types

### Test Coverage

- Empty array deserialization
- Simple array with elements
- Arrays with NULL elements
- Error handling for malformed data
- Multi-dimensional array rejection
- Boundary conditions

## Future Enhancements

### Planned Features

1. **ToSql Implementation**: Support for serializing arrays to the database
2. **NULL Element Support**: Proper `Vec<Option<T>>` support
3. **Multi-dimensional Arrays**: Support for 2D and higher dimensional arrays
4. **Range Types**: Complete implementation of PostgreSQL range types
5. **Complex Element Types**: Support for arrays of custom types

### Range Types (Planned)

Range types will support:
- Integer ranges (`int4range`, `int8range`)
- Numeric ranges (`numrange`)
- Timestamp ranges (`tsrange`, `tstzrange`)
- Date ranges (`daterange`)

## Implementation Notes

### Design Decisions

1. **Conservative Approach**: The implementation prioritizes correctness and safety over feature completeness
2. **PostgreSQL Compatibility**: Follows PostgreSQL binary protocol specifications
3. **Error Handling**: Comprehensive error checking for malformed data
4. **Type Safety**: Leverages Rust's type system for compile-time safety

### Performance Considerations

1. **Zero-copy Deserialization**: Where possible, avoids unnecessary data copying
2. **Efficient Parsing**: Uses `byteorder` crate for efficient binary parsing
3. **Memory Management**: Proper handling of variable-length data

## Contributing

When contributing to complex types support:

1. **Follow PostgreSQL Specifications**: Ensure compatibility with PostgreSQL binary formats
2. **Add Comprehensive Tests**: Include both positive and negative test cases
3. **Document Limitations**: Clearly document any current limitations
4. **Consider Performance**: Optimize for common use cases
5. **Maintain Type Safety**: Leverage Rust's type system for safety

## References

- [PostgreSQL Array Types Documentation](https://www.postgresql.org/docs/current/arrays.html)
- [PostgreSQL Binary Protocol](https://www.postgresql.org/docs/current/protocol-message-formats.html)
- [GaussDB Documentation](https://docs.opengauss.org/)
- [Diesel Type System](https://docs.diesel.rs/diesel/sql_types/index.html)
