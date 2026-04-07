mod normalize;
mod server;

use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Clone)]
pub struct AppConfig {
    pub output_dir: String,
    pub port: u16,
    pub host: String,
    pub speaker_id: i32,
    pub model_dir: String,
}

impl AppConfig {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: AppConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn output_path(&self) -> PathBuf {
        PathBuf::from(&self.output_dir)
    }

    pub fn model_dir(&self) -> PathBuf {
        PathBuf::from(&self.model_dir)
    }
}

/// Безпечна назва файлу з тексту
pub fn safe_filename(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9'
            | 'а'..='я' | 'А'..='Я' | 'і' | 'ї' | 'є' | 'ґ' | 'І' | 'Ї' | 'Є' | 'Ґ'
            | ' ' | '-' | '_' | '(' | ')' | '.' | ',' | '!' | '?' | '\'' | '"' => c,
            _ => '_',
        })
        .collect::<String>()
        .replace("__", "_")
        .replace("  ", " ")
        .trim()
        .to_string()
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config_path = "config.json";

    let config = match AppConfig::load(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Помилка завантаження {}: {}", config_path, e);
            eprintln!("Використовуються значення за замовчуванням");
            AppConfig {
                output_dir: "./output".to_string(),
                port: 8080,
                host: "0.0.0.0".to_string(),
                speaker_id: 2,
                model_dir: "./models/piper-uk_UK-dmytro-medium".to_string(),
            }
        }
    };

    server::start_server(config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_filename() {
        assert!(safe_filename("Привіт світ").contains("Привіт"));
        assert!(safe_filename("Текст/з/слешем").contains("_"));
        assert!(safe_filename("Нормальний-текст").contains("Нормальний-текст"));
    }
}
