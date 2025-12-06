# Drumini

A small Rust CLAP synth that runs headless on Android (for yadaw) and on desktop CLAP hosts.

Build (Android on-device via Termux)
```sh
pkg update && pkg install -y rust clang cmake ninja pkg-config git
cargo install cargo-ndk
rustup target add aarch64-linux-android
export CARGO_NDK_ON_ANDROID=1
cargo ndk -t arm64-v8a --platform 26 build --release
cp target/aarch64-linux-android/release/libdrumini.so Drumini.clap
```

Use with yadaw (Android)
- Put Drumini.clap in a directory yadaw scans, e.g.:
  - /storage/emulated/0/Android/data/<your.yadaw.package>/files/plugins/clap/ (need adb or shizuku access, yadaw supports it by copying it from external to internal, since newer android devices open everything under storage/emulated/0 in noexec mode)
  - Create a folder that ends with .clap and put the .so file in the folder (compiled file from the binary, rename the .clap file to .so if you ran the cp step given above)
  - or your app-internal: /data/data/<your.yadaw.package>/files/plugins/clap/ (need root perms, similar naming scheme as above)
<!-- - In yadaw, set additional plugin search paths if you added UI to configure them. -->

Build (desktop quick)
- Linux: `cargo build --release && cp target/release/libdrumini.so Drumini.clap`
- Windows (MSVC): `cargo build --release && copy target\release\drumini.dll Drumini.clap`
- macOS: `cargo build --release` then bundle as a .clap, or use NIH‑plug’s bundler (`cargo xtask bundle` if you set it up)

Notes on CLAP poly‑mod
- using normalized_offset and Param::preview_modulated() for per‑voice values and emits NoteEvent::VoiceTerminated when voices end; his plugin also sets capacity on init/resize. For more info, see NoteEvent::PolyModulation, Param, and ClapPlugin::PolyModulationConfig in NIH‑plug docs.

[License](LICENSE)
