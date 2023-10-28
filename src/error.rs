use std::fmt::Debug;

use curl::{easy::Handler, MultiError};
use tokio::sync::{mpsc::error::SendError, oneshot::error::RecvError};

use crate::actor;

/// This the enum of Errors for this crate.
#[derive(Debug)]
pub enum Error<H>
where
    H: Handler + Debug + Send + 'static,
{
    Curl(curl::Error),
    Multi(curl::MultiError),
    TokioRecv(RecvError),
    TokioSend(SendError<actor::Request<H>>),
}

/// This convert MultiError to our customized
/// Error enum for ease of management of
/// different errors from 3rd party crates.
impl<H> From<MultiError> for Error<H>
where
    H: Handler + Debug + Send + 'static,
{
    fn from(err: MultiError) -> Self {
        Error::Multi(err)
    }
}

/// This convert RecvError to our customized
/// Error enum for ease of management of
/// different errors from 3rd party crates.
impl<H> From<RecvError> for Error<H>
where
    H: Handler + Debug + Send + 'static,
{
    fn from(err: RecvError) -> Self {
        Error::TokioRecv(err)
    }
}

/// This convert SendError to our customized
/// Error enum for ease of management of
/// different errors from 3rd party crates.
impl<H> From<SendError<actor::Request<H>>> for Error<H>
where
    H: Handler + Debug + Send + 'static,
{
    fn from(err: SendError<actor::Request<H>>) -> Self {
        Error::TokioSend(err)
    }
}

/// This convert curl::Error to our customized
/// Error enum for ease of management of
/// different errors from 3rd party crates.
impl<H> From<curl::Error> for Error<H>
where
    H: Handler + Debug + Send + 'static,
{
    fn from(err: curl::Error) -> Self {
        Error::Curl(err)
    }
}

impl<H> std::fmt::Display for Error<H>
where
    H: Handler + Debug + Send + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Curl(err) => write!(f, "{}", err),
            Error::Multi(err) => write!(f, "{}", err),
            Error::TokioRecv(err) => write!(f, "{}", err),
            Error::TokioSend(err) => write!(f, "{}", err),
        }
    }
}

impl<H> std::error::Error for Error<H> where H: Handler + Debug + Send + 'static {}
