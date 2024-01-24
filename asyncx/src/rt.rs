use crossbeam::channel;
use futures::task::ArcWake;
use std::future::Future;
use std::mem::{forget, ManuallyDrop};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::thread;

struct Runtime {
    scheduled: channel::Receiver<Arc<Task>>,
    executor: channel::Sender<Arc<Task>>,
}

struct Task {
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    executor: channel::Sender<Arc<Task>>,
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let _ = arc_self.executor.send(arc_self.clone());
    }
}

impl Task {
    fn poll(self: Arc<Self>) {
        // let waker = task::waker(self.clone());
        let self_clone = self.clone();
        let waker = waker_fn(move || {
            let _ = self_clone.executor.send(self_clone.clone());
        });
        let mut cx = Context::from_waker(&waker);

        let mut future = self.future.lock().unwrap();
        let rt = future.as_mut().poll(&mut cx);
        println!("poll: {:?}", rt);
    }

    fn spawn<F>(future: F, sender: &channel::Sender<Arc<Task>>)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
            executor: sender.clone(),
        });
        let _ = sender.send(task);
    }

    fn spawn_blocking<T, F>(closure: F) -> SpawnBlocking<T>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        let inner = Arc::new(Mutex::new(Shared {
            val: None,
            waker: None,
        }));

        thread::spawn({
            let inner = inner.clone();

            move || {
                let val = closure();

                let maybe_waker = {
                    let mut guard = inner.lock().unwrap();
                    guard.val = Some(val);
                    guard.waker.take()
                };

                if let Some(waker) = maybe_waker {
                    waker.wake()
                }
            }
        });

        SpawnBlocking(inner)
    }
}

struct SpawnBlocking<T>(Arc<Mutex<Shared<T>>>);

struct Shared<T> {
    val: Option<T>,
    waker: Option<Waker>,
}

impl<T> Future for SpawnBlocking<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut guard = self.0.lock().unwrap();
        if let Some(val) = guard.val.take() {
            return Poll::Ready(val);
        }

        guard.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}

impl Runtime {
    fn new() -> Self {
        let (tx, rx) = channel::unbounded();
        Self {
            scheduled: rx,
            executor: tx,
        }
    }

    fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Task::spawn(future, &self.executor)
    }

    fn run(self) {
        thread::spawn(move || {
            while let Ok(task) = self.scheduled.recv() {
                task.poll()
            }
        });
    }
}

pub fn waker_fn<F: Fn() + Send + Sync + 'static>(f: F) -> Waker {
    let raw = Arc::into_raw(Arc::new(f)) as *const ();
    let vtable = &WakerFn::<F>::VTABLE;
    unsafe { Waker::from_raw(RawWaker::new(raw, vtable)) }
}

struct WakerFn<F>(F);

impl<F: Fn() + Send + Sync + 'static> WakerFn<F> {
    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        Self::clone_waker,
        Self::wake,
        Self::wake_by_ref,
        Self::drop_waker,
    );

    unsafe fn clone_waker(ptr: *const ()) -> RawWaker {
        let arc = ManuallyDrop::new(Arc::from_raw(ptr as *const F));
        forget(arc.clone());
        RawWaker::new(ptr, &Self::VTABLE)
    }

    unsafe fn wake(ptr: *const ()) {
        let arc = Arc::from_raw(ptr as *const F);
        (arc)();
    }

    unsafe fn wake_by_ref(ptr: *const ()) {
        let arc = ManuallyDrop::new(Arc::from_raw(ptr as *const F));
        (arc)();
    }

    unsafe fn drop_waker(ptr: *const ()) {
        drop(Arc::from_raw(ptr as *const F));
    }
}

#[cfg(test)]
mod tests {
    use std::{
        thread::sleep,
        time::{Duration, Instant},
    };

    use crate::delay::Delay;

    use super::{Runtime, Task};

    #[test]
    fn test_spawn() {
        let mut rt = Runtime::new();

        rt.spawn(async {
            let fut = Delay::new(Instant::now() + Duration::from_secs(2));
            fut.await;
        });

        rt.run();
        sleep(Duration::from_secs(3))
    }

    #[test]
    fn test_spawn_blocking() {
        let mut rt = Runtime::new();

        let fut1 = Task::spawn_blocking(|| {
            sleep(Duration::from_secs(2));
            1
        });

        let fut2 = Task::spawn_blocking(|| {
            sleep(Duration::from_secs(2));
            2
        });

        let fut3 = Task::spawn_blocking(|| {
            sleep(Duration::from_secs(2));
            3
        });

        rt.spawn(async {
            let v1 = fut1.await;
            let v2 = fut2.await;
            let v3 = fut3.await;

            assert_eq!((1, 2, 3), (v1, v2, v3));
        });

        rt.run();
        sleep(Duration::from_secs(3))
    }
}
