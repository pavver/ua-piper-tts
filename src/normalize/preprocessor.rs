pub fn step0_fix_paragraphs(text: &str) -> String {
    let mut result = String::with_capacity(text.len() + 32);
    let mut prev_char: Option<char> = None;
    let mut consecutive_newlines = 0;

    for c in text.chars() {
        if c == '\n' {
            consecutive_newlines += 1;
        } else if c == '\r' {
            // Пропускаємо
        } else {
            if consecutive_newlines >= 2 {
                let needs_period = match prev_char {
                    None => false,
                    Some(p) => !matches!(p, '.' | '!' | '?' | '…' | ':' | ';'),
                };
                if needs_period {
                    result.push('.');
                    result.push(' ');
                } else if consecutive_newlines >= 2 {
                    result.push(' ');
                }
            }
            result.push(c);
            prev_char = Some(c);
            consecutive_newlines = 0;
        }
    }

    result
}

// Попередня обробка діапазонів та віднімання
pub fn preprocess_text(text: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            let mut num1_end = i;
            while num1_end < chars.len() && (chars[num1_end].is_ascii_digit() || chars[num1_end] == '.' || chars[num1_end] == ',') {
                num1_end += 1;
            }
            let num1: String = chars[i..num1_end].iter().collect();
            
            if num1_end < chars.len() && chars[num1_end] == '-' {
                let after_dash = num1_end + 1;
                if after_dash < chars.len() && chars[after_dash].is_ascii_digit() {
                    let mut num2_end = after_dash;
                    while num2_end < chars.len() && (chars[num2_end].is_ascii_digit() || chars[num2_end] == '.' || chars[num2_end] == ',') {
                        num2_end += 1;
                    }
                    let num2: String = chars[after_dash..num2_end].iter().collect();
                    
                    let remaining: String = chars[num2_end..].iter().collect();
                    let remaining_trimmed = remaining.trim_start();
                    let unit_candidate: String = remaining_trimmed.split_whitespace().next().unwrap_or("").to_lowercase();
                    
                    let mut is_range = false;
                    let unit_chars: Vec<char> = unit_candidate.chars().collect();
                    let max_len = 10.min(unit_chars.len());
                    for unit_len in (1..=max_len).rev() {
                        let candidate: String = unit_chars[..unit_len].iter().collect();
                        if crate::abbr::UNIT_MAP.contains_key(candidate.as_str()) || candidate.starts_with('°') {
                            is_range = true;
                            break;
                        }
                    }
                    
                    if is_range {
                        result.push_str("від ");
                        result.push_str(&num1);
                        result.push_str(" до ");
                        result.push_str(&num2);
                        result.push_str(&remaining);
                        i = chars.len();
                        continue;
                    } else {
                        result.push_str(&num1);
                        result.push_str(" мінус ");
                        result.push_str(&num2);
                        result.push_str(&chars[num2_end..].iter().collect::<String>());
                        i = chars.len();
                        continue;
                    }
                }
            }
            
            result.push_str(&num1);
            i = num1_end;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    
    result
}
