use crate::*;
use alloc::vec::Vec;

pub type Nodes = Vec<Node>;

pub trait Processor {
    fn set(&mut self, _param: u8, _val: f32) {
        // do nothing
    }

    fn reset(&mut self) {
        // do nothing
    }

    // TODO: seek

    fn process_children_f(&mut self, cn: &mut Nodes) -> Option<FrameF> {
        let mut sum = FrameF::zero();
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
        self.process_frame_f(f)
    }

    fn process_frame_f(&mut self, f: FrameF) -> Option<FrameF> {
        let left = self.process_sample_f(f.left)?;
        let right = match f.right {
            Some(right) => {
                let right = self.process_sample_f(right)?;
                Some(right)
            }
            None => None,
        };
        Some(FrameF { left, right })
    }

    fn process_sample_f(&mut self, s: SampleF) -> Option<SampleF> {
        Some(s)
    }
}
