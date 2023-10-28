# async-curl
This will perform curl Easy2 asynchronously for rust-lang using curl::multi and tokio

## How to use with multiple async request

```rust
use async_curl::{actor::CurlActor, response_handler::ResponseHandler};
use curl::easy::Easy2;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let curl = CurlActor::new();
    let mut easy2 = Easy2::new(ResponseHandler::new());
    easy2.url("https://www.rust-lang.org").unwrap();
    easy2.get(true).unwrap();

    let spawn1 = tokio::spawn(async move {
        let response = curl.send_request(easy2).await;
        let mut response = response.unwrap();

        // Response body
        eprintln!(
            "Task 1 : {}",
            String::from_utf8_lossy(&response.get_ref().to_owned().get_data())
        );
        // Response status code
        let status_code = response.response_code().unwrap();
        eprintln!("Task 1 : {}", status_code);
    });

    let curl = CurlActor::new();
    let mut easy2 = Easy2::new(ResponseHandler::new());
    easy2.url("https://www.rust-lang.org").unwrap();
    easy2.get(true).unwrap();

    let spawn2 = tokio::spawn(async move {
        let response = curl.send_request(easy2).await;
        let mut response = response.unwrap();

        // Response body
        eprintln!(
            "Task 2 : {}",
            String::from_utf8_lossy(&response.get_ref().to_owned().get_data())
        );
        // Response status code
        let status_code = response.response_code().unwrap();
        eprintln!("Task 2 : {}", status_code);
    });
    let (_, _) = tokio::join!(spawn1, spawn2);

    Ok(())
}
```