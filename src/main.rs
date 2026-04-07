use std::env;
use std::path::PathBuf;
use std::process::Command;

/// Доступні Piper TTS моделі
/// (short_name, model_dir_name)
const MODELS: &[(&str, &str)] = &[
    (
        "piper_ukrainian_tts",
        "piper-uk_UK-dmytro-medium",
    ),
];

fn main() {
    let text = "Привіт, як у тебе справи? Сьогодні гарний день!";
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("output");
    std::fs::create_dir_all(&output_dir).expect("Не вдалося створити директорію output");

    println!("=== Piper Ukrainian TTS Test ===");
    println!("Текст: {}", text);
    println!();

    // Знаходимо Python скрипт
    let script_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let synth_script = script_dir.join("tts_synth.py");

    if !synth_script.exists() {
        eprintln!("Помилка: tts_synth.py не знайдено!");
        std::process::exit(1);
    }

    for (short_name, model_name) in MODELS {
        let model_dir = get_model_path(model_name);
        let model_onnx = model_dir.join("model.onnx");

        if !model_onnx.exists() {
            eprintln!("[{}] Модель не знайдена: {:?}", short_name, model_onnx);
            eprintln!("     Запустіть: ./download_models.sh");
            println!();
            continue;
        }

        println!("--- Модель: {} ({}) ---", short_name, model_name);

        // Piper має 3 спікери: lada=0, mykyta=1, tetiana=2
        for speaker_id in 0..3 {
            let output_path = output_dir.join(format!("{}_speaker_{}.wav", short_name, speaker_id));

            println!("  Генерація для спікера {} (speaker_id={})...", speaker_id, speaker_id);

            let result = Command::new("python3")
                .arg(&synth_script)
                .arg(text)
                .arg(&output_path)
                .arg(speaker_id.to_string())
                .output();

            match result {
                Ok(output) => {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        println!("  {}", stdout.trim());
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        eprintln!("  Помилка: {}", stderr.lines().last().unwrap_or("невідомо"));
                    }
                }
                Err(e) => {
                    eprintln!("  Помилка запуску: {}", e);
                }
            }
        }
        println!();
    }

    println!("=== Готово! Перевірте файли у: {:?}", output_dir);
}

fn get_model_path(model_name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("models");
    path.push(model_name);
    path
}
