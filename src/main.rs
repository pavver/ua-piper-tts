use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::io::Write;

const MODELS: &[(&str, &str)] = &[(
    "piper_ukrainian_tts",
    "piper-uk_UK-dmytro-medium",
)];

const SPEAKER_ID: i32 = 2; // tetiana — жіночий голос

fn main() {
    let text = "Привіт, як у тебе справи? Сьогодні гарний день!";
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("output");
    fs::create_dir_all(&output_dir).expect("Не вдалося створити директорію output");

    println!("=== Piper Ukrainian TTS Test ===");
    println!("Текст: {}", text);
    println!();

    // Нормалізуємо текст (числа → слова, великі → маленькі)
    let normalized = normalize_text(text);
    println!("Нормалізований: {}", normalized);
    println!();

    let piper_bin = find_piper().expect("piper не знайдено! Встановіть: pip3 install piper-tts");

    for (short_name, model_name) in MODELS {
        let model_dir = get_model_path(model_name);
        let model_onnx = model_dir.join("model.onnx");
        let model_json = model_dir.join("model.onnx.json");

        if !model_onnx.exists() {
            eprintln!("[{}] Модель не знайдена: {:?}", short_name, model_onnx);
            eprintln!("     Запустіть: ./download_models.sh");
            continue;
        }

        let output_path = output_dir.join(format!("{}_speaker_{}.wav", short_name, SPEAKER_ID));

        println!("--- Модель: {} (speaker {}) ---", short_name, SPEAKER_ID);

        let result = Command::new(&piper_bin)
            .arg("--model").arg(&model_onnx)
            .arg("--config").arg(&model_json)
            .arg("--output_file").arg(&output_path)
            .arg("--speaker").arg(SPEAKER_ID.to_string())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .spawn();

        match result {
            Ok(mut child) => {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(normalized.as_bytes());
                }
                let status = child.wait().expect("Не вдалося дочекатись piper");
                if status.success() && output_path.exists() {
                    let size = fs::metadata(&output_path).unwrap().len();
                    println!("  Збережено: {} ({} байт)", output_path.display(), size);
                } else {
                    eprintln!("  Помилка генерації");
                }
            }
            Err(e) => eprintln!("  Помилка запуску piper: {}", e),
        }
        println!();
    }

    println!("=== Готово! Перевірте файли у: {:?}", output_dir);
}

fn normalize_text(text: &str) -> String {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("normalize_text.py");
    if !script.exists() {
        // Fallback: просто нижній регістр
        return text.to_lowercase();
    }
    let output = Command::new("python3")
        .arg(&script)
        .arg(text)
        .output();
    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }
        _ => text.to_lowercase(),
    }
}

fn get_model_path(model_name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("models");
    path.push(model_name);
    path
}

fn find_piper() -> Option<PathBuf> {
    for candidate in &["piper", "/home/radxa/.local/bin/piper"] {
        let path = PathBuf::from(candidate);
        if path.exists() { return Some(path); }
        if let Ok(out) = Command::new("which").arg(candidate).output() {
            if out.status.success() {
                let p = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !p.is_empty() { return Some(PathBuf::from(p)); }
            }
        }
    }
    None
}
