//! Different types of delay for retryable operations.

use std::fmt::{Debug, Formatter, Error as FmtError};
use std::time::Duration;

use rand::distributions::{IndependentSample, Range as RandRange};
use rand::{Closed01, random, ThreadRng, thread_rng};

/// Each retry increases the delay since the last exponentially.
#[derive(Debug)]
pub struct Exponential {
    base: u64,
    current: u64,
}

impl Exponential {
    /// Create a new `Exponential` using the given millisecond duration as the initial delay.
    pub fn from_millis(base: u64) -> Self {
        Exponential {
            base: base,
            current: base,
        }
    }
}

impl Iterator for Exponential {
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        let duration = Duration::from_millis(self.current);

        self.current = self.current * self.base;

        Some(duration)
    }
}

/// Each retry uses a delay which is the sum of the two previous delays.
///
/// Depending on the problem at hand, a fibonacci delay strategy might
/// perform better and lead to better throughput than the `Exponential`
/// strategy.
///
/// See ["A Performance Comparison of Different Backoff Algorithms under Different Rebroadcast Probabilities for MANETs."](http://www.comp.leeds.ac.uk/ukpew09/papers/12.pdf)
/// for more details.
#[derive(Debug)]
pub struct Fibonacci {
    curr: u64,
    next: u64
}

impl Fibonacci {
    /// Create a new `Fibonacci` using the given duration in milliseconds.
    pub fn from_millis(millis: u64) -> Fibonacci {
        Fibonacci{curr: millis, next: millis}
    }
}

impl Iterator for Fibonacci {
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        let next_next = self.curr + self.next;
        let duration = Duration::from_millis(self.curr);

        self.curr = self.next;
        self.next = next_next;

        Some(duration)
    }
}

#[test]
fn fibonacci() {
    let mut iter = Fibonacci::from_millis(10);
    assert_eq!(iter.next(), Some(Duration::from_millis(10)));
    assert_eq!(iter.next(), Some(Duration::from_millis(10)));
    assert_eq!(iter.next(), Some(Duration::from_millis(20)));
    assert_eq!(iter.next(), Some(Duration::from_millis(30)));
    assert_eq!(iter.next(), Some(Duration::from_millis(50)));
    assert_eq!(iter.next(), Some(Duration::from_millis(80)));
}

/// Each retry uses a fixed delay.
#[derive(Debug)]
pub struct Fixed {
    duration: Duration,
}

impl Fixed {
    /// Create a new `Fixed` using the given duration in milliseconds.
    pub fn from_millis(millis: u64) -> Self {
        Fixed {
            duration: Duration::from_millis(millis),
        }
    }
}

impl Iterator for Fixed {
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        Some(self.duration)
    }
}

/// Each retry happens immediately without any delay.
#[derive(Debug)]
pub struct NoDelay;

impl Iterator for NoDelay {
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        Some(Duration::default())
    }
}

/// Each retry uses a duration randomly chosen from a range.
pub struct Range {
    range: RandRange<u64>,
    rng: ThreadRng,
}

impl Range {
    /// Create a new `Range` between the given millisecond durations.
    pub fn from_millis(minimum: u64, maximum: u64) -> Self {
        Range {
            range: RandRange::new(minimum, maximum),
            rng: thread_rng(),
        }
    }
}

impl Iterator for Range {
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        Some(Duration::from_millis(self.range.ind_sample(&mut self.rng)))
    }
}

impl Debug for Range {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "Range {{ range: RandRange<u64>, rng: ThreadRng }}")
    }
}

/// Apply full random jitter to a duration.
pub fn jitter(duration: Duration) -> Duration {
    let Closed01(jitter) = random::<Closed01<f64>>();
    let secs = ((duration.as_secs() as f64) * jitter).ceil() as u64;
    let nanos = ((duration.subsec_nanos() as f64) * jitter).ceil() as u32;
    Duration::new(secs, nanos)
}
