/// Модуль для логування помилок TTS.
/// Записує помилки від Piper у файл для подальшого аналізу.

use std::fs::OpenOptions;
use std::io::Write;

const ERROR_LOG_PATH: &str = "tts_errors.log";

/// Логує помилку TTS разом з текстом, який її викликав.
/// Формат: [timestamp] ПОМИЛКА | текст
pub fn log_tts_error(error: &str, original_text: &str) {
    // Форматуємо рядок: помилка | повний текст
    let log_line = format!("{} | {}\n", error.trim(), original_text.trim());

    // Записуємо у файл (append mode)
    if let Err(e) = write_to_log_file(&log_line) {
        eprintln!("Помилка запису в лог: {}", e);
    }
}

fn write_to_log_file(content: &str) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(ERROR_LOG_PATH)?;

    file.write_all(content.as_bytes())?;
    file.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_format() {
        // Перевіряємо що функція не панікує
        log_tts_error("тест помилки", "тестовий текст");
    }
}
