use crate::modulators::Modulator;
use crate::*;
use alloc::boxed::Box;
use alloc::vec::Vec;
use micromath::F32Ext;

const MODULATE_EVERY: u32 = SAMPLE_RATE / 60;

/// A modulator connected to a parameter of a node.
struct WiredModulator {
    param: u8,
    modulator: Box<dyn Modulator>,
    low: f32,
    range: f32,
    /// The modulator-specific timer.
    ///
    /// Starts at zero (when the modulator is wired).
    time: u32,
}

pub struct Node {
    children: Vec<Self>,
    proc: Box<dyn Processor>,
    modulator: Option<WiredModulator>,
}

impl Node {
    pub(crate) fn new_root() -> Self {
        Self {
            children: Vec::new(),
            proc: Box::new(Mix::new()),
            modulator: None,
        }
    }

    /// Add a child node.
    pub(crate) fn add(&mut self, proc: Box<dyn Processor>) -> Result<u8, NodeError> {
        if self.children.len() >= 4 {
            return Err(NodeError::TooManyChildren);
        }
        #[expect(clippy::cast_possible_truncation)]
        let child_id = self.children.len() as u8;
        let child = Self {
            children: Vec::new(),
            modulator: None,
            proc,
        };
        self.children.push(child);
        Ok(child_id)
    }

    pub(crate) fn get_node(&mut self, path: &[u8]) -> &mut Self {
        let Some(first) = path.first() else {
            return self;
        };
        let node = &mut self.children[*first as usize];
        node.get_node(&path[1..])
    }

    pub(crate) fn next_frame(&mut self) -> Option<Frame> {
        if let Some(modulator) = self.modulator.as_mut() {
            if modulator.time % MODULATE_EVERY == 0 {
                let val = modulator.modulator.get(modulator.time);
                let val = F32Ext::mul_add(val, modulator.range, modulator.low);
                self.proc.set(modulator.param, val);
            }
            modulator.time += 8;
        }
        self.proc.process_children(&mut self.children)
    }

    pub(crate) fn clear(&mut self) {
        self.children.clear();
    }

    /// Reset the current node (processor and modulator) to the initial state.
    pub fn reset(&mut self) {
        self.proc.reset();
        if let Some(modulator) = self.modulator.as_mut() {
            modulator.time = 0;
        }
    }

    /// Do [`Node::reset`] on the current node and all its children (recursively).
    pub fn reset_all(&mut self) {
        self.reset();
        for node in &mut self.children {
            node.reset_all();
        }
    }

    /// Set modulator for the given parameter.
    ///
    /// The `low` is the lowest value produced by the modulator
    /// and `high` is the highest.
    ///
    /// If `low` is smaller than `high`, the modulator output is inversed.
    /// For example, modulating [`Pause`] with `low=0` and `high=1`
    /// will make it first paused and then playing while modulating it with
    /// `low=1` and `high=0` will first have the node playing and then paused.
    pub fn modulate(&mut self, param: u8, lfo: Box<dyn Modulator>, low: f32, high: f32) {
        let modulator = WiredModulator {
            param,
            modulator: lfo,
            time: 0,
            low,
            range: high - low,
        };
        self.modulator = Some(modulator);
    }
}
