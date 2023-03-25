use curl::{easy::Handler, MultiError};
use std::fmt::Debug;
use tokio::sync::{mpsc::error::SendError, oneshot::error::RecvError};

use crate::async_curl;

#[derive(Debug)]
pub struct AsyncCurlError(pub String);

impl From<MultiError> for AsyncCurlError {
    fn from(err: MultiError) -> Self {
        AsyncCurlError(format!("{:?}", err))
    }
}

impl From<RecvError> for AsyncCurlError {
    fn from(err: RecvError) -> Self {
        AsyncCurlError(format!("{:?}", err))
    }
}

impl<H> From<SendError<async_curl::Request<H>>> for AsyncCurlError
where
    H: Handler + Debug + Send + 'static,
{
    fn from(err: SendError<async_curl::Request<H>>) -> Self {
        AsyncCurlError(format!("{:?}", err))
    }
}
