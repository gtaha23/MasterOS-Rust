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
            // Wait for the next timer interrupt before re-checking, instead of
            // immediately re-queuing and busy-spinning at full CPU speed.
            // Any interrupt (timer or otherwise) will resume execution here.
            x86_64::instructions::hlt();
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

pub async fn sleep(ms: u64) {
    SleepFuture::new(ms).await;
}