use crate::basic_types::*;
use core::marker::PhantomData;
use firefly_device::*;

pub trait Source {
    fn advance(&mut self) -> Option<Frame>;
}

pub struct Mono {}
pub struct Stereo {}

/// Audio file in WAV format with PCM encoding.
pub struct Wav<C> {
    file: <DeviceImpl as Device>::Read,
    _ch: PhantomData<C>,
}

impl<C> Wav<C> {
    pub fn new(file: <DeviceImpl as Device>::Read) -> Self {
        Self {
            file,
            _ch: PhantomData,
        }
    }
}

impl Source for Wav<Mono> {
    fn advance(&mut self) -> Option<Frame> {
        todo!()
    }
}

impl Source for Wav<Stereo> {
    fn advance(&mut self) -> Option<Frame> {
        todo!()
    }
}
