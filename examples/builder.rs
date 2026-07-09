use std::fs::{File, OpenOptions};
use std::io::Write as _;
use std::path::Path;
use std::sync::Arc;

use async_curl::{
    dep::curl::easy::{Handler, WriteError},
    AsyncCurl, CurlActor,
};
use http::HeaderMap;

#[derive(Clone, Debug, Default)]
pub struct ResponseHandler {
    header: Option<HeaderMap>,
    body: Option<Vec<u8>>,
    status: Option<u16>,
    output_file: Option<Arc<File>>,
}

impl Handler for ResponseHandler {
    /// This will store the response from the server
    /// to the data vector.
    fn write(&mut self, stream: &[u8]) -> Result<usize, WriteError> {
        if let Some(file) = self.output_file.as_mut() {
            let _ = file.write_all(stream);
        }

        if let Some(ref mut data) = self.body {
            data.extend_from_slice(stream);
        }
        Ok(stream.len())
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

        self.output_file = Some(Arc::new(file));
        Ok(self)
    }

    pub fn output_file_only<P: AsRef<Path>>(mut self, path: P) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;

        self.output_file = Some(Arc::new(file));
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

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let actor = CurlActor::new();

    body_only(actor.clone()).await?;
    download_file_only(actor.clone()).await?;
    download_file_with_body(actor).await?;
    Ok(())
}
