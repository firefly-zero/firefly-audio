use crate::*;
use alloc::vec;
use alloc::vec::Vec;
use micromath::F32Ext;

/// Do nothing: only mix the children and pass the mix forward with no changes.
pub struct Mix {}

impl Mix {
    pub fn new() -> Self {
        Self {}
    }
}

impl Processor for Mix {}

/// Set the gain level for every sample.
pub struct Gain {
    lvl: f32,
}

impl Gain {
    pub fn new(lvl: f32) -> Self {
        Self { lvl }
    }
}

impl Processor for Gain {
    fn process_sample(&mut self, s: Sample) -> Option<Sample> {
        Some(s * self.lvl)
    }
}

/// When the source ends, start it from the beginning.
pub struct Loop {}

impl Loop {
    pub fn new() -> Self {
        Self {}
    }
}

impl Processor for Loop {
    fn process_children(&mut self, cn: &mut Vec<Node>) -> Option<Frame> {
        let mut sum = Frame::zero();
        for node in cn.iter_mut() {
            let f = match node.next_frame() {
                Some(f) => f,
                None => {
                    node.reset();
                    node.next_frame()?
                }
            };
            sum = sum + &f;
        }
        Some(sum / cn.len() as f32)
    }
}

/// Play children in order, one at a time.
pub struct Concat {}

impl Concat {
    // TODO: support fade-in/fade-out
    pub fn new() -> Self {
        Self {}
    }
}

impl Processor for Concat {
    fn process_children(&mut self, cn: &mut Vec<Node>) -> Option<Frame> {
        for node in cn {
            if let Some(f) = node.next_frame() {
                return Some(f);
            }
        }
        None
    }
}

/// Delay the input only for one tick.
pub struct OneDelay {
    prev: Frame,
}

impl OneDelay {
    pub fn new() -> Self {
        Self {
            prev: Frame::zero(),
        }
    }
}

impl Processor for OneDelay {
    fn process_frame(&mut self, f: Frame) -> Option<Frame> {
        let res = self.prev.clone();
        self.prev = f;
        Some(res)
    }
}

/// Delay the input for the given number of samples.
pub struct Delay {
    buf: Vec<Frame>,
    i: usize,
}

impl Delay {
    pub fn new(size: usize) -> Self {
        Self {
            buf: vec![Frame::zero(); size],
            i: 0,
        }
    }
}

impl Processor for Delay {
    // TODO: process_children shouldn't stop when the source ends.
    // We should exhaust the buffer first.
    fn process_frame(&mut self, f: Frame) -> Option<Frame> {
        self.buf[self.i] = f;
        self.i = self.i.wrapping_add(1);
        if self.i >= self.buf.len() {
            self.i = 0;
        }
        Some(self.buf[self.i].clone())
    }
}

pub struct Pan {
    left_weight: f32,
    right_weight: f32,
}

impl Pan {
    pub fn new(v: f32) -> Self {
        let (left_weight, right_weight) = pan_weights(v);
        Self {
            left_weight,
            right_weight,
        }
    }
}

#[inline]
fn pan_weights(v: f32) -> (f32, f32) {
    let v = v.clamp(-1., 1.);
    let angle = (v + 1.) * core::f32::consts::FRAC_PI_4;
    let (sin, cos) = F32Ext::sin_cos(angle);
    (cos, sin)
}

impl Processor for Pan {
    fn process_frame(&mut self, f: Frame) -> Option<Frame> {
        let left = f.left * self.left_weight;
        let right = f.right.map(|s| s * self.right_weight);
        Some(Frame { left, right })
    }
}

/// Fade in the input for the given number of samples.
///
/// The fade-in is linear. If you need a non-linear fade-in, use modulated [`Gain`].
pub struct FadeIn {
    start_gain: f32,
    total: Position,
    elapsed: Position,
}

impl FadeIn {
    pub fn new(start_gain: f32, duration: Position) -> Self {
        Self {
            start_gain,
            total: duration,
            elapsed: 0,
        }
    }
}

impl Processor for FadeIn {
    fn reset(&mut self) {
        self.elapsed = 0;
    }

    fn process_sample(&mut self, s: Sample) -> Option<Sample> {
        if self.elapsed >= self.total {
            return Some(s);
        }
        let ratio = (self.elapsed / self.total) as f32;
        let gain = self.start_gain + (1. - ratio);
        self.elapsed += 1;
        Some(s * gain)
    }
}

/// A node that can pause the audio stream.
pub struct Pause {
    paused: bool,
}

impl Pause {
    pub fn new() -> Self {
        Self { paused: false }
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn play(&mut self) {
        self.paused = false;
    }
}

impl Processor for Pause {
    fn reset(&mut self) {
        self.paused = false;
    }

    fn process_sample(&mut self, s: Sample) -> Option<Sample> {
        if self.paused {
            return Some(Sample::ZERO);
        }
        Some(s)
    }
}

/// Tracks the current position (elapsed time) of the audio stream.
pub struct TrackPosition {
    elapsed: Position,
}

impl TrackPosition {
    pub fn new() -> Self {
        Self { elapsed: 0 }
    }
}

impl Processor for TrackPosition {
    fn reset(&mut self) {
        self.elapsed = 0;
    }

    fn process_sample(&mut self, s: Sample) -> Option<Sample> {
        self.elapsed += 1;
        Some(s)
    }
}

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
    pub fn new(phase: f32, freq: f32) -> Self {
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
        for sample in &mut element {
            *sample = self.phase;
            self.phase += self.freq * SAMPLE_DURATION;
            self.phase -= self.phase.floor();
        }
        let element = Sample::new(element);
        let s = element * Sample::TAU;
        let s = s.sin();
        Some(Frame::mono(s))
    }
}
