use anyhow::{anyhow, Result};
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use std::fs::File;

/// Decoded audio as interleaved f32 samples at the original sample rate.
pub struct DecodedAudio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_secs: f64,
}

pub fn decode_file(path: &str) -> Result<DecodedAudio> {
    let file = File::open(path).map_err(|e| anyhow!("Cannot open {}: {}", path, e))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = std::path::Path::new(path).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let meta_opts = MetadataOptions::default();
    let fmt_opts = FormatOptions::default();
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .map_err(|e| anyhow!("Probe failed: {}", e))?;

    let mut format = probed.format;
    let track = format
        .default_track()
        .ok_or_else(|| anyhow!("No default track"))?;

    let track_id = track.id;
    let codec_params = track.codec_params.clone();
    let sample_rate = codec_params.sample_rate.unwrap_or(48000);
    let channels = codec_params
        .channels
        .map(|c| c.count() as u16)
        .unwrap_or(2);

    let dec_opts = DecoderOptions::default();
    let mut decoder = symphonia::default::get_codecs()
        .make(&codec_params, &dec_opts)
        .map_err(|e| anyhow!("Decoder error: {}", e))?;

    let mut samples: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(_) => break,
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                append_samples(&decoded, &mut samples);
            }
            Err(_) => continue,
        }
    }

    let total_frames = samples.len() as f64 / channels as f64;
    let duration_secs = total_frames / sample_rate as f64;

    Ok(DecodedAudio {
        samples,
        sample_rate,
        channels,
        duration_secs,
    })
}

fn append_samples(buf: &AudioBufferRef<'_>, out: &mut Vec<f32>) {
    match buf {
        AudioBufferRef::F32(b) => {
            for frame_idx in 0..b.frames() {
                for ch in 0..b.spec().channels.count() {
                    out.push(b.chan(ch)[frame_idx]);
                }
            }
        }
        AudioBufferRef::S16(b) => {
            for frame_idx in 0..b.frames() {
                for ch in 0..b.spec().channels.count() {
                    out.push(b.chan(ch)[frame_idx] as f32 / i16::MAX as f32);
                }
            }
        }
        AudioBufferRef::S32(b) => {
            for frame_idx in 0..b.frames() {
                for ch in 0..b.spec().channels.count() {
                    out.push(b.chan(ch)[frame_idx] as f32 / i32::MAX as f32);
                }
            }
        }
        _ => {}
    }
}
