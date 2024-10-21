use core::ops::{Add, Div};

pub type Position = u32;
pub const SAMPLE_RATE: Position = 44_100;
pub const SAMPLE_DURATION: f32 = 1.0 / SAMPLE_RATE as f32;

pub type SampleF = wide::f32x8;
pub type SampleI = wide::i16x8;

#[derive(Clone)]
pub struct FrameF {
    pub left: SampleF,
    pub right: Option<SampleF>,
}

impl FrameF {
    pub(crate) const fn zero() -> Self {
        Self {
            left: SampleF::ZERO,
            right: None,
        }
    }

    #[must_use]
    pub const fn mono(s: SampleF) -> Self {
        Self {
            left: s,
            right: None,
        }
    }

    #[must_use]
    pub const fn stereo(l: SampleF, r: SampleF) -> Self {
        Self {
            left: l,
            right: Some(r),
        }
    }
}

impl Add<&Self> for FrameF {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self {
        let left = self.left + rhs.left;
        let right = match (self.right, rhs.right) {
            (None, None) => None,
            (None, Some(r)) | (Some(r), None) => Some(r),
            (Some(a), Some(b)) => Some(a + b),
        };
        Self { left, right }
    }
}

impl Div<f32> for FrameF {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        let left = self.left / rhs;
        let right = self.right.map(|r| r / rhs);
        Self { left, right }
    }
}
