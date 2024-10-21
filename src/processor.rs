use crate::*;
use alloc::vec::Vec;

pub type Nodes = Vec<Node>;

pub trait Processor {
    // TODO: seek

    fn set(&mut self, _param: u8, _val: f32) {
        // do nothing
    }

    fn reset(&mut self) {
        // do nothing
    }
}

pub trait ProcessorF: Processor {
    fn process_children(&mut self, cn: &mut Nodes) -> Option<FrameF> {
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
        self.process_frame(f)
    }

    fn process_frame(&mut self, f: FrameF) -> Option<FrameF> {
        let left = self.process_sample(f.left)?;
        let right = match f.right {
            Some(right) => {
                let right = self.process_sample(right)?;
                Some(right)
            }
            None => None,
        };
        Some(FrameF { left, right })
    }

    fn process_sample(&mut self, s: SampleF) -> Option<SampleF> {
        Some(s)
    }
}

pub trait ProcessorI: Processor {
    fn process_frame_i(&mut self, f: FrameI) -> Option<FrameI> {
        let left = self.process_sample_i(f.left)?;
        let right = match f.right {
            Some(right) => {
                let right = self.process_sample_i(right)?;
                Some(right)
            }
            None => None,
        };
        Some(FrameI { left, right })
    }

    fn process_sample_i(&mut self, s: SampleI) -> Option<SampleI> {
        Some(s)
    }
}
