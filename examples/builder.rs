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
