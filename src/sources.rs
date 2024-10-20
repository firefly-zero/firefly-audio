//! Processors that have 0 inputs and produce new values.
//!
//! Includes oscillators, file readers, audio samples, etc.
use crate::*;
use alloc::vec::Vec;
use micromath::F32Ext;

/// A sound source that is always stopped.
pub struct Empty {}

impl Empty {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl Processor for Empty {
    fn process_children(&mut self, _cn: &mut Vec<Node>) -> Option<Frame> {
        None
    }
}

/// A sound source that produces zeros. Forever.
pub struct Zero {}

impl Zero {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl Processor for Zero {
    fn process_children(&mut self, _cn: &mut Vec<Node>) -> Option<Frame> {
        Some(Frame::zero())
    }
}

/// Sine wave oscillator.
pub struct Sine {
    step: f32,
    phase: f32,
    initial_phase: f32,
}

impl Sine {
    #[must_use]
    pub fn new(freq: f32, phase: f32) -> Self {
        Self {
            step: freq * SAMPLE_DURATION,
            phase,
            initial_phase: phase,
        }
    }
}

impl Processor for Sine {
    fn reset(&mut self) {
        self.phase = self.initial_phase;
    }

    fn set(&mut self, param: u8, val: f32) {
        if param == 0 {
            self.step = val * SAMPLE_DURATION;
        }
    }

    fn process_children(&mut self, _cn: &mut Vec<Node>) -> Option<Frame> {
        let mut element = [0f32; 8];
        let mut phase = self.phase;
        for sample in &mut element {
            *sample = phase;
            phase = F32Ext::fract(phase + self.step);
        }
        self.phase = phase;
        let element = Sample::new(element);
        let s = element * Sample::TAU;
        let s = s.sin();
        Some(Frame::mono(s))
    }
}

/// Square wave oscillator.
pub struct Square {
    step: f32,
    phase: f32,
    initial_phase: f32,
}

impl Square {
    #[must_use]
    pub fn new(freq: f32, phase: f32) -> Self {
        Self {
            step: freq * SAMPLE_DURATION,
            phase,
            initial_phase: phase,
        }
    }
}

impl Processor for Square {
    fn reset(&mut self) {
        self.phase = self.initial_phase;
    }

    fn set(&mut self, param: u8, val: f32) {
        if param == 0 {
            self.step = val * SAMPLE_DURATION;
        }
    }

    fn process_children(&mut self, _cn: &mut Vec<Node>) -> Option<Frame> {
        let mut samples = [0f32; 8];
        let mut phase = self.phase;
        for sample in &mut samples {
            let dec = F32Ext::fract(phase);
            *sample = if dec >= 0.5 { 1. } else { -1. };
            phase = F32Ext::fract(phase + self.step);
        }
        self.phase = phase;
        let s = Sample::new(samples);
        Some(Frame::mono(s))
    }
}

/// Sawtooth wave oscillator.
pub struct Sawtooth {
    step: f32,
    phase: f32,
    initial_phase: f32,
}

impl Sawtooth {
    #[must_use]
    pub fn new(freq: f32, phase: f32) -> Self {
        Self {
            step: freq * SAMPLE_DURATION,
            phase,
            initial_phase: phase,
        }
    }
}

impl Processor for Sawtooth {
    fn reset(&mut self) {
        self.phase = self.initial_phase;
    }

    fn set(&mut self, param: u8, val: f32) {
        if param == 0 {
            self.step = val * SAMPLE_DURATION;
        }
    }

    fn process_children(&mut self, _cn: &mut Vec<Node>) -> Option<Frame> {
        let mut samples = [0f32; 8];
        let mut phase = self.phase;
        for sample in &mut samples {
            *sample = phase;
            phase = F32Ext::fract(phase + self.step);
        }
        self.phase = phase;
        let s = Sample::new(samples);
        let s = s * 2. - 1.;
        Some(Frame::mono(s))
    }
}

/// Triangle wave oscillator.
pub struct Triangle {
    step: f32,
    phase: f32,
    initial_phase: f32,
}

impl Triangle {
    #[must_use]
    pub fn new(freq: f32, phase: f32) -> Self {
        Self {
            step: freq * SAMPLE_DURATION,
            phase,
            initial_phase: phase,
        }
    }
}

impl Processor for Triangle {
    fn reset(&mut self) {
        self.phase = self.initial_phase;
    }

    fn set(&mut self, param: u8, val: f32) {
        if param == 0 {
            self.step = val * SAMPLE_DURATION;
        }
    }

    fn process_children(&mut self, _cn: &mut Vec<Node>) -> Option<Frame> {
        let mut samples = [0f32; 8];
        let mut phase = self.phase;
        for sample in &mut samples {
            *sample = phase;
            phase = F32Ext::fract(phase + self.step);
        }
        self.phase = phase;
        let s = Sample::new(samples);
        let s = (s * 4. - 2.).abs() - 1.;
        Some(Frame::mono(s))
    }
}

/// Generate a white noise
pub struct Noise {
    prev: wide::i32x8,
}

impl Noise {
    #[must_use]
    pub fn new(seed: i32) -> Self {
        Self {
            prev: wide::i32x8::new([
                seed,
                seed + 1,
                seed + 2,
                seed + 3,
                seed + 4,
                seed + 5,
                seed + 6,
                seed + 7,
            ]),
        }
    }
}

impl Processor for Noise {
    fn process_children(&mut self, _cn: &mut Vec<Node>) -> Option<Frame> {
        // xorshift RNG algorithm
        // TODO: spectogram shows that it might be not uniformly distributed.
        let mut x = self.prev;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.prev = x;
        let s = Sample::from_i32x8(x);
        let s = s / i32::MAX as f32;
        Some(Frame::mono(s))
    }
}

pub enum AudioFileError {
    TooShort,
    BadMagicNumber,
    BadSampleRate,
}
/// Play audio from a reader (audio file).
pub struct Reader<R: embedded_io::Read> {
    reader: R,
    sample_rate: u16,
    is16: bool,
    stereo: bool,
    adpcm: bool,
}

impl<R: embedded_io::Read> Reader<R> {
    /// Create file reader source from a file in the Firefly Zero format.
    ///
    /// # Errors
    ///
    /// Returns an error if the file header is invalid.
    pub fn from_file(mut reader: R) -> Result<Self, AudioFileError> {
        let mut header = [0u8; 4];
        let res = reader.read_exact(&mut header);
        if res.is_err() {
            return Err(AudioFileError::TooShort);
        }
        if header[0] != 0x31 {
            return Err(AudioFileError::BadMagicNumber);
        }
        let sample_rate = u16::from_le_bytes([header[1], header[2]]);
        if sample_rate != 44100 {
            return Err(AudioFileError::BadSampleRate);
        }
        Ok(Self {
            reader,
            sample_rate,
            stereo: header[3] & 0b_100 != 0,
            is16: header[3] & 0b_010 != 0,
            adpcm: header[3] & 0b_001 != 0,
        })
    }
}

impl<R: embedded_io::Read> Processor for Reader<R> {
    fn process_children(&mut self, _cn: &mut Vec<Node>) -> Option<Frame> {
        let f = match (self.is16, self.stereo) {
            // 8 bit mono
            (false, false) => {
                let mut buf = [0u8; 8];
                self.reader.read_exact(&mut buf).ok()?;
                let s = Sample::new(u8s_to_f32s(buf));
                Frame::mono(s)
            }
            // 8 bit stereo
            (false, true) => {
                let mut buf = [0u8; 16];
                self.reader.read_exact(&mut buf).ok()?;
                let left = Sample::new(u8s_to_f32s_left(buf));
                let right = Sample::new(u8s_to_f32s_right(buf));
                Frame::stereo(left, right)
            }
            // 16 bit mono
            (true, false) => {
                let mut buf = [0u8; 16];
                self.reader.read_exact(&mut buf).ok()?;
                let s = Sample::new(u16s_to_f32s(buf));
                Frame::mono(s)
            }
            // 16 bit stereo
            (true, true) => {
                let mut buf = [0u8; 32];
                self.reader.read_exact(&mut buf).ok()?;
                let left = Sample::new(u16s_to_f32s_left(buf));
                let right = Sample::new(u16s_to_f32s_right(buf));
                Frame::stereo(left, right)
            }
        };
        Some(f)
    }
}

fn u8s_to_f32s(us: [u8; 8]) -> [f32; 8] {
    [
        u8_to_f32(us[0]),
        u8_to_f32(us[1]),
        u8_to_f32(us[2]),
        u8_to_f32(us[3]),
        u8_to_f32(us[4]),
        u8_to_f32(us[5]),
        u8_to_f32(us[6]),
        u8_to_f32(us[7]),
    ]
}

fn u8s_to_f32s_left(us: [u8; 16]) -> [f32; 8] {
    [
        u8_to_f32(us[0]),
        u8_to_f32(us[2]),
        u8_to_f32(us[4]),
        u8_to_f32(us[6]),
        u8_to_f32(us[8]),
        u8_to_f32(us[10]),
        u8_to_f32(us[12]),
        u8_to_f32(us[14]),
    ]
}

fn u8s_to_f32s_right(us: [u8; 16]) -> [f32; 8] {
    [
        u8_to_f32(us[1]),
        u8_to_f32(us[3]),
        u8_to_f32(us[5]),
        u8_to_f32(us[7]),
        u8_to_f32(us[9]),
        u8_to_f32(us[11]),
        u8_to_f32(us[13]),
        u8_to_f32(us[15]),
    ]
}

fn u16s_to_f32s(us: [u8; 16]) -> [f32; 8] {
    [
        u16_to_f32(us[0], us[1]),
        u16_to_f32(us[2], us[3]),
        u16_to_f32(us[4], us[5]),
        u16_to_f32(us[6], us[7]),
        u16_to_f32(us[8], us[9]),
        u16_to_f32(us[10], us[11]),
        u16_to_f32(us[12], us[13]),
        u16_to_f32(us[14], us[15]),
    ]
}

fn u16s_to_f32s_left(us: [u8; 32]) -> [f32; 8] {
    [
        u16_to_f32(us[0], us[1]),
        u16_to_f32(us[4], us[5]),
        u16_to_f32(us[8], us[9]),
        u16_to_f32(us[12], us[13]),
        u16_to_f32(us[16], us[17]),
        u16_to_f32(us[20], us[21]),
        u16_to_f32(us[24], us[25]),
        u16_to_f32(us[28], us[29]),
    ]
}

fn u16s_to_f32s_right(us: [u8; 32]) -> [f32; 8] {
    [
        u16_to_f32(us[2], us[3]),
        u16_to_f32(us[6], us[7]),
        u16_to_f32(us[10], us[11]),
        u16_to_f32(us[14], us[15]),
        u16_to_f32(us[18], us[19]),
        u16_to_f32(us[22], us[23]),
        u16_to_f32(us[26], us[27]),
        u16_to_f32(us[30], us[31]),
    ]
}

fn u8_to_f32(u: u8) -> f32 {
    f32::from(u) / f32::from(u8::MAX) * 2. - 1.
}

fn u16_to_f32(l: u8, r: u8) -> f32 {
    let u = u16::from_le_bytes([l, r]);
    f32::from(u) / f32::from(u16::MAX) * 2. - 1.
}

// TODO: pulse (https://github.com/NibbleRealm/twang/blob/v0/src/osc/pulse.rs)
