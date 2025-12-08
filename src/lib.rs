mod drum_engine;
mod dsp;
mod kits;
mod params;

use crate::dsp::fast_tanh;
use drum_engine::{DrumSlot, N_SLOTS, SLOT_TYPES};
use nih_plug::prelude::*;
use params::{DrumParams, DrumSlotParams, MasterParams};
use std::num::NonZeroU32;
use std::sync::Arc;

// Plugin struct

pub struct Drumini {
    params: Arc<DrumParams>,
    sample_rate: f32,
    slots: [DrumSlot; N_SLOTS],

    comp: SimpleComp,
    reverb: SimpleReverb,
}

impl Default for Drumini {
    fn default() -> Self {
        let sr = 44100.0;
        let params = Arc::new(DrumParams::default());
        let slots = core::array::from_fn(|i| DrumSlot::new(SLOT_TYPES[i], sr));

        Self {
            params,
            sample_rate: sr,
            slots,
            comp: SimpleComp::new(sr),
            reverb: SimpleReverb::new(sr),
        }
    }
}

// Plugin impl

impl Plugin for Drumini {
    const NAME: &'static str = "Drumini";
    const VENDOR: &'static str = "me";
    const URL: &'static str = "https://github.com";
    const EMAIL: &'static str = "me@later.com";
    const VERSION: &'static str = "0.1.0";

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: NonZeroU32::new(2),
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _io: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _ctx: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate.max(1.0);
        for slot in &mut self.slots {
            slot.set_sample_rate(self.sample_rate);
        }
        self.comp.set_sample_rate(self.sample_rate);
        self.reverb.set_sample_rate(self.sample_rate);
        true
    }

    fn reset(&mut self) {
        for (i, slot) in self.slots.iter_mut().enumerate() {
            *slot = DrumSlot::new(SLOT_TYPES[i], self.sample_rate);
        }
        self.comp.reset();
        self.reverb.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer<'_>,
        _aux: &mut AuxiliaryBuffers<'_>,
        ctx: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let params = self.params.clone();
        let mut next_event = ctx.next_event();

        for (sample_idx, mut frame) in buffer.iter_samples().enumerate() {
            // Sample-accurate events
            while let Some(ev) = next_event {
                if ev.timing() != sample_idx as u32 {
                    break;
                }

                match ev {
                    NoteEvent::NoteOn { note, velocity, .. } => {
                        if let Some(slot_idx) = note_to_slot(note) {
                            let vel = velocity.clamp(0.0, 1.0);
                            let p = params.as_ref();
                            let slot_params = match_slot_params(slot_idx, p);
                            let master = &p.master;
                            self.slots[slot_idx].trigger(vel, slot_params, master);
                        }
                    }
                    NoteEvent::NoteOff { .. } => {
                        // One-shot drums, ignore for now
                    }
                    _ => {}
                }

                next_event = ctx.next_event();
            }

            // Render and mix slots
            let mut l = 0.0f32;
            let mut r = 0.0f32;

            {
                let p = params.as_ref();
                let master = &p.master;

                for (i, slot) in self.slots.iter_mut().enumerate() {
                    let slot_params = match_slot_params(i, p);
                    let y = slot.process(slot_params, master);

                    let pan = slot_params.pan.value().clamp(-1.0, 1.0);
                    let level = slot_params.level.value();

                    let (gain_l, gain_r) = pan_to_gains(pan);
                    l += y * level * gain_l;
                    r += y * level * gain_r;
                }

                // Master drive (saturation)
                let drive = master.drive.value().clamp(0.0, 1.0);
                if drive > 0.0 {
                    let drive_gain = 1.0 + drive * 4.0;
                    let makeup = 1.0 / (1.0 + drive * 2.0);
                    l = fast_tanh(l * drive_gain) * makeup;
                    r = fast_tanh(r * drive_gain) * makeup;
                }

                // Master compressor
                let comp_amt = master.comp.value().clamp(0.0, 1.0);
                let (cl, cr) = self.comp.process(l, r, comp_amt);
                l = cl;
                r = cr;

                // Simple room-ish reverb
                let rev_amt = master.reverb.value().clamp(0.0, 1.0);
                let (rl, rr) = self.reverb.process(l, r, rev_amt);
                l = rl;
                r = rr;
            }

            let mut channels = frame.iter_mut();
            if let Some(out_l) = channels.next() {
                *out_l = l;
            }
            if let Some(out_r) = channels.next() {
                *out_r = r;
            }
        }

        ProcessStatus::Normal
    }
}

// Helpers

fn pan_to_gains(pan: f32) -> (f32, f32) {
    // Simple equal-power panning
    let x = (pan + 1.0) * 0.5; // 0..1
    let theta = x * std::f32::consts::FRAC_PI_2;
    (theta.cos(), theta.sin())
}

/// Fixed mapping from MIDI notes to slot indices.
fn note_to_slot(note: u8) -> Option<usize> {
    match note {
        36 => Some(0),           // Kick
        38 => Some(1),           // Snare
        39 => Some(2),           // Clap
        42 => Some(3),           // Closed Hat
        46 => Some(4),           // Open Hat
        43 | 45 | 47 => Some(5), // Toms -> Tom slot
        49 => Some(6),           // Perc 1
        51 => Some(7),           // Perc 2
        _ => None,
    }
}

/// Return the DrumSlotParams for a slot index.
fn match_slot_params<'a>(index: usize, params: &'a DrumParams) -> &'a DrumSlotParams {
    match index {
        0 => &params.kick,
        1 => &params.snare,
        2 => &params.clap,
        3 => &params.hat_closed,
        4 => &params.hat_open,
        5 => &params.tom,
        6 => &params.perc1,
        7 => &params.perc2,
        _ => &params.kick,
    }
}

// Simple bus compressor

struct SimpleComp {
    sr: f32,
    env: f32,
    gain_smooth: f32,
    atk_coeff: f32,
    rel_coeff: f32,
}

impl SimpleComp {
    fn new(sr: f32) -> Self {
        let mut s = Self {
            sr: sr.max(1.0),
            env: 0.0,
            gain_smooth: 1.0,
            atk_coeff: 0.0,
            rel_coeff: 0.0,
        };
        s.update_time_constants();
        s
    }

    fn set_sample_rate(&mut self, sr: f32) {
        self.sr = sr.max(1.0);
        self.update_time_constants();
    }

    fn reset(&mut self) {
        self.env = 0.0;
        self.gain_smooth = 1.0;
    }

    fn update_time_constants(&mut self) {
        // Simple fixed times
        let atk_ms = 5.0;
        let rel_ms = 80.0;
        self.atk_coeff = (-1.0 / ((atk_ms / 1000.0) * self.sr)).exp();
        self.rel_coeff = (-1.0 / ((rel_ms / 1000.0) * self.sr)).exp();
    }

    fn process(&mut self, l: f32, r: f32, amount: f32) -> (f32, f32) {
        let amt = amount.clamp(0.0, 1.0);
        if amt <= 0.001 {
            return (l, r);
        }

        let x = l.abs().max(r.abs());
        let target = x;

        if target > self.env {
            self.env = self.atk_coeff * self.env + (1.0 - self.atk_coeff) * target;
        } else {
            self.env = self.rel_coeff * self.env + (1.0 - self.rel_coeff) * target;
        }

        let eps = 1e-8;
        let level_lin = (self.env).max(eps);
        let level_db = 20.0 * level_lin.log10();

        let thr_db = -12.0;
        let ratio = 1.0 + 3.0 * amt; // 1..4
        let mut gain_db = 0.0;

        if level_db > thr_db {
            let over = level_db - thr_db;
            let compressed = over / ratio;
            gain_db = compressed - over; // negative
        }

        let target_gain = 10.0f32.powf(gain_db / 20.0);

        // Smooth gain to avoid zipper noise
        let g_smooth_coeff = 0.5;
        self.gain_smooth = self.gain_smooth * g_smooth_coeff + target_gain * (1.0 - g_smooth_coeff);

        let g = self.gain_smooth;
        (l * g, r * g)
    }
}

// Simple stereo room-ish reverb

struct SimpleReverb {
    sr: f32,
    buf_l: Vec<f32>,
    buf_r: Vec<f32>,
    idx: usize,
    d1_l: usize,
    d2_l: usize,
    d1_r: usize,
    d2_r: usize,
    feedback: f32,
}

impl SimpleReverb {
    fn new(sr: f32) -> Self {
        let mut s = Self {
            sr: sr.max(1.0),
            buf_l: Vec::new(),
            buf_r: Vec::new(),
            idx: 0,
            d1_l: 0,
            d2_l: 0,
            d1_r: 0,
            d2_r: 0,
            feedback: 0.4,
        };
        s.set_sample_rate(sr);
        s
    }

    fn set_sample_rate(&mut self, sr: f32) {
        self.sr = sr.max(1.0);
        self.init_buffers();
    }

    fn reset(&mut self) {
        for x in &mut self.buf_l {
            *x = 0.0;
        }
        for x in &mut self.buf_r {
            *x = 0.0;
        }
        self.idx = 0;
    }

    fn init_buffers(&mut self) {
        // Max ~250 ms buffer
        let max_time = 0.25;
        let len = (self.sr * max_time).round().max(1.0) as usize;

        self.buf_l = vec![0.0; len];
        self.buf_r = vec![0.0; len];
        self.idx = 0;

        // Set a couple of tap delays for a small-room feel
        self.d1_l = ((0.031 * self.sr) as usize).min(len - 1);
        self.d2_l = ((0.053 * self.sr) as usize).min(len - 1);
        self.d1_r = ((0.037 * self.sr) as usize).min(len - 1);
        self.d2_r = ((0.061 * self.sr) as usize).min(len - 1);

        self.feedback = 0.4;
    }

    fn process(&mut self, l: f32, r: f32, amount: f32) -> (f32, f32) {
        let amt = amount.clamp(0.0, 1.0);
        if amt <= 0.001 || self.buf_l.is_empty() {
            return (l, r);
        }

        let len = self.buf_l.len();
        let idx = self.idx;

        let in_mono = (l + r) * 0.5;

        // Read taps
        let tap_idx = |i: usize, d: usize, len: usize| (i + len - d) % len;
        let y1_l = self.buf_l[tap_idx(idx, self.d1_l, len)];
        let y2_l = self.buf_l[tap_idx(idx, self.d2_l, len)];
        let y1_r = self.buf_r[tap_idx(idx, self.d1_r, len)];
        let y2_r = self.buf_r[tap_idx(idx, self.d2_r, len)];

        let wet_l = 0.7 * y1_l + 0.3 * y2_l;
        let wet_r = 0.7 * y1_r + 0.3 * y2_r;

        // Write new value with feedback
        self.buf_l[idx] = in_mono + wet_l * self.feedback;
        self.buf_r[idx] = in_mono + wet_r * self.feedback;

        self.idx = (idx + 1) % len;

        let dry_mul = 1.0 - amt * 0.6;
        let wet_mul = amt;

        let out_l = l * dry_mul + wet_l * wet_mul;
        let out_r = r * dry_mul + wet_r * wet_mul;

        (out_l, out_r)
    }
}

// CLAP metadata

impl ClapPlugin for Drumini {
    const CLAP_ID: &'static str = "dev.example.drumini";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Minimalist electronic drum synth for beginners.");
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Drum,
        ClapFeature::Stereo,
    ];

    fn remote_controls(&self, _context: &mut impl RemoteControlsContext) {}

    const CLAP_MANUAL_URL: Option<&'static str> = Some("Not yet");
    const CLAP_SUPPORT_URL: Option<&'static str> = Some("Not yet");
}

nih_export_clap!(Drumini);
