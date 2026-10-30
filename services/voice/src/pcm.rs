//! PCM helpers: 16 bit mono samples, the 24 kHz to 16 kHz step for the
//! VAD, and WAV files.

use std::{io, path::Path};

/// The rate of the mic, the ASR and the TTS (WP0: the ASR takes 24 kHz
/// only).
pub const RATE: u32 = 24_000;

/// Taps of the low-pass filter in front of the 3:2 step.
const TAPS: u32 = 33;

/// 24 kHz to 16 kHz for the VAD: a windowed-sinc low-pass at 7 kHz, then
/// every sample at 1.5 steps. It keeps the last samples, so a stream of
/// frames gives the same result as one long buffer.
pub struct Downsampler {
    taps: [f32; TAPS as usize],
    history: Vec<f32>,
}

impl Default for Downsampler {
    fn default() -> Self {
        use std::f64::consts::PI;
        let cutoff = 7_000.0 / f64::from(RATE);
        let middle = f64::from(TAPS / 2);
        let taps: Vec<f64> = (0..TAPS)
            .map(|k| {
                let x = f64::from(k) - middle;
                let sinc = if x == 0.0 {
                    2.0 * cutoff
                } else {
                    (2.0 * PI * cutoff * x).sin() / (PI * x)
                };
                let hamming = 0.54 - 0.46 * (2.0 * PI * f64::from(k) / f64::from(TAPS - 1)).cos();
                sinc * hamming
            })
            .collect();
        let sum: f64 = taps.iter().sum();
        #[allow(clippy::cast_possible_truncation)]
        let taps = std::array::from_fn(|k| (taps[k] / sum) as f32);
        Self {
            taps,
            history: vec![0.0; TAPS as usize - 1],
        }
    }
}

impl Downsampler {
    /// Two samples at 16 kHz for each three at 24 kHz, from -1 to 1. The
    /// input length must be a multiple of 3.
    pub fn push(&mut self, input: &[i16]) -> Vec<f32> {
        debug_assert!(input.len().is_multiple_of(3));
        self.history
            .extend(input.iter().map(|&s| f32::from(s) / 32768.0));
        let filtered: Vec<f32> = self
            .history
            .windows(TAPS as usize)
            .map(|w| w.iter().zip(&self.taps).map(|(a, b)| a * b).sum())
            .collect();
        self.history.drain(..input.len());
        filtered
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|[a, b, c]| [*a, f32::midpoint(*b, *c)])
            .collect()
    }
}

/// Little-endian bytes to samples; an odd last byte drops.
#[must_use]
pub fn samples(bytes: &[u8]) -> Vec<i16> {
    bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&b| i16::from_le_bytes(b))
        .collect()
}

/// Samples to little-endian bytes.
#[must_use]
pub fn bytes(samples: &[i16]) -> Vec<u8> {
    samples.iter().flat_map(|s| s.to_le_bytes()).collect()
}

/// Reads a WAV file of 16 bit mono PCM: its rate and samples.
///
/// # Errors
///
/// The file cannot be read, or it is not 16 bit mono PCM.
pub fn read_wav(path: &Path) -> io::Result<(u32, Vec<i16>)> {
    let data = std::fs::read(path)?;
    let bad = |what: &str| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{}: {what}", path.display()),
        )
    };
    if data.len() < 12 || &data[..4] != b"RIFF" || &data[8..12] != b"WAVE" {
        return Err(bad("no WAV file"));
    }
    let (mut at, mut rate) = (12, None);
    while let Some((id, body, next)) = chunk(&data, at) {
        match id {
            b"fmt " => rate = Some(format(body).ok_or_else(|| bad("not 16 bit mono PCM"))?),
            b"data" => return Ok((rate.ok_or_else(|| bad("data before fmt"))?, samples(body))),
            _ => {}
        }
        at = next;
    }
    Err(bad("no data"))
}

/// The chunk at `at`: its id, its body and where the next one starts. A
/// recording that did not end has no length in its data chunk yet, or an
/// old one: its body goes to the end of the file.
fn chunk(data: &[u8], at: usize) -> Option<(&[u8], &[u8], usize)> {
    let id = data.get(at..at + 4)?;
    let len = u32::from_le_bytes(data.get(at + 4..at + 8)?.try_into().ok()?) as usize;
    let body = match data.get(at + 8..at + 8 + len) {
        Some(body) if !(id == b"data" && len == 0) => body,
        _ if id == b"data" => &data[at + 8..],
        _ => return None,
    };
    Some((id, body, at + 8 + len + len % 2))
}

/// The rate of a `fmt ` chunk of 16 bit mono PCM.
fn format(body: &[u8]) -> Option<u32> {
    let field = |at: usize| u16::from_le_bytes([body[at], body[at + 1]]);
    let pcm16 = body.len() >= 16 && (field(0), field(2), field(14)) == (1, 1, 16);
    pcm16.then(|| u32::from_le_bytes(body[4..8].try_into().expect("4 bytes")))
}

/// The header of a WAV file of 16 bit mono PCM; `len` is the number of
/// bytes of the samples, 0 while they still grow.
#[must_use]
pub fn wav_header(rate: u32, len: u32) -> [u8; 44] {
    let mut h = [0; 44];
    h[..4].copy_from_slice(b"RIFF");
    h[4..8].copy_from_slice(&len.saturating_add(36).to_le_bytes());
    h[8..16].copy_from_slice(b"WAVEfmt ");
    h[16..20].copy_from_slice(&16u32.to_le_bytes());
    h[20..22].copy_from_slice(&1u16.to_le_bytes());
    h[22..24].copy_from_slice(&1u16.to_le_bytes());
    h[24..28].copy_from_slice(&rate.to_le_bytes());
    h[28..32].copy_from_slice(&(rate * 2).to_le_bytes());
    h[32..34].copy_from_slice(&2u16.to_le_bytes());
    h[34..36].copy_from_slice(&16u16.to_le_bytes());
    h[36..40].copy_from_slice(b"data");
    h[40..44].copy_from_slice(&len.to_le_bytes());
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(freq: f64, n: u32) -> Vec<i16> {
        (0..n)
            .map(|i| {
                let t = f64::from(i) / f64::from(RATE);
                #[allow(clippy::cast_possible_truncation)]
                let s = (10_000.0 * (std::f64::consts::TAU * freq * t).sin()) as i16;
                s
            })
            .collect()
    }

    /// The level of a sine of amplitude 10000, from 0 to 1.
    fn level(x: &[f32]) -> f32 {
        let n: f32 = x.iter().map(|_| 1.0).sum();
        (x.iter().map(|s| s * s).sum::<f32>() / n).sqrt() / (10_000.0 / 32768.0 / 2f32.sqrt())
    }

    fn down(input: &[i16]) -> Vec<f32> {
        let mut down = Downsampler::default();
        input.chunks(768).flat_map(|c| down.push(c)).collect()
    }

    #[test]
    fn the_downsampler_keeps_speech_and_drops_what_16_khz_cannot_hold() {
        let out = down(&sine(1_000.0, 7_680));
        assert_eq!(out.len(), 5_120, "two out per three in");
        let kept = level(&out[100..]);
        assert!((0.9..1.1).contains(&kept), "1 kHz keeps its level: {kept}");
        // 1 kHz at 16 kHz: 16 samples per period, so one period later the
        // sample repeats.
        assert!((out[1000] - out[1016]).abs() < 0.02);
        let alias = level(&down(&sine(10_000.0, 7_680))[100..]);
        assert!(alias < 0.1, "10 kHz would fold to 6 kHz: {alias}");
    }

    #[test]
    fn a_wav_header_round_trips() {
        let dir = std::env::temp_dir().join(format!("chora-voice-wav-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("a.wav");
        let samples = [1i16, -2, 300, i16::MIN];
        let mut file = wav_header(RATE, 8).to_vec();
        file.extend(bytes(&samples));
        std::fs::write(&path, file).unwrap();
        assert_eq!(read_wav(&path).unwrap(), (RATE, samples.to_vec()));
        std::fs::write(&path, b"RIFF....WAVEfmt ").unwrap();
        assert!(read_wav(&path).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
