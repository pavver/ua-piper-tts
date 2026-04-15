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
| **MP3** | `ffmpeg` installed | 8 kbps, mono, 8 kHz (via libmp3lame) |
| **WAV** | `ffmpeg` not found | PCM WAV (uncompressed) |

Format is determined automatically at startup. For minimal file size, MP3 with 8 kbps bitrate in mono mode is used. Piper outputs raw PCM to stdout, which is piped directly into ffmpeg for streaming MP3 encoding.

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

## Number Normalization

| Input | Spoken As |
|-------|-----------|
| `25°C` | двадцять п'ять градусів це́льсія |
| `36.6°C` | тридцять шість **і** шість градусів це́льсія |
| `9.6°C` | дев'ять цілих шість десятих градусів це́льсія |
| `-5°C` | мінус п'ять градусів це́льсія |
| `100%` | сто відсотків |
| `3.14` | три цілих чотирнадцять сотих |

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
├── Cargo.toml              # num2words, hound, actix-web
├── config.json             # Server configuration
├── src/
│   ├── main.rs             # Entry point, config loading
│   ├── server.rs           # Web server, audio generation (WAV/MP3)
│   ├── normalize.rs        # Text normalization (num2words)
│   └── error_log.rs        # TTS error logging
├── download_models.sh      # Piper model download script
├── models/                 # Models (gitignored)
├── output/                 # Output (gitignored)
└── tts_errors.log          # Error log (gitignored, auto-generated)
```

## License

Code — no restrictions. Model — [Piper Voices](https://huggingface.co/rhasspy/piper-voices).
