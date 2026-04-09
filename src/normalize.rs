/// Нормалізація українського тексту для Piper TTS.
use num2words::Lang;
use num2words::Ukrainian;
use num2words::Language;
use std::collections::HashMap;
use std::sync::LazyLock;

static UNIT_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("v", "вольт"); m.insert("mv", "мілівольт"); m.insert("kv", "кіловольт");
    m.insert("a", "ампер"); m.insert("ma", "міліампер"); m.insert("ka", "кілоампер");
    m.insert("w", "ват"); m.insert("mw", "міліват"); m.insert("kw", "кіловат");
    m.insert("wh", "ват-годин"); m.insert("mwh", "міліват-годин"); m.insert("kwh", "кіловат-годин");
    m.insert("ohm", "ом"); m.insert("kohm", "кілоом"); m.insert("mohm", "мегаом");
    m.insert("f", "фарад"); m.insert("uf", "мікрофарад"); m.insert("nf", "нанофарад"); m.insert("pf", "пікофарад");
    m.insert("hz", "герц"); m.insert("khz", "кілогерц"); m.insert("mhz", "мегагерц"); m.insert("ghz", "гігагерц");
    m.insert("°c", " градусів цельсія"); m.insert("°f", " градусів фаренгейта");
    m.insert("°k", "кельвін"); m.insert("°", " градусів");
    m.insert("%", "відсотків");
    m.insert("pa", "паскаль"); m.insert("hpa", "гектопаскаль"); m.insert("kpa", "кілопаскаль");
    m.insert("lux", "люкс"); m.insert("db", "децибел");
    m.insert("mm", "міліметр"); m.insert("cm", "сантиметр"); m.insert("m", "метр"); m.insert("km", "кілометр");
    m.insert("мм", "міліметр"); m.insert("см", "сантиметр"); m.insert("м", "метр"); m.insert("км", "кілометр");
    m.insert("g", "грам"); m.insert("kg", "кілограм"); m.insert("г", "грам"); m.insert("кг", "кілограм");
    m.insert("l", "літр"); m.insert("ml", "мілілітр"); m.insert("л", "літр"); m.insert("мл", "мілілітр");
    m.insert("mm/s", "міліметрів за секунду"); m.insert("m/s", "метрів за секунду"); m.insert("km/h", "кілометрів на годину");
    m.insert("mmhg", "міліметрів ртутного стовпця");
    m.insert("on", "увімкнено"); m.insert("off", "вимкнено"); m.insert("open", "відчинено");
    m.insert("closed", "зачинено"); m.insert("detected", "виявлено");
    m.insert("clear", "чисто"); m.insert("true", "так"); m.insert("false", "ні");
    m
});

fn decimal_suffix(digits: usize) -> &'static str {
    match digits { 1 => "десятих", 2 => "сотих", 3 => "тисячних", 4 => "десяти тисячних", 5 => "сот тисячних", 6 => "мільйонних", _ => "десятих" }
}

fn int_to_ua(unsigned: &str) -> String {
    if let Ok(n) = unsigned.parse::<f64>() {
        num2words::Num2Words::new(n).lang(Lang::Ukrainian).to_words()
            .unwrap_or(unsigned.to_string()).replace('ʼ', "'")
            .replace("один цілих", "один цілий").replace("два цілих", "дві цілих")
    } else { unsigned.to_string() }
}

fn decimal_to_ua(int_part: &str, dec_part: &str, is_temp: bool) -> String {
    let int_val = int_part.parse::<i64>().unwrap_or(0);
    let int_words = if int_part.is_empty() || int_part == "0" { "нуль".to_string() } else { int_to_ua(int_part) };
    let dec_digits: Vec<_> = dec_part.chars().filter(|c| c.is_ascii_digit()).collect();
    let dec_clean: String = dec_digits.iter().collect();
    if is_temp && int_val > 9 && dec_digits.len() == 1 {
        let digit_word = if let Some(d) = dec_digits.first().and_then(|c| c.to_digit(10)) {
            num2words::Num2Words::new(d as f64).lang(Lang::Ukrainian).to_words().unwrap_or(dec_clean.clone()).replace('ʼ', "'")
        } else { dec_clean };
        format!("{} і {}", int_words, digit_word)
    } else {
        let dec_words = if dec_clean.is_empty() { "нуль".to_string() }
        else if let Ok(n) = dec_clean.parse::<f64>() {
            num2words::Num2Words::new(n).lang(Lang::Ukrainian).to_words().unwrap_or(dec_clean.clone()).replace('ʼ', "'")
        } else { dec_clean };
        format!("{} цілих {} {}", int_words, dec_words, decimal_suffix(dec_digits.len()))
    }
}

fn num_to_ua_cardinal(num_str: &str) -> String {
    if let Ok(n) = num_str.parse::<i64>() {
        num2words::Num2Words::new(n).lang(Lang::Ukrainian).to_words()
            .unwrap_or(num_str.to_string()).replace('ʼ', "'")
            .replace("один цілих", "один цілий").replace("два цілих", "дві цілих")
    } else { num_str.to_string() }
}

fn num_to_ua(num_str: &str) -> String {
    let (sign, unsigned) = if num_str.starts_with('-') { ("мінус ", &num_str[1..]) } else { ("", num_str) };
    if let Some(dot) = unsigned.find(|c| c == '.' || c == ',') {
        format!("{}{}", sign, decimal_to_ua(&unsigned[..dot], &unsigned[dot+1..], false))
    } else if let Ok(n) = unsigned.parse::<f64>() {
        format!("{}{}", sign, num2words::Num2Words::new(n).lang(Lang::Ukrainian).to_words().unwrap_or(unsigned.to_string()).replace('ʼ', "'"))
    } else { format!("{}{}", sign, unsigned) }
}

fn temp_to_ua(num_str: &str) -> String {
    let (sign, unsigned) = if num_str.starts_with('-') { ("мінус ", &num_str[1..]) } else { ("", num_str) };
    if let Some(dot) = unsigned.find(|c| c == '.' || c == ',') {
        format!("{}{}", sign, decimal_to_ua(&unsigned[..dot], &unsigned[dot+1..], true))
    } else if let Ok(n) = unsigned.parse::<f64>() {
        format!("{}{}", sign, num2words::Num2Words::new(n).lang(Lang::Ukrainian).to_words().unwrap_or(unsigned.to_string()).replace('ʼ', "'"))
    } else { format!("{}{}", sign, unsigned) }
}

fn with_stress(word: &str) -> String {
    word.replace("вольт", "во\u{0301}льт").replace("ампер", "ампе\u{0301}р")
        .replace("ват", "ва\u{0301}т").replace("ват-годин", "ва\u{0301}т-годи\u{0301}н")
        .replace("кіловат", "кілова\u{0301}т").replace("кіловат-годин", "кілова\u{0301}т-годи\u{0301}н")
        .replace("міліампер", "міліампе\u{0301}р").replace("міліват", "міліва\u{0301}т")
        .replace("герц", "ге\u{0301}рц").replace("фарад", "фара\u{0301}д")
        .replace("ом", "о\u{0301}м").replace("паскаль", "паска\u{0301}ль")
        .replace("цельсія", "це\u{0301}льсія")
}

fn find_unit(text: &str) -> Option<(usize, &'static str)> {
    let lower = text.to_lowercase();
    // Тільки точні співпадіння (щоб "мінус" не співпадав з "м")
    if let Some(&replacement) = UNIT_MAP.get(lower.as_str()) {
        return Some((lower.len(), replacement));
    }
    None
}

/// Попередня обробка тексту:
/// - "5-10 кг" → "від 5 до 10 кг" (діапазон з одиницею)
/// - "4-3" → "4 мінус 3" (звичайне віднімання)
fn preprocess_text(text: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        // Перевіряємо чи це початок числа
        if chars[i].is_ascii_digit() {
            // Збираємо перше число
            let mut num1_end = i;
            while num1_end < chars.len() && (chars[num1_end].is_ascii_digit() || chars[num1_end] == '.' || chars[num1_end] == ',') {
                num1_end += 1;
            }
            let num1: String = chars[i..num1_end].iter().collect();
            
            // Перевіряємо чи далі йде дефіс і друге число
            if num1_end < chars.len() && chars[num1_end] == '-' {
                let after_dash = num1_end + 1;
                if after_dash < chars.len() && chars[after_dash].is_ascii_digit() {
                    // Збираємо друге число
                    let mut num2_end = after_dash;
                    while num2_end < chars.len() && (chars[num2_end].is_ascii_digit() || chars[num2_end] == '.' || chars[num2_end] == ',') {
                        num2_end += 1;
                    }
                    let num2: String = chars[after_dash..num2_end].iter().collect();
                    
                    // Перевіряємо чи після другого числа є одиниця виміру
                    let remaining: String = chars[num2_end..].iter().collect();
                    let remaining_trimmed = remaining.trim_start();
                    let unit_candidate: String = remaining_trimmed.split_whitespace().next().unwrap_or("").to_lowercase();
                    
                    // Перевіряємо чи це відома одиниця (з урахуванням °c, °f, mm/s тощо)
                    let mut is_range = false;
                    let max_len = 10.min(unit_candidate.len());
                    for unit_len in (1..=max_len).rev() {
                        let candidate = &unit_candidate[..unit_len];
                        if UNIT_MAP.contains_key(candidate) || candidate.starts_with('°') {
                            is_range = true;
                            break;
                        }
                    }
                    
                    if is_range {
                        // Це діапазон: "від X до Y"
                        result.push_str("від ");
                        result.push_str(&num1);
                        result.push_str(" до ");
                        result.push_str(&num2);
                        // Додаємо решту тексту після другого числа
                        result.push_str(&remaining);
                        i = chars.len();
                        continue;
                    } else {
                        // Звичайне віднімання: "X мінус Y"
                        result.push_str(&num1);
                        result.push_str(" мінус ");
                        result.push_str(&num2);
                        // Додаємо решту тексту
                        result.push_str(&chars[num2_end..].iter().collect::<String>());
                        i = chars.len();
                        continue;
                    }
                }
            }
            
            // Не діапазон, просто число
            result.push_str(&num1);
            i = num1_end;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    
    result
}

pub fn normalize_text(text: &str) -> String {
    // Попередня обробка: діапазони та віднімання
    let preprocessed = preprocess_text(text);
    
    let words: Vec<&str> = preprocessed.split_whitespace().collect();
    let mut result = Vec::new();
    let mut i = 0;
    while i < words.len() {
        let word = words[i];
        // Check for ordinal with suffix: "43-ї"
        if let Some(pos) = word.find('-') {
            let num_part = &word[..pos];
            let suffix = &word[pos+1..];
            if let Ok(_n) = num_part.parse::<i64>() {
                if let Some(_gram) = Ukrainian::from_ordinal_suffix(suffix) {
                    if let Ok(ordinal) = Ukrainian::default().ordinal_from_text(word) {
                        result.push(ordinal);
                        i += 1; continue;
                    }
                }
            }
        }
        // Check for context-aware ordinal (e.g. "о 1 годині")
        // Пропускаємо якщо це частина віднімання "X мінус Y"
        if word.parse::<i64>().is_ok() && word != "мінус" {
            let prep = if i > 0 { Some(words[i - 1]) } else { None };
            // Якщо наступне слово "мінус" - це віднімання, не ordinal
            let next_is_minus = if i + 1 < words.len() { words[i + 1] == "мінус" } else { false };
            if !next_is_minus {
                let next_word = if i + 1 < words.len() { Some(words[i + 1]) } else { None };
                if let Ok(n) = word.parse::<i64>() {
                    let (g, d, n_count) = Ukrainian::analyze_ordinal_context(prep, word, next_word);
                    let u = Ukrainian::new(g, n_count, d);
                    if let Ok(ordinal) = u.to_ordinal(n.into()) {
                        result.push(ordinal);
                        i += 1; continue;
                    }
                }
            }
        }
        // Number+unit: "220V"
        if let Some((num_str, unit)) = split_number_unit(word) {
            result.push(num_to_ua(&num_str));
            if let Some((_, u)) = find_unit(&unit.to_lowercase()) {
                result.push(with_stress(u).to_string());
            }
            i += 1; continue;
        }
        let lower = word.to_lowercase();
        if let Some((_, u)) = find_unit(&lower) { result.push(with_stress(u).to_string()); } else { result.push(lower); }
        i += 1;
    }
    result.join(" ")
}

fn split_number_unit(word: &str) -> Option<(String, String)> {
    let chars: Vec<char> = word.chars().collect();
    let mut num_end = 0;
    for (i, c) in chars.iter().enumerate() {
        if c.is_ascii_digit() || (*c == '.' || *c == ',') { num_end = i + 1; } else { break; }
    }
    if num_end > 0 && num_end < chars.len() {
        let num: String = chars[..num_end].iter().collect();
        let unit: String = chars[num_end..].iter().collect();
        if num.parse::<f64>().is_ok() { return Some((num, unit)); }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_decimals() {
        assert!(int_to_ua("36.6").contains("цілих"));
        let r = normalize_text("100%");
        assert!(r.contains("сто"), "100% → {}", r);
    }
    #[test]
    fn test_electric_units() {
        let r = normalize_text("220V");
        assert!(r.contains("двісті"), "220V → {}", r);
    }
    #[test]
    fn test_ordinal_suffix_hyphen() {
        assert_eq!(Ukrainian::default().ordinal_from_text("43-ї").unwrap(), "сорок третьої");
        assert_eq!(Ukrainian::default().ordinal_from_text("43-й").unwrap(), "сорок третій");
    }
    #[test]
    fn test_ha_states() {
        assert!(normalize_text("on").contains("увімкнено"));
        assert!(normalize_text("off").contains("вимкнено"));
    }
    #[test]
    fn test_full_sentence() {
        let r = normalize_text("о 1 годині ночі");
        eprintln!("Result: '{}'", r);
        assert!(r.contains("першій"), "о 1 годині ночі → {}", r);
    }
    #[test]
    fn test_range_with_unit() {
        let r = normalize_text("Вночі буде 0-3°");
        eprintln!("Range: '{}'", r);
        assert!(r.contains("від"), "0-3° → {}", r);
        assert!(r.contains("до"), "0-3° → {}", r);
    }
    #[test]
    fn test_subtraction_without_unit() {
        let r = normalize_text("Обчисліть 4-3");
        eprintln!("Subtraction: '{}'", r);
        assert!(r.contains("мінус"), "4-3 → {}", r);
    }
}

