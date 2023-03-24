use curl::easy::{Easy2, Handler};
use curl::multi::Multi;
use curl::MultiError;

pub struct AsyncCurl;

impl AsyncCurl {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn send_request<H>(&self, easy2: Easy2<H>) -> Result<Easy2<H>, MultiError>
    where
        H: Handler,
    {
        let multi = Multi::new();
        let handle = multi.add2(easy2)?;

        while multi.perform()? > 0 {
            multi.wait(&mut [], std::time::Duration::from_secs(1))?;
        }

        multi.remove2(handle)
    }
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
        easy2.url("https://www.google.com").unwrap();
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
        easy2.url("https://www.google.com").unwrap();
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
