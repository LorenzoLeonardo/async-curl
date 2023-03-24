use curl::easy::WriteError;
use curl::MultiError;
use curl::{
    easy::{Easy2, Handler},
    multi::Multi,
};
use std::fmt::Debug;
use std::io::Read;

#[derive(Debug, Clone, Default)]
pub struct ResponseHandler {
    data: Vec<u8>,
}

impl Handler for ResponseHandler {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.data.extend_from_slice(data);
        Ok(data.len())
    }
}

impl ResponseHandler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_data(self) -> Vec<u8> {
        self.data
    }
}

pub async fn send_request<H>(easy2: Easy2<H>) -> Result<Easy2<H>, MultiError>
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

#[cfg(test)]
mod test {
    use crate::async_curl::send_request;
    use crate::async_curl::Easy2;
    use crate::async_curl::ResponseHandler;

    #[tokio::test]
    async fn test_send_request() {
        let mut easy2 = Easy2::new(ResponseHandler::new());
        easy2.url("https://www.google.com").unwrap();
        easy2.get(true).unwrap();

        let spawn1 = tokio::spawn(async move {
            let result = send_request(easy2).await;
            let result = result.unwrap();
            eprintln!(
                "{:?}",
                String::from_utf8_lossy(&result.get_ref().to_owned().get_data())
            );
        });

        let mut easy2 = Easy2::new(ResponseHandler::new());
        easy2.url("https://www.google.com").unwrap();
        easy2.get(true).unwrap();

        let spawn2 = tokio::spawn(async move {
            let result = send_request(easy2).await;
            let result = result.unwrap();
            eprintln!(
                "{:?}",
                String::from_utf8_lossy(&result.get_ref().to_owned().get_data())
            );
        });

        let (_, _) = tokio::join!(spawn1, spawn2);
    }
}
