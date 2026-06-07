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
    /// In the current implementation, some processors
    /// might reset parameters changed using `set`
    /// and some processors might keep these.
    /// This behavior might change in the future
    /// for individual processors or all processors.
    fn reset(&mut self) {
        // do nothing
    }

    /// Get the next frame for the processor.
    fn process_children(&mut self, cn: &mut [Node]) -> Option<Frame> {
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
