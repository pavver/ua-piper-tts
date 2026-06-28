/// Модуль для роботи з наголосами та словником наголосів.
/// Завантажує основний та кастомний словники наголосів і надає функції для їх застосування.

use std::collections::HashMap;
use std::sync::LazyLock;

/// Словник наголосів української мови.
/// Порядок завантаження:
///   1. custom_stress_dict.txt (вищий пріоритет — перевизначає основний)
///   2. ua_stress_dict.txt (основний словник lang-uk)
///
/// Формат: кожен рядок — слово з наголосом (U+0301 після голосної).
/// Ключ = слово БЕЗ наголосів (lowercase), значення = слово З наголосами.
pub static STRESS_DICT: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let mut dict = HashMap::new();

    // Спочатку основний словник
    let main_path = "data/ua_stress_dict.txt";
    if let Ok(content) = std::fs::read_to_string(main_path) {
        for line in content.lines() {
            let stressed = line.trim();
            if stressed.is_empty() { continue; }
            
            let lower_first = stressed.to_lowercase();
            let unstressed: String = lower_first.chars()
                .filter(|c| *c != '\u{0301}')
                .collect();
            if lower_first != unstressed {
                dict.insert(unstressed, lower_first);
            }
        }
        eprintln!("[stress_dict] Основний: {} слів", dict.len());
    } else {
        eprintln!("[stress_dict] Попередження: не знайдено {}", main_path);
    }

    // Потім custom — перевизначає основний
    let custom_path = "data/custom_stress_dict.txt";
    let mut custom_count = 0;
    if let Ok(content) = std::fs::read_to_string(custom_path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') { continue; }
            
            // Підтримка формату "ключ=значення" для винятків
            let (key, stressed) = if let Some(eq_pos) = trimmed.find('=') {
                let k = trimmed[..eq_pos].trim().to_lowercase();
                let v = trimmed[eq_pos+1..].trim().to_lowercase();
                (k, v)
            } else {
                let lower_first = trimmed.to_lowercase();
                let unstressed: String = lower_first.chars()
                    .filter(|c| *c != '\u{0301}')
                    .collect();
                (unstressed, lower_first)
            };
            
            if !key.is_empty() {
                dict.insert(key, stressed);
                custom_count += 1;
            }
        }
        eprintln!("[stress_dict] Custom: {} слів (пріоритет вище)", custom_count);
    } else {
        eprintln!("[stress_dict] Попередження: не знайдено {} (не обов'язково)", custom_path);
    }

    eprintln!("[stress_dict] Загалом: {} слів", dict.len());
    dict
});

/// Застосовує наголос до слова, якщо воно присутнє в словнику.
pub fn apply_stress(word: &str) -> String {
    let lower = word.to_lowercase();
    if let Some(stressed) = STRESS_DICT.get(&lower) {
        return stressed.clone();
    }
    lower
}

/// Наголошує закінчення одиниць виміру.
pub fn with_stress_units(word: &str) -> String {
    word.replace("вольт", "во\u{0301}льт")
        .replace("ампер", "ампе\u{0301}р")
        .replace("ват", "ва\u{0301}т")
        .replace("ват-годин", "ва\u{0301}т-годи\u{0301}н")
        .replace("кіловат", "кілова\u{0301}т")
        .replace("кіловат-годин", "кілова\u{0301}т-годи\u{0301}н")
        .replace("міліампер", "міліампе\u{0301}р")
        .replace("міліват", "міліва\u{0301}т")
        .replace("герц", "ге\u{0301}рц")
        .replace("фарад", "фара\u{0301}д")
        .replace("ом", "о\u{0301}м")
        .replace("паскаль", "паска\u{0301}ль")
        .replace("цельсія", "це\u{0301}льсія")
}
