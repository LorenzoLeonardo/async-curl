use std::fmt::Debug;
use std::time::Duration;

use curl::easy::{Easy2, Handler};
use curl::multi::Multi;
use log::trace;
use tokio::runtime::Builder;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::oneshot;
use tokio::task::LocalSet;
use tokio::time::sleep;

use crate::error::Error;
/// CurlActor is responsible for performing
/// the contructed Easy2 object at the background
/// to perform it asynchronously.
/// ```
/// use curl::easy::Easy2;
/// use async_curl::response_handler::ResponseHandler;
/// use async_curl::actor::CurlActor;
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
/// use async_curl::{actor::CurlActor, response_handler::ResponseHandler};
/// use curl::easy::Easy2;
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

    /// This will send Easy2 into the background task that will perform
    /// curl asynchronously, await the response in the oneshot receiver and
    /// return Easy2 back to the caller.
    pub async fn send_request(&self, easy2: Easy2<H>) -> Result<Easy2<H>, Error<H>>
    where
        H: Handler + Debug + Send + 'static,
    {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel::<Result<Easy2<H>, Error<H>>>();
        self.request_sender
            .send(Request(easy2, oneshot_sender))
            .await?;
        oneshot_receiver.await?
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

#[cfg(test)]
mod test {
    use std::convert::TryFrom;
    use std::time::Duration;

    use http::StatusCode;
    use wiremock::matchers::method;
    use wiremock::matchers::path;
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::ResponseTemplate;

    use crate::actor::CurlActor;
    use crate::actor::Easy2;
    use crate::response_handler::ResponseHandler;

    async fn start_mock_server(
        node: &str,
        mock_body: String,
        mock_status_code: StatusCode,
    ) -> MockServer {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(node))
            .respond_with(ResponseTemplate::new(mock_status_code).set_body_string(mock_body))
            .mount(&server)
            .await;
        server
    }

    #[tokio::test]
    async fn test_async_requests() {
        const MOCK_BODY_RESPONSE: &str = r#"{"token":"12345"}"#;
        const MOCK_STATUS_CODE: StatusCode = StatusCode::OK;

        let server = start_mock_server(
            "/async-test",
            MOCK_BODY_RESPONSE.to_string(),
            StatusCode::OK,
        )
        .await;
        let url = format!("{}{}", server.uri(), "/async-test");

        let curl = CurlActor::new();
        let mut easy2 = Easy2::new(ResponseHandler::new());
        easy2.url(url.as_str()).unwrap();
        easy2.get(true).unwrap();

        let curl1 = curl.clone();
        let spawn1 = tokio::spawn(async move {
            let result = curl1.send_request(easy2).await;
            let mut result = result.unwrap();
            // Test response body
            assert_eq!(
                String::from_utf8_lossy(&result.get_ref().to_owned().get_data()),
                MOCK_BODY_RESPONSE.to_string()
            );

            // Test response status code
            let status_code = result.response_code().unwrap();

            assert_eq!(
                StatusCode::from_u16(u16::try_from(status_code).unwrap()).unwrap(),
                MOCK_STATUS_CODE
            );
        });

        let mut easy2 = Easy2::new(ResponseHandler::new());
        easy2.url(url.as_str()).unwrap();
        easy2.get(true).unwrap();

        let spawn2 = tokio::spawn(async move {
            let result = curl.send_request(easy2).await;
            let mut result = result.unwrap();
            // Test response body
            assert_eq!(
                String::from_utf8_lossy(&result.get_ref().to_owned().get_data()),
                MOCK_BODY_RESPONSE.to_string()
            );

            // Test response status code
            let status_code = result.response_code().unwrap();
            assert_eq!(
                StatusCode::from_u16(u16::try_from(status_code).unwrap()).unwrap(),
                MOCK_STATUS_CODE
            );
        });

        let (_, _) = tokio::join!(spawn1, spawn2);
    }

    #[tokio::test]
    async fn test_error() {
        let url = "https://no-connection";

        let curl = CurlActor::new();

        let mut easy2 = Easy2::new(ResponseHandler::new());
        easy2.url(url).unwrap();
        easy2.get(true).unwrap();

        let result = curl.send_request(easy2).await;
        let _ = result.unwrap_err();
    }

    #[tokio::test]
    async fn test_concurrency_abort() {
        let url = "https://no-connection";

        let curl = CurlActor::new();

        let curl_handle = tokio::spawn(async move {
            let mut easy2 = Easy2::new(ResponseHandler::new());
            easy2.url(url).unwrap();
            easy2.get(true).unwrap();

            let result = curl.send_request(easy2).await;
            let _ = result.unwrap_err();
            panic!("Not aborted, the future should be aborted.");
        });

        let other_task = tokio::spawn(async move {
            for _n in 0..10 {
                println!("Other task . . . .");
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        });

        other_task.await.unwrap();
        curl_handle.abort();
    }
}
