#!/bin/bash
# Скрипт завантаження українських TTS моделей для sherpa-onnx

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
MODELS_DIR="$SCRIPT_DIR/models"
mkdir -p "$MODELS_DIR"

echo "=== Завантаження українських TTS моделей ==="
echo ""

# Модель 1: vits-piper-uk_UA-lada-x_low
echo "[1/2] Завантаження vits-piper-uk_UA-lada-x_low..."
LADA_DIR="$MODELS_DIR/vits-piper-uk_UA-lada-x_low"
mkdir -p "$LADA_DIR"

if [ -f "$LADA_DIR/model.onnx" ]; then
    echo "  Вже завантажено, пропускаю"
else
    echo "  Завантаження model.onnx..."
    curl -L -o "$LADA_DIR/model.onnx" \
        "https://huggingface.co/csukuangfj/vits-piper-uk_UA-lada-x_low/resolve/main/uk_UA-lada-x_low.onnx"
    echo "  Завантаження tokens.txt..."
    curl -L -o "$LADA_DIR/tokens.txt" \
        "https://huggingface.co/csukuangfj/vits-piper-uk_UA-lada-x_low/resolve/main/tokens.txt"
    echo "  Завантаження lexicon.txt (якщо є)..."
    curl -L -o "$LADA_DIR/lexicon.txt" \
        "https://huggingface.co/csukuangfj/vits-piper-uk_UA-lada-x_low/resolve/main/lexicon.txt" 2>/dev/null || true
    echo "  Готово!"
fi
echo ""

# Модель 2: vits-coqui-uk-mai
echo "[2/2] Завантаження vits-coqui-uk-mai..."
MAI_DIR="$MODELS_DIR/vits-coqui-uk-mai"
mkdir -p "$MAI_DIR"

if [ -f "$MAI_DIR/model.onnx" ]; then
    echo "  Вже завантажено, пропускаю"
else
    echo "  Завантаження model.onnx..."
    curl -L -o "$MAI_DIR/model.onnx" \
        "https://huggingface.co/csukuangfj/vits-coqui-uk-mai/resolve/main/model.onnx"
    echo "  Завантаження tokens.txt..."
    curl -L -o "$MAI_DIR/tokens.txt" \
        "https://huggingface.co/csukuangfj/vits-coqui-uk-mai/resolve/main/tokens.txt"
    echo "  Завантаження lexicon.txt..."
    curl -L -o "$MAI_DIR/lexicon.txt" \
        "https://huggingface.co/csukuangfj/vits-coqui-uk-mai/resolve/main/lexicon.txt"
    echo "  Готово!"
fi
echo ""

# Модель 3: vits-piper-uk_UA-ukrainian_tts-medium (3 спікери)
echo "[3/3] Завантаження vits-piper-uk_UA-ukrainian_tts-medium..."
UKR_TTS_DIR="$MODELS_DIR/vits-piper-uk_UA-ukrainian_tts-medium"
mkdir -p "$UKR_TTS_DIR"

if [ -f "$UKR_TTS_DIR/model.onnx" ]; then
    echo "  Вже завантажено, пропускаю"
else
    echo "  Завантаження архіву з GitHub releases..."
    curl -L -o "$MODELS_DIR/vits-piper-uk_UA-ukrainian_tts-medium.tar.bz2" \
        "https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/vits-piper-uk_UA-ukrainian_tts-medium.tar.bz2"
    echo "  Розпаковка..."
    tar xvf "$MODELS_DIR/vits-piper-uk_UA-ukrainian_tts-medium.tar.bz2" -C "$MODELS_DIR/"
    rm "$MODELS_DIR/vits-piper-uk_UA-ukrainian_tts-medium.tar.bz2"
    echo "  Готово!"
fi
echo ""

echo "=== Всі моделі завантажено! ==="
echo "Розміри файлів:"
du -sh "$MODELS_DIR"/*/
