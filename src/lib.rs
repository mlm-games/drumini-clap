mod drum_engine;
mod dsp;
mod params;

use crate::dsp::fast_tanh;
use drum_engine::{DrumSlot, N_SLOTS, SLOT_TYPES};
use nih_plug::prelude::*;
use params::{DrumParams, DrumSlotParams, MasterParams};
use std::num::NonZeroU32;
use std::sync::Arc;

pub struct MiniDrums {
    params: Arc<DrumParams>,
    sample_rate: f32,
    slots: [DrumSlot; N_SLOTS],
}

impl Default for MiniDrums {
    fn default() -> Self {
        let sr = 44100.0;
        let params = Arc::new(DrumParams::default());

        let slots = core::array::from_fn(|i| DrumSlot::new(SLOT_TYPES[i], sr));

        Self {
            params,
            sample_rate: sr,
            slots,
        }
    }
}

impl Plugin for MiniDrums {
    const NAME: &'static str = "MiniDrums";
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
        true
    }

    fn reset(&mut self) {
        for (i, slot) in self.slots.iter_mut().enumerate() {
            *slot = DrumSlot::new(SLOT_TYPES[i], self.sample_rate);
        }
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
            // Handle sample-accurate events
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
                        // One-shot drums; ignore NoteOff for now
                    }
                    _ => {}
                }

                next_event = ctx.next_event();
            }

            // Render all slots and mix to stereo
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

                // Simple master drive (saturation)
                let drive = master.drive.value().clamp(0.0, 1.0);
                if drive > 0.0 {
                    let drive_gain = 1.0 + drive * 6.0;
                    let makeup = 1.0 / (1.0 + drive * 2.0);
                    l = fast_tanh(l * drive_gain) * makeup;
                    r = fast_tanh(r * drive_gain) * makeup;
                }

                // Placeholder: master.comp and master.reverb can be added here later
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

/// Helper: return the DrumSlotParams for a slot index.
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

// CLAP metadata
impl ClapPlugin for MiniDrums {
    const CLAP_ID: &'static str = "dev.example.minidrums";
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

nih_export_clap!(MiniDrums);
