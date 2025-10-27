# Deployment Document — Synth

**Project Name:** Synth  
**Repository:** Synth  
**Author:** [Your Name]  
**Language/Frameworks:** Rust, CPAL, EGUI/EFRAME  

---

## 1. System Requirements

- **OS:** Windows, macOS, Linux  
- **Rust Toolchain:** Rust 1.72+ (stable recommended)  
- **Dependencies:**  
  - `cpal = "0.15"` – for audio output  
  - `egui = "0.27"` – for GUI  
  - `eframe = { version = "0.27", features = ["wgpu"] }` – GUI framework  
  - `anyhow = "1"` – error handling  

---

## 2. Installation Steps

1. **Install Rust**  
   Follow instructions at [https://rustup.rs](https://rustup.rs) to install Rust and Cargo.

2. **Clone the Repository**  
   ```bash
   git clone https://github.com/username/Synth.git
   cd Synth
   ```

3. **Verify Dependencies**
   The `Cargo.toml` already includes all required crates. Cargo will automatically fetch them on build.

---

## 3. Build Instructions

1. **Compile the Project**

   ```bash
   cargo build --release
   ```

2. **Run the Application**

   ```bash
   cargo run --release
   ```

3. **Expected Output**

   * GUI window titled `"Rust Synth Prototype"`
   * Default preset loaded: `"Ryan & Josh Allen (romantic)"`
   * Continuous monophonic tone output
   * Buttons to switch presets, adjust oscillator mix, detune, and gain
   * Disco mode with flashing colors and spammy ads

---

## 4. Running Notes

* **Audio:** Requires a default output device. Errors will be printed if none is found.
* **GUI:** Interactive controls include:

  * Load presets (Ryan & Josh, Laura Les)
  * Adjust oscillator mix, detune, gain, master volume
  * Enable/disable disco mode
* **Disco Mode:** Changes background color and displays repeated “ads.”
* **Threading:** Audio runs in a separate thread to avoid blocking GUI.

---

## 5. Deployment Considerations

* **Windows:** Ensure Visual Studio Build Tools are installed for linking.
* **macOS:** Xcode Command Line Tools may be required for building native code.
* **Linux:** ALSA/PulseAudio should be present; install development headers if needed.
* **Performance:** Running in release mode is recommended for smooth audio.

---

## 6. Future Enhancements

* Add **more presets** to represent more “people.”
* Implement **mouse-following visual effects** in disco mode.
* Add **MIDI input** support for live control.