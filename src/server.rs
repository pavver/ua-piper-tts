use crate::normalize::normalize_text;
use crate::AppConfig;
use crate::error_log::log_tts_error;
use actix_web::{web, App, HttpResponse, HttpServer, get, post};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::io::Write;


// ==================== Запити та відповіді ====================

#[derive(Deserialize)]
pub struct TtsRequest {
    pub text: String,
}

#[derive(Serialize)]
pub struct TtsErrorResponse {
    pub success: bool,
    pub error: String,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub output_dir: String,
    pub speaker_id: i32,
}

// ==================== State ====================

pub struct AppState {
    pub config: AppConfig,
}

// ==================== Endpoints ====================

#[get("/health")]
async fn health(data: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        output_dir: data.config.output_dir.clone(),
        speaker_id: data.config.speaker_id,
    })
}

#[post("/tts")]
async fn tts(req: web::Json<TtsRequest>, data: web::Data<AppState>) -> HttpResponse {
    let text = req.text.trim();
    if text.is_empty() {
        return HttpResponse::BadRequest().json(TtsErrorResponse {
            success: false,
            error: "Порожній текст".to_string(),
        });
    }

    // Нормалізуємо текст
    let normalized = normalize_text(text);

    let use_mp3 = has_ffmpeg();

    // Генерація аудіо в пам'яті
    let result = if use_mp3 {
        generate_mp3_in_memory(&normalized, &data.config)
            .map(|bytes| (bytes, "audio/mpeg"))
    } else {
        generate_wav_in_memory(&normalized, &data.config)
            .map(|bytes| (bytes, "audio/wav"))
    };

    match result {
        Ok((bytes, mime_type)) => HttpResponse::Ok()
            .content_type(mime_type)
            .body(bytes),
        Err(e) => HttpResponse::InternalServerError().json(TtsErrorResponse {
            success: false,
            error: format!("Помилка генерації: {}", e),
        }),
    }
}

// ==================== Генерація аудіо ====================

/// Генерація WAV через Piper CLI в пам'ять (stdout)
fn generate_wav_in_memory(
    normalized_text: &str,
    config: &AppConfig,
) -> Result<Vec<u8>, String> {
    let piper_bin = find_piper().ok_or("piper не знайдено! pip3 install piper-tts")?;
    let model_onnx = config.model_dir().join("model.onnx");
    let model_json = config.model_dir().join("model.onnx.json");

    if !model_onnx.exists() {
        return Err(format!("Модель не знайдена: {:?}", model_onnx));
    }

    // За замовчуванням piper пише WAV у stdout, якщо не вказано --output_file
    let mut child = Command::new(&piper_bin)
        .arg("--model").arg(&model_onnx)
        .arg("--config").arg(&model_json)
        .arg("--speaker").arg(config.speaker_id.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Не вдалося запустити piper: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        let mut text_to_send = normalized_text.to_string();
        if !text_to_send.ends_with('\n') {
            text_to_send.push('\n');
        }
        stdin.write_all(text_to_send.as_bytes())
            .map_err(|e| format!("Помилка запису в stdin: {}", e))?;
    }

    let output = child.wait_with_output()
        .map_err(|e| format!("Не вдалося дочекатись piper: {}", e))?;

    // Перевіряємо stderr на помилки
    if !output.stderr.is_empty() {
        let stderr_text = String::from_utf8_lossy(&output.stderr);
        if !stderr_text.trim().is_empty() {
            log_tts_error(&stderr_text, normalized_text);
        }
    }

    if !output.status.success() {
        let stderr_text = String::from_utf8_lossy(&output.stderr);
        return Err(format!("piper завершився з кодом {:?}: {}", output.status.code(), stderr_text.trim()));
    }

    Ok(output.stdout)
}

/// Генерація MP3: piper --output-raw (PCM) → ffmpeg → MP3 в пам'яті (stdout)
fn generate_mp3_in_memory(
    normalized_text: &str,
    config: &AppConfig,
) -> Result<Vec<u8>, String> {
    let piper_bin = find_piper().ok_or("piper не знайдено!")?;
    let model_onnx = config.model_dir().join("model.onnx");
    let model_json = config.model_dir().join("model.onnx.json");

    if !model_onnx.exists() {
        return Err(format!("Модель не знайдена: {:?}", model_onnx));
    }

    // Piper → stdout (raw PCM s16le, 22050 Hz, mono)
    let mut piper = Command::new(&piper_bin)
        .arg("--model").arg(&model_onnx)
        .arg("--config").arg(&model_json)
        .arg("--output-raw")
        .arg("--speaker").arg(config.speaker_id.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Не вдалося запустити piper: {}", e))?;

    // Пишемо текст в piper
    if let Some(mut stdin) = piper.stdin.take() {
        let mut text_to_send = normalized_text.to_string();
        if !text_to_send.ends_with('\n') {
            text_to_send.push('\n');
        }
        stdin.write_all(text_to_send.as_bytes())
            .map_err(|e| format!("Помилка запису в piper: {}", e))?;
    }

    // ffmpeg: stdin (raw PCM) → stdout (MP3)
    let ffmpeg = Command::new("ffmpeg")
        .arg("-y")
        .arg("-f").arg("s16le")
        .arg("-ar").arg("22050")
        .arg("-ac").arg("1")
        .arg("-i").arg("pipe:0")
        .arg("-codec:a").arg("libmp3lame")
        .arg("-b:a").arg("32k")
        .arg("-ac").arg("1")
        .arg("-ar").arg("22050")
        .arg("-f").arg("mp3")
        .arg("pipe:1")
        .stdin(piper.stdout.take().ok_or("Не вдалося отримати stdout від piper")?)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Не вдалося запустити ffmpeg: {}", e))?;

    let ffmpeg_output = ffmpeg.wait_with_output()
        .map_err(|e| format!("Не вдалося дочекатись ffmpeg: {}", e))?;

    // Чекаємо завершення Piper і отримуємо stderr
    let piper_output = piper.wait_with_output()
        .map_err(|e| format!("Не вдалося дочекатись piper: {}", e))?;

    // Перевіряємо stderr від Piper на помилки
    if !piper_output.stderr.is_empty() {
        let piper_stderr = String::from_utf8_lossy(&piper_output.stderr);
        if !piper_stderr.trim().is_empty() {
            log_tts_error(&piper_stderr, normalized_text);
        }
    }

    if !piper_output.status.success() {
        let piper_stderr = String::from_utf8_lossy(&piper_output.stderr);
        return Err(format!("piper завершився з кодом {:?}: {}", piper_output.status.code(), piper_stderr.trim()));
    }

    if !ffmpeg_output.status.success() {
        let stderr = String::from_utf8_lossy(&ffmpeg_output.stderr);
        return Err(format!("ffmpeg помилка: {}", stderr.trim()));
    }

    Ok(ffmpeg_output.stdout)
}

/// Перевірка чи встановлено ffmpeg
fn has_ffmpeg() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
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

// ==================== Запуск сервера ====================

pub async fn start_server(config: AppConfig) -> std::io::Result<()> {
    let host = config.host.clone();
    let port = config.port;
    let format_str = if has_ffmpeg() { "MP3" } else { "WAV" };

    println!("╔══════════════════════════════════════════╗");
    println!("║  Sherpa-UA TTS Server                    ║");
    println!("╠══════════════════════════════════════════╣");
    println!("║  Адреса: http://{}:{}              ║", host, port);
    println!("║  POST /tts   - Генерація аудіо           ║");
    println!("║  GET  /health - Перевірка стану           ║");
    println!("║  Формат: {}                          ║", format_str);
    println!("║  Speaker: {}                            ║", config.speaker_id);
    println!("║  Output: {}               ║", config.output_dir);
    println!("╚══════════════════════════════════════════╝");
    println!();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState { config: config.clone() }))
            .service(tts)
            .service(health)
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
