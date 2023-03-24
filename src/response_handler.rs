use curl::easy::Handler;
use curl::easy::WriteError;
use std::fmt::Debug;

/// A handler of Easy2
/// ```
/// use curl::easy::Easy2;
/// use async_curl::response_handler::ResponseHandler;
///
/// # fn main() {
/// let easy2 = Easy2::new(ResponseHandler::new());
///
/// println!("{:?}", easy2.get_ref());
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct ResponseHandler {
    data: Vec<u8>,
}

impl Handler for ResponseHandler {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.data.extend_from_slice(data);
        Ok(data.len())
    }
}

impl ResponseHandler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_data(self) -> Vec<u8> {
        self.data
    }
}
