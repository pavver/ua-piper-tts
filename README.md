# Sherpa-UA TTS

Rust-проект для генерації аудіо з тексту українською мовою.

## Як це працює

Rust-код викликає Python-скрипт `tts_synth.py`, який використовує бібліотеку **Piper TTS** для синтезу мовлення.
Piper — це швидка нейронна система TTS яка працює на CPU без підключення до інтернету.

## Моделі

| Модель | Спікери | Якість |
|--------|---------|--------|
| `piper-uk_UK-dmytro-medium` | 3 (lada, mykyta, tetiana) | Хороша |

### Спікери
- **speaker_0 (lada)** — жіночий голос
- **speaker_1 (mykyta)** — чоловічий голос
- **speaker_2 (tetiana)** — жіночий голос (за замовчуванням)

## Встановлення на новій машині

### Системні вимоги

- **ОС:** Linux (x86_64 або aarch64)
- **Python:** 3.8+
- **Rust:** 1.70+
- **Інтернет:** для завантаження моделей (~74 MB)

### Крок 1: Встановіть Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

### Крок 2: Встановіть Piper TTS

```bash
pip3 install piper-tts
```

Якщо використовуєте системний Python (Debian/Ubuntu):
```bash
pip3 install --break-system-packages piper-tts
# або в віртуальному середовищі:
python3 -m venv .venv
source .venv/bin/activate
pip3 install piper-tts
```

### Крок 3: Клонуйте репозиторій

```bash
git clone <url-репозиторію>
cd sherpa-ua-tts
```

### Крок 4: Завантажте модель

```bash
chmod +x download_models.sh
./download_models.sh
```

Це завантажить модель Piper для української мови (~74 MB) у директорію `models/`.

### Крок 5: Зберіть та запустіть

```bash
cargo run
```

Готові WAV-файли з'являться у директорії `output/`:
- `piper_ukrainian_tts_speaker_0.wav` — lada
- `piper_ukrainian_tts_speaker_1.wav` — mykyta
- `piper_ukrainian_tts_speaker_2.wav` — tetiana

## Структура проекту

```
├── Cargo.toml              # Залежності
├── src/main.rs             # Rust код (виклик Python subprocess)
├── tts_synth.py            # Python скрипт для синтезу мови
├── download_models.sh      # Скрипт завантаження моделей
├── models/                 # Завантажені моделі (в gitignore)
│   └── piper-uk_UK-dmytro-medium/
│       ├── model.onnx          # ONNX модель Piper
│       ├── model.onnx.json     # Конфігурація моделі
│       └── espeak-ng-data/     # Дані espeak-ng для фонемізації
├── output/                 # Згенеровані WAV файли (в gitignore)
└── README.md
```

## Використання

### Зміна тексту

Відредагуйте змінну `text` у `src/main.rs`:

```rust
let text = "Ваш текст українською мовою.";
```

### Використання окремого спікера

Запустіть Python-скрипт напряму:

```bash
# Спікер 0 (lada)
python3 tts_synth.py "текст" output.wav 0

# Спікер 1 (mykyta)
python3 tts_synth.py "текст" output.wav 1

# Спікер 2 (tetiana)
python3 tts_synth.py "текст" output.wav 2
```

### Збірка release-версії

```bash
cargo build --release
./target/release/sherpa-ua-tts
```

## Відомі обмеження

1. **Великі літери:** Модель не підтримує великі літери. Текст автоматично конвертується в нижній регістр перед синтезом.
2. **Piper CLI:** Piper CLI генерує пошкоджене аудіо на деяких системах. Тому проект використовує Python бібліотеку `piper-tts` напряму через subprocess.

## Залежності

| Компонент | Версія | Призначення |
|-----------|--------|-------------|
| Rust | 1.70+ | Компіляція проекту |
| Python | 3.8+ | Виконання tts_synth.py |
| piper-tts | будь-яка | Нейронний синтез мови |
| onnxruntime | (автоматично) | ONNX інференс |
| curl | будь-яка | Завантаження моделей |

## Ліцензія

- Код проекту: без обмежень
- Модель Piper: перевіряйте [MODEL_CARD](https://huggingface.co/rhasspy/piper-voices) у репозиторії Piper Voices
