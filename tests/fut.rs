use std::{
    future::Future,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::Poll,
    thread,
};

//异步块中使用
#[test]
fn test_async_block() {
    let ret = laes::execute(async { 100 });
    assert_eq!(100, ret);
}

//同步函数或者方法调用异步方法中使用
#[test]
fn test_async_func() {
    //ten async function
    fn do_business() {
        //`await` is only allowed inside `async` functions and blocks
        //get_conn().await;
        let conn = laes::execute(get_conn());
    }

    struct Conn;
    //the async function
    async fn get_conn() -> Conn {
        //do something
        Conn
    }
}

//自定义future使用
#[test]
fn test_impl_future() {
    struct CustomeFuture(Arc<AtomicBool>, usize);

    impl Future for CustomeFuture {
        type Output = usize;
        fn poll(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            //do something
            self.1 += 1;
            // finish
            if self.0.load(Ordering::Acquire) {
                Poll::Ready(self.1)
            } else {
                if self.1 == 1 {
                    let v = Arc::clone(&self.0);
                    let waker = cx.waker().clone();
                    thread::spawn(move || {
                        v.store(true, Ordering::Release);
                        waker.wake();
                    });
                }
                Poll::Pending
            }
        }
    }
    let fut = CustomeFuture(Arc::new(AtomicBool::new(false)), 0);
    assert_eq!(2, laes::execute(fut));
}
