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
┌─────────────┐     ┌────────────────┐     ┌────────────┐
│   Rust code │────▶│ normalize/     │────▶│  Piper CLI │
│  main.rs    │     │ (token pipeline)────▶│  (subproc) │
└─────────────┘     └────────────────┘     └────────────┘
                           │                      │
                      Normalized            WAV/MP3 file
                      text (lowercase,          in output/
                      numbers as words,
                      stress marks)
```

### The Tokenization Pipeline

The normalizer uses a modular three-phase pipeline:
1. **Lexer** (`lexer.rs`): Segments text into raw token fragments (words, numbers, punctuation, spaces, times, dates).
2. **Parser** (`parser.rs`): Analyzes preceding/following tokens to resolve semantic relations (attaching prepositions, merging units, applying multiplier scales).
3. **Generator** (`generator.rs`): Translates tokens into standard Ukrainian speech phrases, applying grammar rules and stress marks.

### Key Components

| File/Folder | Purpose |
|-------------|---------|
| `src/main.rs` | Entry point, config loading, module imports |
| `src/server.rs` | Web server (actix-web), request handling, WAV/MP3 generation |
| `src/normalize/` | Modular normalization pipeline: lexer, parser, generator, preprocessor |
| `src/error_log.rs` | Logs Piper TTS errors to `tts_errors.log` for analysis |
| `download_models.sh` | Downloads Piper model from HuggingFace |
| `models/piper-uk_UK-dmytro-medium/` | ONNX model + espeak-ng data |

### Dependencies

```toml
hound = "3.5"       # Audio handling
num2words = { git = "https://github.com/pavver/num2words_v2" } # Fork with Ukrainian grammar fixes
actix-web = "4"     # Web framework
actix-files = "0.6" # Static file serving
serde = "1"         # Serialization
serde_json = "1"    # JSON handling
tokio = "1"         # Async runtime
```

### External Dependencies (not Rust)

- **piper-tts** (`pip3 install piper-tts`) — provides `piper` CLI binary
- **ONNX Runtime** — auto-installed with piper-tts
- **ffmpeg** (optional) — for MP3 encoding (raw PCM piped to ffmpeg with libmp3lame codec)

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

### Apostrophe

`num2words` uses U+02BC (`ʼ`), but Piper only understands U+0027 (`'`).
Solution: `.replace('ʼ', "'")` after every conversion.

### Thousand Multipliers (к / k)

Technical numbers followed by `к` or `k` (case-insensitive, optionally with trailing dots or separated by spaces) are multiplied by `1000`:
- `1.5к кг` ➔ `"тисяча п'ятсот кілограмів"`
- `2.5 к` ➔ `"дві тисячі п'ятсот"`
- `1 k.` ➔ `"тисяча"`

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

### 6. Piper TTS errors with unknown characters
**Cause:** Piper couldn't recognize some special characters and output errors to stderr, which were ignored.
**Solution:** Added `error_log.rs` module that captures Piper stderr and writes errors to `tts_errors.log` along with the triggering text for analysis.

---

## 📐 Development Rules

1. **Rust only + Piper CLI** — no Python scripts in main flow
2. **Normalization in Rust** — `num2words` crate for numbers
3. **speaker_2 default** — tetiana (female voice)
4. **Lowercase required** — mandatory before passing to Piper
5. **Stress marks** — U+0301 for key words (це́льсія)
6. **Tests required** — every change to the normalization logic must be verified by tests in `src/normalize/mod.rs`
7. **Clean build** — `cargo build` must pass without warnings
8. **Error logging** — all Piper errors logged to `tts_errors.log` via `error_log.rs`

### How to Add a New Special Character

1. If it's a punctuation character, update `split_punctuation` in `src/normalize/lexer.rs`.
2. If it's a multiplier or context unit, update the parser in `src/normalize/parser.rs` and the unit map `UNIT_MAP` in `src/abbr.rs`.
3. Add a test in `src/normalize/mod.rs` inside the `mod tests` block.

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

# Run specific test with logs
cargo test <test_name> -- --nocapture
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
├── Cargo.toml              # num2words git fork, actix-web, serde
├── Cargo.lock              # Fixed versions
├── config.json             # Server configuration
├── README.md               # English documentation
├── README_UK.md            # Ukrainian documentation
├── AGENTS.md               # This file — AI agent instructions
├── download_models.sh      # Model download script
├── scripts/
│   ├── install_deps.sh     # Dependency installer
│   ├── build.sh            # Build script (with cross-compilation)
│   ├── deploy.sh           # Deployment & service manager
│   ├── update.sh           # Pull, rebuild, restart from GitHub
│   ├── quickstart.sh       # One-command setup
│   └── ua-piper-tts.service  # Systemd service template
├── src/
│   ├── main.rs             # Entry point, config loading
│   ├── server.rs           # Web server, audio generation (WAV/MP3)
│   ├── normalize/          # Modular normalizer pipeline
│   │   ├── mod.rs          # Entry point and tests
│   │   ├── types.rs        # RawToken, Token, NumberValue
│   │   ├── helpers.rs      # Number formats (int_to_ua, decimal_to_ua, num_to_ua)
│   │   ├── preprocessor.rs # Paragraphs & mathematical range scanner
│   │   ├── lexer.rs        # Scanner and character tokenization
│   │   ├── parser.rs       # Semantic parser & unit lookup
│   │   └── generator.rs    # Text renderer
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

- [ ] Add more Ukrainian models (if available)
- [ ] Speaker selection support

## 🌟 Completed Features

- [x] Transitioned normalizer to modular Lexer-Parser-Generator architecture
- [x] Log Piper TTS errors to file for analysis (`error_log.rs`)
- [x] Automated deployment scripts (install, build, deploy)
- [x] Systemd service integration
- [x] Cross-compilation support (x86_64, aarch64, armv7)
- [x] Automated update script (pull, rebuild, restart)

---

*Last updated: 28.06.2026*
