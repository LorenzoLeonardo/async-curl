//! async-curl: An asynchronous implementation to perform curl operations with tokio.
//! This requires a dependency with the [curl](https://crates.io/crates/curl) and [tokio](https://crates.io/crates/tokio) crates
//!
pub mod actor;
pub mod error;
pub mod response_handler;
