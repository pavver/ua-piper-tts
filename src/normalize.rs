/// Нормалізація українського тексту для Piper TTS.
/// Використовує крейт num2words для конвертації чисел в українські слова.

use num2words::Lang;

/// Суфікси для десяткових дробів за кількістю цифр після коми
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

/// Конвертує число (ціле) в українські слова.
fn int_to_ua(unsigned: &str) -> String {
    if let Ok(n) = unsigned.parse::<f64>() {
        num2words::Num2Words::new(n)
            .lang(Lang::Ukrainian)
            .to_words()
            .unwrap_or(unsigned.to_string())
            .replace('ʼ', "'")
    } else {
        unsigned.to_string()
    }
}

/// Конвертує десяткове число в українські слова.
/// Для °C: якщо ціла > 9 → "і [цифра]" (короткий формат для температури)
/// Інакше → "цілих ... десятих/сотих"
fn decimal_to_ua(int_part: &str, dec_part: &str, is_temp: bool) -> String {
    let int_val = int_part.parse::<i64>().unwrap_or(0);
    let int_words = if int_part.is_empty() || int_part == "0" {
        "нуль".to_string()
    } else {
        int_to_ua(int_part)
    };

    // Десяткова частина — конвертуємо цифри окремо або як ціле
    let dec_digits: Vec<_> = dec_part.chars().filter(|c| c.is_ascii_digit()).collect();
    let dec_clean: String = dec_digits.iter().collect();

    if is_temp && int_val > 9 && dec_digits.len() == 1 {
        // Короткий формат: "36 і шість"
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
        // Повний формат: "цілих ... десятих/сотих"
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

/// Конвертує число (включно з десятковим) в українські слова.
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

/// Конвертує число для температури (°C).
/// Якщо ціла > 9 і одна цифра після коми → "і [цифра]"
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

pub fn normalize_text(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        // Числа (з десятковими та знаком мінус)
        if c.is_ascii_digit() || (c == '-' && chars.peek().map_or(false, |n| n.is_ascii_digit())) {
            let mut num = String::from(c);
            while let Some(&next) = chars.peek() {
                if next.is_ascii_digit() || next == '.' || next == ',' {
                    num.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            // Перевіряємо чи далі йде °C, %, $ тощо
            let next_chars: String = chars.clone().take(3).collect();

            if next_chars.starts_with('°') {
                chars.next(); // °
                let _ws: String = chars.clone().take_while(|c| c.is_whitespace()).collect();
                if let Some(n) = chars.peek() {
                    if *n == 'c' || *n == 'C' {
                        chars.next();
                        out.push_str(&format!("{} градусів це\u{0301}льсія", temp_to_ua(&num)));
                        continue;
                    } else if *n == 'f' || *n == 'F' {
                        chars.next();
                        out.push_str(&format!("{} градусів фаренгейта", temp_to_ua(&num)));
                        continue;
                    }
                }
                out.push_str(&format!("{} градусів", temp_to_ua(&num)));
                continue;
            }
            if next_chars.starts_with('%') {
                chars.next();
                out.push_str(&format!("{} відсотків", num_to_ua(&num)));
                continue;
            }
            if next_chars.starts_with('$') {
                chars.next();
                out.push_str(&format!("{} доларів", num_to_ua(&num)));
                continue;
            }
            if next_chars.starts_with('#') {
                chars.next();
                out.push_str(&format!("номер {}", num_to_ua(&num)));
                continue;
            }
            out.push_str(&num_to_ua(&num));
        } else if c == '+' {
            out.push_str(" плюс ");
        } else if c == '=' {
            out.push_str(" дорівнює ");
        } else if c == '&' {
            out.push_str(" і ");
        } else {
            out.extend(c.to_lowercase());
        }
    }

    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimals() {
        assert!(num_to_ua("36.6").contains("цілих"));
        assert!(num_to_ua("36.6").contains("десятих"));
        assert!(num_to_ua("3.14").contains("сотих"));
        assert!(num_to_ua("0.001").contains("тисячних"));
        assert!(normalize_text("25°C").contains("градусів"));
        assert!(normalize_text("25°C").contains("це\u{0301}льсія"));
        assert!(normalize_text("100%").contains("відсотків"));
    }

    #[test]
    fn test_temp_short_format() {
        // Велике число (> 9) → короткий формат "і"
        assert!(temp_to_ua("36.6").contains(" і "), "'{}' має містити ' і '", temp_to_ua("36.6"));
        assert!(!temp_to_ua("36.6").contains("цілих"), "'{}' не має містити 'цілих'", temp_to_ua("36.6"));

        // Маленьке число (≤ 9) → повний формат
        assert!(temp_to_ua("9.6").contains("цілих"), "'{}' має містити 'цілих'", temp_to_ua("9.6"));
        assert!(temp_to_ua("9.6").contains("десятих"), "'{}' має містити 'десятих'", temp_to_ua("9.6"));

        // Звичайні числа не змінюються
        assert!(num_to_ua("36.6").contains("цілих"));
        assert!(num_to_ua("36.6").contains("десятих"));
    }
}
