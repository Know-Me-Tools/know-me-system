// TJ-ARCH-MOB-001 compliant
//! Microphone capture via `cpal`. Runs the platform audio callback on its own
//! OS thread (cpal's contract); samples are forwarded to Rust through a bounded
//! channel and resampled to the mono 16kHz f32 PCM whisper.cpp requires. Never
//! touches the Tokio runtime directly — `Recorder::stop` does the resample/join
//! work, so callers should invoke it from `spawn_blocking`.
use crate::error::{ScribeError, ScribeResult};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc;

/// Sample rate whisper.cpp's ggml models are trained/expect.
pub const WHISPER_SAMPLE_RATE: u32 = 16_000;

/// A live microphone recording. Drop or call `stop` to finalise.
pub struct Recorder {
    stream: cpal::Stream,
    rx: mpsc::Receiver<Vec<f32>>,
    input_rate: u32,
    input_channels: u16,
}

impl Recorder {
    /// Open the default input device and start streaming. Fails fast (Transient
    /// — the caller should surface a permission/no-mic prompt) if no default
    /// input device is available, mirroring how mobile mic-permission denial
    /// should read as "user can fix this", not a crash.
    pub fn start() -> ScribeResult<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| ScribeError::Microphone("no default input device".into()))?;
        let config = device
            .default_input_config()
            .map_err(|e| ScribeError::Microphone(format!("no input config: {e}")))?;

        let input_rate = config.sample_rate();
        let input_channels = config.channels();
        let (tx, rx) = mpsc::channel::<Vec<f32>>();

        let err_tx = tx.clone();
        let stream = device
            .build_input_stream(
                config.into(),
                move |data: &[f32], _| {
                    let _ = tx.send(data.to_vec());
                },
                move |err| {
                    tracing::warn!(%err, "cpal input stream error");
                    // Sending an empty frame on error unblocks a caller waiting
                    // on the channel rather than hanging forever; `stop()` still
                    // reports EmptyRecording if nothing usable arrived.
                    let _ = err_tx.send(Vec::new());
                },
                None,
            )
            .map_err(|e| ScribeError::Microphone(format!("failed to open stream: {e}")))?;

        stream
            .play()
            .map_err(|e| ScribeError::Microphone(format!("failed to start stream: {e}")))?;

        Ok(Self {
            stream,
            rx,
            input_rate,
            input_channels,
        })
    }

    /// Stop capturing and return mono 16kHz f32 PCM ready for `transcribe`.
    pub fn stop(self) -> ScribeResult<Vec<f32>> {
        self.stream.pause().ok();
        drop(self.stream);

        let mut interleaved = Vec::new();
        while let Ok(chunk) = self.rx.try_recv() {
            interleaved.extend(chunk);
        }

        let mono = downmix_to_mono(&interleaved, self.input_channels);
        let resampled = resample_linear(&mono, self.input_rate, WHISPER_SAMPLE_RATE);
        if resampled.is_empty() {
            return Err(ScribeError::EmptyRecording);
        }
        Ok(resampled)
    }
}

/// Average all channels down to mono. A no-op when already mono.
fn downmix_to_mono(interleaved: &[f32], channels: u16) -> Vec<f32> {
    let channels = channels.max(1) as usize;
    if channels == 1 {
        return interleaved.to_vec();
    }
    interleaved
        .chunks(channels)
        .map(|frame| frame.iter().sum::<f32>() / frame.len() as f32)
        .collect()
}

/// Minimal linear resampler. Good enough for speech-to-text (whisper.cpp itself
/// tolerates modest resampling artifacts far better than, say, music); a
/// windowed-sinc resampler would be the upgrade if transcription quality on
/// non-48kHz/44.1kHz devices ever becomes a measured problem.
fn resample_linear(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if samples.is_empty() || from_rate == to_rate {
        return samples.to_vec();
    }
    let ratio = to_rate as f64 / from_rate as f64;
    let out_len = ((samples.len() as f64) * ratio).round() as usize;
    (0..out_len)
        .map(|i| {
            let src_pos = i as f64 / ratio;
            let idx = src_pos.floor() as usize;
            let frac = (src_pos - idx as f64) as f32;
            let a = samples.get(idx).copied().unwrap_or(0.0);
            let b = samples.get(idx + 1).copied().unwrap_or(a);
            a + (b - a) * frac
        })
        .collect()
}
