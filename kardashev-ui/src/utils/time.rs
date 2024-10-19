use std::{
    collections::VecDeque,
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
    time::Duration,
};

use futures::{
    FutureExt,
    StreamExt,
};
pub use web_time::Instant;

fn duration_to_millis(duration: Duration) -> u32 {
    duration.as_millis().try_into().expect("duration too long")
}

#[derive(Debug)]
pub struct Interval {
    inner: gloo_timers::future::IntervalStream,
}

impl Interval {
    fn new(period: Duration) -> Self {
        Self {
            inner: gloo_timers::future::IntervalStream::new(duration_to_millis(period)),
        }
    }

    pub async fn tick(&mut self) {
        self.inner.next().await.unwrap()
    }

    pub fn poll_tick(&mut self, cx: &mut Context) -> Poll<()> {
        self.inner.poll_next_unpin(cx).map(|result| result.unwrap())
    }
}

pub fn interval(period: Duration) -> Interval {
    Interval::new(period)
}

#[derive(Debug)]
pub struct Sleep {
    inner: gloo_timers::future::TimeoutFuture,
}

impl Sleep {
    fn new(duration: Duration) -> Sleep {
        Self {
            inner: gloo_timers::future::TimeoutFuture::new(duration_to_millis(duration)),
        }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.inner.poll_unpin(cx)
    }
}

pub fn sleep(duration: Duration) -> Sleep {
    Sleep::new(duration)
}

#[derive(Debug)]
pub struct TicksPerSecond {
    times: VecDeque<Instant>,
    num_past: usize,
}

impl TicksPerSecond {
    pub fn new(num_past: usize) -> Self {
        Self {
            times: VecDeque::with_capacity(num_past),
            num_past,
        }
    }

    pub fn push(&mut self, time: Instant) {
        if self.times.len() == self.num_past {
            self.times.pop_front();
        }
        self.times.push_back(time);
    }

    pub fn push_now(&mut self) {
        self.push(Instant::now());
    }

    pub fn clear(&mut self) {
        self.times.clear();
    }

    pub fn tps(&self) -> Option<f32> {
        (self.times.len() > 1).then(|| {
            self.times.len() as f32
                / self
                    .times
                    .front()
                    .unwrap()
                    .duration_since(*self.times.back().unwrap())
                    .as_secs_f32()
        })
    }
}
