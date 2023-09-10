use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::Instant;

use futures::task::AtomicWaker;

pub struct Delay {
    when: Instant,
    waker: Option<Arc<Mutex<Waker>>>,
}

impl Delay {
    pub fn new(when: Instant) -> Self {
        Self { when, waker: None }
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if let Some(waker) = &self.waker {
            let mut waker = waker.lock().unwrap();
            if !waker.will_wake(cx.waker()) {
                println!("replace waker, cx: {:?}", cx);
                *waker = cx.waker().clone();
            } else {
                println!("do not replace, cx: {:?}", cx);
            }
        } else {
            println!("no waker");
            let when = self.when;
            let waker = Arc::new(Mutex::new(cx.waker().clone()));
            self.waker = Some(waker.clone());

            thread::spawn(move || {
                let now = Instant::now();

                if now < when {
                    thread::sleep(when - now);
                }
                let waker = waker.lock().unwrap();
                waker.wake_by_ref();
            });
        }

        if Instant::now() >= self.when {
            println!("ready");
            Poll::Ready(())
        } else {
            println!("pending");
            Poll::Pending
        }
    }
}

pub struct DelayAtomic {
    shared: Arc<Shared>,
}

struct Shared {
    when: Instant,
    waker: AtomicWaker,
}

impl DelayAtomic {
    #[allow(dead_code)]
    pub fn new(when: Instant) -> Self {
        let shared = Shared {
            when,
            waker: AtomicWaker::new(),
        };
        Self {
            shared: Arc::new(shared),
        }
    }
}

impl Future for DelayAtomic {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        self.shared.waker.register(cx.waker());
        let shared = Arc::clone(&self.shared);

        thread::spawn(move || {
            let now = Instant::now();

            if now < shared.when {
                thread::sleep(shared.when - now);
            }
            shared.waker.wake();
        });

        if Instant::now() >= self.shared.when {
            println!("ready");
            Poll::Ready(())
        } else {
            println!("pending");
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{future::poll_fn, time::Duration};

    use tokio::time::sleep;

    use super::*;

    #[tokio::test]
    async fn test_atomic_waker() {
        let when = Instant::now() + Duration::from_secs(1);
        let mut delay = Some(DelayAtomic::new(when));

        poll_fn(move |cx| {
            let mut delay = delay.take().unwrap();
            let res = Pin::new(&mut delay).poll(cx);
            assert!(res.is_pending());
            tokio::spawn(async move {
                delay.await;
            });

            Poll::Ready(())
        })
        .await;

        sleep(Duration::from_secs(2)).await
    }
}
