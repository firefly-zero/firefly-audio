use crate::basic_types::*;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ops::Range;

pub trait Behavior {
    fn reset(&mut self) {
        // do nothing
    }

    // TODO: seek

    fn process_children(&mut self, cn: &mut Vec<Node>) -> Option<Frame> {
        let mut sum = Frame::zero();
        for node in cn.iter_mut() {
            sum = sum + &node.next_frame()?;
        }
        Some(sum / cn.len() as f32)
    }

    fn process_frame(&mut self, f: &Frame) -> Option<Frame> {
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
    id: u32,
    range: Range<u32>,
    children: Vec<Node>,
    behavior: Box<dyn Behavior>,
}

impl Node {
    pub fn add(&mut self, parent: u32, b: Box<dyn Behavior>) -> bool {
        let Some(node) = self.get_node(parent) else {
            return false;
        };
        if node.children.len() >= 4 {
            return false;
        }
        let child = Self {
            id: todo!(),
            range: todo!(),
            children: Vec::new(),
            behavior: b,
        };
        node.children.push(child);
        true
    }

    fn get_node(&mut self, id: u32) -> Option<&mut Self> {
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
    }
}
