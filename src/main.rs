mod normalize;

use normalize::normalize_text;
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
    let text = "Зараз температура 125°C, а вчора було 36.6°C";
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("output");
    fs::create_dir_all(&output_dir).expect("Не вдалося створити директорію output");

    println!("=== Piper Ukrainian TTS Test ===");
    println!("Текст: {}", text);
    println!();

    let normalized = normalize_text(text);
    println!("Нормалізований: {}", normalized);
    println!();

    let piper_bin = find_piper().expect("piper не знайдено! pip3 install piper-tts");

    for (short_name, model_name) in MODELS {
        let model_dir = get_model_path(model_name);
        let model_onnx = model_dir.join("model.onnx");
        let model_json = model_dir.join("model.onnx.json");

        if !model_onnx.exists() {
            eprintln!("[{}] Модель не знайдена: {:?}", short_name, model_onnx);
            eprintln!("     ./download_models.sh");
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

    println!("=== Готово! {:?}", output_dir);
}

fn get_model_path(model_name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("models");
    path.push(model_name);
    path
}

fn find_piper() -> Option<PathBuf> {
    for c in &["piper", "/home/radxa/.local/bin/piper"] {
        let p = PathBuf::from(c);
        if p.exists() { return Some(p); }
        if let Ok(o) = Command::new("which").arg(c).output() {
            if o.status.success() {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if !s.is_empty() { return Some(PathBuf::from(s)); }
            }
        }
    }
    None
}
