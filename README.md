# Sherpa-UA TTS

Rust-проект для генерації аудіо з тексту українською мовою.

## Як це працює

Проект використовує **Piper TTS** через subprocess виклик з Rust коду. 
Piper — це швидка нейронна система TTS яка працює на CPU.

## Моделі

| Модель | Спікери | Якість |
|--------|---------|--------|
| `piper-uk_UK-dmytro-medium` | 3 (lada, mykyta, tetiana) | Хороша |

### Спікери
- **speaker_0 (lada)** — жіночий голос
- **speaker_1 (mykyta)** — чоловічий голос  
- **speaker_2 (tetiana)** — жіночий голос

## Швидкий старт

### 1. Встановлення залежностей

```bash
pip3 install --user piper-tts
```

### 2. Завантаження моделей

```bash
./download_models.sh
```

### 3. Запуск

```bash
cargo run
```

Аудіо-файли будуть збережені в директорії `output/`.

## Структура проекту

```
├── Cargo.toml              # Залежності (hound)
├── src/main.rs             # Основний код TTS (виклик Piper через subprocess)
├── download_models.sh      # Скрипт завантаження моделей
├── models/                 # Завантажені моделі (не комітяться)
├── output/                 # Згенеровані WAV файли (не комітяться)
└── README.md
```

## Додавання нового тексту

Відредагуйте змінну `text` у `src/main.rs`:

```rust
let text = "ваш текст тут";
```

## Залежності

- Rust (cargo)
- Python 3 + piper-tts (`pip3 install piper-tts`)
- curl (для завантаження моделей)

## Ліцензія

Моделі мають різні ліцензії — перевіряйте MODEL_CARD у кожній моделі.
