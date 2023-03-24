use curl::easy::Handler;
use curl::easy::WriteError;
use std::fmt::Debug;

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
