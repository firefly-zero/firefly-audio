//! A collection of Low-Frequency Oscillators.

/// A Low-Frequency Oscillator. Used for modulation.
pub trait LFO {
    fn get(&self, now: u32) -> f32;
}

/// Emit first value before the given moment and the second value after.
pub struct Switch {
    v1: f32,
    v2: f32,
    time: u32,
}

impl Switch {
    pub fn new(v1: f32, v2: f32, time: u32) -> Self {
        Self { v1, v2, time }
    }
}

impl LFO for Switch {
    fn get(&self, now: u32) -> f32 {
        if now < self.time {
            self.v1
        } else {
            self.v2
        }
    }
}

pub struct RampUp {
    start: f32,
    end: f32,
    start_at: u32,
    end_at: u32,
}

impl RampUp {
    pub fn new(start: f32, end: f32, start_at: u32, end_at: u32) -> Self {
        Self {
            start,
            end,
            start_at,
            end_at,
        }
    }
}

impl LFO for RampUp {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switch() {
        let lfo = Switch::new(2., 4., 10);
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
        let lfo = RampUp::new(2., 4., 10, 20);
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
}
