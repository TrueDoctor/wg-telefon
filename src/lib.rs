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

static mut CONTEXT: Option<Context> = None;

#[derive(Debug)]
pub struct Opt {
    /// The input audio device to use
    pub input_device: String,

    /// The output audio device to use
    pub output_device: String,

    /// Specify the delay between input and output
    pub buffer_length: f32,
}

use core::ffi::c_int;
#[no_mangle]
pub extern "C" fn cinit(buffer_ms: f32) -> c_int {
    match {
        init(Opt {
            input_device: "default".to_string(),
            output_device: "default".to_string(),
            buffer_length: buffer_ms,
        })
    } {
        Ok(_) => 0,
        Err(_) => 1,
    }
}
#[no_mangle]
pub extern "C" fn cfree() {
    let context = unsafe { CONTEXT.as_mut() }.unwrap_or_else(|| panic!("Context not initialized"));
    let context = context as *mut Context;
    unsafe {
        std::mem::drop(context.read());
    }
}

/// # Safety
/// The caller must ensure that the buffer is large enough to hold `buffer_len` samples.
#[no_mangle]
pub unsafe extern "C" fn read_samples(buffer: *mut f32, buffer_len: usize) -> isize {
    let context = unsafe { CONTEXT.as_mut() }.unwrap_or_else(|| panic!("Context not initialized"));

    let mut i = 0;
    for sample in context.c_in.pop_iter() {
        *buffer.offset(i) = sample;
        i += 1;
        if i >= buffer_len as isize {
            break;
        }
    }
    i
}

/// # Safety
/// The caller must ensure that the buffer is large enough to hold `buffer_len` samples.
#[no_mangle]
pub unsafe extern "C" fn write_samples(buffer: *mut f32, buffer_len: usize) -> isize {
    let context = unsafe { CONTEXT.as_mut() }.unwrap_or_else(|| panic!("Context not initialized"));

    let data = unsafe { std::slice::from_raw_parts(buffer, buffer_len) };
    let mut i = 0;
    for &sample in data {
        if context.c_out.push(sample).is_ok() {
            i += 1;
        } else {
            break;
        }
    }
    i
}

type Consumer = ringbuf::Consumer<f32, Arc<ringbuf::SharedRb<f32, Vec<MaybeUninit<f32>>>>>;
type Producer = ringbuf::Producer<f32, Arc<ringbuf::SharedRb<f32, Vec<MaybeUninit<f32>>>>>;

struct Context {
    _input_stream: Stream,
    _output_stream: Stream,
    c_in: Consumer,
    c_out: Producer,
}

pub fn init(config: Opt) -> anyhow::Result<()> {
    // Conditionally compile with jack if the feature is specified.
    let host = cpal::default_host();
    let opt = config;

    // Find devices.
    let input_device = if opt.input_device == "default" {
        host.default_input_device()
    } else {
        host.input_devices()?
            .find(|x| x.name().map(|y| y == opt.input_device).unwrap_or(false))
    }
    .expect("failed to find input device");

    let output_device = if opt.output_device == "default" {
        host.default_output_device()
    } else {
        host.output_devices()?
            .find(|x| x.name().map(|y| y == opt.output_device).unwrap_or(false))
    }
    .expect("failed to find output device");

    println!("Using input device: \"{}\"", input_device.name()?);
    println!("Using output device: \"{}\"", output_device.name()?);

    // We'll try and use the same configuration between streams to keep it simple.
    let config: cpal::StreamConfig = input_device.default_input_config()?.into();

    // Create a delay in case the input and output devices aren't synced.
    let buffer_frames = (opt.buffer_length / 1_000.0) * config.sample_rate.0 as f32;
    let buffer_samples = buffer_frames as usize * config.channels as usize;

    // The buffer to share samples
    let input_ring = HeapRb::<f32>::new(buffer_samples * 2);
    let output_ring = HeapRb::<f32>::new(buffer_samples * 2);
    let (mut mic, c_in) = input_ring.split();
    let (c_out, mut speaker) = output_ring.split();

    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        let mut output_fell_behind = false;
        for &sample in data {
            if let Err(e) = mic.push(sample) {
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
    let input_stream = input_device.build_input_stream(&config, input_data_fn, err_fn)?;
    let output_stream = output_device.build_output_stream(&config, output_data_fn, err_fn)?;
    println!("Successfully built streams.");

    // Play the streams.
    println!(
        "Starting the input and output streams with `{}` milliseconds of latency.",
        opt.buffer_length
    );
    input_stream.play()?;
    output_stream.play()?;

    // Run for 3 seconds before closing.
    //println!("Playing for 3 seconds... ");
    //std::thread::sleep(std::time::Duration::from_secs(30));
    let context = Context {
        _input_stream: input_stream,
        _output_stream: output_stream,
        c_in,
        c_out,
    };
    unsafe {
        CONTEXT = Some(context);
    }
    Ok(())
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}
