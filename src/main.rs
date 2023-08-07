extern crate cpal;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rand::seq::SliceRandom;
use std::f64::consts::PI;

fn main() {
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let config = device.default_output_config().unwrap();

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()),
        _ => panic!("Unrecognized sample format"),
    }
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig)
where
    T: cpal::Sample + std::convert::From<f32> + cpal::SizedSample,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    const TWELVE_TONE_EQUAL_TEMPERAMENT: [f32; 12] = [
        261.626, 277.183, 293.665, 311.127, 329.628, 349.228, 369.994, 391.995, 415.305, 440.000,
        466.164, 493.883,
    ];

    //random selection of 3 notes from the 12 tone equal temperament scale
    let freqs: Vec<f32> = TWELVE_TONE_EQUAL_TEMPERAMENT
        .choose_multiple(&mut rand::thread_rng(), 3)
        .cloned()
        .collect();

    let mut sample_clock = 0f32;
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(channels) {
                    let value: T = {
                        sample_clock += 1.0;
                        let t = sample_clock / sample_rate;
                        make_tone_waves::<T>(freqs.clone(), &t)
                    };
                    for sample in frame.iter_mut() {
                        *sample = value;
                    }
                }
            },
            err_fn,
            None,
        )
        .unwrap();

    stream.play().unwrap();
    std::thread::sleep(std::time::Duration::from_secs(10));
}

fn make_tone_waves<T>(freqs: Vec<f32>, &t: &f32) -> T
where
    T: cpal::Sample + std::convert::From<f32> + cpal::SizedSample,
{
    let mut y = 0.0;
    for freq in freqs {
        let a = 0.6 * (freq * t * 2.0 * PI as f32).sin() * (-0.0015 * freq * t).exp();
        let b = a + 0.4 * (2.0 * freq * t * 2.0 * PI as f32).sin() * (-0.0015 * freq * t).exp();
        let c = b.powi(3);
        y += c * (1.0 + 16.0 * t * (-6.0 * t).exp());
    }
    (y * 0.5 + 0.5).try_into().unwrap()
}
