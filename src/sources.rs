//! Processors that have 0 inputs and produce new values.
//!
//! Includes oscillators, file readers, audio samples, etc.
use crate::*;
use alloc::vec::Vec;
use micromath::F32Ext;

/// A sound source that is always stopped.
pub struct Empty {}

impl Empty {
    #[must_use]
    pub const fn new() -> Self {
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
    #[must_use]
    pub const fn new() -> Self {
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
    step: f32,
    phase: f32,
    initial_phase: f32,
}

impl Sine {
    #[must_use]
    pub fn new(freq: f32, phase: f32) -> Self {
        Self {
            step: freq * SAMPLE_DURATION,
            phase,
            initial_phase: phase,
        }
    }
}

impl Processor for Sine {
    fn reset(&mut self) {
        self.phase = self.initial_phase;
    }

    fn set(&mut self, param: u8, val: f32) {
        if param == 0 {
            self.step = val * SAMPLE_DURATION;
        }
    }

    fn process_children(&mut self, _cn: &mut Vec<Node>) -> Option<Frame> {
        let mut element = [0f32; 8];
        let mut phase = self.phase;
        for sample in &mut element {
            *sample = phase;
            phase = F32Ext::fract(phase + self.step);
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
    step: f32,
    phase: f32,
    initial_phase: f32,
}

impl Square {
    #[must_use]
    pub fn new(freq: f32, phase: f32) -> Self {
        Self {
            step: freq * SAMPLE_DURATION,
            phase,
            initial_phase: phase,
        }
    }
}

impl Processor for Square {
    fn reset(&mut self) {
        self.phase = self.initial_phase;
    }

    fn set(&mut self, param: u8, val: f32) {
        if param == 0 {
            self.step = val * SAMPLE_DURATION;
        }
    }

    fn process_children(&mut self, _cn: &mut Vec<Node>) -> Option<Frame> {
        let mut samples = [0f32; 8];
        let mut phase = self.phase;
        for sample in &mut samples {
            let dec = F32Ext::fract(phase);
            *sample = if dec >= 0.5 { 1. } else { -1. };
            phase = F32Ext::fract(phase + self.step);
        }
        self.phase = phase;
        let s = Sample::new(samples);
        Some(Frame::mono(s))
    }
}

/// Sawtooth wave oscillator.
pub struct Sawtooth {
    step: f32,
    phase: f32,
    initial_phase: f32,
}

impl Sawtooth {
    #[must_use]
    pub fn new(freq: f32, phase: f32) -> Self {
        Self {
            step: freq * SAMPLE_DURATION,
            phase,
            initial_phase: phase,
        }
    }
}

impl Processor for Sawtooth {
    fn reset(&mut self) {
        self.phase = self.initial_phase;
    }

    fn set(&mut self, param: u8, val: f32) {
        if param == 0 {
            self.step = val * SAMPLE_DURATION;
        }
    }

    fn process_children(&mut self, _cn: &mut Vec<Node>) -> Option<Frame> {
        let mut samples = [0f32; 8];
        let mut phase = self.phase;
        for sample in &mut samples {
            *sample = phase;
            phase = F32Ext::fract(phase + self.step);
        }
        self.phase = phase;
        let s = Sample::new(samples);
        let s = s * 2. - 1.;
        Some(Frame::mono(s))
    }
}

/// Triangle wave oscillator.
pub struct Triangle {
    step: f32,
    phase: f32,
    initial_phase: f32,
}

impl Triangle {
    #[must_use]
    pub fn new(freq: f32, phase: f32) -> Self {
        Self {
            step: freq * SAMPLE_DURATION,
            phase,
            initial_phase: phase,
        }
    }
}

impl Processor for Triangle {
    fn reset(&mut self) {
        self.phase = self.initial_phase;
    }

    fn set(&mut self, param: u8, val: f32) {
        if param == 0 {
            self.step = val * SAMPLE_DURATION;
        }
    }

    fn process_children(&mut self, _cn: &mut Vec<Node>) -> Option<Frame> {
        let mut samples = [0f32; 8];
        let mut phase = self.phase;
        for sample in &mut samples {
            *sample = phase;
            phase = F32Ext::fract(phase + self.step);
        }
        self.phase = phase;
        let s = Sample::new(samples);
        let s = (s * 4. - 2.).abs() - 1.;
        Some(Frame::mono(s))
    }
}

/// Generate a white noise
pub struct Noise {
    prev: wide::i32x8,
}

impl Noise {
    #[must_use]
    pub fn new(seed: i32) -> Self {
        Self {
            prev: wide::i32x8::new([
                seed,
                seed + 1,
                seed + 2,
                seed + 3,
                seed + 4,
                seed + 5,
                seed + 6,
                seed + 7,
            ]),
        }
    }
}

impl Processor for Noise {
    fn process_children(&mut self, _cn: &mut Vec<Node>) -> Option<Frame> {
        // xorshift RNG algorithm
        // TODO: spectogram shows that it might be not uniformly distributed.
        let mut x = self.prev;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.prev = x;
        let s = Sample::from_i32x8(x);
        let s = s / i32::MAX as f32;
        Some(Frame::mono(s))
    }
}

// TODO: pulse (https://github.com/NibbleRealm/twang/blob/v0/src/osc/pulse.rs)
