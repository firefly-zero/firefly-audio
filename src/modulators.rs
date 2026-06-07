//! A collection of modulators.

use crate::SAMPLE_DURATION;
use micromath::F32Ext;

/// An audio node parameter modulator.
///
/// Includes both [envelopes] and [LFOs].
///
/// All modulators produce value between 0 and 1.
/// The range of the modulated parameter value is adjusted via
/// `low` and `high` args of [`Node::modulate`][crate::Node::modulate].
///
/// [envelopes]: https://en.wikipedia.org/wiki/Envelope_(music)
/// [LFOs]: https://en.wikipedia.org/wiki/Low-frequency_oscillation
pub trait Modulator {
    /// Get the modulator value at the given time (in samples).
    ///
    /// The value is guaranteed to be in the range from 0 to 1 (inclusive).
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
    time: u32,
}

impl Hold {
    #[must_use]
    pub const fn new(time: u32) -> Self {
        Self { time }
    }
}

impl Modulator for Hold {
    fn get(&self, now: u32) -> f32 {
        if now < self.time {
            0.
        } else {
            1.
        }
    }
}

/// Linearly ramp up (or cut down) from one value to another on the given time interval.
pub struct Linear {
    start_at: u32,
    end_at: u32,
}

impl Linear {
    #[must_use]
    pub const fn new(start_at: u32, end_at: u32) -> Self {
        Self { start_at, end_at }
    }
}

impl Modulator for Linear {
    fn get(&self, now: u32) -> f32 {
        if now <= self.start_at {
            return 0.;
        }
        if now >= self.end_at {
            return 1.;
        }
        let duration = self.end_at.saturating_sub(self.start_at);
        if duration == 0 {
            return 1.;
        }
        let elapsed = now - self.start_at;
        elapsed as f32 / duration as f32
    }
}

/// Sine wave low-frequency oscillator.
pub struct Sine {
    s: f32,
}

impl Sine {
    #[must_use]
    pub const fn new(freq: f32) -> Self {
        let s = core::f32::consts::TAU * freq * SAMPLE_DURATION;
        Self { s }
    }
}

impl Modulator for Sine {
    #[expect(clippy::manual_midpoint)]
    fn get(&self, now: u32) -> f32 {
        let s = F32Ext::sin(self.s * now as f32);
        (s + 1.) / 2.
    }
}

/// Pulse wave low-frequency oscillator.
///
/// Pulse wave is a slight generalization of square wave where the duty cycle
/// (the ratio of time between pulse and the wave period) can be configured.
///
/// For switching from one value to another only once, use [`Hold`] instead.
pub struct Pulse {
    period: u32,
    pulse_t: u32,
}

impl Pulse {
    #[must_use]
    pub const fn new(v1_t: u32, v2_t: u32) -> Self {
        Self {
            period: v1_t + v2_t,
            pulse_t: v1_t,
        }
    }

    #[must_use]
    pub const fn new_square(period: u32) -> Self {
        Self {
            period,
            pulse_t: period / 2,
        }
    }
}

impl Modulator for Pulse {
    fn get(&self, now: u32) -> f32 {
        let step = now % self.period;
        if step < self.pulse_t {
            0.
        } else {
            1.
        }
    }
}

/// Triangle wave low-frequency oscillator.
///
/// The period of up and down cycle can be configured independently
/// allowing for assymetric wave. With both equal, it is the classic [triangle wave].
/// With the down period zero, it is a [sawtooth wave].
///
/// [triangle wave]: https://en.wikipedia.org/wiki/Triangle_wave
/// [sawtooth wave]: https://en.wikipedia.org/wiki/Sawtooth_wave
pub struct Triangle {
    t1: u32,
    t2: u32,
}

impl Triangle {
    #[must_use]
    pub const fn new(t1: u32, t2: u32) -> Self {
        Self { t1, t2 }
    }

    #[must_use]
    pub const fn new_symmetric(period: u32) -> Self {
        Self {
            t1: period / 2,
            t2: period / 2,
        }
    }

    #[must_use]
    pub const fn new_sawtooth(period: u32) -> Self {
        Self { t1: period, t2: 0 }
    }
}

impl Modulator for Triangle {
    fn get(&self, now: u32) -> f32 {
        let step = now % (self.t1 + self.t2);
        if step < self.t1 {
            step as f32 / self.t1 as f32
        } else {
            1. - (step - self.t1) as f32 / self.t2 as f32
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
pub struct Adsr {
    attack: u32,
    decay: u32,
    sustain: u32,
    sustain_level: f32,
    release: u32,
}

impl Adsr {
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

impl Modulator for Adsr {
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
        let lfo = Hold::new(10);
        assert_eq!(lfo.get(0), 0.);
        assert_eq!(lfo.get(6), 0.);
        assert_eq!(lfo.get(9), 0.);

        assert_eq!(lfo.get(10), 1.);
        assert_eq!(lfo.get(11), 1.);
        assert_eq!(lfo.get(12), 1.);
        assert_eq!(lfo.get(21), 1.);
        assert_eq!(lfo.get(100), 1.);
    }

    #[test]
    fn ramp_up() {
        let lfo = Linear::new(10, 20);
        assert_eq!(lfo.get(0), 0.);
        assert_eq!(lfo.get(8), 0.);
        assert_eq!(lfo.get(10), 0.);

        assert_eq!(lfo.get(20), 1.);
        assert_eq!(lfo.get(23), 1.);
        assert_eq!(lfo.get(100), 1.);

        assert_eq!(lfo.get(13), 0.3);
        assert_eq!(lfo.get(15), 0.5);
        assert_eq!(lfo.get(17), 0.7);
    }

    #[test]
    fn sine() {
        const R: u32 = 44_100; // sample rate
        let lfo = Sine::new(1.);
        assert_eq!(lfo.get(0), 0.5);
        assert!(lfo.get(1) > 0.);
        assert_eq!(lfo.get(R / 4), 1.);
        assert_close(lfo.get(R / 2), 0.5);
        assert!(lfo.get(R / 2 + 1) < 0.5);
        assert_eq!(lfo.get(R * 3 / 4), 0.);
        assert_close(lfo.get(R), 0.5);
    }

    #[test]
    fn pulse() {
        let lfo = Pulse::new(5, 5);
        assert_eq!(lfo.get(0), 0.);
        assert_eq!(lfo.get(1), 0.);
        assert_eq!(lfo.get(4), 0.);

        assert_eq!(lfo.get(5), 1.);
        assert_eq!(lfo.get(6), 1.);
        assert_eq!(lfo.get(9), 1.);

        assert_eq!(lfo.get(10), 0.);
        assert_eq!(lfo.get(11), 0.);
        assert_eq!(lfo.get(12), 0.);
        assert_eq!(lfo.get(14), 0.);

        assert_eq!(lfo.get(15), 1.);
        assert_eq!(lfo.get(16), 1.);
    }

    #[test]
    fn triangle() {
        let lfo = Triangle::new(5, 5);
        assert_eq!(lfo.get(0), 0.);
        assert_eq!(lfo.get(3), 0.6);
        assert_eq!(lfo.get(5), 1.);
        assert_eq!(lfo.get(7), 0.6);
        assert_eq!(lfo.get(10), 0.);
    }

    #[test]
    fn adsr() {
        let lfo = Adsr::new(10, 20, 30, 0.5, 40);
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
