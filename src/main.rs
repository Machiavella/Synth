// src/main.rs
// Requires Cargo.toml with: cpal = "0.15", egui = "0.27", eframe = { version = "0.27", features = ["wgpu"] }, anyhow = "1"

use std::f32::consts::TAU;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};

use eframe::egui;
use eframe::egui::Color32;

/// Helper to store/load f32 in AtomicU32
fn load_f32(a: &AtomicU32) -> f32 {
    f32::from_bits(a.load(Ordering::SeqCst))
}
fn store_f32(a: &AtomicU32, v: f32) {
    a.store(v.to_bits(), Ordering::SeqCst)
}

/// Preset descriptor (pure data)
#[derive(Clone)]
struct Preset {
    name: &'static str,
    osc_mix: f32,
    detune: f32,
    gain: f32,
}

impl Preset {
    fn ryan_josh() -> Self {
        Self {
            name: "Ryan & Josh Allen (romantic)",
            osc_mix: 0.25,
            detune: 2.0,
            gain: 0.45,
        }
    }
    fn laura_les() -> Self {
        Self {
            name: "Laura Les (fast hyperpopish)",
            osc_mix: 0.85,
            detune: 8.0,
            gain: 0.75,
        }
    }
}

/// Shared state between UI and audio. All fields audio reads are atomic (lock-free).
struct SharedState {
    // human readable preset name for the UI:
    preset_name: Mutex<String>,

    // synth params stored as atomics (f32 via AtomicU32)
    osc_mix: AtomicU32,
    detune: AtomicU32,
    gain: AtomicU32,

    // master gain
    master_gain: AtomicU32,

    // disco mode and ad tick
    disco: AtomicBool,
    ad_tick: AtomicU32,

    // frequency (Hz) for demo tone
    freq_hz: AtomicU32,
}

impl SharedState {
    fn new() -> Self {
        let preset = Preset::ryan_josh();
        let s = SharedState {
            preset_name: Mutex::new(preset.name.to_string()),
            osc_mix: AtomicU32::new(preset.osc_mix.to_bits()),
            detune: AtomicU32::new(preset.detune.to_bits()),
            gain: AtomicU32::new(preset.gain.to_bits()),
            master_gain: AtomicU32::new(0.8f32.to_bits()),
            disco: AtomicBool::new(false),
            ad_tick: AtomicU32::new(0),
            freq_hz: AtomicU32::new((220.0f32).to_bits()), // default 220Hz
        };
        s
    }

    fn apply_preset(&self, p: &Preset) {
        if let Ok(mut name) = self.preset_name.lock() {
            *name = p.name.to_string();
        }
        store_f32(&self.osc_mix, p.osc_mix);
        store_f32(&self.detune, p.detune);
        store_f32(&self.gain, p.gain);
    }
}

// ---------- GUI + App ----------

struct SynthApp {
    state: Arc<SharedState>,
}

impl eframe::App for SynthApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Disco color cycling if enabled
        let disco_on = self.state.disco.load(Ordering::SeqCst);
        if disco_on {
            // increment ad tick
            self.state.ad_tick.fetch_add(1, Ordering::SeqCst);

            let t = (Instant::now().elapsed().as_millis() as f32 / 200.0).sin();
            let accent = Color32::from_rgb(
                ((t * 0.5 + 0.5) * 255.0) as u8,
                ((-t * 0.8 + 0.5) * 255.0) as u8,
                ((t * 0.2 + 0.4) * 255.0) as u8,
            );
            let mut visuals = (*ctx.style()).visuals.clone();
            visuals.widgets.inactive.bg_fill = accent;
            ctx.set_visuals(visuals);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Rust Synth Prototype — built-in presets (egui) ");
            ui.horizontal(|ui| {
                if ui.button("Load: Ryan & Josh Allen (romantic)").clicked() {
                    self.state.apply_preset(&Preset::ryan_josh());
                }
                if ui.button("Load: Laura Les (fast hyperpopish)").clicked() {
                    self.state.apply_preset(&Preset::laura_les());
                }
                let mut disco_val = self.state.disco.load(Ordering::SeqCst);
                let mut disco_bool = disco_val;
                if ui.checkbox(&mut disco_bool, "Disco mode").changed() {
                    self.state.disco.store(disco_bool, Ordering::SeqCst);
                }
            });

            ui.separator();

            // show and tweak current preset atomics (read current values)
            let osc_mix = load_f32(&self.state.osc_mix);
            let detune = load_f32(&self.state.detune);
            let gain = load_f32(&self.state.gain);
            let mut osc_mix_mut = osc_mix;
            let mut detune_mut = detune;
            let mut gain_mut = gain;

            ui.label(format!(
                "Preset: {}",
                self.state.preset_name.lock().unwrap()
            ));
            ui.add(egui::Slider::new(&mut osc_mix_mut, 0.0..=1.0).text("osc mix"));
            ui.add(
                egui::DragValue::new(&mut detune_mut)
                    .speed(0.1)
                    .clamp_range(-100.0..=100.0)
                    .prefix("detune: "),
            );
            ui.add(egui::Slider::new(&mut gain_mut, 0.0..=2.0).text("gain"));

            if ui.button("Apply changes").clicked() {
                store_f32(&self.state.osc_mix, osc_mix_mut);
                store_f32(&self.state.detune, detune_mut);
                store_f32(&self.state.gain, gain_mut);
            }

            ui.separator();

            let mut mg = load_f32(&self.state.master_gain);
            if ui
                .add(egui::Slider::new(&mut mg, 0.0..=2.0).text("master gain"))
                .changed()
            {
                store_f32(&self.state.master_gain, mg);
            }

            ui.separator();
            ui.label("Advertisement area (disco mode spams this when enabled):");
            if self.state.disco.load(Ordering::SeqCst) {
                for i in 0..6 {
                    ui.colored_label(
                        Color32::LIGHT_YELLOW,
                        format!("🔥 SUPER SYNTH SALE! BUY NOW — LIMITED TIME! ({})", i),
                    );
                }
            } else {
                ui.label("(disco mode disabled)");
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.small("Demo: monophonic continuous tone driven by preset parameters.");
            });
        });

        ctx.request_repaint_after(Duration::from_millis(16));
    }
}

// ---------- Audio: CPAL stream builders ----------

fn start_audio_thread(state: Arc<SharedState>) -> Result<()> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::anyhow!("No output device"))?;
    let cfg = device.default_output_config()?;
    let sample_rate = cfg.sample_rate().0 as f32;
    let config: StreamConfig = cfg.clone().into();

    // spawn appropriate stream based on format
    let stream = match cfg.sample_format() {
        SampleFormat::F32 => build_stream_f32(&device, &config, sample_rate, state.clone())?,
        SampleFormat::I16 => build_stream_i16(&device, &config, sample_rate, state.clone())?,
        SampleFormat::U16 => build_stream_u16(&device, &config, sample_rate, state.clone())?,
        _ => return Err(anyhow::anyhow!("Unsupported sample format")),
    };

    stream.play()?;
    // keep thread alive while audio plays
    loop {
        thread::sleep(Duration::from_millis(200));
    }
}

/// Basic oscillator: two slightly-detuned sines mixed
fn synth_sample(sample_phase: f32, osc_mix: f32, detune: f32, gain: f32, master: f32) -> f32 {
    // detune: interpret as cents-ish fraction scaled small
    let detune_frac = detune * 0.001; // small demo scaling
    let a = (sample_phase * TAU).sin();
    let b = ((sample_phase + detune_frac).fract() * TAU).sin();
    ((1.0 - osc_mix) * a + osc_mix * b) * gain * master
}

fn build_stream_f32(
    device: &cpal::Device,
    config: &StreamConfig,
    sample_rate: f32,
    state: Arc<SharedState>,
) -> Result<cpal::Stream, anyhow::Error> {
    let channels = config.channels as usize;
    let mut phase: f32 = 0.0;

    let stream = device.build_output_stream(
        config,
        move |data: &mut [f32], _| {
            let osc_mix = load_f32(&state.osc_mix);
            let detune = load_f32(&state.detune);
            let gain = load_f32(&state.gain);
            let master = load_f32(&state.master_gain);
            // read freq once
            let freq = load_f32(&state.freq_hz);

            let step = freq / sample_rate;
            for frame in data.chunks_mut(channels) {
                let s = synth_sample(phase.fract(), osc_mix, detune, gain, master);
                for sample in frame.iter_mut() {
                    *sample = s;
                }
                phase = (phase + step) % 1.0;
            }
        },
        |err| eprintln!("audio err: {}", err),
        None,
    )?;

    Ok(stream)
}

fn build_stream_i16(
    device: &cpal::Device,
    config: &StreamConfig,
    sample_rate: f32,
    state: Arc<SharedState>,
) -> Result<cpal::Stream, anyhow::Error> {
    let channels = config.channels as usize;
    let mut phase: f32 = 0.0;

    let stream = device.build_output_stream(
        config,
        move |data: &mut [i16], _| {
            let osc_mix = load_f32(&state.osc_mix);
            let detune = load_f32(&state.detune);
            let gain = load_f32(&state.gain);
            let master = load_f32(&state.master_gain);
            let freq = load_f32(&state.freq_hz);

            let step = freq / sample_rate;
            for frame in data.chunks_mut(channels) {
                let s = synth_sample(phase.fract(), osc_mix, detune, gain, master);
                // clamp & scale to i16
                let scaled = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                for sample in frame.iter_mut() {
                    *sample = scaled;
                }
                phase = (phase + step) % 1.0;
            }
        },
        |err| eprintln!("audio err: {}", err),
        None,
    )?;

    Ok(stream)
}

fn build_stream_u16(
    device: &cpal::Device,
    config: &StreamConfig,
    sample_rate: f32,
    state: Arc<SharedState>,
) -> Result<cpal::Stream, anyhow::Error> {
    let channels = config.channels as usize;
    let mut phase: f32 = 0.0;

    let stream = device.build_output_stream(
        config,
        move |data: &mut [u16], _| {
            let osc_mix = load_f32(&state.osc_mix);
            let detune = load_f32(&state.detune);
            let gain = load_f32(&state.gain);
            let master = load_f32(&state.master_gain);
            let freq = load_f32(&state.freq_hz);

            let step = freq / sample_rate;
            for frame in data.chunks_mut(channels) {
                let s = synth_sample(phase.fract(), osc_mix, detune, gain, master);
                // convert from [-1,1] to [0, u16::MAX]
                let scaled = (((s.clamp(-1.0, 1.0) * 0.5) + 0.5) * u16::MAX as f32) as u16;
                for sample in frame.iter_mut() {
                    *sample = scaled;
                }
                phase = (phase + step) % 1.0;
            }
        },
        |err| eprintln!("audio err: {}", err),
        None,
    )?;

    Ok(stream)
}

// ---------- main ----------

fn main() {
    let shared = Arc::new(SharedState::new());

    // apply initial preset
    shared.apply_preset(&Preset::ryan_josh());

    // spawn audio thread
    {
        let s = shared.clone();
        thread::spawn(move || {
            if let Err(e) = start_audio_thread(s) {
                eprintln!("Audio thread error: {:?}", e);
            }
        });
    }

    // run eframe GUI
    let options = eframe::NativeOptions::default();
    let app = SynthApp { state: shared };
    if let Err(e) = eframe::run_native(
        "Rust Synth Prototype",
        options,
        Box::new(|_cc| Box::new(app)),
    ) {
        eprintln!("eframe error: {:?}", e);
    }
}
