#!/usr/bin/env python3
"""Нормалізація українського тексту для Piper TTS.

Перетворює числа, спецсимволи та великі літери у текст, який Piper може озвучити.

Встановлення:
    pip3 install num2words

Використання:
    echo "25°C" | python3 normalize_text.py
    python3 normalize_text.py "Сьогодні 25°C, завтра -5°C"
"""

import sys
import re
from num2words import num2words


def _num(n):
    """Число → українські слова."""
    try:
        if '.' in str(n) or ',' in str(n):
            return num2words(float(str(n).replace(',', '.')), lang='uk')
        return num2words(int(n), lang='uk')
    except (ValueError, OverflowError):
        return str(n).lower()


def normalize_text(text: str) -> str:
    result = text

    # Комбінації: число + символ (обробляємо першими)
    patterns = [
        (r'(-?\d+(?:[.,]\d+)?)\s*°\s*[Cc]',  lambda m: _num(m.group(1)) + ' градусів цельсія'),
        (r'(-?\d+(?:[.,]\d+)?)\s*°\s*[Ff]',  lambda m: _num(m.group(1)) + ' градусів фаренгейта'),
        (r'(-?\d+(?:[.,]\d+)?)\s*°\s*[Kk]',  lambda m: _num(m.group(1)) + ' градусів кельвіна'),
        (r'(-?\d+)\s*°',                     lambda m: _num(m.group(1)) + ' градусів'),
        (r'(\d+)\s*%',                       lambda m: _num(m.group(1)) + ' відсотків'),
        (r'\$\s*(\d+(?:[.,]\d+)?)',           lambda m: _num(m.group(1)) + ' доларів'),
        (r'€\s*(\d+(?:[.,]\d+)?)',            lambda m: _num(m.group(1)) + ' євро'),
        (r'₴\s*(\d+(?:[.,]\d+)?)',            lambda m: _num(m.group(1)) + ' гривень'),
        (r'#\s*(\d+)',                        lambda m: 'номер ' + _num(m.group(1))),
    ]

    for pattern, repl in patterns:
        result = re.sub(pattern, repl, result)

    # Залишкові числа (не поруч з буквами)
    result = re.sub(r'(?<![а-яіїєґa-z])\s*-?\d+(?:[.,]\d+)?(?![а-яіїєґa-z])',
                    lambda m: _num(m.group(0).strip()), result)

    # Одиночні символи
    for sym, word in [('+', ' плюс '), ('=', ' дорівнює '), ('&', ' і ')]:
        result = result.replace(sym, word)

    return re.sub(r'\s+', ' ', result.lower()).strip()


def main():
    text = ' '.join(sys.argv[1:]) if len(sys.argv) > 1 else sys.stdin.read()
    if not text.strip():
        print("Порожній ввід!", file=sys.stderr)
        sys.exit(1)
    print(normalize_text(text))


if __name__ == "__main__":
    main()
