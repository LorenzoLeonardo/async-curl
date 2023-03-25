use curl::easy::{Easy2, Handler};
use curl::multi::Multi;
use curl::MultiError;
use std::fmt::Debug;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::oneshot;
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

impl<H> AsyncCurl<H>
where
    H: Handler + Debug + Send + 'static,
{
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel::<Request<H>>(1);
        tokio::spawn(async move {
            while let Some(res) = rx.recv().await {
                let response = perform_curl_multi(res.0).await;
                res.1.send(response).unwrap()
            }
        });

        Self { sender: tx }
    }

    pub async fn send_request(&self, easy2: Easy2<H>) -> Result<Easy2<H>, MultiError>
    where
        H: Handler + Debug + Send + 'static,
    {
        let (tx, rx) = oneshot::channel::<Result<Easy2<H>, MultiError>>();
        self.sender.send(Request(easy2, tx)).await.unwrap();
        rx.await.unwrap()
    }
}

#[derive(Debug)]
struct Request<H: Handler + Debug + Send + 'static>(
    Easy2<H>,
    oneshot::Sender<Result<Easy2<H>, MultiError>>,
);

pub async fn perform_curl_multi<H: Handler>(easy2: Easy2<H>) -> Result<Easy2<H>, MultiError> {
    let multi = Multi::new();
    let handle = multi.add2(easy2)?;

    while multi.perform()? > 0 {
        multi.wait(&mut [], std::time::Duration::from_secs(1))?;
    }

    multi.remove2(handle)
}

#[cfg(test)]
mod test {

    use crate::async_curl::AsyncCurl;
    use crate::async_curl::Easy2;
    use crate::response_handler::ResponseHandler;

    #[tokio::test]
    async fn test_send_request() {
        let curl = AsyncCurl::new();
        let mut easy2 = Easy2::new(ResponseHandler::new());
        easy2.url("https://www.rust-lang.org").unwrap();
        easy2.get(true).unwrap();

        let spawn1 = tokio::spawn(async move {
            let result = curl.send_request(easy2).await;
            let result = result.unwrap();
            eprintln!(
                "{:?}",
                String::from_utf8_lossy(&result.get_ref().to_owned().get_data())
            );
        });

        let curl = AsyncCurl::new();
        let mut easy2 = Easy2::new(ResponseHandler::new());
        easy2.url("https://www.rust-lang.org").unwrap();
        easy2.get(true).unwrap();

        let spawn2 = tokio::spawn(async move {
            let result = curl.send_request(easy2).await;
            let result = result.unwrap();
            eprintln!(
                "{:?}",
                String::from_utf8_lossy(&result.get_ref().to_owned().get_data())
            );
        });

        let (_, _) = tokio::join!(spawn1, spawn2);
    }
}
