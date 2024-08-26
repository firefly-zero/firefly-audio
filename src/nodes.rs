use crate::*;
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
