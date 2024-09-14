use crate::modulators::Modulator;
use crate::*;
use alloc::boxed::Box;
use alloc::vec::Vec;

const MODULATE_EVERY: u32 = SAMPLE_RATE / 60;

// A modulator connected to a parameter of a node.
struct WiredModulator {
    param: u8,
    modulator: Box<dyn Modulator>,
    time: u32,
}

pub struct Node {
    children: Vec<Node>,
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
        const MAX_NODES: u32 = 4;
        if self.children.len() as u32 >= MAX_NODES {
            return Err(NodeError::TooManyChildren);
        }
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
                self.proc.set(modulator.param, val);
            }
            modulator.time += 8;
        }
        self.proc.process_children(&mut self.children)
    }

    pub(crate) fn clear(&mut self) {
        self.children.clear();
    }

    /// Reset the current node processor to its initial state.
    pub fn reset(&mut self) {
        self.proc.reset();
    }

    /// Reset the current node and all its children.
    pub fn reset_all(&mut self) {
        self.proc.reset();
        for node in &mut self.children {
            node.reset_all();
        }
    }

    // TODO: reset modulator

    /// Set modulator for the given parameter.
    pub fn modulate(&mut self, param: u8, lfo: Box<dyn Modulator>) {
        let modulator = WiredModulator {
            param,
            modulator: lfo,
            time: 0,
        };
        self.modulator = Some(modulator);
    }
}
