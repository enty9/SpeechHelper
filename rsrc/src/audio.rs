use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Data, Sample, SampleRate, SampleFormat, StreamConfig, BufferSize};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::time::Duration;
use std::thread;
use wavekat_vad::{VoiceActivityDetector, backends::webrtc::{WebRtcVad, WebRtcVadMode}};

pub fn getAudio(recording: Arc<AtomicBool>, samples: Arc<Mutex<Vec<f32>>>) -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or("Микрофон не найден")?;

    let config = device.default_input_config()?;
    let stream_config: cpal::StreamConfig = config.clone().into();
    let samples_clone = Arc::clone(&samples);

    let value = recording.clone();
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => {
            device.build_input_stream(
                stream_config,

                move |data: &[f32], _| {
                    if value.load(Ordering::Relaxed) {
                        let mut samples = samples_clone.lock().unwrap();

                        samples.extend_from_slice(data);
                    }
                },

                |err| eprintln!("Ошибка микрофона: {}", err),

                None,
            )?
        }

        cpal::SampleFormat::I16 => {
            device.build_input_stream(
                stream_config,

                move |data: &[i16], _| {
                    if value.load(Ordering::Relaxed) {
                        let mut samples = samples_clone.lock().unwrap();

                        for &sample in data {
                            samples.push(
                                sample as f32 / i16::MAX as f32
                            );
                        }
                    }
                },

                |err| eprintln!("Ошибка микрофона: {}", err),

                None,
            )?
        }

        cpal::SampleFormat::U16 => {
            device.build_input_stream(
                stream_config,

                move |data: &[u16], _| {
                    if value.load(Ordering::Relaxed) {
                        let mut samples = samples_clone.lock().unwrap();

                        for &sample in data {
                            let sample =
                                sample as f32 / u16::MAX as f32;

                            samples.push(sample * 2.0 - 1.0);
                        }
                    }
                },

                |err| eprintln!("Ошибка микрофона: {}", err),

                None,
            )?
        }

        _ => return Err("Неподдерживаемый формат".into()),
    };

    stream.play()?;

    while recording.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(10));
    }

    drop(stream);

    Ok(())
}

pub fn playAudio(){

}

pub fn isSpeech(samples: &Vec<f32>) -> bool {
    let new_samples: Vec<i16> = samples.iter().map(move |&x|{ 
        let x = x.clamp(-1.0, 1.0);
        (x * i16::MAX as f32) as i16
    }).collect();

    let mut vad = WebRtcVad::new(16000, WebRtcVadMode::Aggressive).unwrap();
    let result = vad.process(&new_samples, 16000).unwrap();

    if result == 1.0 {
        true
    } else {
        false
    }
}