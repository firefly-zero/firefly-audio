use crate::*;
use alloc::vec;
use alloc::vec::Vec;

/// Do nothing: only mix the children and pass the mix forward with no changes.
pub struct Mix {}

impl Mix {
    pub fn new() -> Self {
        Self {}
    }
}

impl Behavior for Mix {}

/// Set the gain level for every sample.
pub struct Gain {
    lvl: f32,
}

impl Gain {
    pub fn new(lvl: f32) -> Self {
        Self { lvl }
    }
}

impl Behavior for Gain {
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

impl Behavior for Loop {
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

impl Behavior for Concat {
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

impl Behavior for OneDelay {
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

impl Behavior for Delay {
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

impl Behavior for Sine {
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
