use crate::basic_types::*;

pub trait Effect {
    fn apply(&mut self, vs: &Values, f: Frame) -> Frame {
        let (left, right) = f;
        (
            self.apply_mono(vs, left),
            right.map(|r| self.apply_mono(vs, r)),
        )
    }

    fn apply_mono(&mut self, vs: &Values, f: Sample) -> Sample;
}

pub struct GainFx {
    idx: usize,
}

impl GainFx {
    pub fn new(idx: usize) -> Self {
        Self { idx }
    }
}

impl Effect for GainFx {
    fn apply_mono(&mut self, vs: &Values, f: Sample) -> Sample {
        let Some(v) = vs.get(self.idx) else { return f };
        let v = v.clamp(0., 1.);
        f * v
    }
}
