use hello::hello_server::{Hello, HelloServer};
use hello::{HelloRequest, HelloResponse};
use tonic::{transport::Server, Request, Response, Status};

pub mod hello {
    tonic::include_proto!("hello");
}

#[derive(Debug, Default)]
pub struct HelloService {}

#[tonic::async_trait]
impl Hello for HelloService {
    async fn send_message(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        println!("get request: {:?}", request);

        let req = request.into_inner();

        let reply = HelloResponse {
            ok: true,
            message: format!("hello: {}", req.name),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:5000".parse()?;
    println!("grpc server start: {}", addr);

    let hello_service = HelloService::default();

    Server::builder()
        .add_service(HelloServer::new(hello_service))
        .serve(addr)
        .await?;

    Ok(())
}
