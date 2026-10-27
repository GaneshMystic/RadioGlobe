# Build Guide

Cross-platform compilation and setup guide for **RadioGlobe**.

---

## Prerequisites

Regardless of your operating system, building `RadioGlobe` requires:
1. **Rust Toolchain** (1.75+ or stable) — [rustup.rs](https://rustup.rs/)
2. **`mpv` Audio Player** — Used by the audio engine backend (`src/player.rs`)

---

## Platform-Specific Instructions

### 🐧 Linux (Ubuntu / Debian / Arch / Fedora)

1. **Install dependencies**:
   - Ubuntu / Debian: `sudo apt update && sudo apt install mpv build-essential`
   - Arch Linux: `sudo pacman -S mpv base-devel`
   - Fedora: `sudo dnf install mpv gcc`

2. **Clone & Build**:
   ```bash
   git clone https://github.com/yourusername/radioglobe.git
   cd radioglobe
   cargo build --release
   ```

3. **Binary Location**: `./target/release/radioglobe`

---

### 🍏 macOS

1. **Install dependencies via Homebrew**:
   ```brew install mpv rust```

2. **Clone & Build**:
   ```bash
   git clone https://github.com/yourusername/radioglobe.git
   cd radioglobe
   cargo build --release
   ```

3. **Binary Location**: `./target/release/radioglobe`

---

### 🪟 Windows (WSL2 or Native PowerShell)

#### Option A: WSL2 (Recommended for best terminal rendering)
Follow the Linux instructions inside your Ubuntu/WSL2 terminal.

#### Option B: Native Windows (PowerShell)
1. **Install dependencies**:
   - Install Rust via [rustup-init.exe](https://rustup.rs/)
   - Install `mpv` via Chocolatey or Scoop:
     ```powershell
     choco install mpv
     # or
     scoop install mpv
     ```
     *(Ensure `mpv.exe` is added to your system `PATH`)*

2. **Clone & Build**:
   ```powershell
   git clone https://github.com/yourusername/radioglobe.git
   cd radioglobe
   cargo build --release
   ```

3. **Binary Location**: `.\target\release\radioglobe.exe`

---

## Running in Release Mode

For zero-latency map panning and smooth audio streaming, always compile and run with `--release`:

```bash
cargo run --release
```
