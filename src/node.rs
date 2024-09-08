use crate::*;
use alloc::boxed::Box;
use alloc::vec::Vec;

pub type Nodes = Vec<Node>;

pub trait Processor {
    fn reset(&mut self) {
        // do nothing
    }

    // TODO: seek

    fn process_children(&mut self, cn: &mut Nodes) -> Option<Frame> {
        let mut sum = Frame::zero();
        let mut count = 0;
        for node in cn.iter_mut() {
            let Some(frame) = node.next_frame() else {
                continue;
            };
            sum = sum + &frame;
            count += 1;
        }
        if count == 0 {
            return None;
        }
        let f = sum / count as f32;
        self.process_frame(f)
    }

    fn process_frame(&mut self, f: Frame) -> Option<Frame> {
        let left = self.process_sample(f.left)?;
        let right = match f.right {
            Some(right) => {
                let right = self.process_sample(right)?;
                Some(right)
            }
            None => None,
        };
        Some(Frame { left, right })
    }

    fn process_sample(&mut self, s: Sample) -> Option<Sample> {
        Some(s)
    }
}

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
