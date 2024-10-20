use core::ops::{Add, Div};

pub type Position = u32;
pub const SAMPLE_RATE: Position = 44_100;
pub const SAMPLE_DURATION: f32 = 1.0 / SAMPLE_RATE as f32;

pub type Sample = wide::f32x8;

#[derive(Clone)]
pub struct Frame {
    pub left: Sample,
    pub right: Option<Sample>,
}

impl Frame {
    pub(crate) const fn zero() -> Self {
        Self {
            left: Sample::ZERO,
            right: None,
        }
    }

    #[must_use]
    pub const fn mono(s: Sample) -> Self {
        Self {
            left: s,
            right: None,
        }
    }

    #[must_use]
    pub const fn stereo(l: Sample, r: Sample) -> Self {
        Self {
            left: l,
            right: Some(r),
        }
    }
}

impl Add<&Self> for Frame {
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

impl Div<f32> for Frame {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        let left = self.left / rhs;
        let right = self.right.map(|r| r / rhs);
        Self { left, right }
    }
}
