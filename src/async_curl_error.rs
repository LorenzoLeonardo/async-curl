use crate::async_curl;
use curl::{easy::Handler, MultiError};
use std::{
    error,
    fmt::{self, Debug},
};
use tokio::sync::{mpsc::error::SendError, oneshot::error::RecvError};

#[derive(Debug)]
pub struct AsyncCurlError(pub String);

/// This convert MultiError to our customized
/// AsyncCurlError for ease of management of
/// different errors from 3rd party crates.
impl From<MultiError> for AsyncCurlError {
    fn from(err: MultiError) -> Self {
        AsyncCurlError(format!("{:?}", err))
    }
}

/// This convert RecvError to our customized
/// AsyncCurlError for ease of management of
/// different errors from 3rd party crates.
impl From<RecvError> for AsyncCurlError {
    fn from(err: RecvError) -> Self {
        AsyncCurlError(format!("{:?}", err))
    }
}

/// This convert SendError to our customized
/// AsyncCurlError for ease of management of
/// different errors from 3rd party crates.
impl<H> From<SendError<async_curl::Request<H>>> for AsyncCurlError
where
    H: Handler + Debug + Send + 'static,
{
    fn from(err: SendError<async_curl::Request<H>>) -> Self {
        AsyncCurlError(format!("{:?}", err))
    }
}

impl fmt::Display for AsyncCurlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl error::Error for AsyncCurlError {}
