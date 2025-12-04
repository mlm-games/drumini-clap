use crate::drum_engine::N_SLOTS;
use crate::params::{DrumParams, DrumSlotParams, MasterParams};

pub struct Kit<'a> {
    pub name: &'a str,
    pub make: fn() -> DrumParams,
}

pub const FACTORY_KITS: &[Kit<'_>] = &[
    Kit {
        name: "Init",
        make: kit_init,
    },
    Kit {
        name: "808 Clean",
        make: kit_808_clean,
    },
    Kit {
        name: "EDM Punch",
        make: kit_edm_punch,
    },
    Kit {
        name: "Minimal Tech",
        make: kit_minimal_tech,
    },
    Kit {
        name: "Lo-Fi",
        make: kit_lofi,
    },
];

fn kit_init() -> DrumParams {
    DrumParams::default()
}

fn kit_808_clean() -> DrumParams {
    DrumParams {
        kick: DrumSlotParams::from_values("Kick", 1.0, 0.0, 0.40, 360.0, 0.55, -2.0, 0.10),
        snare: DrumSlotParams::from_values("Snare", 0.9, 0.0, 0.65, 220.0, 0.75, 0.0, 0.20),
        clap: DrumSlotParams::from_values("Clap", 0.8, 0.0, 0.75, 190.0, 0.85, 0.0, 0.20),
        hat_closed: DrumSlotParams::from_values(
            "Hat Closed",
            0.65,
            -0.1,
            0.85,
            70.0,
            0.50,
            0.0,
            0.10,
        ),
        hat_open: DrumSlotParams::from_values("Hat Open", 0.7, -0.1, 0.85, 320.0, 0.40, 0.0, 0.10),
        tom: DrumSlotParams::from_values("Tom", 0.8, 0.05, 0.55, 260.0, 0.40, -2.0, 0.10),
        perc1: DrumSlotParams::from_values("Perc1", 0.7, 0.2, 0.70, 220.0, 0.50, 0.0, 0.20),
        perc2: DrumSlotParams::from_values("Perc2", 0.7, 0.3, 0.55, 220.0, 0.50, 0.0, 0.20),
        master: MasterParams::from_values(0.15, 0.25, 0.15, 0.0, 0.45),
    }
}

fn kit_edm_punch() -> DrumParams {
    DrumParams {
        kick: DrumSlotParams::from_values("Kick", 1.1, 0.0, 0.55, 280.0, 0.85, 0.0, 0.15),
        snare: DrumSlotParams::from_values("Snare", 1.0, 0.0, 0.75, 190.0, 0.85, 2.0, 0.20),
        clap: DrumSlotParams::from_values("Clap", 0.9, 0.0, 0.80, 200.0, 0.90, 0.0, 0.15),
        hat_closed: DrumSlotParams::from_values(
            "Hat Closed",
            0.75,
            -0.2,
            0.90,
            90.0,
            0.60,
            0.0,
            0.10,
        ),
        hat_open: DrumSlotParams::from_values("Hat Open", 0.8, -0.2, 0.90, 380.0, 0.50, 0.0, 0.10),
        tom: DrumSlotParams::from_values("Tom", 0.85, 0.1, 0.60, 260.0, 0.45, 0.0, 0.10),
        perc1: DrumSlotParams::from_values("Perc1", 0.8, 0.25, 0.75, 240.0, 0.60, 2.0, 0.20),
        perc2: DrumSlotParams::from_values("Perc2", 0.8, 0.35, 0.65, 240.0, 0.55, -2.0, 0.20),
        master: MasterParams::from_values(0.35, 0.55, 0.20, 0.0, 0.55),
    }
}

fn kit_minimal_tech() -> DrumParams {
    DrumParams {
        kick: DrumSlotParams::from_values("Kick", 1.0, 0.0, 0.35, 260.0, 0.65, -1.0, 0.15),
        snare: DrumSlotParams::from_values("Snare", 0.8, 0.05, 0.55, 170.0, 0.65, -2.0, 0.15),
        clap: DrumSlotParams::from_values("Clap", 0.75, 0.1, 0.65, 160.0, 0.70, 0.0, 0.20),
        hat_closed: DrumSlotParams::from_values(
            "Hat Closed",
            0.65,
            -0.2,
            0.75,
            70.0,
            0.50,
            0.0,
            0.10,
        ),
        hat_open: DrumSlotParams::from_values("Hat Open", 0.7, -0.25, 0.75, 320.0, 0.45, 0.0, 0.10),
        tom: DrumSlotParams::from_values("Tom", 0.75, 0.15, 0.45, 230.0, 0.35, -1.0, 0.10),
        perc1: DrumSlotParams::from_values("Perc1", 0.65, 0.2, 0.60, 220.0, 0.50, 0.0, 0.15),
        perc2: DrumSlotParams::from_values("Perc2", 0.65, 0.3, 0.55, 220.0, 0.45, 0.0, 0.15),
        master: MasterParams::from_values(0.25, 0.40, 0.10, 0.0, 0.45),
    }
}

fn kit_lofi() -> DrumParams {
    DrumParams {
        kick: DrumSlotParams::from_values("Kick", 0.9, -0.05, 0.30, 240.0, 0.40, -3.0, 0.25),
        snare: DrumSlotParams::from_values("Snare", 0.85, 0.05, 0.40, 210.0, 0.50, -4.0, 0.30),
        clap: DrumSlotParams::from_values("Clap", 0.8, 0.0, 0.50, 190.0, 0.55, -2.0, 0.30),
        hat_closed: DrumSlotParams::from_values(
            "Hat Closed",
            0.6,
            -0.1,
            0.55,
            90.0,
            0.40,
            -4.0,
            0.20,
        ),
        hat_open: DrumSlotParams::from_values(
            "Hat Open", 0.65, -0.1, 0.55, 420.0, 0.35, -4.0, 0.20,
        ),
        tom: DrumSlotParams::from_values("Tom", 0.7, 0.1, 0.45, 260.0, 0.40, -3.0, 0.20),
        perc1: DrumSlotParams::from_values("Perc1", 0.75, 0.15, 0.50, 260.0, 0.45, -2.0, 0.30),
        perc2: DrumSlotParams::from_values("Perc2", 0.75, 0.25, 0.45, 260.0, 0.45, -4.0, 0.30),
        master: MasterParams::from_values(0.55, 0.35, 0.30, -1.0, 0.40),
    }
}
