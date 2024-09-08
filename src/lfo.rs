//! A collection of Low-Frequency Oscillators.

/// A Low-Frequency Oscillator. Used for modulation.
pub trait LFO {
    fn get(&mut self, now: u32) -> f32;
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
    fn get(&mut self, now: u32) -> f32 {
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
    fn get(&mut self, now: u32) -> f32 {
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
        let ratio = (duration - now) as f32 / duration as f32;
        (self.end - self.start) * ratio
    }
}
