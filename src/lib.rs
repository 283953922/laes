
//!一个简单的轻量异步执行器，该执行器主要用于在同步环境中调用异步代码。
//! 此异步异步执行器主要适合:同步异步块,同步异步方法,同步自定义Future。


#![forbid(unsafe_code)]
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Condvar, Mutex, PoisonError},
    task::{Context, Poll, Wake, Waker},
};

#[derive(Default)]
struct CWaker {
    cond: Condvar,
    notified: Mutex<bool>,
}
impl CWaker {
    fn park(&self) {
        let mut guard = self.notified.lock().unwrap_or_else(PoisonError::into_inner);
        //直到被通知(唤醒)
        while !*guard {
            guard = self
                .cond
                .wait(guard)
                .unwrap_or_else(PoisonError::into_inner);
        }
        *guard = false;
    }
}

impl Wake for CWaker {
    fn wake(self: std::sync::Arc<Self>) {
        self.wake_by_ref()
    }
    fn wake_by_ref(self: &std::sync::Arc<Self>) {
        //被通知（唤醒）
        *self.notified.lock().unwrap_or_else(PoisonError::into_inner) = true;
        self.cond.notify_one();
    }
}
///Block on [Future]:future将在当前线程中被执行。
/// 
pub fn execute<T>(f: impl Future<Output = T>) -> T {
    poll(std::pin::pin!(f))
}

//poll future直到完成
fn poll<T>(mut pinned: Pin<&mut dyn Future<Output = T>>) -> T {
    let wake = Arc::new(CWaker::default());
    let waker = Waker::from(Arc::clone(&wake));
    let mut cx = Context::from_waker(&waker);
    loop {
        match pinned.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => wake.park(),
        }
    }
}
