/// Нормалізація українського тексту для Piper TTS.
/// Використовує крейт num2words для конвертації чисел в українські слова.

use num2words::Lang;
use std::collections::HashMap;
use std::sync::LazyLock;

// ==================== Словник одиниць виміру ====================

static UNIT_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    // Напруга
    m.insert("v", "вольт");
    m.insert("mv", "мілівольт");
    m.insert("kv", "кіловольт");
    // Струм
    m.insert("a", "ампер");
    m.insert("ma", "міліампер");
    m.insert("ka", "кілоампер");
    m.insert("ua", "мікроампер");
    // Потужність
    m.insert("w", "ват");
    m.insert("mw", "міліват");
    m.insert("kw", "кіловат");
    // Енергія
    m.insert("wh", "ват-годин");
    m.insert("mwh", "міліват-годин");
    m.insert("kwh", "кіловат-годин");
    // Опір
    m.insert("ohm", "ом");
    m.insert("kohm", "кілоом");
    m.insert("mohm", "мегаом");
    // Ємність
    m.insert("f", "фарад");
    m.insert("uf", "мікрофарад");
    m.insert("nf", "нанофарад");
    m.insert("pf", "пікофарад");
    // Частота
    m.insert("hz", "герц");
    m.insert("khz", "кілогерц");
    m.insert("mhz", "мегагерц");
    m.insert("ghz", "гігагерц");
    // Температура
    m.insert("°c", " градусів цельсія");
    m.insert("°f", " градусів фаренгейта");
    m.insert("°k", " кельвін");
    m.insert("°", " градусів");
    // Інше
    m.insert("%", " відсотків");
    m.insert("pa", "паскаль");
    m.insert("hpa", "гектопаскаль");
    m.insert("kpa", "кілопаскаль");
    m.insert("lux", "люкс");
    m.insert("db", "децибел");
    m.insert("mm", "міліметр");
    m.insert("cm", "сантиметр");
    m.insert("m", "метр");
    m.insert("km", "кілометр");
    m.insert("mm/s", "міліметрів за секунду");
    m.insert("m/s", "метрів за секунду");
    m.insert("km/h", "кілометрів на годину");
    m.insert("g", "грам");
    m.insert("kg", "кілограм");
    m.insert("l", "літр");
    m.insert("ml", "мілілітр");
    // HA стани
    m.insert("on", "увімкнено");
    m.insert("off", "вимкнено");
    m.insert("open", "відчинено");
    m.insert("closed", "зачинено");
    m.insert("detected", "виявлено");
    m.insert("clear", "чисто");
    m.insert("true", "так");
    m.insert("false", "ні");
    m
});

// ==================== Наголоси ====================

fn with_stress(word: &str) -> String {
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

// ==================== Конвертація чисел ====================

fn decimal_suffix(digits: usize) -> &'static str {
    match digits {
        1 => "десятих",
        2 => "сотих",
        3 => "тисячних",
        4 => "десяти тисячних",
        5 => "сот тисячних",
        6 => "мільйонних",
        _ => "десятих",
    }
}

fn int_to_ua(unsigned: &str) -> String {
    if let Ok(n) = unsigned.parse::<f64>() {
        num2words::Num2Words::new(n)
            .lang(Lang::Ukrainian)
            .to_words()
            .unwrap_or(unsigned.to_string())
            .replace('ʼ', "'")
            // num2words баг: "один цілих" → "один цілий"
            .replace("один цілих", "один цілий")
            .replace("два цілих", "дві цілих")
    } else {
        unsigned.to_string()
    }
}

fn decimal_to_ua(int_part: &str, dec_part: &str, is_temp: bool) -> String {
    let int_val = int_part.parse::<i64>().unwrap_or(0);
    let int_words = if int_part.is_empty() || int_part == "0" {
        "нуль".to_string()
    } else {
        int_to_ua(int_part)
    };

    let dec_digits: Vec<_> = dec_part.chars().filter(|c| c.is_ascii_digit()).collect();
    let dec_clean: String = dec_digits.iter().collect();

    if is_temp && int_val > 9 && dec_digits.len() == 1 {
        let digit_word = if let Some(d) = dec_digits.first().and_then(|c| c.to_digit(10)) {
            num2words::Num2Words::new(d as f64)
                .lang(Lang::Ukrainian)
                .to_words()
                .unwrap_or(dec_clean.clone())
                .replace('ʼ', "'")
        } else {
            dec_clean
        };
        format!("{} і {}", int_words, digit_word)
    } else {
        let dec_words = if dec_clean.is_empty() {
            "нуль".to_string()
        } else if let Ok(n) = dec_clean.parse::<f64>() {
            num2words::Num2Words::new(n)
                .lang(Lang::Ukrainian)
                .to_words()
                .unwrap_or(dec_clean.clone())
                .replace('ʼ', "'")
        } else {
            dec_clean
        };
        format!(
            "{} цілих {} {}",
            int_words,
            dec_words,
            decimal_suffix(dec_digits.len())
        )
    }
}

fn num_to_ua(num_str: &str) -> String {
    let (sign, unsigned) = if num_str.starts_with('-') {
        ("мінус ", &num_str[1..])
    } else {
        ("", num_str)
    };

    if let Some(dot) = unsigned.find(|c| c == '.' || c == ',') {
        let int_part = &unsigned[..dot];
        let dec_part = &unsigned[dot + 1..];
        format!("{}{}", sign, decimal_to_ua(int_part, dec_part, false))
    } else if let Ok(n) = unsigned.parse::<f64>() {
        format!(
            "{}{}",
            sign,
            num2words::Num2Words::new(n)
                .lang(Lang::Ukrainian)
                .to_words()
                .unwrap_or(unsigned.to_string())
                .replace('ʼ', "'")
        )
    } else {
        format!("{}{}", sign, unsigned)
    }
}

fn temp_to_ua(num_str: &str) -> String {
    let (sign, unsigned) = if num_str.starts_with('-') {
        ("мінус ", &num_str[1..])
    } else {
        ("", num_str)
    };

    if let Some(dot) = unsigned.find(|c| c == '.' || c == ',') {
        let int_part = &unsigned[..dot];
        let dec_part = &unsigned[dot + 1..];
        format!("{}{}", sign, decimal_to_ua(int_part, dec_part, true))
    } else if let Ok(n) = unsigned.parse::<f64>() {
        format!(
            "{}{}",
            sign,
            num2words::Num2Words::new(n)
                .lang(Lang::Ukrainian)
                .to_words()
                .unwrap_or(unsigned.to_string())
                .replace('ʼ', "'")
        )
    } else {
        format!("{}{}", sign, unsigned)
    }
}

// ==================== Головний нормалізатор ====================

/// Шукає найдовшу відповідність одиниці виміру з UNIT_MAP.
/// Повертає (довжина_збігу, заміна) або None.
fn find_unit(text: &str) -> Option<(usize, &'static str)> {
    let lower = text.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();

    // Try longest prefix first
    for end in (1..=chars.len()).rev() {
        let candidate: String = chars[..end].iter().collect();
        if let Some(&replacement) = UNIT_MAP.get(candidate.as_str()) {
            return Some((end, replacement));
        }
    }
    None
}

pub fn normalize_text(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        // Числа
        if c.is_ascii_digit() || (c == '-' && chars.peek().map_or(false, |n| n.is_ascii_digit())) {
            let mut num = String::from(c);
            while let Some(&next) = chars.peek() {
                if next.is_ascii_digit() || next == '.' || next == ',' {
                    num.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            // Збираємо одиницю виміру
            let unit_str: String = chars.clone()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '°' || *c == '/' || *c == 'μ')
                .collect();

            let unit_lower = unit_str.to_lowercase();
            let is_temp = unit_lower.starts_with('°');

            let num_words = if is_temp {
                temp_to_ua(&num)
            } else {
                num_to_ua(&num)
            };

            if !unit_str.is_empty() {
                let unit_char_len = unit_str.chars().count();
                for _ in 0..unit_char_len {
                    chars.next();
                }

                if let Some((_len, replacement)) = find_unit(&unit_lower) {
                    let replacement = with_stress(replacement);
                    out.push_str(&format!(" {} ", num_words));
                    out.push_str(&replacement);
                } else {
                    out.push_str(&format!(" {}", num_words));
                }
            } else {
                out.push_str(&num_words);
            }
        } else if c == '+' {
            out.push_str(" плюс ");
        } else if c == '=' {
            out.push_str(" дорівнює ");
        } else if c == '&' {
            out.push_str(" і ");
        } else {
            // Перевіряємо чи це початок HA стану (on/off/open/closed тощо)
            // Збираємо слово починаючи з поточної позиції
            let word_start: String = std::iter::once(c)
                .chain(chars.clone().take_while(|c| c.is_alphabetic()))
                .collect();
            let word_lower = word_start.to_lowercase();

            if let Some((_, replacement)) = find_unit(&word_lower) {
                // Це HA стан або одиниця без числа
                let replacement = with_stress(replacement);
                out.push_str(&replacement);
                let char_count = word_start.chars().count();
                for _ in 0..char_count - 1 {
                    chars.next();
                }
            } else {
                out.extend(c.to_lowercase());
            }
        }
    }

    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ==================== Тести ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimals() {
        assert!(num_to_ua("36.6").contains("цілих"));
        assert!(num_to_ua("36.6").contains("десятих"));
        // % одиниця — перевіряємо що число конвертоване
        let r = normalize_text("100%");
        assert!(r.contains("сто"), "100% → {}", r);
    }

    #[test]
    fn test_temp_short_format() {
        assert!(temp_to_ua("36.6").contains(" і "));
        assert!(!temp_to_ua("36.6").contains("цілих"));
        assert!(temp_to_ua("9.6").contains("цілих"));
    }

    #[test]
    fn test_electric_units() {
        let r = normalize_text("220V");
        assert!(r.contains("двісті"), "220V → {}", r);

        let r = normalize_text("1.5A");
        assert!(r.contains("цілих") || r.contains("ціла"), "1.5A → {}", r);

        let r = normalize_text("500mA");
        assert!(r.contains("п'ятсот"), "500mA → {}", r);

        let r = normalize_text("1500W");
        assert!(r.contains("тисяча"), "1500W → {}", r);

        let r = normalize_text("2.3kW");
        assert!(r.contains("цілих"), "2.3kW → {}", r);

        let r = normalize_text("5.5kWh");
        assert!(r.contains("цілих"), "5.5kWh → {}", r);
        assert!(r.contains("годин"), "5.5kWh → {}", r);
    }

    #[test]
    fn test_frequency() {
        let r = normalize_text("50Hz");
        assert!(r.contains("п'ятдесят"), "50Hz → {}", r);

        let r = normalize_text("2.4GHz");
        assert!(r.contains("цілих"), "2.4GHz → {}", r);
    }

    #[test]
    fn test_ha_states() {
        let r = normalize_text("on");
        assert!(r.contains("увімкнено"), "on → {}", r);

        let r = normalize_text("off");
        assert!(r.contains("вимкнено"), "off → {}", r);

        let r = normalize_text("open");
        assert!(r.contains("відчинено"), "open → {}", r);

        let r = normalize_text("closed");
        assert!(r.contains("зачинено"), "closed → {}", r);
    }

    #[test]
    fn test_voltage_variants() {
        let r = normalize_text("12V");
        assert!(r.contains("дванадцять"), "12V → {}", r);

        let r = normalize_text("3.3V");
        assert!(r.contains("три цілих"), "3.3V → {}", r);
    }

    #[test]
    fn test_current_variants() {
        let r = normalize_text("10A");
        assert!(r.contains("десять"), "10A → {}", r);

        let r = normalize_text("250mA");
        assert!(r.contains("двісті"), "250mA → {}", r);
    }

    #[test]
    fn test_power_variants() {
        let r = normalize_text("100W");
        assert!(r.contains("сто"), "100W → {}", r);

        let r = normalize_text("1.5kW");
        assert!(r.contains("цілих") || r.contains("ціла"), "1.5kW → {}", r);
    }
}
