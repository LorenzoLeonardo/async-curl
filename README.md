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

## Using AsyncCurl builder using Easy2 Curl perform

```rust
use async_curl::{AsyncCurl, CurlActor};
use curl::easy::{Handler, WriteError};

#[derive(Debug, Clone, Default)]
pub struct ResponseHandler {
    data: Option<Vec<u8>>,
}

impl Handler for ResponseHandler {
    /// This will store the response from the server
    /// to the data vector.
    fn write(&mut self, stream: &[u8]) -> Result<usize, WriteError> {
        if self.data.is_none() {
            self.data = Some(stream.to_vec());
        } else {
            if let Some(ref mut data) = self.data {
                data.extend_from_slice(stream);
            }
        }
        Ok(stream.len())
    }
}

impl ResponseHandler {
    /// Instantiation of the ResponseHandler
    /// and initialize the data vector.
    pub fn new() -> Self {
        Self::default()
    }

    /// This will give the data to the receiving variable
    pub fn take(&mut self) -> Option<Vec<u8>> {
        self.data.take()
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let actor = CurlActor::new();
    let collector = ResponseHandler::new();

    let mut curl = AsyncCurl::new(actor, collector)
        .url("https://www.rust-lang.org/")
        .unwrap()
        .finalize()
        .perform()
        .await
        .unwrap();

    let body = curl.get_mut().take();
    let status = curl.response_code().unwrap() as u16;

    println!("Body: {:?}", body);
    println!("Status: {status}");
}
```

## Using AsyncCurl builder using Easy2 Curl-Multi perform

```rust
use async_curl::{AsyncCurl, CurlActor};
use curl::easy::{Handler, WriteError};

#[derive(Debug, Clone, Default)]
pub struct ResponseHandler {
    data: Option<Vec<u8>>,
}

impl Handler for ResponseHandler {
    /// This will store the response from the server
    /// to the data vector.
    fn write(&mut self, stream: &[u8]) -> Result<usize, WriteError> {
        if self.data.is_none() {
            self.data = Some(stream.to_vec());
        } else {
            if let Some(ref mut data) = self.data {
                data.extend_from_slice(stream);
            }
        }
        Ok(stream.len())
    }
}

impl ResponseHandler {
    /// Instantiation of the ResponseHandler
    /// and initialize the data vector.
    pub fn new() -> Self {
        Self::default()
    }

    /// This will give the data to the receiving variable
    pub fn take(&mut self) -> Option<Vec<u8>> {
        self.data.take()
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let actor = CurlActor::new();
    let collector = ResponseHandler::new();

    let mut curl = AsyncCurl::new(actor, collector)
        .url("https://www.rust-lang.org/")
        .unwrap()
        .finalize()
        .use_multi_transfer()
        .perform()
        .await
        .unwrap();

    let body = curl.get_mut().take();
    let status = curl.response_code().unwrap() as u16;

    println!("Body: {:?}", body);
    println!("Status: {status}");
}
```

## Supplying Tokio Runtime (current-thread)

```rust
use async_curl::{AsyncCurl, CurlActor};
use curl::easy::{Handler, WriteError};
use tokio::runtime::Builder;

#[derive(Debug, Clone, Default)]
pub struct ResponseHandler {
    data: Option<Vec<u8>>,
}

impl Handler for ResponseHandler {
    /// This will store the response from the server
    /// to the data vector.
    fn write(&mut self, stream: &[u8]) -> Result<usize, WriteError> {
        if self.data.is_none() {
            self.data = Some(stream.to_vec());
        } else {
            if let Some(ref mut data) = self.data {
                data.extend_from_slice(stream);
            }
        }
        Ok(stream.len())
    }
}

impl ResponseHandler {
    /// Instantiation of the ResponseHandler
    /// and initialize the data vector.
    pub fn new() -> Self {
        Self::default()
    }

    /// This will give the data to the receiving variable
    pub fn take(&mut self) -> Option<Vec<u8>> {
        self.data.take()
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let runtime = Builder::new_current_thread().enable_all().build().unwrap();
    let actor = CurlActor::new_runtime(runtime);
    let collector = ResponseHandler::new();

    let mut curl = AsyncCurl::new(actor, collector)
        .url("https://www.rust-lang.org/")
        .unwrap()
        .finalize()
        .perform()
        .await
        .unwrap();

    let body = curl.get_mut().take();
    let status = curl.response_code().unwrap() as u16;

    println!("Body: {:?}", body);
    println!("Status: {status}");
}
```

## Supplying Tokio Runtime (multi-thread)

```rust
use async_curl::{AsyncCurl, CurlActor};
use curl::easy::{Handler, WriteError};
use tokio::runtime::Builder;

#[derive(Debug, Clone, Default)]
pub struct ResponseHandler {
    data: Option<Vec<u8>>,
}

impl Handler for ResponseHandler {
    /// This will store the response from the server
    /// to the data vector.
    fn write(&mut self, stream: &[u8]) -> Result<usize, WriteError> {
        if self.data.is_none() {
            self.data = Some(stream.to_vec());
        } else {
            if let Some(ref mut data) = self.data {
                data.extend_from_slice(stream);
            }
        }
        Ok(stream.len())
    }
}

impl ResponseHandler {
    /// Instantiation of the ResponseHandler
    /// and initialize the data vector.
    pub fn new() -> Self {
        Self::default()
    }

    /// This will give the data to the receiving variable
    pub fn take(&mut self) -> Option<Vec<u8>> {
        self.data.take()
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let runtime = Builder::new_multi_thread().enable_all().build().unwrap();
    let actor = CurlActor::new_runtime(runtime);
    let collector = ResponseHandler::new();

    let mut curl = AsyncCurl::new(actor, collector)
        .url("https://www.rust-lang.org/")
        .unwrap()
        .finalize()
        .perform()
        .await
        .unwrap();

    let body = curl.get_mut().take();
    let status = curl.response_code().unwrap() as u16;

    println!("Body: {:?}", body);
    println!("Status: {status}");
}
```

## More Complex Examples

```rust
use std::fs::{File, OpenOptions};
use std::io::{Read, Write as _};
use std::path::Path;
use std::sync::{Arc, Mutex};

use async_curl::{
    dep::curl::easy::{Handler, ReadError, WriteError},
    AsyncCurl, CurlActor,
};
use http::HeaderMap;

#[derive(Clone, Debug, Default)]
pub struct ResponseHandler {
    header: Option<HeaderMap>,
    body: Option<Vec<u8>>,
    status: Option<u16>,
    output_file: Option<Arc<Mutex<File>>>,
    input_file: Option<Arc<Mutex<File>>>,
}

impl Handler for ResponseHandler {
    /// This will store the response from the server
    /// to the data vector.
    fn write(&mut self, stream: &[u8]) -> Result<usize, WriteError> {
        if let Some(file) = self.output_file.as_ref() {
            let mut guard = file.lock().map_err(|_| WriteError::Pause)?;
            guard.write_all(stream).map_err(|_| WriteError::Pause)?;
        }

        if let Some(ref mut data) = self.body {
            data.extend_from_slice(stream);
        }
        Ok(stream.len())
    }

    fn read(&mut self, data: &mut [u8]) -> Result<usize, ReadError> {
        if let Some(file) = self.input_file.as_ref() {
            let mut guard = file.lock().map_err(|_| ReadError::Abort)?;
            guard.read(data).map_err(|_| ReadError::Abort)
        } else {
            Ok(0)
        }
    }

    fn header(&mut self, data: &[u8]) -> bool {
        let line = match std::str::from_utf8(data) {
            Ok(line) => line.trim(),
            Err(_) => return true,
        };

        if line.is_empty() {
            return true;
        }

        if let Some(status_text) = line.strip_prefix("HTTP/") {
            if let Some(status_code) = status_text.split_whitespace().nth(1) {
                if let Ok(code) = status_code.parse::<u16>() {
                    self.status = Some(code);
                }
            }
            return true;
        }

        let Some((name, value)) = line.split_once(':') else {
            return true;
        };

        let header_name = match http::header::HeaderName::from_bytes(name.trim().as_bytes()) {
            Ok(name) => name,
            Err(_) => return true,
        };
        let header_value = match http::header::HeaderValue::from_bytes(value.trim().as_bytes()) {
            Ok(value) => value,
            Err(_) => return true,
        };

        let header_map = self.header.get_or_insert_with(HeaderMap::new);
        header_map.append(header_name, header_value);

        true
    }
}

impl ResponseHandler {
    /// Instantiation of the ResponseHandler
    /// and initialize the data vector.
    pub fn new() -> Self {
        Self {
            body: Some(Vec::new()),
            ..Self::default()
        }
    }

    /// Create a handler that writes the body stream directly to a file.
    pub fn with_output_file<P: AsRef<Path>>(mut self, path: P) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;

        self.output_file = Some(Arc::new(Mutex::new(file)));
        Ok(self)
    }

    /// Create a handler that will read upload data from a local file.
    pub fn with_input_file<P: AsRef<Path>>(mut self, path: P) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        self.input_file = Some(Arc::new(Mutex::new(file)));
        Ok(self)
    }

    pub fn output_file_only<P: AsRef<Path>>(mut self, path: P) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;
        self.output_file = Some(Arc::new(Mutex::new(file)));
        self.body = None;
        Ok(self)
    }

    /// This will build an http::Response from the collected headers, body, and status.
    pub fn take_response(&mut self) -> Result<http::Response<Vec<u8>>, http::Error> {
        let mut builder = http::Response::builder();

        if let Some(status) = self.status.take() {
            builder = builder.status(status);
        }

        if let Some(headers) = self.header.take() {
            for (name, value) in headers.iter() {
                builder = builder.header(name.clone(), value.clone());
            }
        }

        builder.body(self.body.take().unwrap_or_default())
    }
}

async fn download_file_only(
    actor: CurlActor<ResponseHandler>,
) -> Result<(), Box<dyn std::error::Error>> {
    let collector = ResponseHandler::new().output_file_only("downloaded1.html")?;

    let mut curl = AsyncCurl::new(actor, collector)
        .url("https://www.rust-lang.org/")?
        .finalize()
        .perform()
        .await?;

    let response = curl.get_mut().take_response()?;

    println!("Header: {:?}", response.headers());
    println!("Body: {:?}", response.body());
    println!("Status: {}", response.status());
    Ok(())
}

async fn download_file_with_body(
    actor: CurlActor<ResponseHandler>,
) -> Result<(), Box<dyn std::error::Error>> {
    let collector = ResponseHandler::new().with_output_file("downloaded2.html")?;

    let mut curl = AsyncCurl::new(actor, collector)
        .url("https://www.rust-lang.org/")?
        .finalize()
        .perform()
        .await?;

    let response = curl.get_mut().take_response()?;

    println!("Header: {:?}", response.headers());
    println!("Body: {:?}", response.body());
    println!("Status: {}", response.status());
    Ok(())
}

async fn body_only(actor: CurlActor<ResponseHandler>) -> Result<(), Box<dyn std::error::Error>> {
    let collector = ResponseHandler::new();

    let mut curl = AsyncCurl::new(actor, collector)
        .url("https://www.rust-lang.org/")?
        .finalize()
        .perform()
        .await?;

    let response = curl.get_mut().take_response()?;

    println!("Header: {:?}", response.headers());
    println!("Body: {:?}", response.body());
    println!("Status: {}", response.status());
    Ok(())
}

async fn upload_file(actor: CurlActor<ResponseHandler>) -> Result<(), Box<dyn std::error::Error>> {
    let path_ref = "<file to be uploaded>";
    let file_size = std::fs::metadata(path_ref)?.len();

    let collector = ResponseHandler::new().with_input_file(path_ref)?;

    let mut curl = AsyncCurl::new(actor, collector)
        .url("<upload url>")?
        .upload(true)?
        .in_filesize(file_size)?
        .finalize()
        .perform()
        .await?;

    let response = curl.get_mut().take_response()?;

    println!("Upload headers: {:?}", response.headers());
    println!("Upload body: {:?}", response.body());
    println!("Upload status: {}", response.status());

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let actor = CurlActor::new();

    let inner = actor.clone();
    let h1 = tokio::spawn(async move {
        let _ = body_only(inner).await;
    });

    let inner = actor.clone();
    let h2 = tokio::spawn(async move {
        let _ = download_file_only(inner).await;
    });

    let inner = actor.clone();
    let h3 = tokio::spawn(async move {
        let _ = download_file_with_body(inner).await;
    });

    let inner = actor.clone();
    let h4 = tokio::spawn(async move {
        let _ = upload_file(inner).await;
    });

    let _ = tokio::join!(h1, h2, h3, h4);
    Ok(())
}
```