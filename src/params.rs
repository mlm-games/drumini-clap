use nih_plug::prelude::*;

/// Top-level parameters: 8 drum slots + master section.
#[derive(Params)]
pub struct DrumParams {
    #[nested(id_prefix = "kick", group = "Kick")]
    pub kick: DrumSlotParams,

    #[nested(id_prefix = "snare", group = "Snare")]
    pub snare: DrumSlotParams,

    #[nested(id_prefix = "clap", group = "Clap")]
    pub clap: DrumSlotParams,

    #[nested(id_prefix = "hatc", group = "Closed Hat")]
    pub hat_closed: DrumSlotParams,

    #[nested(id_prefix = "hato", group = "Open Hat")]
    pub hat_open: DrumSlotParams,

    #[nested(id_prefix = "tom", group = "Tom")]
    pub tom: DrumSlotParams,

    #[nested(id_prefix = "pc1", group = "Perc 1")]
    pub perc1: DrumSlotParams,

    #[nested(id_prefix = "pc2", group = "Perc 2")]
    pub perc2: DrumSlotParams,

    #[nested(group = "Master")]
    pub master: MasterParams,
}

/// Parameters for a single drum slot (Kick/Snare/…)
#[derive(Params)]
pub struct DrumSlotParams {
    /// Output level for this slot
    #[id = "lvl"]
    pub level: FloatParam,

    /// Stereo pan (-1 = left, 0 = center, 1 = right)
    #[id = "pan"]
    pub pan: FloatParam,

    /// Macro: brightness / filter / oscillator color
    #[id = "ton"]
    pub tone: FloatParam,

    /// Macro: amplitude decay (in ms)
    #[id = "dec"]
    pub decay: FloatParam,

    /// Macro: transient attack / snap
    #[id = "snp"]
    pub snap: FloatParam,

    /// Pitch offset in semitones
    #[id = "pit"]
    pub pitch: FloatParam,

    /// Humanization amount (randomization of level/decay/pitch)
    #[id = "hum"]
    pub humanize: FloatParam,
}

/// Global/master controls.
#[derive(Params)]
pub struct MasterParams {
    /// Master drive/saturation
    #[id = "drv"]
    pub drive: FloatParam,

    /// Bus compression amount
    #[id = "cmp"]
    pub comp: FloatParam,

    /// Send reverb amount
    #[id = "rev"]
    pub reverb: FloatParam,

    /// Global kit pitch (for toms / 808 styles)
    #[id = "ktp"]
    pub kit_pitch: FloatParam,

    /// Velocity curve / sensitivity
    #[id = "vel"]
    pub velocity_curve: FloatParam,
}

impl Default for DrumParams {
    fn default() -> Self {
        Self {
            kick: DrumSlotParams::default_kick(),
            snare: DrumSlotParams::default_snare(),
            clap: DrumSlotParams::default_clap(),
            hat_closed: DrumSlotParams::default_hat_closed(),
            hat_open: DrumSlotParams::default_hat_open(),
            tom: DrumSlotParams::default_tom(),
            perc1: DrumSlotParams::default_perc1(),
            perc2: DrumSlotParams::default_perc2(),
            master: MasterParams::default(),
        }
    }
}

impl DrumSlotParams {
    /// Construct a slot with explicit values for all macros.
    pub fn from_values(
        level: f32,
        pan: f32,
        tone: f32,
        decay_ms: f32,
        snap: f32,
        pitch_st: f32,
        humanize: f32,
    ) -> Self {
        Self {
            level: FloatParam::new("Level", level, FloatRange::Linear { min: 0.0, max: 2.0 })
                .with_unit("×"),

            pan: FloatParam::new(
                "Pan",
                pan,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
                },
            ),

            tone: FloatParam::new("Tone", tone, FloatRange::Linear { min: 0.0, max: 1.0 }),

            decay: FloatParam::new(
                "Decay",
                decay_ms,
                FloatRange::Skewed {
                    min: 10.0,
                    max: 2000.0,
                    factor: 0.4,
                },
            )
            .with_unit("ms"),

            snap: FloatParam::new("Snap", snap, FloatRange::Linear { min: 0.0, max: 1.0 }),

            pitch: FloatParam::new(
                "Pitch",
                pitch_st,
                FloatRange::Linear {
                    min: -24.0,
                    max: 24.0,
                },
            )
            .with_unit("st"),

            humanize: FloatParam::new(
                "Humanize",
                humanize,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
        }
    }

    pub fn default_kick() -> Self {
        // Punchy, slightly darker, medium-long decay
        Self::from_values(0.9, 0.0, 0.4, 300.0, 0.6, 0.0, 0.2)
    }

    pub fn default_snare() -> Self {
        // Bright, snappy, medium decay
        Self::from_values(0.9, 0.0, 0.6, 200.0, 0.7, 0.0, 0.2)
    }

    pub fn default_clap() -> Self {
        // Bright, snappy, shorter decay
        Self::from_values(0.8, 0.0, 0.7, 180.0, 0.8, 0.0, 0.2)
    }

    pub fn default_hat_closed() -> Self {
        // Short, bright
        Self::from_values(0.7, -0.1, 0.8, 80.0, 0.5, 0.0, 0.1)
    }

    pub fn default_hat_open() -> Self {
        // Longer, bright
        Self::from_values(0.7, -0.1, 0.8, 450.0, 0.4, 0.0, 0.1)
    }

    pub fn default_tom() -> Self {
        // Medium decay, mid tone
        Self::from_values(0.8, 0.1, 0.5, 260.0, 0.4, 0.0, 0.1)
    }

    pub fn default_perc1() -> Self {
        // Slightly bright, medium decay
        Self::from_values(0.7, 0.2, 0.7, 220.0, 0.5, 0.0, 0.2)
    }

    pub fn default_perc2() -> Self {
        // More mid, similar decay
        Self::from_values(0.7, 0.3, 0.5, 220.0, 0.5, 0.0, 0.2)
    }
}

impl MasterParams {
    pub fn from_values(
        drive: f32,
        comp: f32,
        reverb: f32,
        kit_pitch: f32,
        velocity_curve: f32,
    ) -> Self {
        Self {
            drive: FloatParam::new("Drive", drive, FloatRange::Linear { min: 0.0, max: 1.0 }),
            comp: FloatParam::new("Comp", comp, FloatRange::Linear { min: 0.0, max: 1.0 }),
            reverb: FloatParam::new("Reverb", reverb, FloatRange::Linear { min: 0.0, max: 1.0 }),
            kit_pitch: FloatParam::new(
                "Kit Pitch",
                kit_pitch,
                FloatRange::Linear {
                    min: -12.0,
                    max: 12.0,
                },
            )
            .with_unit("st"),
            velocity_curve: FloatParam::new(
                "Velocity",
                velocity_curve,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
        }
    }
}

impl Default for MasterParams {
    fn default() -> Self {
        Self::from_values(0.1, 0.3, 0.2, 0.0, 0.5)
    }
}
