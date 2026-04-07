#!/bin/bash
# Скрипт завантаження українських TTS моделей для sherpa-onnx

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
MODELS_DIR="$SCRIPT_DIR/models"
mkdir -p "$MODELS_DIR"

echo "=== Завантаження Piper Ukrainian TTS моделей ==="
echo ""

# Piper модель: uk_UK-dmytro-medium
echo "[1/1] Завантаження piper-uk_UK-dmytro-medium..."
PIPER_DIR="$MODELS_DIR/piper-uk_UK-dmytro-medium"
mkdir -p "$PIPER_DIR"

if [ -f "$PIPER_DIR/model.onnx" ]; then
    echo "  Вже завантажено, пропускаю"
else
    echo "  Завантаження моделі (може зайняти час)..."
    curl -L -o "$PIPER_DIR/model.onnx" \
        "https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/uk/uk_UA/ukrainian_tts/medium/uk_UA-ukrainian_tts-medium.onnx"
    curl -L -o "$PIPER_DIR/model.onnx.json" \
        "https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/uk/uk_UA/ukrainian_tts/medium/uk_UA-ukrainian_tts-medium.onnx.json"
    echo "  Готово!"
fi
echo ""

echo "=== Всі моделі завантажено! ==="
echo "Розміри файлів:"
du -sh "$MODELS_DIR"/*/
