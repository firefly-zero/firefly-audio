use crate::*;
use micromath::F32Ext;

/// Do nothing: only mix the children and pass the mix forward with no changes.
pub struct Mix {}

impl Mix {
    pub fn new() -> Self {
        Self {}
    }
}

impl Processor for Mix {}

/// Mix the inputs, stop everything if at least one input is stopped.
pub struct AllForOne {}

impl AllForOne {
    pub fn new() -> Self {
        Self {}
    }
}

impl Processor for AllForOne {
    fn process_children(&mut self, cn: &mut Nodes) -> Option<Frame> {
        let mut sum = Frame::zero();
        if cn.is_empty() {
            return None;
        }
        for node in cn.iter_mut() {
            sum = sum + &node.next_frame()?;
        }
        let f = sum / cn.len() as f32;
        self.process_frame(f)
    }
}

/// Set the gain level for every sample.
pub struct Gain {
    lvl: f32,
}

impl Gain {
    pub fn new(lvl: f32) -> Self {
        Self { lvl }
    }
}

impl Processor for Gain {
    fn set(&mut self, param: u8, val: f32) {
        if param == 0 {
            self.lvl = val;
        }
    }

    fn process_sample(&mut self, s: Sample) -> Option<Sample> {
        Some(s * self.lvl)
    }
}

/// When the source ends, start it from the beginning.
///
/// If any of the sources doesn't restart on reset,
/// the node stops.
pub struct Loop {}

impl Loop {
    pub fn new() -> Self {
        Self {}
    }
}

impl Processor for Loop {
    fn process_children(&mut self, cn: &mut Nodes) -> Option<Frame> {
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

impl Processor for Concat {
    fn process_children(&mut self, cn: &mut Nodes) -> Option<Frame> {
        for node in cn {
            if let Some(f) = node.next_frame() {
                return Some(f);
            }
        }
        None
    }
}

pub struct Pan {
    left_weight: f32,
    right_weight: f32,
}

impl Pan {
    pub fn new(v: f32) -> Self {
        let (left_weight, right_weight) = pan_weights(v);
        Self {
            left_weight,
            right_weight,
        }
    }
}

#[inline]
fn pan_weights(v: f32) -> (f32, f32) {
    let v = v.clamp(-1., 1.);
    let angle = (v + 1.) * core::f32::consts::FRAC_PI_4;
    let (sin, cos) = F32Ext::sin_cos(angle);
    (cos, sin)
}

impl Processor for Pan {
    fn process_frame(&mut self, f: Frame) -> Option<Frame> {
        let left = f.left * self.left_weight;
        let right = f.right.map(|s| s * self.right_weight);
        Some(Frame { left, right })
    }
}

/// A node that can mute the audio stream.
pub struct Mute {
    muted: bool,
}

impl Mute {
    pub fn new() -> Self {
        Self { muted: false }
    }

    pub fn mute(&mut self) {
        self.muted = true;
    }

    pub fn unmute(&mut self) {
        self.muted = false;
    }
}

impl Processor for Mute {
    fn reset(&mut self) {
        self.muted = false;
    }

    fn process_sample(&mut self, s: Sample) -> Option<Sample> {
        if self.muted {
            return Some(Sample::ZERO);
        }
        Some(s)
    }
}

/// A node that can pause the audio stream.
pub struct Pause {
    paused: bool,
}

impl Pause {
    pub fn new() -> Self {
        Self { paused: false }
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn play(&mut self) {
        self.paused = false;
    }
}

impl Processor for Pause {
    fn reset(&mut self) {
        self.paused = false;
    }

    fn process_children(&mut self, cn: &mut Nodes) -> Option<Frame> {
        if self.paused {
            return None;
        }
        Mix::new().process_children(cn)
    }
}

/// Tracks the current position (elapsed time) of the audio stream.
pub struct TrackPosition {
    elapsed: Position,
}

impl TrackPosition {
    pub fn new() -> Self {
        Self { elapsed: 0 }
    }
}

impl Processor for TrackPosition {
    fn reset(&mut self) {
        self.elapsed = 0;
    }

    fn process_sample(&mut self, s: Sample) -> Option<Sample> {
        self.elapsed += 1;
        Some(s)
    }
}

/// Low-pass/high-pass filter.
#[derive(Default)]
pub struct LowHighPass {
    low: bool,
    freq: u32,
    q: f32,

    x_n1: Sample,
    x_n2: Sample,
    y_n1: Sample,
    y_n2: Sample,

    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

impl LowHighPass {
    pub fn new(low: bool, freq: u32, q: f32) -> Self {
        let mut res = Self {
            low,
            freq,
            q,
            ..Default::default()
        };
        res.update_coefs();
        res
    }

    fn update_coefs(&mut self) {
        let w0 = core::f32::consts::TAU * self.freq as f32 / SAMPLE_RATE as f32;
        let cos_w0 = w0.cos();
        let alpha = w0.sin() / (2. * self.q);

        if self.low {
            let b1 = 1. - cos_w0;
            let b0 = b1 / 2.;
            let b2 = b0;
            let a0 = 1. + alpha;
            let a1 = -2. * cos_w0;
            let a2 = 1. - alpha;

            self.b0 = b0 / a0;
            self.b1 = b1 / a0;
            self.b2 = b2 / a0;
            self.a1 = a1 / a0;
            self.a2 = a2 / a0;
        } else {
            let b0 = (1. + cos_w0) / 2.;
            let b1 = -1. - cos_w0;
            let b2 = b0;
            let a0 = 1. + alpha;
            let a1 = -2. * cos_w0;
            let a2 = 1. - alpha;

            self.b0 = b0 / a0;
            self.b1 = b1 / a0;
            self.b2 = b2 / a0;
            self.a1 = a1 / a0;
            self.a2 = a2 / a0;
        }
    }
}

impl Processor for LowHighPass {
    fn reset(&mut self) {
        self.y_n2 = Sample::ZERO;
        self.x_n2 = Sample::ZERO;
        self.y_n1 = Sample::ZERO;
        self.x_n1 = Sample::ZERO;
    }

    fn process_sample(&mut self, s: Sample) -> Option<Sample> {
        let bx0 = self.b0 * s;
        let bx1 = self.b1 * self.x_n1;
        let bx2 = self.b2 * self.x_n2;
        let ay1 = self.a1 * self.y_n1;
        let ay2 = self.a2 * self.y_n2;
        let result = bx0 + bx1 + bx2 - ay1 - ay2;

        self.y_n2 = self.y_n1;
        self.x_n2 = self.x_n1;
        self.y_n1 = result;
        self.x_n1 = s;

        Some(result)
    }
}

/// Take the left (and discard the right) channel from a stereo source.
pub struct TakeLeft {}

impl TakeLeft {
    pub fn new() -> Self {
        Self {}
    }
}

impl Processor for TakeLeft {
    fn process_children(&mut self, cn: &mut Nodes) -> Option<Frame> {
        let mut sum = Sample::ZERO;
        for node in cn.iter_mut() {
            sum += &node.next_frame()?.left;
        }
        let s = sum / cn.len() as f32;
        Some(Frame::mono(s))
    }
}

/// Take the right (and discard the left) channel from a stereo source.
pub struct TakeRight {}

impl TakeRight {
    pub fn new() -> Self {
        Self {}
    }
}

impl Processor for TakeRight {
    fn process_children(&mut self, cn: &mut Nodes) -> Option<Frame> {
        let mut sum = Sample::ZERO;
        for node in cn.iter_mut() {
            if let Some(right) = &node.next_frame()?.right {
                sum += right;
            }
        }
        let s = sum / cn.len() as f32;
        Some(Frame::mono(s))
    }
}

/// Swap left and right channels.
pub struct Swap {}

impl Swap {
    pub fn new() -> Self {
        Self {}
    }
}

impl Processor for Swap {
    fn process_frame(&mut self, f: Frame) -> Option<Frame> {
        if let Some(right) = f.right {
            Some(Frame::stereo(right, f.left))
        } else {
            Some(f)
        }
    }
}
