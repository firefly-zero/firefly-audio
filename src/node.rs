use crate::*;
use alloc::boxed::Box;
use alloc::vec::Vec;

pub struct Node {
    pub(crate) children: Vec<Node>,
    proc: Box<dyn Processor>,
}

impl Node {
    pub(crate) fn new_root() -> Self {
        Self {
            children: Vec::new(),
            proc: Box::new(Mix::new()),
        }
    }

    pub(crate) fn add(&mut self, proc: Box<dyn Processor>) -> Result<u8, NodeError> {
        const MAX_NODES: u32 = 4;
        if self.children.len() as u32 >= MAX_NODES {
            return Err(NodeError::TooManyChildren);
        }
        let child_id = self.children.len() as u8;
        let child = Self {
            children: Vec::new(),
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
        self.proc.process_children(&mut self.children)
    }

    /// Reset the current node processor to its initial state.
    pub fn reset(&mut self) {
        self.proc.reset();
    }

    /// Reset the current node and all its children.
    pub fn reset_all(&mut self) {
        self.proc.reset();
        for node in self.children.iter_mut() {
            node.reset_all();
        }
    }

    pub fn set_behavior(&mut self, b: Box<dyn Processor>) {
        self.proc = b;
    }
}
