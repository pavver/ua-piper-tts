use super::types::RawToken;

pub fn try_parse_time(word: &str) -> Option<RawToken> {
    if !word.contains(':') {
        return None;
    }
    let parts: Vec<&str> = word.split(':').collect();
    if parts.len() == 2 || parts.len() == 3 {
        if parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit())) {
            let hours = parts[0];
            let minutes = parts[1];
            let seconds = if parts.len() == 3 { Some(parts[2].to_string()) } else { None };
            
            if let (Ok(h), Ok(m)) = (hours.parse::<u8>(), minutes.parse::<u8>()) {
                if h <= 23 && m <= 59 {
                    if let Some(ref s_str) = seconds {
                        if let Ok(s) = s_str.parse::<u8>() {
                            if s > 59 { return None; }
                        } else { return None; }
                    }
                    return Some(RawToken::Time {
                        hours: hours.to_string(),
                        minutes: minutes.to_string(),
                        seconds,
                    });
                }
            }
        }
    }
    None
}

pub fn try_parse_date(word: &str) -> Option<RawToken> {
    if !word.contains('.') {
        return None;
    }
    let parts: Vec<&str> = word.split('.').collect();
    if parts.len() == 2 || parts.len() == 3 {
        if parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit())) {
            let day_str = parts[0];
            let month_str = parts[1];
            let year_str = if parts.len() == 3 { Some(parts[2]) } else { None };
            
            if let (Ok(d), Ok(m)) = (day_str.parse::<u8>(), month_str.parse::<u8>()) {
                if d >= 1 && d <= 31 && m >= 1 && m <= 12 && month_str.len() == 2 {
                    if let Some(y_str) = year_str {
                        if y_str.len() != 4 || y_str.parse::<u16>().is_err() {
                            return None;
                        }
                    }
                    return Some(RawToken::Date {
                        day: day_str.to_string(),
                        month: month_str.to_string(),
                        year: year_str.map(|s| s.to_string()),
                    });
                }
            }
        }
    }
    None
}

pub fn try_parse_number(word: &str) -> Option<RawToken> {
    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() { return None; }
    
    let mut idx = 0;
    let mut is_negative = false;
    if chars[idx] == '-' {
        is_negative = true;
        idx += 1;
    }
    
    let mut int_part = String::new();
    while idx < chars.len() && chars[idx].is_ascii_digit() {
        int_part.push(chars[idx]);
        idx += 1;
    }
    
    if int_part.is_empty() {
        return None;
    }
    
    let mut dec_part = None;
    if idx < chars.len() && (chars[idx] == '.' || chars[idx] == ',') {
        if idx + 1 < chars.len() && chars[idx + 1].is_ascii_digit() {
            idx += 1;
            let mut dp = String::new();
            while idx < chars.len() && chars[idx].is_ascii_digit() {
                dp.push(chars[idx]);
                idx += 1;
            }
            dec_part = Some(dp);
        }
    }
    
    let suffix = if idx < chars.len() {
        Some(chars[idx..].iter().collect::<String>())
    } else {
        None
    };
    
    // ordinal suffixes should not be split as separate units
    if let Some(ref s) = suffix {
        if s.starts_with('-') {
            return Some(RawToken::Number {
                raw: word.to_string(),
                is_negative,
                int_part,
                dec_part,
                suffix: suffix.clone(),
            });
        }
    }
    
    Some(RawToken::Number {
        raw: word.to_string(),
        is_negative,
        int_part,
        dec_part,
        suffix,
    })
}

pub fn parse_word_token(word: &str) -> RawToken {
    if let Some(t) = try_parse_time(word) {
        return t;
    }
    if let Some(d) = try_parse_date(word) {
        return d;
    }
    if let Some(n) = try_parse_number(word) {
        return n;
    }
    RawToken::Word(word.to_string())
}

pub fn split_punctuation(word: &str) -> (String, String, String) {
    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return (String::new(), String::new(), String::new());
    }

    let is_punctuation = |c: char| -> bool {
        matches!(c, '.' | ',' | '!' | '?' | ':' | ';' | '"' | '\'' | '`' | 'ʼ' | '’' | '(' | ')' | '[' | ']' | '{' | '}' | '«' | '»' | '—' | '-')
    };

    let mut start = 0;
    while start < chars.len() && is_punctuation(chars[start]) {
        if chars[start] == '-' && start + 1 < chars.len() && chars[start + 1].is_ascii_digit() {
            break;
        }
        start += 1;
    }

    let mut end = chars.len();
    while end > start && is_punctuation(chars[end - 1]) {
        end -= 1;
    }

    let prefix: String = chars[..start].iter().collect();
    let core: String = chars[start..end].iter().collect();
    let suffix: String = chars[end..].iter().collect();

    (prefix, core, suffix)
}

pub fn tokenize_word(word: &str) -> Vec<RawToken> {
    let (prefix, core, suffix) = split_punctuation(word);
    let mut result = Vec::new();
    
    for c in prefix.chars() {
        result.push(RawToken::Punctuation(c));
    }
    
    if !core.is_empty() {
        result.push(parse_word_token(&core));
    }
    
    for c in suffix.chars() {
        result.push(RawToken::Punctuation(c));
    }
    
    result
}

pub fn tokenize(text: &str) -> Vec<RawToken> {
    let mut tokens = Vec::new();
    let mut current_word = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        if chars[i].is_whitespace() {
            if !current_word.is_empty() {
                tokens.extend(tokenize_word(&current_word));
                current_word.clear();
            }
            let mut ws = String::new();
            while i < chars.len() && chars[i].is_whitespace() {
                ws.push(chars[i]);
                i += 1;
            }
            tokens.push(RawToken::Whitespace(ws));
        } else {
            current_word.push(chars[i]);
            i += 1;
        }
    }
    if !current_word.is_empty() {
        tokens.extend(tokenize_word(&current_word));
    }
    tokens
}
