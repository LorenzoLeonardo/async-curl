//! async-curl: An asynchronous implementation to perform curl operations with tokio.
//! This requires a dependency with the [curl](https://crates.io/crates/curl) and [tokio](https://crates.io/crates/tokio) crates
//!
//! ## perform Curl Easy2 asynchronously
//! ```rust
//! pub mod actor;
//! pub mod error;
//!
//! use async_curl::actor::CurlActor;
//! use curl::easy::{Easy2, Handler, WriteError};
//!
//! #[derive(Debug, Clone, Default)]
//! pub struct ResponseHandler {
//!     data: Vec<u8>,
//! }
//!
//! impl Handler for ResponseHandler {
//!     /// This will store the response from the server
//!     /// to the data vector.
//!     fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
//!         self.data.extend_from_slice(data);
//!         Ok(data.len())
//!     }
//! }
//!
//! impl ResponseHandler {
//!     /// Instantiation of the ResponseHandler
//!     /// and initialize the data vector.
//!     pub fn new() -> Self {
//!         Self::default()
//!     }
//!
//!     /// This will consumed the object and
//!     /// give the data to the caller
//!     pub fn get_data(self) -> Vec<u8> {
//!         self.data
//!     }
//! }
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() {
//!     let actor = CurlActor::new();
//!
//!     let mut easy2 = Easy2::new(ResponseHandler::new());
//!     easy2.url("https://www.rust-lang.org/").unwrap();
//!     easy2.get(true).unwrap();
//!
//!     let actor1 = actor.clone();
//!     let spawn1 = tokio::spawn(async move {
//!     let mut result = actor1.send_request(easy2).await.unwrap();
//!
//!         let response = result.get_ref().to_owned().get_data();
//!         let status = result.response_code().unwrap();
//!
//!         println!("Response: {:?}", response);
//!         println!("Status: {:?}", status);
//!     });
//!
//!     let mut easy2 = Easy2::new(ResponseHandler::new());
//!     easy2.url("https://www.rust-lang.org/").unwrap();
//!     easy2.get(true).unwrap();
//!
//!     let spawn2 = tokio::spawn(async move {
//!         let mut result = actor.send_request(easy2).await.unwrap();
//!
//!         let response = result.get_ref().to_owned().get_data();
//!         let status = result.response_code().unwrap();
//!
//!         println!("Response: {:?}", response);
//!         println!("Status: {:?}", status);
//!     });
//!
//!     let (_, _) = tokio::join!(spawn1, spawn2);
//! }
//! ```
pub mod actor;
pub mod error;
#[cfg(test)]
mod tests;
