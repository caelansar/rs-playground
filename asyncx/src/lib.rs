#![feature(negative_impls)]
#![feature(string_leak)]

mod cancel_decorator;
mod time_decorator;

use std::time::Duration;

use tokio::time::sleep;

#[allow(dead_code)]
async fn request() -> String {
    sleep(Duration::from_secs(2)).await;
    "content".to_string()
}

#[cfg(test)]
mod tests {
    use crate::{cancel_decorator::spawn, request, time_decorator};
    use anyhow::{self, Result};
    use futures::{SinkExt, StreamExt};
    use std::{cell::RefCell, pin::Pin, sync::Arc, time::Duration};
    use tokio::{
        net::TcpListener,
        sync::{mpsc::channel, Mutex},
        time::sleep,
    };
    use tokio_stream::wrappers::ReceiverStream;
    use tokio_util::codec::{Framed, LinesCodec};

    #[tokio::test]
    async fn time_decorator_should_work() {
        let task = request();
        let td = time_decorator::TimeDecorator::new(task);

        let (data, elapsed) = td.await;

        assert_eq!("content", data);
        assert!(elapsed >= Duration::from_secs(2));
    }

    async fn cancelled_task(data: Arc<Mutex<i32>>) {
        sleep(Duration::from_millis(100)).await;
        println!("in cancelled_task");
        let mut lock = data.lock().await;
        *lock += 1;
    }

    #[tokio::test]
    async fn cancel_decorator_should_work() {
        let data1 = Arc::new(Mutex::new(0));
        let data2 = data1.clone();
        let v = spawn(cancelled_task(data1));
        drop(v);
        sleep(Duration::from_millis(150)).await;

        let lock = data2.lock().await;
        assert_eq!(0, *lock);
    }

    #[tokio::test]
    async fn mpsc_should_works() {
        let (tx, rx) = channel(10);

        tx.send(1).await.unwrap();
        tx.send(100).await.unwrap();
        drop(tx);

        let mut stream = ReceiverStream::new(rx);
        while let Some(data) = stream.next().await {
            println!("recv stream: {}", data)
        }
    }

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

        fn init_pinned(self: Pin<&mut Self>) {
            let name_ptr = &self.name as *const String;
            // SAFETY: won't move data since SelfReference is !Unpin
            let s = unsafe { self.get_unchecked_mut() };
            s.name_ptr = name_ptr;
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

    impl !Unpin for SelfReference {}

    #[test]
    fn test_self_reference() {
        let data = move_ptr();
        println!("data: {:?}", data);

        // ERROR
        // data.print_name();
        println!("memory swap");
        mem_swap();
    }

    #[test]
    fn test_pinned_self_reference() {
        move_ptr_pinned();
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

    fn move_ptr_pinned() {
        let mut data = SelfReference::new("aa".to_string());
        let mut data = unsafe { Pin::new_unchecked(&mut data) };

        SelfReference::init_pinned(data.as_mut());

        data.as_ref().print_name();

        let _ = move_pinned(data.as_mut());

        data.as_ref().print_name();
    }

    fn mem_swap() {
        let mut data1 = SelfReference::new("hello".to_string());
        data1.init();

        let mut data2 = SelfReference::new("world".to_string());
        data2.init();

        println!("before swap");
        data1.print_name();
        data2.print_name();

        std::mem::swap(&mut data1, &mut data2);

        println!("after swap");
        data1.print_name();
        data2.print_name();
    }

    fn move_pinned(data: Pin<&mut SelfReference>) -> Pin<&mut SelfReference> {
        data
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
