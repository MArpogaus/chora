//! Silero VAD v6 (the `op18_ifless` export) on `tract`: the speech
//! probability of each 32 ms frame at 16 kHz. v5 and v4 do not load in
//! tract (docs/voice.md, WP0).

use tract_onnx::prelude::*;

/// The model, MIT (`LICENSES/MIT-silero-vad.txt`).
const MODEL: &[u8] = include_bytes!("../model/silero_vad.onnx");
/// Samples per frame at 16 kHz.
pub const FRAME: usize = 512;
/// The last samples of the frame before, which the model wants in front.
const CONTEXT: usize = 64;
/// The shape of the recurrent state.
const STATE: [usize; 3] = [2, 1, 128];

/// The model with its recurrent state.
pub struct Vad {
    plan: TypedSimplePlan<TypedModel>,
    state: Tensor,
    context: [f32; CONTEXT],
}

impl Vad {
    /// Loads the model.
    ///
    /// # Errors
    ///
    /// `tract` cannot load or optimize the model.
    pub fn new() -> TractResult<Self> {
        let model = tract_onnx::onnx().model_for_read(&mut std::io::Cursor::new(MODEL))?;
        // The branches name the batch axis; a 1 there does not unify.
        let batch = model.symbols.sym("batch");
        let [b, len] = [TDim::from(batch.clone()), (CONTEXT + FRAME).into()];
        let state = [STATE[0].into(), b.clone(), STATE[2].into()];
        let plan = model
            .with_input_fact(0, f32::fact([b, len]).into())?
            .with_input_fact(1, tensor0(16000i64).into())?
            .with_input_fact(2, f32::fact(state).into())?
            .into_typed()?
            .concretize_dims(&SymbolValues::default().with(&batch, 1))?
            .into_optimized()?
            .into_runnable()?;
        Ok(Self {
            plan,
            state: Tensor::zero::<f32>(&STATE)?,
            context: [0.0; CONTEXT],
        })
    }

    /// The speech probability, from 0 to 1, of one frame of 512 samples
    /// at 16 kHz from -1 to 1.
    ///
    /// # Errors
    ///
    /// The model fails.
    pub fn probability(&mut self, frame: &[f32; FRAME]) -> TractResult<f32> {
        let mut input = Vec::with_capacity(CONTEXT + FRAME);
        input.extend_from_slice(&self.context);
        input.extend_from_slice(frame);
        self.context.copy_from_slice(&frame[FRAME - CONTEXT..]);
        let input = tract_ndarray::Array2::from_shape_vec((1, CONTEXT + FRAME), input)?;
        let state = std::mem::take(&mut self.state);
        let out = self.plan.run(tvec!(
            input.into_tensor().into(),
            tensor0(16000i64).into(),
            state.into()
        ))?;
        self.state = out[1].clone().into_tensor();
        Ok(out[0].as_slice::<f32>()?[0])
    }

    /// Forgets the state, for example after a gap in the audio.
    pub fn reset(&mut self) {
        self.state = Tensor::zero::<f32>(&STATE).expect("a fixed shape");
        self.context = [0.0; CONTEXT];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pcm::{self, Downsampler};

    /// The probabilities of a 24 kHz WAV file, one per 32 ms.
    fn replay(path: &str) -> Vec<f32> {
        let (rate, samples) = pcm::read_wav(std::path::Path::new(path)).unwrap();
        assert_eq!(rate, pcm::RATE);
        let mut vad = Vad::new().map_err(|e| format!("{e:?}")).unwrap();
        let mut down = Downsampler::default();
        samples
            .as_chunks::<{ crate::audio::FRAME }>()
            .0
            .iter()
            .map(|c| {
                let frame: [f32; FRAME] = down.push(c).try_into().unwrap();
                vad.probability(&frame).unwrap()
            })
            .collect()
    }

    #[test]
    fn silero_finds_the_speech_of_a_tts_clip() {
        // 1 s of quiet room, speech from 1.0 s to 4.04 s, 1.5 s quiet.
        let p = replay(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/data/hallo-chora.wav"
        ));
        let at = |ms: usize| ms / 32;
        let loud = p[..at(900)]
            .iter()
            .chain(&p[at(4500)..])
            .filter(|&&x| x > 0.5)
            .count();
        assert_eq!(loud, 0, "no speech in the quiet room: {p:?}");
        let speech = &p[at(1100)..at(3900)];
        let heard = speech.iter().filter(|&&x| x > 0.5).count();
        assert!(
            heard * 10 > speech.len() * 7,
            "{heard} of {}: {p:?}",
            speech.len()
        );
    }
}
