//! A collection of modulators.

use crate::SAMPLE_DURATION;
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
///
/// For oscillating between two values, use [`Pulse`] instead.
pub struct Hold {
    v1: f32,
    v2: f32,
    time: u32,
}

impl Hold {
    #[must_use]
    pub const fn new(v1: f32, v2: f32, time: u32) -> Self {
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

/// Linearly ramp up (or cut down) from one value to another on the given time interval.
pub struct Linear {
    start: f32,
    end: f32,
    start_at: u32,
    end_at: u32,
}

impl Linear {
    #[must_use]
    pub const fn new(start: f32, end: f32, start_at: u32, end_at: u32) -> Self {
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
        (self.end - self.start).mul_add(ratio, self.start)
    }
}

/// Sine wave low-frequency oscillator.
pub struct Sine {
    s: f32,
    mid: f32,
    amp: f32,
}

impl Sine {
    // TODO: make initial phase configurable.
    #[must_use]
    pub const fn new(freq: f32, low: f32, high: f32) -> Self {
        let s = core::f32::consts::TAU * freq * SAMPLE_DURATION;
        let amp = (high - low) / 2.;
        let mid = low + amp;
        Self { s, mid, amp }
    }
}

impl Modulator for Sine {
    fn get(&self, now: u32) -> f32 {
        let s = F32Ext::sin(self.s * now as f32);
        self.amp.mul_add(s, self.mid)
    }
}

/// Pulse wave low-frequency oscillator.
///
/// Pulse wave is a slight generalization of square wave where the duty cycle
/// (the ratio of time between pulse and the wave period) can be configured.
///
/// For switching from one value to another only once, use [`Hold`] instead.
pub struct Pulse {
    v1: f32,
    v2: f32,
    period: u32,
    #[expect(clippy::struct_field_names)]
    pulse_t: u32,
}

impl Pulse {
    #[must_use]
    pub const fn new(v1: f32, v2: f32, v1_t: u32, v2_t: u32) -> Self {
        Self {
            v1,
            v2,
            period: v1_t + v2_t,
            pulse_t: v1_t,
        }
    }

    #[must_use]
    pub const fn new_square(v1: f32, v2: f32, period: u32) -> Self {
        Self {
            v1,
            v2,
            period,
            pulse_t: period / 2,
        }
    }
}

impl Modulator for Pulse {
    fn get(&self, now: u32) -> f32 {
        let step = now % self.period;
        if step < self.pulse_t {
            self.v1
        } else {
            self.v2
        }
    }
}

/// Attack-decay-sustain-release ([ADSR]) envelope.
///
/// The output:
///
/// 1. jumps from 0 to 1 during `attack`,
/// 2. then goes to `attack_level` during `decay`,
/// 3. holds it for `sustain`,
/// 4. and goes back to 0 during `release`.
///
/// An equivalent of chaining 4 [`Linear`] modulators.
///
/// [ADSR]: https://en.wikipedia.org/wiki/Envelope_(music)#ADSR
pub struct ADSR {
    attack: u32,
    decay: u32,
    sustain: u32,
    sustain_level: f32,
    release: u32,
}

impl ADSR {
    #[must_use]
    pub const fn new(
        attack: u32,
        decay: u32,
        sustain: u32,
        sustain_level: f32,
        release: u32,
    ) -> Self {
        Self {
            attack,
            decay,
            sustain,
            sustain_level,
            release,
        }
    }
}

impl Modulator for ADSR {
    fn get(&self, now: u32) -> f32 {
        if now <= self.attack {
            // Going up to 1.
            now as f32 / self.attack as f32
        } else if now <= self.decay {
            // Going down from 1 to sustain_level.
            let ratio = (self.decay - now) as f32 / (self.decay - self.attack) as f32;
            (1. - self.sustain_level).mul_add(ratio, self.sustain_level)
        } else if now <= self.sustain {
            // Keeping sustain level until release.
            self.sustain_level
        } else if now <= self.release {
            // Going down from sustain_level to 0.
            let ratio = (self.release - now) as f32 / (self.release - self.sustain) as f32;
            self.sustain_level * ratio
        } else {
            // Keeping zero forever.
            0.
        }
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

    #[test]
    fn pulse() {
        let lfo = Pulse::new(2., 4., 5, 5);
        assert_eq!(lfo.get(0), 2.);
        assert_eq!(lfo.get(1), 2.);
        assert_eq!(lfo.get(4), 2.);

        assert_eq!(lfo.get(5), 4.);
        assert_eq!(lfo.get(6), 4.);
        assert_eq!(lfo.get(9), 4.);

        assert_eq!(lfo.get(10), 2.);
        assert_eq!(lfo.get(11), 2.);
        assert_eq!(lfo.get(12), 2.);
        assert_eq!(lfo.get(14), 2.);

        assert_eq!(lfo.get(15), 4.);
        assert_eq!(lfo.get(16), 4.);
    }

    #[test]
    fn adsr() {
        let lfo = ADSR::new(10, 20, 30, 0.5, 40);
        assert_eq!(lfo.get(0), 0.);
        assert_eq!(lfo.get(5), 0.5);
        assert_eq!(lfo.get(10), 1.);
        assert_eq!(lfo.get(15), 0.75);
        assert_eq!(lfo.get(20), 0.5);
        assert_eq!(lfo.get(21), 0.5);
        assert_eq!(lfo.get(30), 0.5);
        assert_eq!(lfo.get(35), 0.25);
        assert_eq!(lfo.get(40), 0.);
        assert_eq!(lfo.get(41), 0.);
        assert_eq!(lfo.get(50), 0.);
    }
}
