//! A collection of modulators.

use crate::SAMPLE_DURATION;
use core::f32;
use micromath::F32Ext;

/// An audio node parameter modulator.
///
/// Includes both [envelopes] and [LFOs].
///
/// [envelopes]: https://en.wikipedia.org/wiki/Envelope_(music)
/// [LFOs]: https://en.wikipedia.org/wiki/Low-frequency_oscillation
pub trait Modulator {
    /// Get the modulator value at the given time (in samples).
    ///
    /// The time usually increases. It can go down if it wraps, which happens only
    /// if the audio plays for a very long time. Or it can be intentionally reset to 0.
    ///
    /// The time value is not sequential: 8 might be followed by 200.
    fn get(&self, now: u32) -> f32;
}

/// Emit first value before the given moment and the second value after.
pub struct Hold {
    v1: f32,
    v2: f32,
    time: u32,
}

impl Hold {
    #[must_use]
    pub fn new(v1: f32, v2: f32, time: u32) -> Self {
        Self { v1, v2, time }
    }
}

impl Modulator for Hold {
    fn get(&self, now: u32) -> f32 {
        if now < self.time {
            self.v1
        } else {
            self.v2
        }
    }
}

/// Linearly ramp up or cut down from one value to another on the given time interval.
pub struct Linear {
    start: f32,
    end: f32,
    start_at: u32,
    end_at: u32,
}

impl Linear {
    #[must_use]
    pub fn new(start: f32, end: f32, start_at: u32, end_at: u32) -> Self {
        Self {
            start,
            end,
            start_at,
            end_at,
        }
    }
}

impl Modulator for Linear {
    fn get(&self, now: u32) -> f32 {
        if now <= self.start_at {
            return self.start;
        }
        if now >= self.end_at {
            return self.end;
        }
        let duration = self.end_at.saturating_sub(self.start_at);
        if duration == 0 {
            return self.end;
        }
        let elapsed = now - self.start_at;
        let ratio = elapsed as f32 / duration as f32;
        self.start + (self.end - self.start) * ratio
    }
}

// Sine wave low-frequency oscillator.
pub struct Sine {
    s: f32,
    mid: f32,
    amp: f32,
}

impl Sine {
    /// TODO: make initial phase configurable.
    #[must_use]
    pub fn new(freq: f32, low: f32, high: f32) -> Self {
        let s = core::f32::consts::TAU * freq * SAMPLE_DURATION;
        let amp = (high - low) / 2.;
        let mid = low + amp;
        Self { s, mid, amp }
    }
}

impl Modulator for Sine {
    fn get(&self, now: u32) -> f32 {
        let s = F32Ext::sin(self.s * now as f32);
        self.mid + self.amp * s
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::*;

    fn assert_close(a: f32, b: f32) {
        let diff = a - b;
        assert!(diff < 0.00001, "{a} != {b}");
        assert!(diff > -0.00001, "{a} != {b}");
    }

    #[test]
    fn switch() {
        let lfo = Hold::new(2., 4., 10);
        assert_eq!(lfo.get(0), 2.);
        assert_eq!(lfo.get(6), 2.);
        assert_eq!(lfo.get(9), 2.);

        assert_eq!(lfo.get(10), 4.);
        assert_eq!(lfo.get(11), 4.);
        assert_eq!(lfo.get(12), 4.);
        assert_eq!(lfo.get(21), 4.);
        assert_eq!(lfo.get(100), 4.);
    }

    #[test]
    fn ramp_up() {
        let lfo = Linear::new(2., 4., 10, 20);
        assert_eq!(lfo.get(0), 2.);
        assert_eq!(lfo.get(8), 2.);
        assert_eq!(lfo.get(10), 2.);

        assert_eq!(lfo.get(20), 4.);
        assert_eq!(lfo.get(23), 4.);
        assert_eq!(lfo.get(100), 4.);

        assert_eq!(lfo.get(13), 2.6);
        assert_eq!(lfo.get(15), 3.);
        assert_eq!(lfo.get(17), 3.4);
    }

    #[test]
    fn cut_down() {
        let lfo = Linear::new(4., 2., 10, 20);
        assert_eq!(lfo.get(0), 4.);
        assert_eq!(lfo.get(8), 4.);
        assert_eq!(lfo.get(10), 4.);

        assert_eq!(lfo.get(20), 2.);
        assert_eq!(lfo.get(23), 2.);
        assert_eq!(lfo.get(100), 2.);

        assert_eq!(lfo.get(13), 3.4);
        assert_eq!(lfo.get(15), 3.);
        assert_eq!(lfo.get(17), 2.6);
    }

    #[test]
    fn sine() {
        const R: u32 = 44_100; // sample rate
        let lfo = Sine::new(1., -1., 1.);
        assert_eq!(lfo.get(0), 0.);
        assert!(lfo.get(1) > 0.);
        assert_eq!(lfo.get(R / 4), 1.);
        assert_close(lfo.get(R / 2), 0.);
        assert!(lfo.get(R / 2 + 1) < 0.);
        assert_eq!(lfo.get(R * 3 / 4), -1.);
        assert_close(lfo.get(R), 0.);
    }
}
