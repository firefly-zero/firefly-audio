use crate::*;
use alloc::vec::Vec;

pub type Nodes = Vec<Node>;

pub trait Processor {
    /// Set the value of the given parameter.
    fn set(&mut self, _param: u8, _val: f32) {
        // do nothing
    }

    /// Reset the processor to the initial state.
    ///
    /// This might or might not affect params changed using [`Processor::set`].
    /// This is doesn't matter since params are usually `set` by a modulator
    /// and when the processor is reset, the modulator is also reset.
    /// They can also be changed using [`Node::set`] and the method
    /// has a warning about this behavior.
    fn reset(&mut self) {
        // do nothing
    }

    /// Get the next frame for the processor.
    fn process_children(&mut self, cn: &mut [Node]) -> Option<Frame> {
        let mut sum = Frame::zero();
        let mut empty = true;
        for node in cn.iter_mut() {
            let Some(frame) = node.next_frame() else {
                continue;
            };
            sum = sum + frame;
            empty = false;
        }
        if empty {
            return None;
        }
        self.process_frame(sum)
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
