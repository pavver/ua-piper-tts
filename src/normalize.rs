/// Нормалізація українського тексту для Piper TTS.
/// Використовує крейт num2words для конвертації чисел в українські слова.

use num2words::Lang;

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
                } else { break; }
            }
            // Перевіряємо чи далі йде °C, %, $ тощо
            let next_chars: String = chars.clone().take(3).collect();

            if next_chars.starts_with('°') {
                chars.next();
                let _ws: String = chars.clone().take_while(|c| c.is_whitespace()).collect();
                if let Some(n) = chars.peek() {
                    if *n == 'c' || *n == 'C' {
                        chars.next();
                        out.push_str(&format!("{} градусів це\u{0301}льсія", num_to_ua(&num)));
                        continue;
                    } else if *n == 'f' || *n == 'F' {
                        chars.next();
                        out.push_str(&format!("{} градусів фаренгейта", num_to_ua(&num)));
                        continue;
                    }
                }
                out.push_str(&format!("{} градусів", num_to_ua(&num)));
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

    // Замінюємо apostrophe num2words (ʼ U+02BC) на звичайний (ʹ U+02B9 або ')
    out.replace('ʼ', "'").split_whitespace().collect::<Vec<_>>().join(" ")
}

fn num_to_ua(num_str: &str) -> String {
    if let Some(dot) = num_str.find(|c| c == '.' || c == ',') {
        let dec_part = &num_str[dot + 1..];
        let int_words = num2words::Num2Words::new(num_str[..dot].parse::<f64>().unwrap())
            .lang(Lang::Ukrainian).to_words().unwrap_or(num_str[..dot].to_string());
        let dec_words: Vec<_> = dec_part.chars()
            .filter_map(|c| c.to_digit(10))
            .map(|d| num2words::Num2Words::new(d as f64).lang(Lang::Ukrainian).to_words()
                .unwrap_or_else(|_| d.to_string()))
            .collect();
        format!("{} кома {}", int_words, dec_words.join(" "))
    } else {
        num2words::Num2Words::new(num_str.parse::<f64>().unwrap())
            .lang(Lang::Ukrainian).to_words().unwrap_or(num_str.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        assert!(normalize_text("25°C").contains("градусів"));
        assert!(normalize_text("100%").contains("відсотків"));
        assert!(normalize_text("25°C").contains("це\u{0301}льсія"));
    }
}
