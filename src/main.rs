use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::io::Write;

/// Доступні Piper TTS моделі
const MODELS: &[(&str, &str)] = &[
    (
        "piper_ukrainian_tts",
        "piper-uk_UK-dmytro-medium",
    ),
];

fn main() {
    let text = "Привіт, як у тебе справи? Сьогодні гарний день!";
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("output");
    fs::create_dir_all(&output_dir).expect("Не вдалося створити директорію output");

    println!("=== Piper Ukrainian TTS Test ===");
    println!("Текст: {}", text);
    println!();

    // Знаходимо Piper бінарник
    let piper_bin = find_piper();
    if piper_bin.is_none() {
        eprintln!("Помилка: piper не знайдено! Встановіть: pip3 install piper-tts");
        std::process::exit(1);
    }
    let piper_bin = piper_bin.unwrap();

    for (short_name, model_name) in MODELS {
        let model_dir = get_model_path(model_name);
        let model_onnx = model_dir.join("model.onnx");
        let model_json = model_dir.join("model.onnx.json");

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

            // Конвертуємо в нижній регістр бо модель не підтримує великі літери
            let lower_text = text.to_lowercase();

            let result = Command::new(&piper_bin)
                .arg("--model").arg(&model_onnx)
                .arg("--config").arg(&model_json)
                .arg("--output_file").arg(&output_path)
                .arg("--speaker").arg(speaker_id.to_string())
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::null())
                .spawn();

            match result {
                Ok(mut child) => {
                    // Пишемо текст в stdin
                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(lower_text.as_bytes());
                    }

                    let status = child.wait().expect("Не вдалося дочекатись piper");
                    if status.success() {
                        if output_path.exists() {
                            let size = fs::metadata(&output_path).unwrap().len();
                            println!("  Збережено: {} ({} байт)", output_path.display(), size);
                        } else {
                            eprintln!("  Помилка: файл не створено");
                        }
                    } else {
                        eprintln!("  Помилка: piper завершився з кодом {:?}", status.code());
                    }
                }
                Err(e) => {
                    eprintln!("  Помилка запуску piper: {}", e);
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

fn find_piper() -> Option<PathBuf> {
    // Спробуємо знайти piper бінарник
    let candidates = [
        "piper",
        "/home/radxa/.local/bin/piper",
    ];

    for candidate in &candidates {
        let path = PathBuf::from(candidate);
        if path.exists() {
            return Some(path);
        }
        // Спробуємо через which
        let result = Command::new("which").arg(candidate).output().ok()?;
        if result.status.success() {
            let path_str = String::from_utf8_lossy(&result.stdout).trim().to_string();
            if !path_str.is_empty() {
                return Some(PathBuf::from(path_str));
            }
        }
    }
    None
}
