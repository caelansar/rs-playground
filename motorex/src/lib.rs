#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

use std::future::Future;

use http::{Request, Response, StatusCode};
use hyper::Body;
use motore::{BoxError, Service};

pub struct Context {
    log_id: usize,
}

#[derive(Clone)]
pub struct LogService<S> {
    inner: S,
}

impl<Req, S> Service<Context, Req> for LogService<S>
where
    Req: 'static + Send,
    S: Service<Context, Req> + 'static + Send + Sync,
    Context: 'static + Send,
    S::Error: Send + Sync + Into<BoxError>,
{
    type Response = S::Response;

    type Error = BoxError;

    type Future<'cx> = impl Future<Output = Result<S::Response, Self::Error>> + Send + 'cx;

    fn call<'cx, 's>(&'s self, cx: &'cx mut Context, req: Req) -> Self::Future<'cx>
    where
        's: 'cx,
    {
        async move {
            let resp = self.inner.call(cx, req).await;
            match resp {
                Ok(_) => {
                    println!("succss, log id: {}", cx.log_id);
                }
                Err(_) => {
                    println!("failed, log id: {}", cx.log_id);
                }
            }
            resp.map_err(Into::into)
        }
    }
}

#[derive(Clone)]
struct HelloWorld;

impl<Cx> Service<Cx, Request<Body>> for HelloWorld {
    type Response = Response<String>;

    type Error = http::Error;

    type Future<'cx> = impl Future<Output = Result<Self::Response, Self::Error>> + 'cx where Cx: 'cx;

    fn call<'cx, 's>(&'s self, _cx: &'cx mut Cx, _req: Request<Body>) -> Self::Future<'cx>
    where
        's: 'cx,
    {
        let body = "hello, world".to_string();
        let resp = Response::builder()
            .status(StatusCode::OK)
            .body(body)
            .expect("Unable to create `http::Response`");
        async { Ok(resp) }
    }
}

#[cfg(test)]
mod tests {
    use hyper::Server;
    use motore::{builder::ServiceBuilder, service::TowerAdapter};
    use std::net::SocketAddr;
    use std::net::TcpListener;
    use tower::make::Shared;

    use super::*;

    #[tokio::test]
    async fn motore_works() {
        let addr = run_in_background();

        let client = reqwest::Client::builder().build().unwrap();

        let response = client
            .get(&format!("http://{}", addr))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.text().await.unwrap(), "hello, world".to_string());
    }

    async fn serve_forever(listener: TcpListener) {
        let motore_service = ServiceBuilder::new()
            .service(LogService {
                inner: HelloWorld {},
            })
            .tower(|tower_req| (Context { log_id: 100 }, tower_req));

        Server::from_tcp(listener)
            .unwrap()
            .serve(Shared::new(motore_service))
            .await
            .expect("server error");
    }

    fn run_in_background() -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Could not bind ephemeral socket");
        let addr = listener.local_addr().unwrap();

        println!("Listening on {}", addr);

        tokio::spawn(async move {
            serve_forever(listener).await;
        });

        addr
    }
}
