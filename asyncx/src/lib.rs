use std::time::Duration;

use tokio::time::sleep;

#[allow(dead_code)]
async fn request() -> String {
    sleep(Duration::from_secs(2)).await;
    "content".to_string()
}

#[cfg(test)]
mod tests {
    use crate::request;
    use anyhow::{self, Result};
    use futures::{SinkExt, StreamExt};
    use std::time::Duration;
    use tokio::{net::TcpListener, time::sleep};
    use tokio_util::codec::{Framed, LinesCodec};

    #[tokio::test]
    async fn read_content_should_works() {
        let mut handles = Vec::new();
        for _ in 0..10 {
            handles.push(tokio::spawn(request()));
        }

        let mut output = Vec::with_capacity(handles.len());
        for handle in handles {
            output.push(handle.await.unwrap());
        }
        for res in output {
            println!("response {}", res);
        }
    }

    #[tokio::test]
    async fn spawn_should_works() {
        for _ in 0..10 {
            tokio::spawn(async move {
                let resp = request().await;
                println!("resp: {:?}", resp);
            });
        }
        sleep(Duration::from_secs(3)).await;
    }

    #[tokio::test]
    async fn select_sould_works() {
        tokio::select! {
            res = run_server() => {
                if let Err(err) = res {
                    println!("err {}", err.to_string());
                }
            }
            _ = sleep(Duration::from_secs(10)) => {
                println!("timeout");
            }
        }
    }

    async fn run_server() -> Result<()> {
        let addr = "0.0.0.0:5000";
        let listener = TcpListener::bind(addr).await.unwrap();

        loop {
            let (stream, addr) = listener.accept().await.unwrap();
            println!("receive conn from: {}", addr);
            tokio::spawn(async move {
                let framed = Framed::new(stream, LinesCodec::new());
                let (mut w, mut r) = framed.split();
                for line in r.next().await {
                    w.send(format!("got : {}", line?)).await.unwrap();
                }
                Ok::<_, anyhow::Error>(())
            });
        }
    }
}
