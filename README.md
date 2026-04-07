# Sherpa-UA TTS

Rust-проект для генерації аудіо з тексту українською мовою.

## Як це працює

Rust викликає **Piper TTS** (CLI) через subprocess. 
Перед синтезом текст нормалізується Python-скриптом (`num2words`):
- Числа → українські слова (25 → "двадцять п'ять")
- Спецсимволи → слова (°C → "градусів цельсія")
- Великі літери → маленькі (модель не підтримує uppercase)

## Моделі

| Модель | Спікери | Якість |
|--------|---------|--------|
| `piper-uk_UK-dmytro-medium` | 3 (lada, mykyta, tetiana) | Хороша |

За замовчуванням використовується **speaker_2 (tetiana)** — жіночий голос.

## Встановлення

### Вимоги
- Linux (x86_64 або aarch64)
- Python 3.8+
- Rust 1.70+

### Крок 1: Встановіть залежності

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Piper TTS + нормалізація
pip3 install piper-tts num2words
```

### Крок 2: Клонуйте та завантажте модель

```bash
git clone <url>
cd sherpa-ua-tts
chmod +x download_models.sh
./download_models.sh  # ~74 MB
```

### Крок 3: Запуск

```bash
cargo run
```

Результат: `output/piper_ukrainian_tts_speaker_2.wav`

## Зміна тексту та спікера

У `src/main.rs`:

```rust
let text = "Ваш текст українською.";
const SPEAKER_ID: i32 = 2; // 0=lada, 1=mykyta, 2=tetiana
```

## Підтримувані спецсимволи

| Ввід | Вимовляється як |
|------|-----------------|
| `25°C` | двадцять п'ять градусів цельсія |
| `-5°C` | мінус п'ять градусів цельсія |
| `100%` | сто відсотків |
| `$50` | п'ятдесят доларів |
| `#123` | номер сто двадцять три |
| `3.14` | три кома чотирнадцять |

## Ліцензія

Код — без обмежень. Модель — [Piper Voices](https://huggingface.co/rhasspy/piper-voices).
