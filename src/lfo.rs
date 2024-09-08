//! A collection of Low-Frequency Oscillators.

/// A Low-Frequency Oscillator. Used for modulation.
pub trait LFO {
    fn reset(&mut self);
    fn get(&mut self, now: u32) -> Option<f32>;
}

/// Emit the given value once at the given time.
pub struct Once {
    value: f32,
    time: u32,
    emitted: bool,
}

impl Once {
    pub fn new(value: f32, time: u32) -> Self {
        Self {
            value,
            time,
            emitted: false,
        }
    }
}

impl LFO for Once {
    fn reset(&mut self) {
        self.emitted = false
    }

    fn get(&mut self, now: u32) -> Option<f32> {
        if self.emitted {
            return None;
        }
        if now < self.time {
            return None;
        }
        self.emitted = true;
        Some(self.value)
    }
}
