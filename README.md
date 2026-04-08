# UA-Piper-TTS

Rust project for generating audio from Ukrainian text.

## How It Works

Rust calls **Piper TTS** (CLI) via subprocess.
Before synthesis, text is normalized using pure Rust code with the `num2words` crate:
- Numbers → Ukrainian words (25 → "двадцять п'ять")
- Special characters → words (°C → "градусів це́льсія" with stress mark)
- Uppercase → lowercase (model doesn't support uppercase)

## Audio Format

| Format | Condition | Parameters |
|--------|-----------|------------|
| **MP3** | `ffmpeg` installed | 32 kbps, mono, 22.05 kHz |
| **WAV** | `ffmpeg` not found | PCM WAV (uncompressed, 22050 Hz) |

Format is determined automatically at startup. MP3 with 32 kbps bitrate provides clear speech quality with small file size (~8 KB per phrase).

## Web Server (REST API)

### Start

```bash
cargo run
```

### Configuration (`config.json`)

```json
{
    "output_dir": "./output",
    "port": 9000,
    "host": "0.0.0.0",
    "speaker_id": 2,
    "model_dir": "./models/piper-uk_UK-dmytro-medium"
}
```

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/health` | Server health check |
| `POST` | `/tts` | Generate audio |
| `GET` | `/output/{filename}` | Download generated audio file |

### TTS Request Body

```json
{
  "text": "Привіт світе",
  "overwrite": false,
  "filename": "custom_name"
}
```

| Parameter | Required | Description |
|-----------|----------|-------------|
| `text` | Yes | Text to synthesize |
| `overwrite` | No | `true` = regenerate even if file exists (default: false) |
| `filename` | No | Custom filename (without extension). Auto-generated from text if omitted. |

### Example Request

```bash
curl -X POST http://localhost:9000/tts \
  -H "Content-Type: application/json" \
  -d '{"text": "Привіт світе", "overwrite": false}'
```

### Example Response

```json
{
  "success": true,
  "filename": "Привіт світе.mp3",
  "message": "Audio generated successfully",
  "already_exists": false
}
```

Parameter `overwrite`: `true` — regenerate file, `false` — skip if exists.

## Models

| Model | Speakers | Quality |
|-------|----------|---------|
| `piper-uk_UK-dmytro-medium` | 3 (lada, mykyta, tetiana) | Good |

Default uses **speaker_2 (tetiana)** — female voice.

## Installation

### Requirements
- Linux (x86_64 or aarch64)
- Python 3.8+ (Piper CLI only)
- Rust 1.70+

### Quick Install (Recommended)

One-command setup: install dependencies, build, deploy, and start:

```bash
./scripts/quickstart.sh
```

### Manual Installation

#### Step 1: Install Dependencies

```bash
./scripts/install_deps.sh
```

This script will:
- Detect your architecture (x86_64, aarch64, armv7)
- Install system packages (gcc, openssl, ffmpeg, etc.)
- Install Rust (if not present)
- Install Piper TTS
- Optionally download the Ukrainian model

#### Step 2: Build

```bash
# Release build for current architecture
./scripts/build.sh

# Cross-compile for ARM64
./scripts/build.sh -a aarch64

# Cross-compile for ARMv7
./scripts/build.sh -a armv7

# Clean build
./scripts/build.sh -c
```

#### Step 3: Deploy & Run

```bash
# Install as systemd service (requires sudo)
sudo ./scripts/deploy.sh install

# Start the service
sudo ./scripts/deploy.sh start

# Check status
./scripts/deploy.sh status

# View logs
sudo ./scripts/deploy.sh logs -f
```

### Alternative: Run Directly

```bash
cargo run
```

Server will start at `http://0.0.0.0:9000`. Audio is saved to `output/`.

## Changing Speaker

In `config.json`:

```json
{
    "speaker_id": 2  // 0=lada, 1=mykyta, 2=tetiana
}
```

## Service Management

After deploying with `./scripts/deploy.sh install`:

| Command | Description |
|---------|-------------|
| `sudo ./scripts/deploy.sh start` | Start the service |
| `sudo ./scripts/deploy.sh stop` | Stop the service |
| `sudo ./scripts/deploy.sh restart` | Restart the service |
| `./scripts/deploy.sh status` | Check service status |
| `sudo ./scripts/deploy.sh logs -f` | Follow service logs |
| `sudo ./scripts/deploy.sh uninstall` | Remove the service |

The service runs as `tts` system user with security hardening enabled.

## Updating

To update to the latest version:

```bash
# Pull latest changes, rebuild, and restart
./scripts/update.sh

# Update from a specific branch
./scripts/update.sh -b develop

# Force rebuild even if no new commits
./scripts/update.sh -f

# Dry run — check what would be done
./scripts/update.sh -n
```

The update script will:
1. Verify you're in the project root
2. Check if the service is deployed
3. Pull latest changes from GitHub (stops if already up to date)
4. Rebuild the project
5. Redeploy and restart the service

## Text Normalization

### How It Works

Normalization is a 4-step pipeline. Each step can be called separately via `normalize_text_debug()` for debugging:

| Step | Function | Description |
|------|----------|-------------|
| 1 | `step1_tokenize()` | Splits text into tokens (numbers+units, words, symbols) |
| 2 | `step2_convert_numbers()` | Converts numbers → Ukrainian words, units → full names |
| 3 | `step3_apply_stress()` | Adds stress marks via dictionaries |
| 4 | `step4_cleanup()` | Normalizes whitespace |

### Stress Dictionaries

The project uses **two** stress dictionaries:

| File | Source | Size | Priority |
|------|--------|------|----------|
| `data/ua_stress_dict.txt` | [lang-uk/ukrainian-word-stress-dictionary](https://github.com/lang-uk/ukrainian-word-stress-dictionary) | ~2.9M words | Lower |
| `data/custom_stress_dict.txt` | Our custom dictionary | Growing | **Higher** |

**How priority works:** main dictionary is loaded first, then custom — the latter **overrides** main dictionary entries. This allows fixing incorrect stress marks or adding new words.

**Custom dictionary format:**
```
# Comments start with #
вологі́сть          — new word with stress
сі́мсот             — override existing
міліме́трів         — unit of measurement
```

Stress is marked with U+0301 (combining acute accent) after a vowel.

### What Gets Normalized

| Type | Input | Output |
|------|-------|--------|
| Numbers | `25` | двадцять п'ять |
| Temperatures | `36.6°C` | тридцять шість **і** шість градусів це́льсія |
| Decimals ≤ 9 | `9.6°C` | дев'ять цілих шість десятих градусів це́льсія |
| Negative | `-5°C` | мінус п'ять градусів це́льсія |
| Percentages | `100%` | сто відсотків |
| Units (Cyrillic) | `5 м`, `10 см`, `200 г` | п'ять метрів, десять сантиметрів, двісті грамів |
| Units (Latin) | `220V`, `50Hz` | двісті двадцять вольт, п'ятдесят герц |
| Pressure | `760 мм ртутного стовпця` | сімсот шістдесят міліметрів ртутного стовпця |
| HA states | `on`, `off`, `open` | увімкнено, вимкнено, відчинено |

### Unit Declension

| Number | Form | Example |
|--------|------|---------|
| **1** | singular | один метр, один грам |
| **2-4** | plural | два метри, три кілограми |
| **5+** | genitive plural | п'ять метрів, десять грамів |

### Temperature Rule
- Integer part **> 9**: short format `"36 і шість"`
- Integer part **≤ 9**: full format `"дев'ять цілих шість десятих"`

## Error Logging

The application automatically captures errors from Piper TTS and writes them to `tts_errors.log`.

### Log Format

```
[Piper error message] | [Text that triggered the error]
```

This allows you to:
- Quickly find problematic text fragments
- Have full context for error analysis
- Fix text or model settings accordingly

## Project Structure

```
├── Cargo.toml                  # num2words, hound, actix-web
├── config.json                 # Server configuration
├── src/
│   ├── main.rs                 # Entry point, config loading
│   ├── server.rs               # Web server, audio generation (WAV/MP3)
│   ├── normalize.rs            # 4-step text normalization
│   └── error_log.rs            # TTS error logging
├── download_models.sh          # Piper model download script
├── data/
│   ├── ua_stress_dict.txt      # Main stress dictionary (lang-uk, 2.9M words)
│   └── custom_stress_dict.txt  # Custom stress dictionary (higher priority)
├── models/                     # Models (gitignored)
├── output/                     # Output (gitignored)
└── tts_errors.log              # Error log (gitignored, auto-generated)
```

## License

Code — no restrictions. Model — [Piper Voices](https://huggingface.co/rhasspy/piper-voices).
