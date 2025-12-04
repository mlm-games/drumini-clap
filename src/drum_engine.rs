use crate::dsp::{fast_tanh, flush_denormals};
use crate::params::{DrumSlotParams, MasterParams};
use core::f32::consts::PI;

pub const N_SLOTS: usize = 8;

#[derive(Copy, Clone, Debug)]
pub enum SlotType {
    Kick,
    Snare,
    Clap,
    HatClosed,
    HatOpen,
    Tom,
    Perc1,
    Perc2,
}

pub const SLOT_TYPES: [SlotType; N_SLOTS] = [
    SlotType::Kick,
    SlotType::Snare,
    SlotType::Clap,
    SlotType::HatClosed,
    SlotType::HatOpen,
    SlotType::Tom,
    SlotType::Perc1,
    SlotType::Perc2,
];

pub struct DrumSlot {
    pub kind: SlotType,
    pub sample_rate: f32,

    active: bool,
    env: f32,
    decay_coef: f32,

    velocity: f32,

    // PRNG + simple noise state
    noise_state: u32,
    noise_lp: f32, // for simple one-pole HP (snare/hats/clap)

    // Pitched body
    osc_phase: f32,
    base_freq: f32,

    // Per-hit humanization
    human_amp: f32,
    human_pitch: f32,     // in semitones
    human_decay_mul: f32, // 1 ± something
}

impl DrumSlot {
    pub fn new(kind: SlotType, sample_rate: f32) -> Self {
        Self {
            kind,
            sample_rate: sample_rate.max(1.0),
            active: false,
            env: 0.0,
            decay_coef: 0.999,
            velocity: 0.0,
            noise_state: 1,
            noise_lp: 0.0,
            osc_phase: 0.0,
            base_freq: 100.0,
            human_amp: 1.0,
            human_pitch: 0.0,
            human_decay_mul: 1.0,
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr.max(1.0);
    }

    /// Trigger a new drum hit for this slot, using slot/master params for humanization & decay.
    pub fn trigger(&mut self, velocity: f32, slot_params: &DrumSlotParams, master: &MasterParams) {
        self.active = true;
        self.env = 1.0;
        self.noise_lp = 0.0;

        // Velocity curve
        let v_curve = master.velocity_curve.value().clamp(0.0, 1.0);
        let shape = 0.5 + v_curve; // 0.5..1.5
        self.velocity = velocity.clamp(0.0, 1.0).powf(shape);

        // Reseed RNG
        self.noise_state = self
            .noise_state
            .wrapping_mul(1664525)
            .wrapping_add(1013904223);

        // Humanization
        let h = slot_params.humanize.value();
        if h > 0.0 {
            let r1 = self.random_bipolar();
            let r2 = self.random_bipolar();
            let r3 = self.random_bipolar();
            self.human_amp = 1.0 + r1 * 0.15 * h; // ±15%
            self.human_pitch = r2 * 3.0 * h; // ±3 semitones
            self.human_decay_mul = 1.0 + r3 * 0.5 * h; // ±50%
        } else {
            self.human_amp = 1.0;
            self.human_pitch = 0.0;
            self.human_decay_mul = 1.0;
        }

        // Exponential decay from ms param
        let decay_ms = slot_params.decay.value().max(5.0);
        let decay_sec = (decay_ms / 1000.0) * self.human_decay_mul;
        let tau = decay_sec.max(0.001);
        self.decay_coef = (-1.0 / (tau * self.sample_rate)).exp();

        // Base pitch per slot
        let base = match self.kind {
            SlotType::Kick => 55.0,
            SlotType::Snare => 180.0,
            SlotType::Clap => 250.0,
            SlotType::HatClosed => 8000.0,
            SlotType::HatOpen => 7000.0,
            SlotType::Tom => 140.0,
            SlotType::Perc1 => 400.0,
            SlotType::Perc2 => 700.0,
        };

        let pitch_offset = slot_params.pitch.value() + master.kit_pitch.value() + self.human_pitch;
        let ratio = 2.0f32.powf(pitch_offset / 12.0);
        self.base_freq = (base * ratio).clamp(20.0, 12000.0);
        self.osc_phase = 0.0;
    }

    /// Render one sample for this slot.
    pub fn process(&mut self, slot_params: &DrumSlotParams, master: &MasterParams) -> f32 {
        if !self.active {
            return 0.0;
        }

        self.env *= self.decay_coef;
        if self.env < 1e-4 {
            self.env = 0.0;
            self.active = false;
            return 0.0;
        }

        let env = self.env;
        let sample = match self.kind {
            SlotType::Kick => self.render_kick(env, slot_params),
            SlotType::Snare => self.render_snare(env, slot_params),
            SlotType::Clap => self.render_clap(env, slot_params),
            SlotType::HatClosed => self.render_hat_closed(env, slot_params),
            SlotType::HatOpen => self.render_hat_open(env, slot_params),
            SlotType::Tom => self.render_tom(env, slot_params),
            SlotType::Perc1 => self.render_perc1(env, slot_params),
            SlotType::Perc2 => self.render_perc2(env, slot_params),
        };

        // Global per-hit scaling
        let mut out = sample * env * self.velocity * self.human_amp;

        // Simple master drive is handled later; here just a gentle per-slot saturator
        out = fast_tanh(out);
        flush_denormals(out)
    }

    #[inline]
    fn random_bipolar(&mut self) -> f32 {
        self.noise_state = self
            .noise_state
            .wrapping_mul(1664525)
            .wrapping_add(1013904223);

        let bits = 0x3F800000 | (self.noise_state >> 9);
        let f = f32::from_bits(bits) - 1.0;
        f * 2.0 - 1.0
    }

    #[inline]
    fn next_noise(&mut self) -> f32 {
        self.random_bipolar() * 0.7
    }

    #[inline]
    fn next_sine(&mut self, freq: f32) -> f32 {
        let inc = 2.0 * PI * freq / self.sample_rate;
        self.osc_phase += inc;
        if self.osc_phase > 2.0 * PI {
            self.osc_phase -= 2.0 * PI;
        }
        self.osc_phase.sin()
    }

    // equal-power-ish LP-based highpass on noise: returns HP component
    #[inline]
    fn hp_noise(&mut self, noise: f32, cutoff_hz: f32) -> f32 {
        let fc = cutoff_hz.clamp(200.0, self.sample_rate * 0.45);
        let alpha = 1.0 - (-2.0 * PI * fc / self.sample_rate).exp();
        self.noise_lp += alpha * (noise - self.noise_lp);
        noise - self.noise_lp
    }

    // Slot-specific engines

    fn render_kick(&mut self, env: f32, p: &DrumSlotParams) -> f32 {
        let tone = p.tone.value(); // 0..1
        let snap = p.snap.value();

        // Pitch sweep: more tone -> deeper sweep
        let sweep_semitones = 30.0 * (0.3 + 0.7 * tone);
        let sweep = sweep_semitones * env * env;
        let freq = self.base_freq * 2.0f32.powf(sweep / 12.0);

        let mut body = self.next_sine(freq);
        body = fast_tanh(body * (1.0 + 3.0 * snap)); // more snap => more distortion

        // Attack click: short, bright noise
        let click_env = env.powf(0.3);
        let noise = self.next_noise();
        let click = self.hp_noise(noise, 4000.0 + 4000.0 * tone) * snap * click_env;

        body * 0.9 + click * 0.4
    }

    fn render_snare(&mut self, _env: f32, p: &DrumSlotParams) -> f32 {
        let tone = p.tone.value();
        let snap = p.snap.value();

        // Pitched body around base_freq
        let body = self.next_sine(self.base_freq);

        // Bright noise band
        let noise = self.next_noise();
        let noise_hp = self.hp_noise(noise, 2000.0 + 6000.0 * tone);

        let body_mix = 0.4 * (1.0 - tone); // darker tone -> more body
        let noise_mix = 0.8 + 0.4 * snap; // snap -> more noise

        body * body_mix + noise_hp * noise_mix
    }

    fn render_clap(&mut self, env: f32, p: &DrumSlotParams) -> f32 {
        let tone = p.tone.value();
        let snap = p.snap.value();

        let noise = self.next_noise();
        // Medium band noise
        let band = self.hp_noise(noise, 800.0 + 1200.0 * (1.0 - tone));

        // Faux "multi-burst": emphasize early envelope region
        let burst = (env.powf(0.3) * (1.0 + 0.6 * snap)).min(1.5);
        band * burst
    }

    fn render_hat_closed(&mut self, env: f32, p: &DrumSlotParams) -> f32 {
        let tone = p.tone.value();
        let snap = p.snap.value();

        let noise = self.next_noise();
        let noise_hp = self.hp_noise(noise, 6000.0 + 6000.0 * tone);

        // Very snappy decay shape
        let shape = env.powf(2.5 - 1.5 * snap);

        noise_hp * shape * (0.8 + 0.4 * snap)
    }

    fn render_hat_open(&mut self, env: f32, p: &DrumSlotParams) -> f32 {
        let tone = p.tone.value();
        let snap = p.snap.value();

        let noise = self.next_noise();
        let noise_hp = self.hp_noise(noise, 5000.0 + 5000.0 * tone);

        let shape = env.powf(1.2 + 0.8 * snap); // more snap -> slightly faster

        noise_hp * shape * (0.9 + 0.3 * snap)
    }

    fn render_tom(&mut self, _env: f32, p: &DrumSlotParams) -> f32 {
        let tone = p.tone.value();

        let body = self.next_sine(self.base_freq);
        let noise = self.next_noise();
        let noise_hp = self.hp_noise(noise, 1500.0 + 3000.0 * tone);

        body * 0.9 + noise_hp * 0.3
    }

    fn render_perc1(&mut self, env: f32, p: &DrumSlotParams) -> f32 {
        let tone = p.tone.value();

        let noise = self.next_noise();
        let noise_hp = self.hp_noise(noise, 2500.0 + 6000.0 * tone);

        // Slight metallic ring via a pitched element
        let body = self.next_sine(self.base_freq * (1.5 + 0.5 * tone));

        let burst = env.powf(0.7);
        body * 0.3 + noise_hp * 0.9 * burst
    }

    fn render_perc2(&mut self, env: f32, p: &DrumSlotParams) -> f32 {
        let tone = p.tone.value();

        let body = self.next_sine(self.base_freq * (1.0 + tone));
        let noise = self.next_noise();
        let noise_hp = self.hp_noise(noise, 2000.0 + 5000.0 * tone);

        let shape = env.powf(0.9);
        body * 0.6 * shape + noise_hp * 0.5 * shape
    }
}
