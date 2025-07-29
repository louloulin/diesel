# Diesel-GaussDB

A GaussDB backend for the Diesel ORM framework.

## Overview

Diesel-GaussDB provides GaussDB database support for the Diesel ORM, enabling Rust applications to work with GaussDB and OpenGauss databases using Diesel's type-safe query builder.

## Features

- **GaussDB Compatibility**: Full support for GaussDB and OpenGauss databases
- **PostgreSQL Protocol**: Leverages PostgreSQL compatibility for maximum feature support
- **Type Safety**: Compile-time verified queries with Diesel's type system
- **Authentication**: Support for GaussDB's SHA256 and MD5_SHA256 authentication methods
- **Async Support**: Optional async/await support with tokio-postgres
- **Connection Pooling**: Compatible with Diesel's r2d2 connection pooling

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
diesel = { version = "2.2", features = ["postgres"] }
diesel-gaussdb = "0.1.0-alpha"
```

### Basic Usage

```rust
use diesel::prelude::*;
use diesel_gaussdb::GaussDBConnection;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = "gaussdb://username:password@localhost:5432/database_name";
    let mut connection = GaussDBConnection::establish(&database_url)?;
    
    // Use Diesel as normal
    // ...
    
    Ok(())
}
```

### Async Usage

```rust
use diesel_async::prelude::*;
use diesel_gaussdb::AsyncGaussDBConnection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = "gaussdb://username:password@localhost:5432/database_name";
    let mut connection = AsyncGaussDBConnection::establish(&database_url).await?;
    
    // Use Diesel async as normal
    // ...
    
    Ok(())
}
```

## Supported Features

- [x] Basic CRUD operations
- [x] Transactions
- [x] Connection pooling
- [x] Type-safe queries
- [ ] GaussDB-specific data types
- [ ] GaussDB-specific functions
- [ ] Advanced authentication methods

## Development Status

This project is in **alpha** stage. The API may change and some features are still being implemented.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
