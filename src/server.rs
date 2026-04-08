use crate::normalize::normalize_text;
use crate::safe_filename;
use crate::AppConfig;
use crate::error_log::log_tts_error;
use actix_web::{web, App, HttpResponse, HttpServer, get, post, Responder};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::io::Write;
use std::fs;

// ==================== Запити та відповіді ====================

#[derive(Deserialize)]
pub struct TtsRequest {
    pub text: String,
    #[serde(default)]
    pub overwrite: bool,
}

#[derive(Serialize)]
pub struct TtsResponse {
    pub success: bool,
    pub filename: String,
    pub message: String,
    pub already_exists: bool,
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

/// GET /output/{filename} — віддача згенерованого аудіо-файлу
async fn get_output(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let filename = path.into_inner();

    // Захист від path traversal
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return HttpResponse::BadRequest().body("Некоректне ім'я файлу");
    }

    let file_path = data.config.output_path().join(&filename);

    if !file_path.exists() {
        return HttpResponse::NotFound().body(format!("Файл не знайдено: {}", filename));
    }

    // Визначаємо content-type
    let content_type = if filename.ends_with(".mp3") {
        "audio/mpeg"
    } else {
        "audio/wav"
    };

    match fs::read(&file_path) {
        Ok(bytes) => HttpResponse::Ok()
            .content_type(content_type)
            .insert_header(("Accept-Ranges", "bytes"))
            .body(bytes),
        Err(_) => HttpResponse::InternalServerError().body("Не вдалося прочитати файл"),
    }
}

#[post("/tts")]
async fn tts(req: web::Json<TtsRequest>, data: web::Data<AppState>) -> HttpResponse {
    let text = req.text.trim();
    if text.is_empty() {
        return HttpResponse::BadRequest().json(TtsResponse {
            success: false,
            filename: String::new(),
            message: "Порожній текст".to_string(),
            already_exists: false,
        });
    }

    // Формуємо ім'я файлу (MP3 або WAV)
    let filename_base = safe_filename(text);
    let extension = if has_lame() { "mp3" } else { "wav" };
    let filename = format!("{}.{}", filename_base, extension);
    let output_path = data.config.output_path().join(&filename);

    // Перевіряємо чи файл вже існує
    if output_path.exists() && !req.overwrite {
        return HttpResponse::Ok().json(TtsResponse {
            success: true,
            filename: filename.clone(),
            message: "Файл вже існує, overwrite=false".to_string(),
            already_exists: true,
        });
    }

    // Забезпечуємо існування директорії
    if let Err(e) = fs::create_dir_all(data.config.output_path()) {
        return HttpResponse::InternalServerError().json(TtsResponse {
            success: false,
            filename: String::new(),
            message: format!("Не вдалося створити директорію: {}", e),
            already_exists: false,
        });
    }

    // Нормалізуємо текст
    let normalized = normalize_text(text);

    // Генерація аудіо
    let result = if extension == "mp3" {
        generate_mp3(&normalized, &output_path, &data.config)
    } else {
        generate_wav(&normalized, &output_path, &data.config)
    };

    match result {
        Ok(()) => HttpResponse::Ok().json(TtsResponse {
            success: true,
            filename: filename.clone(),
            message: "Аудіо згенеровано успішно".to_string(),
            already_exists: false,
        }),
        Err(e) => HttpResponse::InternalServerError().json(TtsResponse {
            success: false,
            filename: String::new(),
            message: format!("Помилка генерації: {}", e),
            already_exists: false,
        }),
    }
}

// ==================== Генерація аудіо ====================

/// Генерація WAV через Piper CLI
fn generate_wav(
    normalized_text: &str,
    output_path: &PathBuf,
    config: &AppConfig,
) -> Result<(), String> {
    let piper_bin = find_piper().ok_or("piper не знайдено! pip3 install piper-tts")?;
    let model_onnx = config.model_dir().join("model.onnx");
    let model_json = config.model_dir().join("model.onnx.json");

    if !model_onnx.exists() {
        return Err(format!("Модель не знайдена: {:?}", model_onnx));
    }

    let mut child = Command::new(&piper_bin)
        .arg("--model").arg(&model_onnx)
        .arg("--config").arg(&model_json)
        .arg("--output_file").arg(output_path)
        .arg("--speaker").arg(config.speaker_id.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())  // Перехоплюємо stderr
        .spawn()
        .map_err(|e| format!("Не вдалося запустити piper: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(normalized_text.as_bytes())
            .map_err(|e| format!("Помилка запису в stdin: {}", e))?;
    }

    let output = child.wait_with_output()
        .map_err(|e| format!("Не вдалося дочекатись piper: {}", e))?;

    // Перевіряємо stderr на помилки
    if !output.stderr.is_empty() {
        let stderr_text = String::from_utf8_lossy(&output.stderr);
        // Якщо є помилки в stderr, логуємо їх
        if !stderr_text.trim().is_empty() {
            log_tts_error(&stderr_text, normalized_text);
        }
    }

    if !output.status.success() {
        let stderr_text = String::from_utf8_lossy(&output.stderr);
        return Err(format!("piper завершився з кодом {:?}: {}", output.status.code(), stderr_text.trim()));
    }

    if !output_path.exists() {
        return Err("Файл не створено".to_string());
    }

    Ok(())
}

/// Генерація MP3: piper --output-raw (PCM stdout) → ffmpeg → MP3 файл
fn generate_mp3(
    normalized_text: &str,
    output_path: &PathBuf,
    config: &AppConfig,
) -> Result<(), String> {
    let piper_bin = find_piper().ok_or("piper не знайдено!")?;
    let model_onnx = config.model_dir().join("model.onnx");
    let model_json = config.model_dir().join("model.onnx.json");

    if !model_onnx.exists() {
        return Err(format!("Модель не знайдена: {:?}", model_onnx));
    }

    // Piper з --output-raw: пише сирий PCM (s16le, 22050 Hz, mono) на stdout
    let mut piper = Command::new(&piper_bin)
        .arg("--model").arg(&model_onnx)
        .arg("--config").arg(&model_json)
        .arg("--output-raw")            // сирий PCM на stdout
        .arg("--speaker").arg(config.speaker_id.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Не вдалося запустити piper: {}", e))?;

    // Пишемо текст в piper
    if let Some(mut stdin) = piper.stdin.take() {
        stdin.write_all(normalized_text.as_bytes())
            .map_err(|e| format!("Помилка запису в piper: {}", e))?;
    }

    // ffmpeg: читає сирий PCM з stdin → MP3 файл
    // Формат входу: s16le (16-bit little-endian), 22050 Hz, 1 канал (mono)
    let ffmpeg_output = Command::new("ffmpeg")
        .arg("-y")                      // перезапис
        .arg("-f").arg("s16le")         // формат входу: сирий PCM
        .arg("-ar").arg("22050")        // sample rate Piper
        .arg("-ac").arg("1")            // mono
        .arg("-i").arg("pipe:0")        // stdin
        .arg("-codec:a").arg("libmp3lame")
        .arg("-b:a").arg("8k")          // 8 kbps
        .arg("-ac").arg("1")            // mono
        .arg("-ar").arg("8000")         // 8 kHz
        .arg(output_path)               // вихід
        .stdin(piper.stdout.take().ok_or("Не вдалося отримати stdout від piper")?)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Не вдалося запустити ffmpeg: {}", e))?;

    // Чекаємо завершення Piper і перехоплюємо stderr
    let piper_output = piper.wait_with_output()
        .map_err(|e| format!("Не вдалося дочекатись piper: {}", e))?;

    if !piper_output.status.success() {
        let piper_stderr = String::from_utf8_lossy(&piper_output.stderr);
        if !piper_stderr.trim().is_empty() {
            log_tts_error(&piper_stderr, normalized_text);
        }
        return Err(format!("piper завершився з кодом {:?}: {}", piper_output.status.code(), piper_stderr.trim()));
    }

    if !ffmpeg_output.status.success() {
        let stderr = String::from_utf8_lossy(&ffmpeg_output.stderr);
        return Err(format!("ffmpeg помилка: {}", stderr.trim()));
    }

    if !output_path.exists() {
        return Err("MP3 файл не створено".to_string());
    }

    Ok(())
}

/// Перевірка чи встановлено ffmpeg
fn has_lame() -> bool {
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
    let format_str = if has_lame() { "MP3" } else { "WAV" };

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
            .route("/output/{filename}", web::get().to(get_output))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
