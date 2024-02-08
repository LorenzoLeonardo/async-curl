use async_curl::{actor::CurlActor, curl::AsyncCurl};
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
