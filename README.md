# async-curl

This will perform curl Easy2 asynchronously for rust-lang via an Actor using tokio

[![Latest Version](https://img.shields.io/crates/v/async-curl.svg)](https://crates.io/crates/async-curl)
[![License](https://img.shields.io/github/license/LorenzoLeonardo/async-curl.svg)](LICENSE)
[![Documentation](https://docs.rs/async-curl/badge.svg)](https://docs.rs/async-curl)
[![Build Status](https://github.com/LorenzoLeonardo/async-curl/workflows/Rust/badge.svg)](https://github.com/LorenzoLeonardo/async-curl/actions)
[![Downloads](https://img.shields.io/crates/d/async-curl)](https://crates.io/crates/async-curl)

## How to use with multiple async request

```rust
use async_curl::{Actor, CurlActor};
use curl::easy::{Easy2, Handler, WriteError};

#[derive(Debug, Clone, Default)]
pub struct ResponseHandler {
    data: Vec<u8>,
}

impl Handler for ResponseHandler {
    /// This will store the response from the server
    /// to the data vector.
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.data.extend_from_slice(data);
        Ok(data.len())
    }
}

impl ResponseHandler {
    /// Instantiation of the ResponseHandler
    /// and initialize the data vector.
    pub fn new() -> Self {
        Self::default()
    }

    /// This will consumed the object and
    /// give the data to the caller
    pub fn get_data(self) -> Vec<u8> {
        self.data
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let actor = CurlActor::new();

    let mut easy2 = Easy2::new(ResponseHandler::new());
    easy2.url("https://www.rust-lang.org/").unwrap();
    easy2.get(true).unwrap();

    let actor1 = actor.clone();
    let spawn1 = tokio::spawn(async move {
        let mut result = actor1.send_request(easy2).await.unwrap();

        let response = result.get_ref().to_owned().get_data();
        let status = result.response_code().unwrap();

        println!("Response: {:?}", response);
        println!("Status: {:?}", status);
    });

    let mut easy2 = Easy2::new(ResponseHandler::new());
    easy2.url("https://www.rust-lang.org/").unwrap();
    easy2.get(true).unwrap();

    let spawn2 = tokio::spawn(async move {
        let mut result = actor.send_request(easy2).await.unwrap();

        let response = result.get_ref().to_owned().get_data();
        let status = result.response_code().unwrap();

        println!("Response: {:?}", response);
        println!("Status: {:?}", status);
    });

    let (_, _) = tokio::join!(spawn1, spawn2);
}
```
