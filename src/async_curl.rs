use curl::easy::{Easy2, Handler};
use curl::multi::Multi;
use std::fmt::Debug;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::oneshot;

use crate::async_curl_error::AsyncCurlError;
/// AsyncCurl is responsible for performing
/// the contructed Easy2 object by passing
/// it into send_request
/// ```
/// use curl::easy::Easy2;
/// use async_curl::response_handler::ResponseHandler;
/// use async_curl::async_curl::AsyncCurl;
///
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>>{
/// let curl = AsyncCurl::new();
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
pub struct AsyncCurl<H>
where
    H: Handler + Debug + Send + 'static,
{
    sender: Sender<Request<H>>,
}

impl<H> Default for AsyncCurl<H>
where
    H: Handler + Debug + Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<H> AsyncCurl<H>
where
    H: Handler + Debug + Send + 'static,
{
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel::<Request<H>>(1);
        tokio::spawn(async move {
            while let Some(res) = rx.recv().await {
                let response = perform_curl_multi(res.0).await;
                if let Err(res) = res.1.send(response) {
                    eprintln!("Warning! The receiver has been dropped. {:?}", res);
                }
            }
        });

        Self { sender: tx }
    }

    pub async fn send_request(&self, easy2: Easy2<H>) -> Result<Easy2<H>, AsyncCurlError>
    where
        H: Handler + Debug + Send + 'static,
    {
        let (tx, rx) = oneshot::channel::<Result<Easy2<H>, AsyncCurlError>>();
        self.sender.send(Request(easy2, tx)).await?;
        rx.await?
    }
}

#[derive(Debug)]
pub(crate) struct Request<H: Handler + Debug + Send + 'static>(
    Easy2<H>,
    oneshot::Sender<Result<Easy2<H>, AsyncCurlError>>,
);

pub async fn perform_curl_multi<H: Handler>(easy2: Easy2<H>) -> Result<Easy2<H>, AsyncCurlError> {
    let multi = Multi::new();
    let handle = multi.add2(easy2)?;

    while multi.perform()? > 0 {
        multi.wait(&mut [], std::time::Duration::from_secs(1))?;
    }

    multi.remove2(handle).map_err(AsyncCurlError::from)
}

#[cfg(test)]
mod test {

    use http::StatusCode;
    use wiremock::matchers::method;
    use wiremock::matchers::path;
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::ResponseTemplate;

    use crate::async_curl::AsyncCurl;
    use crate::async_curl::Easy2;
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
        const PATH: &str = "/test";
        const MOCK_BODY_RESPONSE: &str = r#"{"token":"12345"}"#;
        const MOCK_STATUS_CODE: StatusCode = StatusCode::OK;

        let server = start_mock_server(
            "/async-test",
            MOCK_BODY_RESPONSE.to_string(),
            StatusCode::OK,
        )
        .await;
        let url = format!("{}{}", server.uri(), "/async-test");

        let curl = AsyncCurl::new();
        let mut easy2 = Easy2::new(ResponseHandler::new());
        easy2.url(url.as_str()).unwrap();
        easy2.get(true).unwrap();

        let spawn1 = tokio::spawn(async move {
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
                StatusCode::from_u16(status_code.try_into().unwrap()).unwrap(),
                MOCK_STATUS_CODE
            );
        });

        let curl = AsyncCurl::new();
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
                StatusCode::from_u16(status_code.try_into().unwrap()).unwrap(),
                MOCK_STATUS_CODE
            );
        });

        let (_, _) = tokio::join!(spawn1, spawn2);
    }
}
