//! Feeds back the input stream directly into the output stream.
//!
//! Assumes that the input and output devices can use the same stream configuration and that they
//! support the f32 sample format.
//!
//! Uses a delay of `LATENCY_MS` milliseconds in case the default input and output streams are not
//! precisely synchronised.

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Stream,
};
use ringbuf::HeapRb;
use std::mem::MaybeUninit;
use std::sync::Arc;

#[derive(Debug)]
pub struct Options {
    /// The input audio device to use
    pub input_device: String,

    /// The output audio device to use
    pub output_device: String,

    /// Specify the delay between input and output in ms
    pub buffer_length: f32,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            input_device: "default".to_string(),
            output_device: "default".to_string(),
            buffer_length: 30.,
        }
    }
}

type Consumer = ringbuf::Consumer<f32, Arc<ringbuf::SharedRb<f32, Vec<MaybeUninit<f32>>>>>;
type Producer = ringbuf::Producer<f32, Arc<ringbuf::SharedRb<f32, Vec<MaybeUninit<f32>>>>>;

pub enum Input {
    Mic(Stream),
    GPIO,
    None,
}

pub struct Context {
    _input_stream: Input,
    _output_stream: Stream,
    c_in: Consumer,
    c_out: Producer,
}

pub fn create_context(config: Options) -> anyhow::Result<Context> {
    // Conditionally compile with jack if the feature is specified.
    let host = cpal::default_host();
    let opt = config;

    let output_device = if opt.output_device == "default" {
        host.default_output_device()
    } else {
        host.output_devices()?
            .find(|x| x.name().map(|y| y == opt.output_device).unwrap_or(false))
    }
    .expect("failed to find output device");

    //println!("Using input device: \"{}\"", input_device.name()?);
    println!("Using output device: \"{}\"", output_device.name()?);

    // We'll try and use the same configuration between streams to keep it simple.
    let config: cpal::StreamConfig = output_device.default_output_config()?.into();

    // Create a delay in case the input and output devices aren't synced.
    let buffer_frames = (opt.buffer_length / 1_000.0) * config.sample_rate.0 as f32;
    let buffer_samples = buffer_frames as usize * config.channels as usize;

    // The buffer to share samples
    let input_ring = HeapRb::<f32>::new(buffer_samples * 2);
    let output_ring = HeapRb::<f32>::new(buffer_samples * 2);
    let (mut mic, c_in) = input_ring.split();
    let (c_out, mut speaker) = output_ring.split();

    #[cfg(not(feature = "gpio"))]
    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        let mut output_fell_behind = false;
        for &sample in data {
            if let Err(_) = mic.push(sample) {
                output_fell_behind = true;
            }
        }
        if output_fell_behind {
            eprintln!("output stream fell behind: try increasing latency");
        }
    };

    let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        let mut input_fell_behind = false;
        for sample in data {
            *sample = match speaker.pop() {
                Some(s) => s,
                None => {
                    input_fell_behind = true;
                    0.0
                }
            };
        }
        if input_fell_behind {
            eprintln!("input stream fell behind: try increasing latency");
        }
    };

    // Build streams.
    println!(
        "Attempting to build both streams with f32 samples and `{:?}`.",
        config
    );

    // Find devices.
    #[cfg(not(feature = "gpio"))]
    let input_stream = if opt.input_device == "default" {
        host.default_input_device()
    } else {
        host.input_devices()?
            .find(|x| x.name().map(|y| y == opt.input_device).unwrap_or(false))
    }
    .and_then(|device| {
        device
            .build_input_stream(&config, input_data_fn, err_fn)
            .ok()
    })
    .map(Input::Mic)
    .unwrap_or(Input::None);

    #[cfg(feature = "gpio")]
    let input_stream = Input::GPIO;

    let output_stream = output_device.build_output_stream(&config, output_data_fn, err_fn)?;
    println!("Successfully built streams.");

    // Play the streams.
    println!(
        "Starting the input and output streams with `{}` milliseconds of latency.",
        opt.buffer_length
    );

    match &input_stream {
        Input::Mic(stream) => {
            stream.play()?;
        }
        #[cfg(feature = "gpio")]
        Input::GPIO => {
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_millis(100));
                mic.push(0.0).unwrap();
            });
        }
        _ => (),
    };
    output_stream.play()?;

    Ok(Context {
        _input_stream: input_stream,
        _output_stream: output_stream,
        c_in,
        c_out,
    })
}

impl Context {
    pub fn read_samples(&mut self, buffer: &mut [f32]) -> usize {
        let mut i = 0;
        for sample in self.c_in.pop_iter() {
            buffer[i] = sample;
            i += 1;
            if i >= buffer.len() {
                break;
            }
        }
        i
    }

    pub fn write_samples(&mut self, data: &[f32]) -> isize {
        let mut i = 0;
        for sample in data {
            if self.c_out.push(*sample).is_ok() {
                i += 1;
            } else {
                break;
            }
        }
        i
    }
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}
