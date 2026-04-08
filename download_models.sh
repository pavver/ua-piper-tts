#!/bin/bash
# Скрипт завантаження українських TTS моделей для Piper

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
MODELS_DIR="$SCRIPT_DIR/models"
mkdir -p "$MODELS_DIR"

echo "=== Завантаження Piper Ukrainian TTS моделей ==="
echo ""

# Piper модель: uk_UA-ukrainian_tts-medium
echo "[1/1] Завантаження piper-uk_UK-dmytro-medium..."
PIPER_DIR="$MODELS_DIR/piper-uk_UK-dmytro-medium"
mkdir -p "$PIPER_DIR"

if [ -f "$PIPER_DIR/model.onnx" ]; then
    echo "  Модель вже завантажена"
else
    echo "  Завантаження моделі (може зайняти час)..."
    curl -L -o "$PIPER_DIR/model.onnx" \
        "https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/uk/uk_UA/ukrainian_tts/medium/uk_UA-ukrainian_tts-medium.onnx"
    curl -L -o "$PIPER_DIR/model.onnx.json" \
        "https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/uk/uk_UA/ukrainian_tts/medium/uk_UA-ukrainian_tts-medium.onnx.json"
    echo "  Готово!"
fi

# espeak-ng дані потрібні для фонемізації
if [ -d "$PIPER_DIR/espeak-ng-data" ]; then
    echo "  espeak-ng-data вже існує, пропускаю"
else
    echo ""
    echo "  Увага: espeak-ng-data відсутній!"
    echo "  Ця модель потребує espeak-ng даних для фонемізації."
    echo ""
    read -rp "  Скопіювати espeak-ng-data з piper-tts пакунку? [Y/n]: " copy_espeak
    if [[ ! "$copy_espeak" =~ ^[Nn]$ ]]; then
        echo "  Шукаю espeak-ng-data..."
        PIPER_DATA_DIR=""

        # Варіант 1: через piper package
        if command -v pip3 &>/dev/null; then
            PIPER_SITE=$(pip3 show piper-tts 2>/dev/null | grep "^Location:" | cut -d' ' -f2-)
            if [ -n "$PIPER_SITE" ]; then
                FOUND=$(find "$PIPER_SITE/piper" -name "espeak-ng-data" -type d 2>/dev/null | head -1)
                if [ -n "$FOUND" ] && [ -d "$FOUND" ]; then
                    PIPER_DATA_DIR="$FOUND"
                fi
            fi
        fi

        # Варіант 2: системний espeak-ng
        if [ -z "$PIPER_DATA_DIR" ]; then
            for dir in /usr/share/espeak-ng-data /usr/local/share/espeak-ng-data /usr/lib/espeak-ng-data; do
                if [ -d "$dir" ]; then
                    PIPER_DATA_DIR="$dir"
                    break
                fi
            done
        fi

        if [ -n "$PIPER_DATA_DIR" ] && [ -d "$PIPER_DATA_DIR" ]; then
            echo "  Знайдено: $PIPER_DATA_DIR"
            echo "  Копіювання..."
            cp -r "$PIPER_DATA_DIR" "$PIPER_DIR/espeak-ng-data"
            echo "  Готово!"
        else
            echo "  НЕ вдалося знайти espeak-ng-data!"
            echo "  Спробуйте: sudo apt install espeak-ng"
            echo "  Або скопіюйте вручну з іншої Piper моделі"
        fi
    fi
fi
echo ""

echo "=== Всі моделі завантажено! ==="
echo "Розміри файлів:"
du -sh "$MODELS_DIR"/*/
