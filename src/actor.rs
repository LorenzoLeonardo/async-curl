use std::fmt::Debug;
use std::time::Duration;

use async_trait::async_trait;
use curl::easy::{Easy2, Handler};
use curl::multi::Multi;
use log::trace;
use tokio::runtime::Builder;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::oneshot;
use tokio::task::LocalSet;
use tokio::time::sleep;

use crate::error::Error;

#[async_trait]
pub trait Actor<H>
where
    H: Handler + Debug + Send + 'static,
{
    async fn send_request(&self, easy2: Easy2<H>) -> Result<Easy2<H>, Error<H>>;
}

/// CurlActor is responsible for performing
/// the contructed Easy2 object at the background
/// to perform it asynchronously.
/// ```
/// use async_curl::actor::{Actor, CurlActor};
/// use curl::easy::{Easy2, Handler, WriteError};
///
/// #[derive(Debug, Clone, Default)]
/// pub struct ResponseHandler {
///     data: Vec<u8>,
/// }
///
/// impl Handler for ResponseHandler {
///     /// This will store the response from the server
///     /// to the data vector.
///     fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
///         self.data.extend_from_slice(data);
///         Ok(data.len())
///     }
/// }
///
/// impl ResponseHandler {
///     /// Instantiation of the ResponseHandler
///     /// and initialize the data vector.
///     pub fn new() -> Self {
///         Self::default()
///     }
///
///     /// This will consumed the object and
///     /// give the data to the caller
///     pub fn get_data(self) -> Vec<u8> {
///         self.data
///     }
/// }
///
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>>{
/// let curl = CurlActor::new();
/// let mut easy2 = Easy2::new(ResponseHandler::new());
///
/// easy2.url("https://www.rust-lang.org").unwrap();
/// easy2.get(true).unwrap();
///
/// let response = curl.send_request(easy2).await.unwrap();
/// eprintln!("{:?}", response.get_ref());
///
/// Ok(())
/// # }
/// ```
///
/// Example for multiple request executed
/// at the same time.
///
/// ```
/// use async_curl::actor::{Actor, CurlActor};
/// use curl::easy::{Easy2, Handler, WriteError};
///
/// #[derive(Debug, Clone, Default)]
/// pub struct ResponseHandler {
///     data: Vec<u8>,
/// }
///
/// impl Handler for ResponseHandler {
///     /// This will store the response from the server
///     /// to the data vector.
///     fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
///         self.data.extend_from_slice(data);
///         Ok(data.len())
///     }
/// }
///
/// impl ResponseHandler {
///     /// Instantiation of the ResponseHandler
///     /// and initialize the data vector.
///     pub fn new() -> Self {
///         Self::default()
///     }
///
///     /// This will consumed the object and
///     /// give the data to the caller
///     pub fn get_data(self) -> Vec<u8> {
///         self.data
///     }
/// }
///
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let actor = CurlActor::new();
/// let mut easy2 = Easy2::new(ResponseHandler::new());
/// easy2.url("https://www.rust-lang.org").unwrap();
/// easy2.get(true).unwrap();
///
/// let actor1 = actor.clone();
/// let spawn1 = tokio::spawn(async move {
///     let response = actor1.send_request(easy2).await;
///     let mut response = response.unwrap();
///
///     // Response body
///     eprintln!(
///         "Task 1 : {}",
///         String::from_utf8_lossy(&response.get_ref().to_owned().get_data())
///     );
///     // Response status code
///     let status_code = response.response_code().unwrap();
///     eprintln!("Task 1 : {}", status_code);
/// });
///
/// let mut easy2 = Easy2::new(ResponseHandler::new());
/// easy2.url("https://www.rust-lang.org").unwrap();
/// easy2.get(true).unwrap();
///
/// let spawn2 = tokio::spawn(async move {
///     let response = actor.send_request(easy2).await;
///     let mut response = response.unwrap();
///
///     // Response body
///     eprintln!(
///         "Task 2 : {}",
///         String::from_utf8_lossy(&response.get_ref().to_owned().get_data())
///     );
///     // Response status code
///     let status_code = response.response_code().unwrap();
///     eprintln!("Task 2 : {}", status_code);
/// });
/// let (_, _) = tokio::join!(spawn1, spawn2);
///
/// Ok(())
/// # }
/// ```
///
#[derive(Clone)]
pub struct CurlActor<H>
where
    H: Handler + Debug + Send + 'static,
{
    request_sender: Sender<Request<H>>,
}

impl<H> Default for CurlActor<H>
where
    H: Handler + Debug + Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<H> Actor<H> for CurlActor<H>
where
    H: Handler + Debug + Send + 'static,
{
    /// This will send Easy2 into the background task that will perform
    /// curl asynchronously, await the response in the oneshot receiver and
    /// return Easy2 back to the caller.
    async fn send_request(&self, easy2: Easy2<H>) -> Result<Easy2<H>, Error<H>> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel::<Result<Easy2<H>, Error<H>>>();
        self.request_sender
            .send(Request(easy2, oneshot_sender))
            .await?;
        oneshot_receiver.await?
    }
}

impl<H> CurlActor<H>
where
    H: Handler + Debug + Send + 'static,
{
    /// This creates the new instance of CurlActor to handle Curl perform asynchronously using Curl Multi
    /// in a background thread to avoid blocking of other tasks.
    pub fn new() -> Self {
        let (request_sender, mut request_receiver) = mpsc::channel::<Request<H>>(1);
        let runtime = Builder::new_current_thread().enable_all().build().unwrap();

        std::thread::spawn(move || {
            let local = LocalSet::new();
            local.spawn_local(async move {
                while let Some(Request(easy2, oneshot_sender)) = request_receiver.recv().await {
                    tokio::task::spawn_local(async move {
                        let response = perform_curl_multi(easy2).await;
                        if let Err(res) = oneshot_sender.send(response) {
                            trace!("Warning! The receiver has been dropped. {:?}", res);
                        }
                    });
                }
            });
            runtime.block_on(local);
        });

        Self { request_sender }
    }
}

async fn perform_curl_multi<H: Handler + Debug + Send + 'static>(
    easy2: Easy2<H>,
) -> Result<Easy2<H>, Error<H>> {
    let multi = Multi::new();
    let handle = multi.add2(easy2).map_err(|e| Error::Multi(e))?;

    while multi.perform().map_err(|e| Error::Multi(e))? != 0 {
        let timeout_result = multi
            .get_timeout()
            .map(|d| d.unwrap_or_else(|| Duration::from_secs(2)));

        let timeout = match timeout_result {
            Ok(duration) => duration,
            Err(multi_error) => {
                if !multi_error.is_call_perform() {
                    return Err(Error::Multi(multi_error));
                }
                Duration::ZERO
            }
        };

        if !timeout.is_zero() {
            sleep(Duration::from_millis(200)).await;
        }
    }

    let mut error: Option<Error<H>> = None;
    multi.messages(|msg| {
        if let Some(Err(e)) = msg.result() {
            error = Some(Error::Curl(e));
        }
    });

    if let Some(e) = error {
        Err(e)
    } else {
        multi.remove2(handle).map_err(|e| Error::Multi(e))
    }
}

/// This contains the Easy2 object and a oneshot sender channel when passing into the
/// background task to perform Curl asynchronously.
#[derive(Debug)]
pub struct Request<H: Handler + Debug + Send + 'static>(
    Easy2<H>,
    oneshot::Sender<Result<Easy2<H>, Error<H>>>,
);
