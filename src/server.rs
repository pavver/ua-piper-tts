use crate::normalize::normalize_text;
use crate::safe_filename;
use crate::AppConfig;
use actix_web::{web, App, HttpResponse, HttpServer, post, get};
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
    pub file_path: String,
    pub file_size: u64,
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

#[post("/tts")]
async fn tts(req: web::Json<TtsRequest>, data: web::Data<AppState>) -> HttpResponse {
    let text = req.text.trim();
    if text.is_empty() {
        return HttpResponse::BadRequest().json(TtsResponse {
            success: false,
            file_path: String::new(),
            file_size: 0,
            message: "Порожній текст".to_string(),
            already_exists: false,
        });
    }

    // Формуємо ім'я файлу з оригінального тексту
    let filename = safe_filename(text);
    let wav_name = format!("{}.wav", filename);
    let output_path = data.config.output_path().join(&wav_name);

    // Перевіряємо чи файл вже існує
    if output_path.exists() && !req.overwrite {
        let size = fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);
        return HttpResponse::Ok().json(TtsResponse {
            success: true,
            file_path: output_path.to_string_lossy().to_string(),
            file_size: size,
            message: "Файл вже існує, overwrite=false".to_string(),
            already_exists: true,
        });
    }

    // Забезпечуємо існування директорії
    if let Err(e) = fs::create_dir_all(data.config.output_path()) {
        return HttpResponse::InternalServerError().json(TtsResponse {
            success: false,
            file_path: String::new(),
            file_size: 0,
            message: format!("Не вдалося створити директорію: {}", e),
            already_exists: false,
        });
    }

    // Нормалізуємо текст
    let normalized = normalize_text(text);

    // Генерація через Piper CLI
    let result = generate_audio(&normalized, &output_path, &data.config);

    match result {
        Ok(()) => {
            let size = fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);
            HttpResponse::Ok().json(TtsResponse {
                success: true,
                file_path: output_path.to_string_lossy().to_string(),
                file_size: size,
                message: "Аудіо згенеровано успішно".to_string(),
                already_exists: false,
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(TtsResponse {
            success: false,
            file_path: String::new(),
            file_size: 0,
            message: format!("Помилка генерації: {}", e),
            already_exists: false,
        }),
    }
}

// ==================== Генерація аудіо ====================

fn generate_audio(
    normalized_text: &str,
    output_path: &PathBuf,
    config: &AppConfig,
) -> Result<(), String> {
    // Знаходимо Piper
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
        .spawn()
        .map_err(|e| format!("Не вдалося запустити piper: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(normalized_text.as_bytes())
            .map_err(|e| format!("Помилка запису в stdin: {}", e))?;
    }

    let status = child.wait()
        .map_err(|e| format!("Не вдалося дочекатись piper: {}", e))?;

    if !status.success() {
        return Err(format!("piper завершився з кодом {:?}", status.code()));
    }

    if !output_path.exists() {
        return Err("Файл не створено".to_string());
    }

    Ok(())
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

    println!("╔══════════════════════════════════════════╗");
    println!("║  Sherpa-UA TTS Server                    ║");
    println!("╠══════════════════════════════════════════╣");
    println!("║  Адреса: http://{}:{}              ║", host, port);
    println!("║  POST /tts   - Генерація аудіо           ║");
    println!("║  GET  /health - Перевірка стану           ║");
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
