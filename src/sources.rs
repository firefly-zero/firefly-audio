use core::marker::PhantomData;
use firefly_device::*;

type Amp = f32;

pub trait MonoSource {
    fn next_mono(&mut self) -> Option<Amp>;
}

pub trait StereoSource {
    fn next_stereo(&mut self) -> Option<(Amp, Amp)>;

    fn next_mono(&mut self) -> Option<Amp> {
        let (a, b) = self.next_stereo()?;
        Some((a + b) / 2.)
    }
}

impl<T: MonoSource> StereoSource for T {
    fn next_stereo(&mut self) -> Option<(Amp, Amp)> {
        let a = self.next_mono()?;
        Some((a, a))
    }
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

impl MonoSource for Wav<Mono> {
    fn next_mono(&mut self) -> Option<Amp> {
        todo!()
    }
}

impl StereoSource for Wav<Stereo> {
    fn next_stereo(&mut self) -> Option<(Amp, Amp)> {
        todo!()
    }
}

pub struct Sine {}

impl MonoSource for Sine {
    fn next_mono(&mut self) -> Option<Amp> {
        todo!()
    }
}
