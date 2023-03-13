use anyhow::Result;
use futures::{SinkExt, StreamExt};
use rand::Rng;
use std::{thread, time::Duration};
use tokio::{
    net::TcpListener,
    sync::{mpsc, oneshot},
};
use tokio_util::codec::{Framed, LinesCodec};

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "127.0.0.1:5000";
    let listener = TcpListener::bind(addr).await?;
    println!("listening: {}", addr);

    let (sender, mut receiver) = mpsc::unbounded_channel::<oneshot::Sender<String>>();

    thread::spawn(move || {
        while let Some(reply) = receiver.blocking_recv() {
            let result = gen_string_slow();
            if let Err(e) = reply.send(result) {
                println!("failed to send: {}", e);
            }
        }
    });

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("accepted: {:?}", addr);
        let sender1 = sender.clone();
        tokio::spawn(async move {
            let framed = Framed::new(stream, LinesCodec::new());
            let (mut w, mut r) = framed.split();
            while let Some(Ok(_)) = r.next().await {
                let (reply, reply_receiver) = oneshot::channel();
                sender1.send(reply)?;

                if let Ok(v) = reply_receiver.await {
                    w.send(format!("rand string: {}", v)).await?;
                }
            }
            Ok::<_, anyhow::Error>(())
        });
    }
}

fn gen_string_slow() -> String {
    thread::sleep(Duration::from_secs(2));

    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(7)
        .map(char::from)
        .collect()
}
