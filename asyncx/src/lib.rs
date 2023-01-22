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
            _ = sleep(Duration::from_secs(5)) => {
                println!("timeout");
            }
        }
    }

    #[derive(Debug)]
    struct SelfReference {
        name: String,
        name_ptr: *const String,
    }

    impl SelfReference {
        fn new(name: String) -> Self {
            SelfReference {
                name,
                name_ptr: std::ptr::null(),
            }
        }

        fn init(&mut self) {
            self.name_ptr = &self.name as *const String
        }

        fn print_name(&self) {
            print!(
                "struct {:p}\nname: {:p} name_ptr: {:p}\nname_val: {}, name_ref_val: {}\n",
                self,
                &self.name,
                self.name_ptr,
                self.name,
                unsafe { &*self.name_ptr }
            )
        }
    }

    #[test]
    fn test_self_reference() {
        let data = move_ptr();
        println!("data: {:?}", data);

        // ERROR
        // data.print_name();
        println!("memory swap");
        mem_swap();
    }

    fn move_ptr() -> SelfReference {
        let mut data = SelfReference::new("xx".to_string());
        data.init();

        data.print_name();

        // shadow
        let data = move_data(data);

        // the addr which name_ptr references is changed after move
        // but addr is still valid here, after this function return
        // it will be drop
        data.print_name();
        data
    }

    fn mem_swap() {
        let mut data1 = SelfReference::new("hello".to_string());
        data1.init();

        let mut data2 = SelfReference::new("world".to_string());
        data2.init();

        data1.print_name();
        data2.print_name();

        std::mem::swap(&mut data1, &mut data2);
        data1.print_name();
        data2.print_name();
    }

    fn move_data(data: SelfReference) -> SelfReference {
        data
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
                while let Some(line) = r.next().await {
                    w.send(format!("got : {}", line?)).await.unwrap();
                }
                Ok::<_, anyhow::Error>(())
            });
        }
    }
}
