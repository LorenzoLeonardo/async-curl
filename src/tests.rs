use std::sync::Arc;
use std::time::Duration;

use curl::easy::Easy2;
use curl::easy::Handler;
use curl::easy::WriteError;
use http::status::StatusCode;
use log::LevelFilter;
use tokio::sync::Mutex;
use wiremock::matchers::method;
use wiremock::matchers::path;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;

use crate::actor::Actor;
use crate::actor::CurlActor;
use crate::curl::AsyncCurl;

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
        } else if let Some(ref mut data) = self.data {
            data.extend_from_slice(stream);
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

#[ctor::ctor]
fn setup_test_logger() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("your_crate_name=trace"),
    )
    .filter_level(LevelFilter::Trace)
    .init();
}

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
            String::from_utf8_lossy(&result.get_mut().take().unwrap()),
            MOCK_BODY_RESPONSE.to_string()
        );

        // Test response status code
        let status_code = result.response_code().unwrap() as u16;

        assert_eq!(status_code, MOCK_STATUS_CODE.as_u16());
    });

    let mut easy2 = Easy2::new(ResponseHandler::new());
    easy2.url(url.as_str()).unwrap();
    easy2.get(true).unwrap();

    let spawn2 = tokio::spawn(async move {
        let result = curl.send_request(easy2).await;
        let mut result = result.unwrap();
        // Test response body
        assert_eq!(
            String::from_utf8_lossy(&result.get_mut().take().unwrap()),
            MOCK_BODY_RESPONSE.to_string()
        );

        // Test response status code
        let status_code = result.response_code().unwrap() as u16;
        assert_eq!(status_code, MOCK_STATUS_CODE.as_u16());
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
    const MOCK_BODY_RESPONSE: &str = r#"{"token":"12345"}"#;
    let server = start_mock_server(
        "/async-test",
        MOCK_BODY_RESPONSE.to_string(),
        StatusCode::OK,
    )
    .await;
    let url = format!("{}{}", server.uri(), "/async-test");
    let check_cancelled = Arc::new(Mutex::new(true));
    let curl = CurlActor::new();

    let check_cancelled1 = check_cancelled.clone();
    let http_task = tokio::spawn(async move {
        let mut easy2 = Easy2::new(ResponseHandler::new());
        easy2.url(url.as_str()).unwrap();
        easy2.get(true).unwrap();
        log::trace!("HTTP task . . . .");
        let _ = curl.send_request(easy2).await.unwrap();
        let mut lock = check_cancelled1.lock().await;
        *lock = false;
    });

    let other_task = tokio::spawn(async move {
        for _n in 0..10 {
            log::trace!("Other task . . . .");
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });

    let abort_task = tokio::spawn(async move {
        http_task.abort();
    });

    let (other_task, abort_task) = tokio::join!(other_task, abort_task);
    other_task.unwrap();
    abort_task.unwrap();
    assert!(*check_cancelled.lock().await);
}

#[tokio::test]
async fn test_curl_builder() {
    const MOCK_BODY_RESPONSE: &str = r#"{"token":"12345"}"#;
    let server = start_mock_server(
        "/async-test",
        MOCK_BODY_RESPONSE.to_string(),
        StatusCode::OK,
    )
    .await;
    let url = format!("{}{}", server.uri(), "/async-test");

    let actor = CurlActor::new();
    let collector = ResponseHandler::new();

    let mut curl = AsyncCurl::new(actor, collector)
        .url(url.as_str())
        .unwrap()
        .finalize()
        .perform()
        .await
        .unwrap();

    log::trace!("{:?}", curl);
    let body = curl.get_mut().take();
    let status = curl.response_code().unwrap() as u16;

    assert_eq!(body, Some(MOCK_BODY_RESPONSE.as_bytes().to_vec()));
    assert_eq!(status, StatusCode::OK);
}
