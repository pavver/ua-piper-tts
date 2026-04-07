/// Нормалізація українського тексту для Piper TTS.
/// Перетворює числа, спецсимволи та великі літери у текст з наголосами.

fn ones(n: u32) -> &'static str {
    match n {
        0 => "", 1 => "один", 2 => "два", 3 => "три", 4 => "чотири",
        5 => "п'ять", 6 => "шість", 7 => "сім", 8 => "вісім", 9 => "дев'ять",
        _ => "",
    }
}

fn teens(n: u32) -> &'static str {
    match n {
        10 => "десять", 11 => "одинадцять", 12 => "дванадцять",
        13 => "тринадцять", 14 => "чотирнадцять", 15 => "п'ятнадцять",
        16 => "шістнадцять", 17 => "сімнадцять", 18 => "вісімнадцять",
        19 => "дев'ятнадцять", _ => "",
    }
}

fn tens(n: u32) -> &'static str {
    match n {
        2 => "двадцять", 3 => "тридцять", 4 => "сорок",
        5 => "п'ятдесят", 6 => "шістдесят", 7 => "сімдесят",
        8 => "вісімдесят", 9 => "дев'яносто", _ => "",
    }
}

fn hundreds(n: u32) -> &'static str {
    match n {
        1 => "сто", 2 => "двісті", 3 => "триста", 4 => "чотириста",
        5 => "п'ятсот", 6 => "шістсот", 7 => "сімсот",
        8 => "вісімсот", 9 => "дев'ятсот", _ => "",
    }
}

fn three_digits(n: u32) -> String {
    let mut parts = Vec::new();
    if n >= 100 { parts.push(hundreds(n / 100)); }
    let rem = n % 100;
    if rem >= 10 && rem <= 19 { parts.push(teens(rem)); }
    else {
        if rem >= 20 { parts.push(tens(rem / 10)); }
        let o = rem % 10;
        if o > 0 { parts.push(ones(o)); }
    }
    parts.join(" ")
}

fn number_to_words(n: i64) -> String {
    if n == 0 { return "нуль".into(); }
    let neg = n < 0;
    let mut num = if neg { -n } else { n } as u64;
    let mut parts = Vec::new();
    if neg { parts.push("мінус".into()); }
    if num >= 1_000_000_000 {
        let b = num / 1_000_000_000;
        parts.push(format!("{} мільярд{}", three_digits(b as u32), billion_end(b)));
        num %= 1_000_000_000;
    }
    if num >= 1_000_000 {
        let m = num / 1_000_000;
        parts.push(format!("{} мільйон{}", three_digits(m as u32), million_end(m)));
        num %= 1_000_000;
    }
    if num >= 1_000 {
        let t = num / 1_000;
        let t_str = three_digits(t as u32);
        let t_fixed = t_str.replace("один", "одна").replace("два", "дві");
        parts.push(format!("{} тисяч{}", t_fixed, thousand_end(t)));
        num %= 1_000;
    }
    if num > 0 || parts.is_empty() { parts.push(three_digits(num as u32)); }
    parts.join(" ")
}

fn thousand_end(n: u64) -> &'static str {
    let l = n % 100;
    if (11..=19).contains(&l) { "" } else { match l % 10 { 1 => "а", 2..=4 => "і", _ => "" } }
}
fn million_end(n: u64) -> &'static str {
    let l = n % 100;
    if (11..=19).contains(&l) { "ів" } else { match l % 10 { 1 => "", 2..=4 => "и", _ => "ів" } }
}
fn billion_end(n: u64) -> &'static str {
    let l = n % 100;
    if (11..=19).contains(&l) { "ів" } else { match l % 10 { 1 => "", 2..=4 => "и", _ => "ів" } }
}

fn number_with_decimal(num_str: &str) -> String {
    if let Some(dot) = num_str.find(|c| c == '.' || c == ',') {
        let int_part = &num_str[..dot];
        let dec_part = &num_str[dot + 1..];
        let int_words = if let Ok(n) = int_part.parse::<i64>() {
            number_to_words(n)
        } else { int_part.to_string() };
        format!("{} кома {}", int_words, number_to_words_decimals(dec_part))
    } else if let Ok(n) = num_str.parse::<i64>() {
        number_to_words(n)
    } else { num_str.to_string() }
}

fn number_to_words_decimals(s: &str) -> String {
    let names = ["нуль", "один", "два", "три", "чотири", "п'ять",
                 "шість", "сім", "вісім", "дев'ять"];
    s.chars().filter_map(|c| c.to_digit(10))
        .map(|d| names[d as usize]).collect::<Vec<_>>().join(" ")
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
                } else { break; }
            }
            // Перевіряємо чи далі йде °C, %, $ тощо
            let _ws: String = chars.clone().take_while(|c| c.is_whitespace()).collect();
            let next_chars: String = chars.clone().take(3).collect();

            if next_chars.starts_with('°') || next_chars.starts_with('c') || next_chars.starts_with('C') {
                // °C
                let nc: String = chars.clone().take(2).collect();
                if nc.starts_with('°') {
                    //吃掉 °
                    chars.next();
                    let after: String = chars.clone().take_while(|c| c.is_whitespace()).collect();
                    for _ in 0..after.len() { chars.next(); }
                    if let Some(n) = chars.peek() {
                        if *n == 'c' || *n == 'C' {
                            chars.next();
                            out.push_str(&format!("{} градусів це\u{0301}льсія", number_with_decimal(&num)));
                            continue;
                        }
                    }
                    out.push_str(&format!("{} градусів", number_with_decimal(&num)));
                    continue;
                }
            }
            if next_chars.starts_with('%') {
                chars.next(); // %
                out.push_str(&format!("{} відсотків", number_with_decimal(&num)));
                continue;
            }
            if next_chars.starts_with('$') {
                chars.next(); // $
                out.push_str(&format!("{} доларів", number_with_decimal(&num)));
                continue;
            }
            if next_chars.starts_with('#') {
                chars.next(); // #
                out.push_str(&format!("номер {}", number_with_decimal(&num)));
                continue;
            }
            out.push_str(&number_with_decimal(&num));
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
    fn test_numbers() {
        assert_eq!(number_to_words(0), "нуль");
        assert_eq!(number_to_words(25), "двадцять п'ять");
        assert_eq!(number_to_words(125), "сто двадцять п'ять");
        assert_eq!(number_to_words(-5), "мінус п'ять");
    }

    #[test]
    fn test_normalize() {
        assert!(normalize_text("25°C").contains("градусів"));
        assert!(normalize_text("100%").contains("відсотків"));
    }
}
