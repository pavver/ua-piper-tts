#!/usr/bin/env python3
"""TTS скрипт для генерації аудіо з тексту українською мовою."""

import sys
import wave
import os

def main():
    if len(sys.argv) < 4:
        print("Usage: tts_synth.py <text> <output_path> <speaker_id>")
        sys.exit(1)

    text = sys.argv[1].lower()  # Конвертуємо в нижній регістр
    output_path = sys.argv[2]
    speaker_id = int(sys.argv[3])

    # Шлях до моделі
    script_dir = os.path.dirname(os.path.abspath(__file__))
    model_dir = os.path.join(script_dir, "models", "piper-uk_UK-dmytro-medium")
    model_path = os.path.join(model_dir, "model.onnx")
    config_path = os.path.join(model_dir, "model.onnx.json")

    if not os.path.exists(model_path):
        print(f"Error: Model not found at {model_path}", file=sys.stderr)
        sys.exit(1)

    # Завантажуємо модель
    from piper.voice import PiperVoice
    from piper.config import SynthesisConfig
    
    voice = PiperVoice.load(model_path, config_path=config_path)
    config = SynthesisConfig(speaker_id=speaker_id)

    # Генеруємо аудіо
    chunks = list(voice.synthesize(text, config))
    
    if not chunks:
        print("Error: No audio generated", file=sys.stderr)
        sys.exit(1)

    # Записуємо у WAV
    sample_rate = chunks[0].sample_rate
    with wave.open(output_path, 'wb') as wav_file:
        wav_file.setnchannels(1)
        wav_file.setsampwidth(2)  # 16-bit
        wav_file.setframerate(sample_rate)
        for chunk in chunks:
            wav_file.writeframes(chunk.audio_int16_bytes)

    # Перевіряємо результат
    w = wave.open(output_path, 'rb')
    frames = w.getnframes()
    dur = frames / sample_rate
    w.close()

    print(f"Generated: {output_path} ({dur:.3f}s, {frames} frames)")

if __name__ == "__main__":
    main()
