use sherpa_onnx::{
    GenerationConfig, OfflineTts, OfflineTtsConfig, OfflineTtsModelConfig, OfflineTtsVitsModelConfig,
};
use std::env;
use std::path::PathBuf;

/// Доступні українські TTS моделі
const MODELS: &[(&str, &str)] = &[
    (
        "mai",
        "vits-coqui-uk-mai",
    ),
    (
        "ukrainian_tts",
        "vits-piper-uk_UA-ukrainian_tts-medium",
    ),
];

fn get_model_path(model_name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("models");
    path.push(model_name);
    path
}

fn create_tts_config(model_dir: &PathBuf) -> OfflineTtsConfig {
    // Для моделі ukrainian_tts-medium файл має іншу назву
    let model_file = model_dir.join("uk_UA-ukrainian_tts-medium.onnx");
    let model_path = if model_file.exists() {
        model_file
    } else {
        model_dir.join("model.onnx")
    };
    let tokens_path = model_dir.join("tokens.txt");
    
    // Перевіряємо чи є espeak-ng-data
    let espeak_dir = model_dir.join("espeak-ng-data");
    let data_dir = if espeak_dir.exists() {
        Some(espeak_dir.to_string_lossy().to_string())
    } else {
        None
    };

    OfflineTtsConfig {
        model: OfflineTtsModelConfig {
            vits: OfflineTtsVitsModelConfig {
                model: Some(model_path.to_string_lossy().to_string()),
                tokens: Some(tokens_path.to_string_lossy().to_string()),
                lexicon: None,
                data_dir,
                noise_scale: 0.667,
                noise_scale_w: 0.8,
                length_scale: 1.0,
                ..Default::default()
            },
            num_threads: 2,
            debug: false,
            provider: Some("cpu".to_string()),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn generate_and_save(tts: &OfflineTts, text: &str, output_path: &str, speaker_id: i32) -> bool {
    let config = GenerationConfig {
        sid: speaker_id,
        speed: 1.0,
        ..Default::default()
    };

    match tts.generate_with_config(text, &config, Option::<fn(&[f32], f32) -> bool>::None) {
        Some(audio) => {
            println!(
                "  Згенеровано {} семплів (sample rate: {} Гц)",
                audio.samples().len(),
                audio.sample_rate()
            );
            if audio.save(output_path) {
                println!("  Збережено у: {}", output_path);
                true
            } else {
                eprintln!("  Помилка збереження WAV файлу");
                false
            }
        }
        None => {
            eprintln!("  Помилка генерації аудіо");
            false
        }
    }
}

fn main() {
    let text = "вітаю";
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("output");
    std::fs::create_dir_all(&output_dir).expect("Не вдалося створити директорію output");

    println!("=== Sherpa-ONNX Ukrainian TTS Test ===");
    println!("Текст: {}", text);
    println!();

    for (short_name, model_name) in MODELS {
        let model_path = get_model_path(model_name);

        if !model_path.exists() {
            eprintln!("[{}] Модель не знайдена: {:?}", short_name, model_path);
            eprintln!("     Завантажте модель вручну або запустіть: ./download_models.sh");
            println!();
            continue;
        }

        println!("--- Модель: {} ({}) ---", short_name, model_name);

        let config = create_tts_config(&model_path);
        
        match OfflineTts::create(&config) {
            Some(tts) => {
                let num_speakers = tts.num_speakers();
                // Якщо модель не має спікерів (num_speakers == 0), використовуємо sid = 0
                let speaker_count = if num_speakers > 0 { num_speakers } else { 1 };
                
                println!(
                    "  Модель завантажена. Спікерів: {} (single speaker model)",
                    num_speakers
                );

                for speaker_id in 0..speaker_count {
                    let output_path = output_dir
                        .join(format!("{}_speaker_{}.wav", short_name, speaker_id));
                    println!(
                        "  Генерація для спікера {}...",
                        speaker_id
                    );
                    generate_and_save(
                        &tts,
                        text,
                        &output_path.to_string_lossy(),
                        speaker_id,
                    );
                }
            }
            None => {
                eprintln!("  Не вдалося завантажити модель");
            }
        }
        println!();
    }

    println!("=== Готово! Перевірте файли у: {:?}", output_dir);
}
