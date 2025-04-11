use wasm_bindgen::prelude::*;
use web_sys::console;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Stream;
use oxisynth::MidiEvent;
use std::sync::mpsc::{Receiver, Sender};
use std::io::Cursor;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    Ok(())
}

#[wasm_bindgen]
#[allow(dead_code)]
pub struct Handle(Stream, Sender<MidiEvent>);

impl Handle {
    fn note_on(&mut self, channel: u8, note: u8, velocity: u8) {
        self.1.send(MidiEvent::NoteOn { channel, key: note, vel: velocity }).ok();
    }
    fn note_off(&mut self, channel: u8, note: u8) {
        self.1.send(MidiEvent::NoteOff { channel, key: note }).ok();
    }
    fn program_change(&mut self, channel: u8, program_id: u8) {
        self.1.send(MidiEvent::ProgramChange { channel, program_id }).ok();
    }
    fn control_change(&mut self, channel: u8, control: u8, value: u8) {
        self.1.send(MidiEvent::ControlChange { channel, ctrl: control, value }).ok();
    }
    fn all_notes_off(&mut self, channel: u8) {
        self.1.send(MidiEvent::AllNotesOff { channel }).ok();
    }
    fn all_sounds_off(&mut self, channel: u8) {
        self.1.send(MidiEvent::AllSoundOff { channel }).ok();
    }
    fn pitch_bend(&mut self, channel: u8, value: u16) {
        self.1.send(MidiEvent::PitchBend { channel, value }).ok();
    }
}

#[wasm_bindgen]
pub fn note_on(h: &mut Handle, channel: i32, note: i32, velocity: i32) {
    h.note_on(channel as _, note as _, velocity as _);
}

#[wasm_bindgen]
pub fn note_off(h: &mut Handle, channel: i32, note: i32) {
    h.note_off(channel as _, note as _);
}

#[wasm_bindgen]
pub fn program_change(h: &mut Handle, channel: i32, program_id: i32) {
    h.program_change(channel as _, program_id as _);
}

#[wasm_bindgen] 
pub fn control_change(h: &mut Handle, channel: i32, control: i32, value: i32) {
    h.control_change(channel as _, control as _, value as _);
}

#[wasm_bindgen]
pub fn all_notes_off(h: &mut Handle, channel: i32) {
    h.all_notes_off(channel as _);
}   

#[wasm_bindgen]
pub fn all_sounds_off(h: &mut Handle, channel: i32) {
    h.all_sounds_off(channel as _);
}

#[wasm_bindgen]
pub fn pitch_bend(h: &mut Handle, channel: i32, value: i32) {
    h.pitch_bend(channel as _, value as _);
}

#[wasm_bindgen]
pub fn load_sound_font(sound_font_data: Vec<u8>) -> Handle {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config().unwrap();

    let (tx, rx) = std::sync::mpsc::channel::<MidiEvent>();
    Handle(
        match config.sample_format() {
            cpal::SampleFormat::F32 => run::<f32>(sound_font_data, &device, &config.into(), rx),
            cpal::SampleFormat::I16 => run::<i16>(sound_font_data, &device, &config.into(), rx),
            cpal::SampleFormat::U16 => run::<u16>(sound_font_data, &device, &config.into(), rx),
            _ => panic!("unsupported sample format"),
        },
        tx,
    )
}

fn run<T>(sound_font_data: Vec<u8>, device: &cpal::Device, config: &cpal::StreamConfig, rx: Receiver<MidiEvent>) -> Stream
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let mut synth = {
        let settings = oxisynth::SynthDescriptor {
            sample_rate,
            gain: 1.0,
            ..Default::default()
        };

        let mut synth = oxisynth::Synth::new(settings).unwrap();

        let mut sound_font_cursor = Cursor::new(sound_font_data);
        let font = oxisynth::SoundFont::load(&mut sound_font_cursor).unwrap();

        synth.add_font(font, true);
        synth.set_sample_rate(sample_rate);
        synth.set_gain(1.0);

        synth
    };

    let mut next_value = move || {
        let (l, r) = synth.read_next();

        if let Ok(e) = rx.try_recv() {
            synth.send_event(e).ok();
        }

        (l, r)
    };

    let err_fn = |err| console::error_1(&format!("an error occurred on stream: {}", err).into());

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _| write_data(data, channels, &mut next_value),
            err_fn,
            None,
        )
        .unwrap();
    stream.play().unwrap();
    stream
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> (f32, f32))
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let (l, r) = next_sample();

        let channels = [
            T::from_sample::<f32>(l),
            T::from_sample::<f32>(r)
        ];

        for (id, sample) in frame.iter_mut().enumerate() {
            *sample = channels[id % 2];
        }
    }
}
