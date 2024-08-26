use core::ops::{Add, Div};

pub type Sample = f32;
pub type Values = alloc::vec::Vec<f32>;

pub struct Frame {
    pub left: Sample,
    pub right: Option<Sample>,
}

impl Frame {
    pub(crate) fn zero() -> Self {
        Self {
            left: 0.0,
            right: None,
        }
    }

    pub fn mono(s: Sample) -> Self {
        Self {
            left: s,
            right: None,
        }
    }

    pub fn stereo(l: Sample, r: Sample) -> Self {
        Self {
            left: l,
            right: Some(r),
        }
    }

    pub(crate) fn map<F>(&self, mut f: F) -> Frame
    where
        F: FnMut(Sample) -> Sample,
    {
        let left = f(self.left);
        let right = self.right.map(f);
        Self { left, right }
    }
}

impl Add<&Frame> for Frame {
    type Output = Self;

    fn add(self, rhs: &Frame) -> Self {
        let left = self.left + rhs.left;
        let right = match (self.right, rhs.right) {
            (None, None) => None,
            (None, Some(r)) => Some(r),
            (Some(r), None) => Some(r),
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
