# AGENTS.md — UA-Piper-TTS

> AI Agent instructions for the UA-Piper-TTS project.

---

## 📋 About the Project

**Goal:** Generate Ukrainian audio from text using Rust.
**Platform:** Linux aarch64 (Rockchip RK3588), also works on x86_64.
**Language:** Rust (main), Piper CLI (subprocess).

---

## 🏗 Architecture

```
┌─────────────┐     ┌─────────────────────────────┐     ┌────────────┐
│   Rust code │────▶│       normalize.rs          │────▶│  Piper CLI │
│  main.rs    │     │  4-step pipeline:           │     │  (subproc) │
└─────────────┘     │  1. tokenize()              │     └────────────┘
                    │  2. convert_numbers()        │          │
                    │  3. apply_stress()           │          ▼
                    │  4. cleanup()                │    WAV/MP3 file
                    └─────────────────────────────┘    in output/
                               │
                     Normalized text with
                     stress marks (U+0301)
```

### Key Components

| File | Purpose |
|------|---------|
| `src/main.rs` | Entry point, config loading, module imports |
| `src/server.rs` | Web server (actix-web), request handling, WAV/MP3 generation |
| `src/normalize.rs` | 4-step normalization: tokenize → convert → stress → cleanup |
| `src/error_log.rs` | Logs Piper TTS errors to `tts_errors.log` for analysis |
| `data/ua_stress_dict.txt` | Main stress dictionary (lang-uk, 2.9M words) |
| `data/custom_stress_dict.txt` | Custom stress dictionary (higher priority, user-editable) |
| `download_models.sh` | Downloads Piper model from HuggingFace |
| `models/piper-uk_UK-dmytro-medium/` | ONNX model + espeak-ng data |

### Dependencies

```toml
hound = "3.5"       # Audio handling
num2words = "1.2"   # Number → Ukrainian words conversion
actix-web = "4"     # Web framework
actix-files = "0.6" # Static file serving
serde = "1"         # Serialization
serde_json = "1"    # JSON handling
tokio = "1"         # Async runtime
```

### External Dependencies (not Rust)

- **piper-tts** (`pip3 install piper-tts`) — provides `piper` CLI binary
- **ONNX Runtime** — auto-installed with piper-tts
- **ffmpeg** (optional) — for MP3 encoding (32 kbps, mono, 22.05 kHz)

---

## 🗣 Model

### Piper `uk_UA-ukrainian_tts-medium`

- **Source:** https://huggingface.co/rhasspy/piper-voices
- **Size:** ~74 MB
- **Sample rate:** 22050 Hz
- **Speakers:** 3
  - `0` — lada (female)
  - `1` — mykyta (male)
  - `2` — tetiana (female) ← **default**

### Known Model Limitations

1. **No uppercase support** — text must be converted to lowercase
2. **`Missing phoneme: П/С` warning** — occurs if text is not converted to lowercase
3. **Stress marks** — supported via U+0301 Combining Acute Accent (`це́льсія`)
4. **espeak-ng data** required for phonemization (copied from another model)

---

## 📝 Text Normalization

### Architecture: 4-Step Pipeline

```
Input text → step1_tokenize() → Tokens → step2_convert_numbers() → Words
       → step3_apply_stress() → Stressed words → step4_cleanup() → Output
```

Each step is a separate function. Use `normalize_text_debug()` to inspect intermediate results.

### Stress Dictionaries

Two dictionaries are loaded in order (later entries override earlier ones):

| File | Source | Size | Priority |
|------|--------|------|----------|
| `data/ua_stress_dict.txt` | lang-uk (GitHub) | ~2.9M words | Lower |
| `data/custom_stress_dict.txt` | Our custom file | Growing | **Higher** |

**Custom dictionary format:**
```
# Comments start with #
вологі́сть          — new word with stress mark
сі́мсот             — override existing entry
```

Stress is U+0301 (combining acute accent) placed after the vowel.

### Library: `num2words` (Rust crate)

Converts numbers to Ukrainian words with correct grammar.

### Decimal Rule for Temperature (°C)

| Integer Part | Format | Example |
|--------------|--------|---------|
| **> 9** | `"і [digit]"` | `36.6°C` → `"тридцять шість і шість градусів це́льсія"` |
| **≤ 9** | `"цілих ... десятих/сотих"` | `9.6°C` → `"дев'ять цілих шість десятих градусів це́льсія"` |
| Regular numbers (no °C) | Always `"цілих ..."` | `3.14` → `"три цілих чотирнадцять сотих"` |

### Supported Special Characters

| Symbol | Replacement |
|--------|-------------|
| `°C` / `°c` | ` градусів це́льсія` |
| `°F` / `°f` | ` градусів фаренгейта` |
| `°` (no letter) | ` градусів` |
| `%` | ` відсотків` |
| `+` | ` плюс ` |
| `=` | ` дорівнює ` |
| `&` | ` і ` |

### Home Assistant States

| State | Replacement |
|-------|-------------|
| `on` | `увімкнено` |
| `off` | `вимкнено` |
| `open` | `відчинено` |
| `closed` | `зачинено` |
| `detected` | `виявлено` |
| `clear` | `чисто` |
| `true` | `так` |
| `false` | `ні` |

### Electrical Units

| Unit | Replacement |
|------|-------------|
| `V` / `v` | `во́льт` |
| `mV` / `mv` | `міліво́льт` |
| `kV` / `kv` | `кілово́льт` |
| `A` / `a` | `ампе́р` |
| `mA` / `ma` | `міліампе́р` |
| `W` / `w` | `ва́т` |
| `mW` / `mw` | `міліва́т` |
| `kW` / `kw` | `кілова́т` |
| `kWh` / `kwh` | `кілова́т-годи́н` |
| `Hz` / `hz` | `ге́рц` |
| `kHz` / `khz` | `кілоге́рц` |
| `MHz` / `mhz` | `мегаге́рц` |
| `GHz` / `ghz` | `гігаге́рц` |

### Cyrillic Units

| Unit | Replacement |
|------|-------------|
| `мм` | міліметр(и/ів) |
| `см` | сантиметр(и/ів) |
| `м` | метр(и/ів) |
| `км` | кілометр(и/ів) |
| `г` | грам(и/ів) |
| `кг` | кілограм(и/ів) |
| `л` | літр(и/ів) |
| `мл` | мілілітр(и/ів) |

### Unit Declension

Units are declined based on the preceding number:
- **1** → singular: `один метр`
- **2-4** → plural: `два метри`, `три кілограми`
- **5+** → genitive plural: `п'ять метрів`, `десять грамів`

### Pressure: "мм рт. ст."

Special handling for pressure readings:
- `760 мм ртутного стовпця` → `сімсот шістдесят міліметрів ртутного стовпця`
- `760 мм рт ст` → same result (abbreviated form supported)

### Apostrophe

`num2words` uses U+02BC (`ʼ`), but Piper only understands U+0027 (`'`).
Solution: `.replace('ʼ', "'")` after every conversion.

---

## 🐛 Known Issues & Solutions History

### 1. "Truncated" audio (0.5s instead of 3s)
**Cause:** VITS models with `characters` frontend don't handle `add_blank` correctly in sherpa-onnx.
**Solution:** Switched from sherpa-onnx VITS to Piper.

### 2. "AAAUUU" instead of speech
**Cause:** Piper CLI via stdin pipe generated corrupted audio on some systems.
**Solution:** Verified — Piper CLI works correctly. Problem was elsewhere.

### 3. "Сьогодні" → "йогодні"
**Cause:** Uppercase 'С' skipped by model (not in phoneme_id_map).
**Solution:** `.to_lowercase()` before passing to Piper.

### 4. "цельсія" → "цельсІя" (wrong stress)
**Cause:** Model placed stress on the last syllable.
**Solution:** Added U+0301 after 'е': `це́льсія`.

### 5. "шість кома шість" instead of "шість цілих шість десятих"
**Cause:** num2words uses "кома ..." format by default.
**Solution:** Custom `decimal_to_ua()` logic with correct suffixes.

### 6. Python zoo
**Cause:** Initially used Python script for normalization.
**Solution:** Moved everything to Rust via `num2words` crate.

### 7. Piper TTS errors with unknown characters
**Cause:** Piper couldn't recognize some special characters and output errors to stderr, which were ignored.
**Solution:** Added `error_log.rs` module that captures Piper stderr and writes errors to `tts_errors.log` along with the triggering text for analysis.

---

## 📐 Development Rules

1. **Rust only + Piper CLI** — no Python scripts in main flow
2. **Normalization in Rust** — `num2words` crate for numbers
3. **speaker_2 default** — tetiana (female voice)
4. **Lowercase required** — mandatory before passing to Piper
5. **Stress marks** — U+0301 for key words (це́льсія)
6. **Tests required** — every change to `normalize.rs` must have tests
7. **Clean build** — `cargo build` must pass without warnings
8. **Error logging** — all Piper errors logged to `tts_errors.log` via `error_log.rs`

### How to Add a New Special Character

1. In `normalize.rs` find the `while let Some(c) = chars.next()` block
2. Add `else if` for the new character
3. Add a test in `mod tests`

### How to Add a New Model

1. Update `MODELS` in `main.rs` (if applicable)
2. Update `download_models.sh`
3. Verify `model.onnx` and `model.onnx.json` exist
4. Verify `espeak-ng-data/` presence

---

## 🧪 Commands

```bash
# Build
cargo build

# Run
cargo run

# Release
cargo build --release

# Tests
cargo test

# Tests with output
cargo test -- --nocapture
```

## 🚀 Deployment Scripts

| Script | Purpose |
|--------|---------|
| `scripts/install_deps.sh` | Install all dependencies (auto-detects arch) |
| `scripts/build.sh` | Build project (supports cross-compilation) |
| `scripts/deploy.sh` | Install/manage systemd service |
| `scripts/update.sh` | Pull, rebuild, and restart from GitHub |
| `scripts/quickstart.sh` | One-command full setup |

### Build Script Options

```bash
# Release build for current arch
./scripts/build.sh

# Cross-compile for ARM64
./scripts/build.sh -a aarch64

# Cross-compile for ARMv7
./scripts/build.sh -a armv7

# Clean build
./scripts/build.sh -c
```

### Deploy Script Commands

```bash
sudo ./scripts/deploy.sh install    # Install systemd service
sudo ./scripts/deploy.sh start      # Start service
sudo ./scripts/deploy.sh stop       # Stop service
./scripts/deploy.sh status          # Check status
sudo ./scripts/deploy.sh logs -f    # Follow logs
sudo ./scripts/deploy.sh uninstall  # Remove service
```

### Update Script Commands

```bash
./scripts/update.sh                 # Pull, rebuild, restart
./scripts/update.sh -b develop      # Update from specific branch
./scripts/update.sh -f              # Force rebuild (no new commits)
./scripts/update.sh -n              # Dry run (check only)
```

The update script workflow:
1. Verify project root (Cargo.toml exists)
2. Verify service is deployed
3. Check for uncommitted changes (stash if needed)
4. `git pull` — stops if already up to date
5. `./scripts/build.sh` — rebuild binary
6. `sudo ./scripts/deploy.sh install` — update files
7. `sudo ./scripts/deploy.sh restart` — restart service
8. Restore stashed changes (if any)

---

## 📁 Structure

```
├── Cargo.toml              # num2words = "1.2", actix-web, serde
├── Cargo.lock              # Fixed versions
├── config.json             # Server configuration
├── README.md               # English documentation
├── README_UK.md            # Ukrainian documentation
├── AGENTS.md               # This file — AI agent instructions
├── TODO_NUM2WORDS.md       # num2words library issues
├── download_models.sh      # Model download script
├── scripts/
│   ├── install_deps.sh     # Dependency installer
│   ├── build.sh            # Build script (with cross-compile)
│   ├── deploy.sh           # Deployment & service manager
│   ├── update.sh           # Pull, rebuild, restart from GitHub
│   ├── quickstart.sh       # One-command setup
│   └── ua-piper-tts.service  # Systemd service template
├── src/
│   ├── main.rs             # Entry point, config loading
│   ├── server.rs           # Web server, audio generation (WAV/MP3)
│   ├── normalize.rs        # Text normalization (num2words)
│   └── error_log.rs        # TTS error logging
├── models/                 # .gitignore
│   └── piper-uk_UK-dmytro-medium/
│       ├── model.onnx
│       ├── model.onnx.json
│       └── espeak-ng-data/
├── output/                 # .gitignore
│   └── *.mp3 / *.wav
└── tts_errors.log          # .gitignore — auto-generated errors
```

---

## 🔮 Future Improvements

- [x] Log Piper TTS errors to file for analysis
- [x] Automated deployment scripts (install, build, deploy)
- [x] Systemd service integration
- [x] Cross-compilation support (x86_64, aarch64, armv7)
- [x] Automated update script (pull, rebuild, restart)
- [x] Custom filename parameter in TTS request
- [x] `/output/{filename}` endpoint for file serving
- [x] Improved MP3 quality (32 kbps, 22.05 kHz)
- [x] ffmpeg instead of lame for MP3 encoding
- [x] 4-step normalization pipeline with debug support
- [x] Stress dictionary integration (lang-uk, 2.9M words)
- [x] Custom stress dictionary with higher priority
- [x] Cyrillic unit support (мм, см, м, км, г, кг, л, мл)
- [x] Unit declension based on number (1 метр, 2 метри, 5 метрів)
- [x] Pressure unit special handling (мм рт. ст.)
- [ ] Add ASR (speech recognition) — whisper.cpp or sherpa-onnx ASR
- [ ] Add more Ukrainian models (if available)
- [ ] NPU support (RKNN) — not yet working with Piper
- [ ] Streaming TTS — chunked generation for long texts
- [ ] Caching — don't regenerate identical text (see ua-tts-demo)
- [ ] Speaker selection via CLI argument
- [ ] Auto-fix known errors based on `tts_errors.log`

---

*Last updated: April 7, 2025*
