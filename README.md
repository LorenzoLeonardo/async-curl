# async-curl
This will perform curl Easy2 asynchronously for rust-lang via an Actor using tokio

## How to use with multiple async request

```rust
use async_curl::actor::CurlActor;
use async_curl::response_handler::ResponseHandler;
use curl::easy::Easy2;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let actor = CurlActor::new();

    let mut easy2 = Easy2::new(ResponseHandler::new());
    easy2.url("https://www.rust-lang.org/").unwrap();
    easy2.get(true).unwrap();

    let actor1 = actor.clone();
    let spawn1 = tokio::spawn(async move {
        let mut result = actor1.send_request(easy2).await.unwrap();

        let response = result.get_ref().to_owned().get_data();
        let status = result.response_code().unwrap();

        println!("Response: {:?}", response);
        println!("Status: {:?}", status);
    });

    let mut easy2 = Easy2::new(ResponseHandler::new());
    easy2.url("https://www.rust-lang.org/").unwrap();
    easy2.get(true).unwrap();

    let spawn2 = tokio::spawn(async move {
        let mut result = actor.send_request(easy2).await.unwrap();

        let response = result.get_ref().to_owned().get_data();
        let status = result.response_code().unwrap();

        println!("Response: {:?}", response);
        println!("Status: {:?}", status);
    });

    let (_, _) = tokio::join!(spawn1, spawn2);
}
```