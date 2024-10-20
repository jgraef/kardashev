use std::{
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
    measurement_start: Option<Instant>,
    measurement_duration: Duration,
    measurement: Option<f32>,
    num_ticks: usize,
}

impl TicksPerSecond {
    pub fn new(measurement_duration: Duration) -> Self {
        Self {
            measurement_start: None,
            measurement_duration,
            measurement: None,
            num_ticks: 0,
        }
    }

    pub fn push(&mut self, time: Instant) {
        if let Some(measurement_start) = self.measurement_start {
            let measurement_duration = time.duration_since(measurement_start);
            if measurement_duration > self.measurement_duration {
                self.measurement =
                    Some((self.num_ticks as f32) / measurement_duration.as_secs_f32());
                self.measurement_start = Some(time);
                self.num_ticks = 1;
            }
            else {
                self.num_ticks += 1;
            }
        }
        else {
            self.measurement_start = Some(time);
            self.num_ticks = 1;
        }
    }

    pub fn push_now(&mut self) {
        self.push(Instant::now());
    }

    pub fn clear(&mut self) {
        self.measurement_start = None;
    }

    pub fn tps(&self) -> Option<f32> {
        self.measurement
    }
}
