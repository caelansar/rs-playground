use hello::hello_client::HelloClient;
use hello::HelloRequest;

pub mod hello {
    tonic::include_proto!("hello");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = HelloClient::connect("http://127.0.0.1:5000").await?;

    let req = tonic::Request::new(HelloRequest {
        name: "cae".to_string(),
    });

    let resp = client.send_message(req).await?;

    println!("get response: {:?}", resp);

    Ok(())
}
