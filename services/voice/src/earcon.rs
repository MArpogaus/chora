//! Earcons: short synthesized tones for the state changes.

use crate::pcm::RATE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Earcon {
    Wake,
    Sleep,
    Cancel,
    Error,
}

impl Earcon {
    /// The notes: frequency in Hz and length in ms.
    fn notes(self) -> &'static [(f64, u32)] {
        match self {
            Self::Wake => &[(660.0, 80), (880.0, 120)],
            Self::Sleep => &[(880.0, 80), (660.0, 120)],
            Self::Cancel => &[(440.0, 60), (330.0, 100)],
            Self::Error => &[(220.0, 250)],
        }
    }

    /// The PCM at 24 kHz; `volume` from 0 to 1.
    pub fn pcm(self, volume: f64) -> Vec<i16> {
        let amplitude = volume.clamp(0.0, 1.0) * 16_000.0;
        let fade = f64::from(RATE) * 0.005;
        let mut pcm = Vec::new();
        for &(freq, ms) in self.notes() {
            let n = RATE * ms / 1000;
            pcm.extend((0..n).map(|i| {
                let (i, n) = (f64::from(i), f64::from(n));
                // 5 ms fades keep the tone free of clicks.
                let envelope = (i / fade).min((n - i) / fade).min(1.0);
                let t = i / f64::from(RATE);
                let s = amplitude * envelope * (std::f64::consts::TAU * freq * t).sin();
                #[allow(clippy::cast_possible_truncation)]
                let s = s as i16;
                s
            }));
        }
        pcm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rms(samples: &[i16]) -> f64 {
        let n: f64 = samples.iter().map(|_| 1.0).sum();
        (samples.iter().map(|&s| f64::from(s).powi(2)).sum::<f64>() / n).sqrt()
    }

    #[test]
    fn earcons_are_short_soft_and_scale_with_the_volume() {
        let wake = Earcon::Wake.pcm(1.0);
        assert_eq!(wake.len(), 24 * 200, "200 ms");
        assert!(wake.first().unwrap().abs() < 100 && wake.last().unwrap().abs() < 100);
        let half = rms(&Earcon::Wake.pcm(0.5)) / rms(&wake);
        assert!((half - 0.5).abs() < 0.01, "{half}");
        assert!(Earcon::Error.pcm(0.0).iter().all(|&s| s == 0));
    }
}
