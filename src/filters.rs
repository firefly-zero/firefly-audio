pub trait MonoFilter {
    fn filter_mono(&mut self, a: f32) -> f32;
}

pub trait StereoFilter {
    fn filter_stereo(&mut self, a: (f32, f32)) -> (f32, f32);
}

impl<F: MonoFilter> StereoFilter for F {
    fn filter_stereo(&mut self, a: (f32, f32)) -> (f32, f32) {
        (self.filter_mono(a.0), self.filter_mono(a.1))
    }
}

pub struct Volume {
    v: f32,
}

impl Volume {
    pub fn new(v: f32) -> Self {
        debug_assert!(v >= 0.);
        debug_assert!(v <= 1.);
        Self { v }
    }
}

impl MonoFilter for Volume {
    fn filter_mono(&mut self, a: f32) -> f32 {
        a * self.v
    }
}
