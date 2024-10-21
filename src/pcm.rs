use crate::*;
use alloc::vec::Vec;
use core::fmt::Display;

pub enum PcmError {
    TooShort,
    BadMagicNumber,
    BadSampleRate(u16),
}

impl Display for PcmError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TooShort => write!(f, "file is too short"),
            Self::BadMagicNumber => write!(f, "bad magic number"),
            Self::BadSampleRate(sr) => write!(f, "bad sample rate: expected 44100, got {sr}"),
        }
    }
}

/// Play audio from a pulse-code modulated audio file.
pub struct Pcm<R: embedded_io::Read> {
    reader: R,
    _sample_rate: u16,
    is16: bool,
    stereo: bool,
    _adpcm: bool,
}

impl<R: embedded_io::Read> Pcm<R> {
    /// Create the source from a file in the Firefly Zero format.
    ///
    /// # Errors
    ///
    /// Returns an error if the file header is invalid.
    pub fn from_file(mut reader: R) -> Result<Self, PcmError> {
        let mut header = [0u8; 4];
        let res = reader.read_exact(&mut header);
        if res.is_err() {
            return Err(PcmError::TooShort);
        }
        if header[0] != 0x31 {
            return Err(PcmError::BadMagicNumber);
        }
        let sample_rate = u16::from_le_bytes([header[2], header[3]]);
        if sample_rate != 44100 {
            return Err(PcmError::BadSampleRate(sample_rate));
        }
        Ok(Self {
            reader,
            _sample_rate: sample_rate,
            stereo: header[1] & 0b_100 != 0,
            is16: header[1] & 0b_010 != 0,
            _adpcm: header[1] & 0b_001 != 0,
        })
    }
}

impl<R: embedded_io::Read> Processor for Pcm<R> {
    fn process_children(&mut self, _cn: &mut Vec<Node>) -> Option<Frame> {
        let f = match (self.is16, self.stereo) {
            // 8 bit mono
            (false, false) => {
                let mut buf = [0u8; 8];
                self.reader.read_exact(&mut buf).ok()?;
                let s = Sample::new(i8s_to_f32s(buf));
                Frame::mono(s)
            }
            // 8 bit stereo
            (false, true) => {
                let mut buf = [0u8; 16];
                self.reader.read_exact(&mut buf).ok()?;
                let left = Sample::new(i8s_to_f32s_left(buf));
                let right = Sample::new(i8s_to_f32s_right(buf));
                Frame::stereo(left, right)
            }
            // 16 bit mono
            (true, false) => {
                let mut buf = [0u8; 16];
                self.reader.read_exact(&mut buf).ok()?;
                let s = Sample::new(i16s_to_f32s(buf));
                Frame::mono(s)
            }
            // 16 bit stereo
            (true, true) => {
                let mut buf = [0u8; 32];
                self.reader.read_exact(&mut buf).ok()?;
                let left = Sample::new(i16s_to_f32s_left(buf));
                let right = Sample::new(i16s_to_f32s_right(buf));
                Frame::stereo(left, right)
            }
        };
        Some(f)
    }
}

fn i8s_to_f32s(us: [u8; 8]) -> [f32; 8] {
    [
        i8_to_f32(us[0]),
        i8_to_f32(us[1]),
        i8_to_f32(us[2]),
        i8_to_f32(us[3]),
        i8_to_f32(us[4]),
        i8_to_f32(us[5]),
        i8_to_f32(us[6]),
        i8_to_f32(us[7]),
    ]
}

fn i8s_to_f32s_left(us: [u8; 16]) -> [f32; 8] {
    [
        i8_to_f32(us[0]),
        i8_to_f32(us[2]),
        i8_to_f32(us[4]),
        i8_to_f32(us[6]),
        i8_to_f32(us[8]),
        i8_to_f32(us[10]),
        i8_to_f32(us[12]),
        i8_to_f32(us[14]),
    ]
}

fn i8s_to_f32s_right(us: [u8; 16]) -> [f32; 8] {
    [
        i8_to_f32(us[1]),
        i8_to_f32(us[3]),
        i8_to_f32(us[5]),
        i8_to_f32(us[7]),
        i8_to_f32(us[9]),
        i8_to_f32(us[11]),
        i8_to_f32(us[13]),
        i8_to_f32(us[15]),
    ]
}

fn i16s_to_f32s(us: [u8; 16]) -> [f32; 8] {
    [
        i16_to_f32(us[0], us[1]),
        i16_to_f32(us[2], us[3]),
        i16_to_f32(us[4], us[5]),
        i16_to_f32(us[6], us[7]),
        i16_to_f32(us[8], us[9]),
        i16_to_f32(us[10], us[11]),
        i16_to_f32(us[12], us[13]),
        i16_to_f32(us[14], us[15]),
    ]
}

fn i16s_to_f32s_left(us: [u8; 32]) -> [f32; 8] {
    [
        i16_to_f32(us[0], us[1]),
        i16_to_f32(us[4], us[5]),
        i16_to_f32(us[8], us[9]),
        i16_to_f32(us[12], us[13]),
        i16_to_f32(us[16], us[17]),
        i16_to_f32(us[20], us[21]),
        i16_to_f32(us[24], us[25]),
        i16_to_f32(us[28], us[29]),
    ]
}

fn i16s_to_f32s_right(us: [u8; 32]) -> [f32; 8] {
    [
        i16_to_f32(us[2], us[3]),
        i16_to_f32(us[6], us[7]),
        i16_to_f32(us[10], us[11]),
        i16_to_f32(us[14], us[15]),
        i16_to_f32(us[18], us[19]),
        i16_to_f32(us[22], us[23]),
        i16_to_f32(us[26], us[27]),
        i16_to_f32(us[30], us[31]),
    ]
}

fn i8_to_f32(u: u8) -> f32 {
    #[expect(clippy::cast_possible_wrap)]
    let i = u as i8;
    f32::from(i) / f32::from(i8::MAX)
}

fn i16_to_f32(l: u8, r: u8) -> f32 {
    let i = i16::from_le_bytes([l, r]);
    f32::from(i) / f32::from(i16::MAX)
}
