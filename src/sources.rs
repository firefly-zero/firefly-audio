use crate::*;
use alloc::vec::Vec;

/// A sound source that is always stopped.
pub struct Empty {}

impl Empty {
    pub fn new() -> Self {
        Self {}
    }
}

impl Processor for Empty {
    fn process_children(&mut self, _cn: &mut Vec<Node>) -> Option<Frame> {
        None
    }
}

/// A sound source that produces zeros. Forever.
pub struct Zero {}

impl Zero {
    pub fn new() -> Self {
        Self {}
    }
}

impl Processor for Zero {
    fn process_children(&mut self, _cn: &mut Vec<Node>) -> Option<Frame> {
        Some(Frame::zero())
    }
}

/// Generate sine wave oscillator.
pub struct Sine {
    freq: f32,
    phase: f32,
    initial_phase: f32,
}

impl Sine {
    pub fn new(freq: f32, phase: f32) -> Self {
        Self {
            freq,
            phase,
            initial_phase: phase,
        }
    }
}

impl Processor for Sine {
    fn reset(&mut self) {
        self.phase = self.initial_phase;
    }

    fn process_children(&mut self, _cn: &mut Vec<Node>) -> Option<Frame> {
        let mut element = [0f32; 8];
        let mut phase = self.phase;
        for sample in &mut element {
            *sample = phase;
            phase += self.freq * SAMPLE_DURATION;
            phase -= floor(phase);
        }
        self.phase = phase;
        let element = Sample::new(element);
        let s = element * Sample::TAU;
        let s = s.sin();
        Some(Frame::mono(s))
    }
}

fn floor(x: f32) -> f32 {
    let mut res = (x as i32) as f32;
    if x < res {
        res -= 1.0;
    }
    res
}

/// Generate a white noise
pub struct Noise {
    seed: i32,
}

impl Noise {
    pub fn new(seed: i32) -> Self {
        Self { seed }
    }
}

impl Processor for Noise {
    fn process_children(&mut self, _cn: &mut Vec<Node>) -> Option<Frame> {
        let mut samples = [0f32; 8];
        for sample in samples.iter_mut() {
            let mut x = self.seed;
            if x == 0 {
                x = 1;
            }
            // xorshift RNG algorithm
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            self.seed = x;
            *sample = x as f32;
        }
        let samples = Sample::new(samples);
        let samples = samples / u32::MAX as f32;
        Some(Frame::mono(samples))
    }
}
