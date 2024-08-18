use crate::filters::*;
use crate::sources::*;
use alloc::boxed::Box;
use alloc::vec::Vec;

pub struct Channels {
    channels: [Option<Channel>; 8],
}

impl Channels {
    pub fn new() -> Self {
        Self {
            channels: [None, None, None, None, None, None, None, None],
        }
    }

    pub fn next_stereo(&mut self) -> (f32, f32) {
        todo!()
    }
}

pub enum Channel {
    Mono(MonoChannel),
    Stereo(StereoChannel),
}

pub struct MonoChannel {
    pub sources: Vec<Box<dyn MonoSource>>,
    pub filters: Vec<Box<dyn MonoFilter>>,
}

pub struct StereoChannel {
    pub sources: Vec<Box<dyn StereoSource>>,
    pub filters: Vec<Box<dyn StereoFilter>>,
}
