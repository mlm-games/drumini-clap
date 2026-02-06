# Drumini

A minimal algorithmic drum synthesizer CLAP plugin with 8 slots, per-slot macros, and master effects. Runs on Linux, Windows, and Android (headless).

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## The Pitch

Drumini gives you playable electronic drums without sample libraries. Each hit is synthesized in real-time using oscillators and noise, giving you immediate control over tone, snap, and decay. Made for quick beat-making when you want drums now, not after browsing 500 snare samples.

Runs on desktop DAWs and Android (via [yadaw](https://github.com/atsushieno/yadaw)) for mobile production.

## Features

- **8 Drum Slots**: Kick, Snare, Clap, Closed Hat, Open Hat, Tom, Perc1, Perc2
- **Per-Slot Controls**: Level, Pan, Tone, Decay, Snap, Pitch, Humanize
- **Master Section**: Drive (saturation), Compressor, Reverb, Kit Pitch, Velocity Curve
- **5 Factory Kits**: Init, 808 Clean, EDM Punch, Minimal Tech, Lo-Fi
- **Algorithmic Synthesis**: Sine wave bodies + filtered noise, no samples required
- **MIDI Triggered**: Standard GM drum map (C1=Kick, D1=Snare, etc.)
- **Lightweight**: Single-file CLAP plugin, no external dependencies

## Installation

### Desktop (Linux/Windows)

Download the latest release for your platform and rename:
- Linux: `Drumini-linux.clap` → `Drumini.clap`
- Windows: `Drumini-windows.clap` → `Drumini.clap`

Place in your CLAP plugin folder (typically `~/.clap/` on Linux, or your DAW's plugin path).

### Android (yadaw)

```bash
pkg update && pkg install -y rust clang cmake ninja pkg-config git
cargo install cargo-ndk
rustup target add aarch64-linux-android
export CARGO_NDK_ON_ANDROID=1
cargo ndk -t arm64-v8a --platform 26 build --release
cp target/aarch64-linux-android/release/libdrumini.so Drumini.clap
```

Install in yadaw:
- Copy `Drumini.clap` to yadaw's CLAP folder (via ADB/Shizuku):
  - `/storage/emulated/0/Android/data/<yadaw.package>/files/plugins/clap/`
  - Or create `Drumini.clap/` folder and place the `.so` inside
- Rescan plugins in yadaw

### Arch Linux (AUR)

```bash
yay -S drumini-clap-bin
# or
paru -S drumini-clap-bin
```

## Usage

Load Drumini in any CLAP host. Send MIDI notes to trigger:

| MIDI Note | Drum     |
|-----------|----------|
| 36 (C1)   | Kick     |
| 38 (D1)   | Snare    |
| 39 (Eb1)  | Clap     |
| 42 (F#1)  | Hat Closed |
| 46 (Bb1)  | Hat Open |
| 43-47     | Tom      |
| 49        | Perc 1   |
| 51        | Perc 2   |

### Quick Tweaks

- **Kick not punchy?** Increase Snap, decrease Decay
- **Snare too thin?** Raise Tone, add Snap
- **Hats too harsh?** Lower Tone, reduce Decay
- **Everything too static?** Add Humanize (randomizes level/pitch/decay per hit)
- **Needs glue?** Dial in Compressor, add subtle Drive

[PLACEHOLDER: add demo GIF here showing parameter tweaking in a host]

## Building from Source

Requirements: Rust 1.70+, CLAP host for testing

```bash
git clone https://github.com/mlm-games/drumini-clap
cd drumini-clap

# Desktop Linux
cargo build --release
cp target/release/libdrumini.so Drumini.clap

# Windows (MSVC)
cargo build --release
copy target\release\drumini.dll Drumini.clap

# macOS (experimental)
cargo build --release
# Bundle as .clap or use NIH-plug bundler
```

## Architecture

```
lib.rs          Plugin entry, MIDI handling, master effects
drum_engine.rs  8 synthesis engines (kick, snare, etc.)
params.rs       Parameter definitions and defaults
kits.rs         Factory kit presets
dsp.rs          Utilities: fast tanh, PolyBLEP osc, ZDF-SVF
```

Synthesis per slot:
- **Kick**: Pitch-swept sine + click noise
- **Snare**: Pitched body + bright noise
- **Clap**: Multi-burst filtered noise
- **Hats**: High-passed noise with snap shaping
- **Toms/Perc**: Body + filtered noise blend

Master chain: Saturation → Compressor → Reverb

## Contributing

PRs welcome. Areas that need work:
- Preset loading/saving (waiting on yadaw (clack)'s preset handling)
- Additional synthesis algorithms
- Parameter smoothing improvements

Dev setup is standard Cargo—no special build tools needed.

## Credits

Built with [nih-plug](https://github.com/robbert-vdh/nih-plug) by Robbert van der Helm.

## License

MIT License—see [LICENSE](LICENSE).
