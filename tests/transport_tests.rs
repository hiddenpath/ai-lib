use ai_lib::transport::HttpTransport;
use reqwest::Client;
use std::time::Duration;

#[test]
fn can_construct_transport_from_reqwest_client() {
    let client = Client::builder().timeout(Duration::from_secs(5)).build().unwrap();
    let transport = HttpTransport::with_reqwest_client(client, Duration::from_secs(5));
    // ensure boxed conversion works
    let _boxed = transport.boxed();
}
