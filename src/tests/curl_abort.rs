use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use curl::easy::{Easy2, Handler, WriteError};
use http::StatusCode;
use tokio::runtime::Builder;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

use crate::{Actor, CurlActor};

#[derive(Debug, Clone, Default)]
pub struct ResponseHandler {
    data: Option<Vec<u8>>,
    abort: Arc<AtomicBool>,
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

    fn progress(&mut self, _dltotal: f64, _dlnow: f64, _ultotal: f64, _ulnow: f64) -> bool {
        // Return true to continue the transfer, false to abort
        self.abort.load(Ordering::Relaxed) == false
    }
}

impl ResponseHandler {
    /// Instantiation of the ResponseHandler
    /// and initialize the data vector.
    pub fn new(abort: Arc<AtomicBool>) -> Self {
        Self { data: None, abort }
    }
}

async fn start_mock_server_with_delay(
    node: &str,
    mock_body: String,
    mock_status_code: StatusCode,
    delay: Duration,
) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(node))
        .respond_with(
            ResponseTemplate::new(mock_status_code)
                .set_delay(delay)
                .set_body_string(mock_body),
        )
        .mount(&server)
        .await;
    server
}

#[tokio::test(flavor = "current_thread")]
async fn test_transfer_abort_current_thread_curl_multi_perform() {
    const MOCK_BODY_RESPONSE: &str = r#"{"token":"12345"}"#;
    let server = start_mock_server_with_delay(
        "/async-test",
        MOCK_BODY_RESPONSE.to_string(),
        StatusCode::OK,
        Duration::from_millis(50),
    )
    .await;
    let url = format!("{}{}", server.uri(), "/async-test");
    let runtime = Builder::new_current_thread().enable_all().build().unwrap();
    let main_curl = CurlActor::new_runtime(runtime).use_multi_transfer();
    let curl = main_curl.clone();
    let cancel = Arc::new(AtomicBool::new(false));
    let http_cancel = cancel.clone();
    let http_task = tokio::spawn(async move {
        let mut easy2 = Easy2::new(ResponseHandler::new(http_cancel));
        easy2.url(url.as_str()).unwrap();
        easy2.get(true).unwrap();
        easy2.progress(true).unwrap();
        log::trace!("[test_transfer_abort_current_thread_curl_multi_perform] HTTP task . . . .");
        let result = curl.send_request(easy2).await;

        println!(
            "[test_transfer_abort_current_thread_curl_multi_perform] HTTP task result: {:?}",
            result
        );
        result
    });

    let other_task = tokio::spawn(async move {
        for _n in 0..10 {
            log::trace!(
                "[test_transfer_abort_current_thread_curl_multi_perform] Other task . . . ."
            );
            tokio::time::sleep(Duration::from_millis(1)).await;
            log::trace!(
                "[test_transfer_abort_current_thread_curl_multi_perform] Other task completed"
            );
        }
    });

    let abort_cancel = cancel.clone();
    let abort_task = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(5)).await;
        log::trace!(
            "[test_transfer_abort_current_thread_curl_multi_perform] Aborting HTTP task . . . ."
        );
        abort_cancel.store(true, Ordering::Relaxed);
        log::trace!("[test_transfer_abort_current_thread_curl_multi_perform] HTTP task aborted");
    });

    let (other_task, abort_task) = tokio::join!(other_task, abort_task);
    other_task.unwrap();
    abort_task.unwrap();

    let result = http_task.await.unwrap().unwrap_err();
    match result {
        crate::Error::Curl(e) => {
            assert_eq!(e.code(), curl_sys::CURLE_ABORTED_BY_CALLBACK);
        }
        _ => panic!(
            "[test_transfer_abort_current_thread_curl_multi_perform] Expected Curl error, got {:?}",
            result
        ),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_transfer_abort_multi_thread_curl_multi_perform() {
    const MOCK_BODY_RESPONSE: &str = r#"{"token":"12345"}"#;
    let server = start_mock_server_with_delay(
        "/async-test",
        MOCK_BODY_RESPONSE.to_string(),
        StatusCode::OK,
        Duration::from_millis(50),
    )
    .await;
    let url = format!("{}{}", server.uri(), "/async-test");
    let runtime = Builder::new_multi_thread().enable_all().build().unwrap();
    let main_curl = CurlActor::new_runtime(runtime).use_multi_transfer();
    let curl = main_curl.clone();
    let cancel = Arc::new(AtomicBool::new(false));
    let http_cancel = cancel.clone();
    let http_task = tokio::spawn(async move {
        let mut easy2 = Easy2::new(ResponseHandler::new(http_cancel));
        easy2.url(url.as_str()).unwrap();
        easy2.get(true).unwrap();
        easy2.progress(true).unwrap();
        log::trace!("[test_transfer_abort_multi_thread_curl_multi_perform] HTTP task . . . .");
        let result = curl.send_request(easy2).await;

        println!(
            "[test_transfer_abort_multi_thread_curl_multi_perform] HTTP task result: {:?}",
            result
        );
        result
    });

    let other_task = tokio::spawn(async move {
        for _n in 0..10 {
            log::trace!("[test_transfer_abort_multi_thread_curl_multi_perform] Other task . . . .");
            tokio::time::sleep(Duration::from_millis(1)).await;
            log::trace!(
                "[test_transfer_abort_multi_thread_curl_multi_perform] Other task completed"
            );
        }
    });

    let abort_cancel = cancel.clone();
    let abort_task = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(5)).await;
        log::trace!(
            "[test_transfer_abort_multi_thread_curl_multi_perform] Aborting HTTP task . . . ."
        );
        abort_cancel.store(true, Ordering::Relaxed);
        log::trace!("[test_transfer_abort_multi_thread_curl_multi_perform] HTTP task aborted");
    });

    let (other_task, abort_task) = tokio::join!(other_task, abort_task);
    other_task.unwrap();
    abort_task.unwrap();

    let result = http_task.await.unwrap().unwrap_err();
    match result {
        crate::Error::Curl(e) => {
            assert_eq!(e.code(), curl_sys::CURLE_ABORTED_BY_CALLBACK);
        }
        _ => panic!(
            "[test_transfer_abort_multi_thread_curl_multi_perform] Expected Curl error, got {:?}",
            result
        ),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_transfer_abort_current_thread_curl_easy2_perform() {
    const MOCK_BODY_RESPONSE: &str = r#"{"token":"12345"}"#;
    let server = start_mock_server_with_delay(
        "/async-test",
        MOCK_BODY_RESPONSE.to_string(),
        StatusCode::OK,
        Duration::from_millis(50),
    )
    .await;
    let url = format!("{}{}", server.uri(), "/async-test");
    let runtime = Builder::new_current_thread().enable_all().build().unwrap();
    let main_curl = CurlActor::new_runtime(runtime);
    let curl = main_curl.clone();
    let cancel = Arc::new(AtomicBool::new(false));
    let http_cancel = cancel.clone();
    let http_task = tokio::spawn(async move {
        let mut easy2 = Easy2::new(ResponseHandler::new(http_cancel));
        easy2.url(url.as_str()).unwrap();
        easy2.get(true).unwrap();
        easy2.progress(true).unwrap();
        log::trace!("[test_transfer_abort_current_thread_curl_easy2_perform] HTTP task . . . .");
        let result = curl.send_request(easy2).await;

        println!(
            "[test_transfer_abort_current_thread_curl_easy2_perform] HTTP task result: {:?}",
            result
        );
        result
    });

    let other_task = tokio::spawn(async move {
        for _n in 0..10 {
            log::trace!(
                "[test_transfer_abort_current_thread_curl_easy2_perform] Other task . . . ."
            );
            tokio::time::sleep(Duration::from_millis(1)).await;
            log::trace!(
                "[test_transfer_abort_current_thread_curl_easy2_perform] Other task completed"
            );
        }
    });

    let abort_cancel = cancel.clone();
    let abort_task = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(5)).await;
        log::trace!(
            "[test_transfer_abort_current_thread_curl_easy2_perform] Aborting HTTP task . . . ."
        );
        abort_cancel.store(true, Ordering::Relaxed);
        log::trace!("[test_transfer_abort_current_thread_curl_easy2_perform] HTTP task aborted");
    });

    let (other_task, abort_task) = tokio::join!(other_task, abort_task);
    other_task.unwrap();
    abort_task.unwrap();

    let result = http_task.await.unwrap().unwrap_err();
    match result {
        crate::Error::Curl(e) => {
            assert_eq!(e.code(), curl_sys::CURLE_ABORTED_BY_CALLBACK);
        }
        _ => panic!(
            "[test_transfer_abort_current_thread_curl_easy2_perform] Expected Curl error, got {:?}",
            result
        ),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_transfer_abort_multi_thread_curl_easy2_perform() {
    const MOCK_BODY_RESPONSE: &str = r#"{"token":"12345"}"#;
    let server = start_mock_server_with_delay(
        "/async-test",
        MOCK_BODY_RESPONSE.to_string(),
        StatusCode::OK,
        Duration::from_millis(50),
    )
    .await;
    let url = format!("{}{}", server.uri(), "/async-test");
    let runtime = Builder::new_multi_thread().enable_all().build().unwrap();
    let main_curl = CurlActor::new_runtime(runtime);
    let curl = main_curl.clone();
    let cancel = Arc::new(AtomicBool::new(false));
    let http_cancel = cancel.clone();
    let http_task = tokio::spawn(async move {
        let mut easy2 = Easy2::new(ResponseHandler::new(http_cancel));
        easy2.url(url.as_str()).unwrap();
        easy2.get(true).unwrap();
        easy2.progress(true).unwrap();
        log::trace!("[test_transfer_abort_multi_thread_curl_easy2_perform] HTTP task . . . .");
        let result = curl.send_request(easy2).await;

        println!(
            "[test_transfer_abort_multi_thread_curl_easy2_perform] HTTP task result: {:?}",
            result
        );
        result
    });

    let other_task = tokio::spawn(async move {
        for _n in 0..10 {
            log::trace!("[test_transfer_abort_multi_thread_curl_easy2_perform] Other task . . . .");
            tokio::time::sleep(Duration::from_millis(1)).await;
            log::trace!(
                "[test_transfer_abort_multi_thread_curl_easy2_perform] Other task completed"
            );
        }
    });

    let abort_cancel = cancel.clone();
    let abort_task = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(5)).await;
        log::trace!(
            "[test_transfer_abort_multi_thread_curl_easy2_perform] Aborting HTTP task . . . ."
        );
        abort_cancel.store(true, Ordering::Relaxed);
        log::trace!("[test_transfer_abort_multi_thread_curl_easy2_perform] HTTP task aborted");
    });

    let (other_task, abort_task) = tokio::join!(other_task, abort_task);
    other_task.unwrap();
    abort_task.unwrap();

    let result = http_task.await.unwrap().unwrap_err();
    match result {
        crate::Error::Curl(e) => {
            assert_eq!(e.code(), curl_sys::CURLE_ABORTED_BY_CALLBACK);
        }
        _ => panic!(
            "[test_transfer_abort_multi_thread_curl_easy2_perform] Expected Curl error, got {:?}",
            result
        ),
    }
}
