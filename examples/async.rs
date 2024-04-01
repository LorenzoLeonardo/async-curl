use async_curl::{Actor, CurlActor};
use curl::easy::{Easy2, Handler, WriteError};

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

    let mut easy2 = Easy2::new(ResponseHandler::new());
    easy2.url("https://www.rust-lang.org/").unwrap();
    easy2.get(true).unwrap();

    let actor1 = actor.clone();
    let spawn1 = tokio::spawn(async move {
        let mut result = actor1.send_request(easy2).await.unwrap();

        let response = result.get_mut().take();
        let status = result.response_code().unwrap();

        println!("Response: {:?}", response);
        println!("Status: {:?}", status);
    });

    let mut easy2 = Easy2::new(ResponseHandler::new());
    easy2.url("https://www.rust-lang.org/").unwrap();
    easy2.get(true).unwrap();

    let spawn2 = tokio::spawn(async move {
        let mut result = actor.send_request(easy2).await.unwrap();

        let response = result.get_mut().take();
        let status = result.response_code().unwrap();

        println!("Response: {:?}", response);
        println!("Status: {:?}", status);
    });

    let (_, _) = tokio::join!(spawn1, spawn2);
}
