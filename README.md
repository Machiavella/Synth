# README — Synth

# Synth

**Rust Synth Prototype** — a simple monophonic synth with built-in presets, spammy ads, and a disco GUI.

---

## Features

- Three built-in presets:  
  - Ryan & Josh Allen (romantic)  
  - Laura Les (fast hyperpopish)  
  - Extendable for more “people”  
- Real-time audio synthesis using CPAL  
- Adjustable oscillator mix, detune, gain, and master volume  
- Disco mode: flashing colors + spammy GUI ads  
- Thread-safe shared state between audio and GUI  

---

## Installation

```bash
# Clone the repository
git clone https://github.com/username/Synth.git
cd Synth

# Run the synth
cargo run --release
````

**Requirements:**

* Rust 1.72+
* Default audio output device
* Cargo will automatically install required crates

---

## Usage

* Click preset buttons to load different “people” sounds
* Adjust sliders for oscillator mix, detune, and gain
* Enable “Disco Mode” to see flashing colors and repeated ads
* Master volume slider adjusts global gain

---

## Development Notes

* Audio runs in a separate thread using CPAL
* Shared state between GUI and audio uses atomic types for smooth updates
* Designed for easy extension: add new presets, effects, or visualizations

---

## License

[MIT License]