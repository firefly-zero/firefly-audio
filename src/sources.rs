//! Processors that have 0 inputs and produce new values.
//!
//! Includes oscillators, file readers, audio samples, etc.
use crate::*;
use alloc::vec::Vec;
use micromath::F32Ext;

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

/// Sine wave oscillator.
pub struct Sine {
    freq: f32,
    phase: f32,
    initial_phase: f32,
}

impl Sine {
    // TODO: drop phase support
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
            phase -= F32Ext::floor(phase);
        }
        self.phase = phase;
        let element = Sample::new(element);
        let s = element * Sample::TAU;
        let s = s.sin();
        Some(Frame::mono(s))
    }
}

/// Square wave oscillator.
pub struct Square {
    freq: f32,
    phase: f32,
    initial_phase: f32,
}

impl Square {
    pub fn new(freq: f32, phase: f32) -> Self {
        Self {
            freq,
            phase,
            initial_phase: phase,
        }
    }
}

impl Processor for Square {
    fn reset(&mut self) {
        self.phase = self.initial_phase;
    }

    fn process_children(&mut self, _cn: &mut Vec<Node>) -> Option<Frame> {
        let mut samples = [0f32; 8];
        let mut phase = self.phase;
        for sample in &mut samples {
            let dec = phase - F32Ext::floor(phase);
            *sample = if dec >= 0.5 { 1. } else { 0. };
            phase += self.freq * SAMPLE_DURATION;
            phase -= F32Ext::floor(phase);
        }
        self.phase = phase;
        let s = Sample::new(samples);
        Some(Frame::mono(s))
    }
}

/// Sawtooth wave oscillator.
pub struct Sawtooth {
    freq: f32,
    phase: f32,
    initial_phase: f32,
}

impl Sawtooth {
    pub fn new(freq: f32, phase: f32) -> Self {
        Self {
            freq,
            phase,
            initial_phase: phase,
        }
    }
}

impl Processor for Sawtooth {
    fn reset(&mut self) {
        self.phase = self.initial_phase;
    }

    fn process_children(&mut self, _cn: &mut Vec<Node>) -> Option<Frame> {
        let mut samples = [0f32; 8];
        let mut phase = self.phase;
        let dur = 1. / self.freq;
        for sample in &mut samples {
            *sample = phase / dur;
            phase += self.freq * SAMPLE_DURATION;
            phase %= dur;
        }
        self.phase = phase;
        let s = Sample::new(samples);
        Some(Frame::mono(s))
    }
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
        let mut x = self.seed;
        if x == 0 {
            x = 1;
        }
        for sample in samples.iter_mut() {
            // xorshift RNG algorithm
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            *sample = x as f32;
        }
        self.seed = x;
        let samples = Sample::new(samples);
        let samples = samples / u32::MAX as f32;
        Some(Frame::mono(samples))
    }
}
