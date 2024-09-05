use crate::*;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ops::Range;

pub trait Processor {
    fn reset(&mut self) {
        // do nothing
    }

    // TODO: seek

    fn process_children(&mut self, cn: &mut Vec<Node>) -> Option<Frame> {
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
    pub(crate) id: u32,
    range: Range<u32>,
    pub(crate) children: Vec<Node>,
    behavior: Box<dyn Processor>,
}

impl Node {
    pub fn new_root() -> Self {
        Self {
            id: 0,
            range: 0..u32::MAX,
            children: Vec::new(),
            behavior: Box::new(Mix::new()),
        }
    }

    pub fn add(&mut self, b: Box<dyn Processor>) -> Option<u32> {
        const MAX_NODES: u32 = 4;
        if self.children.len() as u32 >= MAX_NODES {
            return None;
        }
        let range_size = (self.range.end - self.range.start) / MAX_NODES;
        let child_id = self.range.start + 1 + range_size * self.children.len() as u32;
        let range_start = child_id;
        let range_end = range_start + range_size;
        debug_assert!(range_start >= self.range.start);
        debug_assert!(range_end <= self.range.end);
        let child = Self {
            id: child_id,
            range: range_start..range_end,
            children: Vec::new(),
            behavior: b,
        };
        self.children.push(child);
        Some(child_id)
    }

    pub fn get_node(&mut self, id: u32) -> Option<&mut Self> {
        if self.id == id {
            return Some(self);
        }
        for node in &mut self.children {
            if node.range.contains(&id) {
                return node.get_node(id);
            }
        }
        None
    }

    pub fn next_frame(&mut self) -> Option<Frame> {
        self.behavior.process_children(&mut self.children)
    }

    pub fn reset(&mut self) {
        self.behavior.reset();
        for node in self.children.iter_mut() {
            node.reset();
        }
    }

    pub fn reset_all(&mut self) {
        self.behavior.reset();
        for node in self.children.iter_mut() {
            node.reset_all();
        }
    }

    pub fn clear(&mut self) {
        self.behavior.reset();
        self.children.clear();
    }
}
