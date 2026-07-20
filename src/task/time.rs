use core::{future::Future,pin::Pin,task::{Context, Poll}};
use crate::pit::uptime_ms;

pub struct SleepFuture {
    end_time_ms: u64,
}

impl SleepFuture {
    pub fn new(duration_ms: u64) -> Self {
        Self {
            end_time_ms: uptime_ms() + duration_ms,
        }
    }
}

impl Future for SleepFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if uptime_ms() >= self.end_time_ms {
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

pub async fn sleep(ms: u64) {
    SleepFuture::new(ms).await;
}
