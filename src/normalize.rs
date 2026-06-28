/// Нормалізація українського тексту для Piper TTS.
/// Архітектура: Лексер (Токенізація) -> Контекстний парсер -> Генератор слів.

use num2words::{UkContext, Lang};
use crate::abbr::{
    is_abbreviation, expand_abbreviation, expand_abbr_contextual,
    get_context_noun, get_unit_form, find_unit, ABBR_MAP, is_preposition,
    Gender
};
use crate::stress::{apply_stress, with_stress_units, STRESS_DICT};

// ==================== Токени та внутрішні структури ====================

#[derive(Debug, Clone, PartialEq)]
pub enum NumberValue {
    Integer(i64),
    Decimal {
        int_part: i64,
        dec_part: u32,
        dec_places: usize,
    },
}

#[derive(Debug, Clone)]
pub enum RawToken {
    Word(String),
    Number {
        raw: String,
        is_negative: bool,
        int_part: String,
        dec_part: Option<String>,
        suffix: Option<String>,
    },
    Time {
        hours: String,
        minutes: String,
        seconds: Option<String>,
    },
    Date {
        day: String,
        month: String,
        year: Option<String>,
    },
    Punctuation(char),
    Whitespace(String),
}

#[derive(Debug, Clone)]
pub enum Token {
    Word {
        text: String,
        is_abbr: bool,
    },
    Number {
        raw: String,
        is_negative: bool,
        value: NumberValue,
        suffix: Option<String>,
        unit: Option<String>,
        context_noun: Option<String>,
        preposition: Option<String>,
        governed_by_decimal: bool,
        gender: Gender,
    },
    Time {
        hours: u8,
        minutes: u8,
        seconds: Option<u8>,
        preposition: Option<String>,
    },
    Date {
        day: u8,
        month: u8,
        year: Option<u16>,
        preposition: Option<String>,
    },
    Punctuation(char),
    Whitespace(String),
}

// ==================== КРОК 0: Попередня обробка тексту ====================

fn step0_fix_paragraphs(text: &str) -> String {
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
fn preprocess_text(text: &str) -> String {
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

// ==================== Допоміжні чисельні конвертори ====================

fn int_to_ua(unsigned: &str) -> String {
    if let Ok(n) = unsigned.parse::<f64>() {
        num2words::Num2Words::new(n)
            .lang(Lang::Ukrainian)
            .to_words()
            .unwrap_or(unsigned.to_string())
            .replace('ʼ', "'")
            .replace("один цілих", "один цілий")
            .replace("два цілих", "дві цілих")
    } else {
        unsigned.to_string()
    }
}

fn decimal_to_ua(int_part: &str, dec_part: &str, is_temp: bool) -> String {
    let int_val = int_part.parse::<i64>().unwrap_or(0);
    let mut int_words = if int_part.is_empty() || int_part == "0" {
        "нуль".to_string()
    } else {
        int_to_ua(int_part)
    };
    let dec_digits: Vec<_> = dec_part.chars().filter(|c| c.is_ascii_digit()).collect();
    let dec_clean: String = dec_digits.iter().collect();

    if is_temp && int_val > 9 && dec_digits.len() == 1 {
        let int_ends_with_one = int_words == "один" || (int_words.ends_with(" один") && !int_words.ends_with("одинадцять"));
        let int_ends_with_two = int_words == "два" || (int_words.ends_with(" два") && !int_words.ends_with("дванадцять"));
        if int_ends_with_one {
            if int_words == "один" {
                int_words = "одна".to_string();
            } else if int_words.ends_with(" один") {
                int_words = int_words[..int_words.len() - 4].to_string() + "одна";
            }
        } else if int_ends_with_two {
            if int_words == "два" {
                int_words = "дві".to_string();
            } else if int_words.ends_with(" два") {
                int_words = int_words[..int_words.len() - 3].to_string() + "дві";
            }
        }

        let digit_val = dec_digits.first().and_then(|c| c.to_digit(10)).unwrap_or(0);
        let digit_word = match digit_val {
            0 => "нуль", 1 => "один", 2 => "два", 3 => "три", 4 => "чотири",
            5 => "п'ять", 6 => "шість", 7 => "сім", 8 => "вісім", 9 => "дев'ять",
            _ => "нуль"
        };
        format!("{} і {}", int_words, digit_word)
    } else {
        let mut dec_words = if dec_clean.is_empty() { "нуль".to_string() }
        else if let Ok(n) = dec_clean.parse::<f64>() {
            num2words::Num2Words::new(n)
                .lang(Lang::Ukrainian)
                .to_words()
                .unwrap_or(dec_clean.clone())
                .replace('ʼ', "'")
        } else { dec_clean };

        let int_ends_with_one = int_words == "один" || (int_words.ends_with(" один") && !int_words.ends_with("одинадцять"));
        let int_ends_with_two = int_words == "два" || (int_words.ends_with(" два") && !int_words.ends_with("дванадцять"));
        if int_ends_with_one {
            if int_words == "один" {
                int_words = "одна".to_string();
            } else if int_words.ends_with(" один") {
                int_words = int_words[..int_words.len() - 4].to_string() + "одна";
            }
        } else if int_ends_with_two {
            if int_words == "два" {
                int_words = "дві".to_string();
            } else if int_words.ends_with(" два") {
                int_words = int_words[..int_words.len() - 3].to_string() + "дві";
            }
        }

        let int_suffix = if int_ends_with_one { "ціла" } else { "цілих" };

        let dec_ends_with_one = dec_words == "один" || (dec_words.ends_with(" один") && !dec_words.ends_with("одинадцять"));
        let dec_ends_with_two = dec_words == "два" || (dec_words.ends_with(" два") && !dec_words.ends_with("дванадцять"));
        if dec_ends_with_one {
            if dec_words == "один" {
                dec_words = "одна".to_string();
            } else if dec_words.ends_with(" один") {
                dec_words = dec_words[..dec_words.len() - 4].to_string() + "одна";
            }
        } else if dec_ends_with_two {
            if dec_words == "два" {
                dec_words = "дві".to_string();
            } else if dec_words.ends_with(" два") {
                dec_words = dec_words[..dec_words.len() - 3].to_string() + "дві";
            }
        }

        let dec_suffix = if dec_ends_with_one {
            match dec_digits.len() {
                1 => "десята",
                2 => "сота",
                3 => "тисячна",
                4 => "десятитисячна",
                _ => "сота",
            }
        } else {
            match dec_digits.len() {
                1 => "десятих",
                2 => "сотих",
                3 => "тисячних",
                4 => "десятитисячних",
                _ => "сотих",
            }
        };

        format!("{} {} {} {}", int_words, int_suffix, dec_words, dec_suffix)
    }
}

fn num_to_ua(num_str: &str, is_temp: bool) -> String {
    let (sign, unsigned) = if num_str.starts_with('-') {
        ("мінус ", &num_str[1..])
    } else { ("", num_str) };

    if let Some(dot) = unsigned.find(|c| c == '.' || c == ',') {
        let int_part = &unsigned[..dot];
        let dec_part = &unsigned[dot + 1..];
        format!("{}{}", sign, decimal_to_ua(int_part, dec_part, is_temp))
    } else if let Ok(n) = unsigned.parse::<f64>() {
        format!("{}{}", sign,
            num2words::Num2Words::new(n)
                .lang(Lang::Ukrainian)
                .to_words()
                .unwrap_or(unsigned.to_string())
                .replace('ʼ', "'"))
    } else { format!("{}{}", sign, unsigned) }
}

// ==================== КРОК 1: Лексер (Токенізація) ====================

fn try_parse_time(word: &str) -> Option<RawToken> {
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

fn try_parse_date(word: &str) -> Option<RawToken> {
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

fn try_parse_number(word: &str) -> Option<RawToken> {
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

fn parse_word_token(word: &str) -> RawToken {
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

fn split_punctuation(word: &str) -> (String, String, String) {
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

fn tokenize_word(word: &str) -> Vec<RawToken> {
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

fn tokenize(text: &str) -> Vec<RawToken> {
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

// ==================== КРОК 2: Контекстний парсер ====================

fn parse_context(raw_tokens: Vec<RawToken>) -> Vec<Token> {
    let mut temp_tokens = Vec::new();
    for rt in raw_tokens {
        match rt {
            RawToken::Whitespace(ws) => temp_tokens.push(Token::Whitespace(ws)),
            RawToken::Punctuation(c) => temp_tokens.push(Token::Punctuation(c)),
            RawToken::Word(w) => {
                let lower = w.to_lowercase();
                let is_abbr = is_abbreviation(&w) || ABBR_MAP.contains_key(lower.as_str());
                temp_tokens.push(Token::Word { text: w, is_abbr });
            }
            RawToken::Time { hours, minutes, seconds, .. } => {
                let h_val = hours.parse::<u8>().unwrap_or(0);
                let m_val = minutes.parse::<u8>().unwrap_or(0);
                let s_val = seconds.map(|s| s.parse::<u8>().unwrap_or(0));
                temp_tokens.push(Token::Time {
                    hours: h_val,
                    minutes: m_val,
                    seconds: s_val,
                    preposition: None,
                });
            }
            RawToken::Date { day, month, year, .. } => {
                let d_val = day.parse::<u8>().unwrap_or(0);
                let m_val = month.parse::<u8>().unwrap_or(0);
                let y_val = year.map(|y| y.parse::<u16>().unwrap_or(0));
                temp_tokens.push(Token::Date {
                    day: d_val,
                    month: m_val,
                    year: y_val,
                    preposition: None,
                });
            }
            RawToken::Number { raw, is_negative, int_part, dec_part, suffix } => {
                let val = match &dec_part {
                    None => NumberValue::Integer(int_part.parse::<i64>().unwrap_or(0)),
                    Some(dp) => NumberValue::Decimal {
                        int_part: int_part.parse::<i64>().unwrap_or(0),
                        dec_part: dp.parse::<u32>().unwrap_or(0),
                        dec_places: dp.len(),
                    },
                };
                temp_tokens.push(Token::Number {
                    raw,
                    is_negative,
                    value: val,
                    suffix,
                    unit: None,
                    context_noun: None,
                    preposition: None,
                    governed_by_decimal: false,
                    gender: Gender::Masculine,
                });
            }
        }
    }
    
    let mut tokens = Vec::new();
    let mut idx = 0;
    while idx < temp_tokens.len() {
        let mut current = temp_tokens[idx].clone();
        
        match &mut current {
            Token::Time { preposition, .. } => {
                if let Some(prep) = find_preceding_word(&temp_tokens, idx) {
                    let p_lower = prep.to_lowercase();
                    if p_lower == "о" || p_lower == "об" {
                        *preposition = Some(prep);
                    }
                }
            }
            Token::Date { preposition, .. } => {
                if let Some(prep) = find_preceding_word(&temp_tokens, idx) {
                    *preposition = Some(prep);
                }
            }
            Token::Number { preposition, context_noun, gender, governed_by_decimal, unit, suffix, value, .. } => {
                if let Some(prep) = find_preceding_word(&temp_tokens, idx) {
                    if is_preposition(&prep) {
                        *preposition = Some(prep);
                    }
                }
                
                let mut resolved_unit = None;
                let mut resolved_ctx_noun = None;
                
                if let Some(suf) = suffix {
                    let lower_suf = suf.to_lowercase();
                    if let Some((_, u)) = find_unit(&lower_suf) {
                        resolved_unit = Some(u.to_string());
                        resolved_ctx_noun = get_context_noun(&lower_suf).map(|s| s.to_string());
                    }
                }
                
                if resolved_unit.is_none() {
                    if let Some((next_word_idx, next_word)) = find_following_word(&temp_tokens, idx) {
                        let lower_next = next_word.to_lowercase();
                        if let Some((end, u)) = find_unit(&lower_next) {
                            if end == lower_next.chars().count() {
                                resolved_unit = Some(u.to_string());
                                resolved_ctx_noun = get_context_noun(&lower_next).map(|s| s.to_string());
                                temp_tokens[next_word_idx] = Token::Whitespace(String::new());
                            }
                        }
                    }
                }
                
                if resolved_ctx_noun.is_none() {
                    if let Some((_, next_word)) = find_following_word(&temp_tokens, idx) {
                        let lower_next = next_word.to_lowercase();
                        if let Some(mapped) = get_context_noun(&lower_next) {
                            resolved_ctx_noun = Some(mapped.to_string());
                        } else if !is_preposition(&next_word) 
                            && !next_word.chars().any(|c| c.is_ascii_digit())
                            && next_word.chars().all(|c| c.is_alphanumeric()) 
                        {
                            resolved_ctx_noun = Some(next_word.clone());
                        }
                    }
                }
                
                *unit = resolved_unit;
                *context_noun = resolved_ctx_noun;
                
                let is_simplified_temp = match value {
                    NumberValue::Decimal { int_part, dec_places, .. } => {
                        let is_temp_unit = unit.as_ref().map(|u| u.starts_with("градус") || u.starts_with("кельвін")).unwrap_or(false);
                        is_temp_unit && *int_part > 9 && *dec_places == 1
                    }
                    _ => false,
                };
                
                match value {
                    NumberValue::Decimal { .. } => {
                        *governed_by_decimal = !is_simplified_temp;
                        *gender = if is_simplified_temp { Gender::Masculine } else { Gender::Feminine };
                    }
                    NumberValue::Integer(_) => {
                        *governed_by_decimal = false;
                        if let Some(suf) = suffix {
                            if suf.ends_with('а') || suf.ends_with('я') {
                                *gender = Gender::Feminine;
                            } else if suf.ends_with('е') || suf.ends_with('є') {
                                *gender = Gender::Neuter;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        
        tokens.push(current);
        idx += 1;
    }
    
    tokens.into_iter().filter(|t| {
        if let Token::Whitespace(ws) = t {
            !ws.is_empty()
        } else {
            true
        }
    }).collect()
}

fn find_preceding_word(tokens: &[Token], start_idx: usize) -> Option<String> {
    let mut i = start_idx;
    while i > 0 {
        i -= 1;
        match &tokens[i] {
            Token::Word { text, .. } => return Some(text.clone()),
            Token::Punctuation(_) | Token::Whitespace(_) => {}
            _ => break,
        }
    }
    None
}

fn find_following_word(tokens: &[Token], start_idx: usize) -> Option<(usize, String)> {
    let mut i = start_idx + 1;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Word { text, .. } => return Some((i, text.clone())),
            Token::Punctuation(_) | Token::Whitespace(_) => {}
            _ => break,
        }
        i += 1;
    }
    None
}

// ==================== КРОК 3: Генератор тексту ====================

fn generate_text(tokens: Vec<Token>) -> String {
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < tokens.len() {
        match &tokens[i] {
            Token::Whitespace(ws) => {
                result.push(ws.clone());
            }
            Token::Punctuation(c) => {
                result.push(c.to_string());
            }
            Token::Word { text, is_abbr } => {
                let lower = text.to_lowercase();
                if let Some((end, u)) = find_unit(&lower) {
                    if end == lower.chars().count() {
                        result.push(apply_stress(u));
                        i += 1;
                        continue;
                    }
                }
                if *is_abbr {
                    if ABBR_MAP.contains_key(lower.as_str()) {
                        let (prev_core, before_prev) = find_preceding_words(&tokens, i);
                        let expanded = expand_abbr_contextual(text, prev_core.as_deref(), before_prev.as_deref());
                        let stressed_parts: Vec<String> = expanded.split_whitespace()
                            .map(|w| apply_stress(w))
                            .collect();
                        result.push(stressed_parts.join(" "));
                    } else if let Some(stressed) = STRESS_DICT.get(&lower) {
                        result.push(stressed.clone());
                    } else {
                        let expanded = expand_abbreviation(text);
                        let stressed_parts: Vec<String> = expanded.split_whitespace()
                            .map(|w| apply_stress(w))
                            .collect();
                        result.push(stressed_parts.join(" "));
                    }
                } else {
                    result.push(apply_stress(text));
                }
            }
            Token::Time { hours, minutes, seconds, preposition } => {
                let mut time_parts = Vec::new();
                
                let prep_str = preposition.as_deref();
                let hours_word = if let Some(p) = prep_str {
                    let p_lower = p.to_lowercase();
                    if p_lower == "о" || p_lower == "об" {
                        UkContext::analyze(Some(p), &hours.to_string(), Some("годині"))
                            .unwrap_or_else(|_| num_to_ua(&hours.to_string(), false))
                    } else {
                        num_to_ua(&hours.to_string(), false)
                    }
                } else {
                    num_to_ua(&hours.to_string(), false)
                };
                time_parts.push(hours_word.replace('ʼ', "'"));
                
                let m_val = *minutes;
                if m_val > 0 {
                    let min_word = if m_val < 10 {
                        format!("нуль {}", num_to_ua(&m_val.to_string(), false))
                    } else {
                        num_to_ua(&m_val.to_string(), false)
                    };
                    time_parts.push(min_word);
                }
                
                if let Some(s_val) = seconds {
                    if *s_val > 0 {
                        let sec_word = if *s_val < 10 {
                            format!("нуль {}", num_to_ua(&s_val.to_string(), false))
                        } else {
                            num_to_ua(&s_val.to_string(), false)
                        };
                        time_parts.push(sec_word);
                    }
                }
                
                result.push(time_parts.join(" "));
            }
            Token::Date { day, month, year, preposition } => {
                let mut date_parts = Vec::new();
                
                let month_name = match month {
                    1 => "січня", 2 => "лютого", 3 => "березня", 4 => "квітня",
                    5 => "травня", 6 => "червня", 7 => "липня", 8 => "серпня",
                    9 => "вересня", 10 => "жовтня", 11 => "листопада", 12 => "грудня",
                    _ => "січня"
                };
                
                let suffix = if let Some(p) = preposition {
                    let p_lower = p.to_lowercase();
                    if matches!(p_lower.as_str(), "до" | "з" | "від" | "після" | "для" | "коло" | "проти" | "біля") {
                        "-го"
                    } else if matches!(p_lower.as_str(), "на" | "о" | "об" | "при") {
                        "-му"
                    } else {
                        "-е"
                    }
                } else {
                    "-е"
                };
                
                let day_query = format!("{}{}", day, suffix);
                let day_word = UkContext::analyze(preposition.as_deref(), &day_query, Some(month_name))
                    .unwrap_or_else(|_| num_to_ua(&day.to_string(), false));
                date_parts.push(day_word.replace('ʼ', "'"));
                date_parts.push(month_name.to_string());
                
                if let Some(y) = year {
                    let year_query = format!("{}-го", y);
                    let year_word = UkContext::analyze(None, &year_query, Some("року"))
                        .unwrap_or_else(|_| num_to_ua(&y.to_string(), false));
                    date_parts.push(format!("{} року", year_word.replace('ʼ', "'")));
                }
                
                result.push(date_parts.join(" "));
            }
            Token::Number { is_negative, value, suffix, unit, context_noun, preposition, governed_by_decimal, gender: _gender, .. } => {
                let sign = if *is_negative { "мінус " } else { "" };
                let mut num_word = match value {
                    NumberValue::Integer(val) => {
                        let is_ordinal = suffix.as_ref().map(|s| s.starts_with('-')).unwrap_or(false);
                        if is_ordinal {
                            let query = format!("{}{}", val, suffix.as_ref().unwrap());
                            UkContext::analyze(None, &query, None)
                                .unwrap_or_else(|_| num_to_ua(&val.to_string(), false))
                        } else {
                            if let Some(ctx) = context_noun {
                                UkContext::analyze(preposition.as_deref(), &val.to_string(), Some(ctx))
                                    .unwrap_or_else(|_| num_to_ua(&val.to_string(), false))
                            } else {
                                UkContext::analyze(preposition.as_deref(), &val.to_string(), None)
                                    .unwrap_or_else(|_| num_to_ua(&val.to_string(), false))
                            }
                        }
                    }
                    NumberValue::Decimal { int_part, dec_part, .. } => {
                        let is_temp = unit.as_ref().map(|u| u.starts_with("градус") || u.starts_with("кельвін")).unwrap_or(false);
                        decimal_to_ua(&int_part.to_string(), &dec_part.to_string(), is_temp)
                    }
                };
                
                num_word = num_word.replace('ʼ', "'");
                let mut output = format!("{}{}", sign, num_word);
                
                if let Some(u) = unit {
                    let prev_word = find_preceding_words(&tokens, i).0;
                    let unit_form = if *governed_by_decimal {
                        get_unit_form("десятих", u, prev_word.as_deref())
                    } else {
                        get_unit_form(&output, u, prev_word.as_deref())
                    };
                    output.push(' ');
                    output.push_str(&with_stress_units(&unit_form));
                }
                
                result.push(output);
            }
        }
        i += 1;
    }
    
    result.join("")
}

fn find_preceding_words(tokens: &[Token], start_idx: usize) -> (Option<String>, Option<String>) {
    let mut first = None;
    let mut second = None;
    let mut i = start_idx;
    while i > 0 {
        i -= 1;
        match &tokens[i] {
            Token::Word { text, .. } => {
                if first.is_none() {
                    first = Some(text.clone());
                } else {
                    second = Some(text.clone());
                    break;
                }
            }
            Token::Number { raw, .. } => {
                if first.is_none() {
                    first = Some(raw.clone());
                } else {
                    second = Some(raw.clone());
                    break;
                }
            }
            Token::Punctuation(_) | Token::Whitespace(_) => {}
            _ => break,
        }
    }
    (first, second)
}

// ==================== Публічний API ====================

/// Повний пайплайн нормалізації тексту.
pub fn normalize_text(text: &str) -> String {
    let prepared = step0_fix_paragraphs(text);
    let preprocessed = preprocess_text(&prepared);
    let raw_tokens = tokenize(&preprocessed);
    let resolved_tokens = parse_context(raw_tokens);
    let generated = generate_text(resolved_tokens);
    
    generated.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ==================== Тести ====================

#[cfg(test)]
mod tests {
    use super::*;

    fn has_text(result: &str, expected: &str) -> bool {
        let stripped: String = result.chars().filter(|c| *c != '\u{0301}').collect();
        stripped.contains(expected)
    }

    #[test]
    fn test_step0_paragraphs() {
        let r = step0_fix_paragraphs("привіт світе\n\nяк справи");
        assert!(has_text(&r, "привіт світе. як справи"));

        let r = step0_fix_paragraphs("привіт світе.\n\nяк справи");
        assert!(has_text(&r, "привіт світе. як справи"));

        let r = step0_fix_paragraphs("як справи?\n\nдобре");
        assert!(has_text(&r, "як справи? добре"));
    }

    #[test]
    fn test_abbreviations() {
        let r = normalize_text("заступник керівника бучанського РТЦК андрій євтушенко");
        assert!(has_text(&r, "районного територіального центру комплектування"));
    }

    #[test]
    fn test_decimals() {
        let r = normalize_text("36.6");
        assert!(has_text(&r, "цілих"));
    }

    #[test]
    fn test_electric_units() {
        let r = normalize_text("220V");
        assert!(has_text(&r, "двісті"));
        assert!(has_text(&r, "вольт"));
    }

    #[test]
    fn test_ha_states() {
        assert!(has_text(&normalize_text("on"), "увімкнено"));
        assert!(has_text(&normalize_text("off"), "вимкнено"));
    }

    #[test]
    fn test_range_with_unit() {
        let r = normalize_text("Вночі буде 0-3°");
        assert!(has_text(&r, "від"));
        assert!(has_text(&r, "до"));
    }

    #[test]
    fn test_subtraction_without_unit() {
        let r = normalize_text("4-3");
        println!("test_subtraction_without_unit RESULT: {:?}", r);
        assert!(has_text(&r, "мінус"));
    }

    #[test]
    fn test_abbreviation_agreement_tts() {
        let r1 = normalize_text("1 СБУ");
        println!("test_abbreviation_agreement_tts r1: {:?}", r1);
        assert!(has_text(&r1, "одна служба безпеки україни"));

        let r2 = normalize_text("2 СБУ");
        println!("test_abbreviation_agreement_tts r2: {:?}", r2);
        assert!(has_text(&r2, "дві служби безпеки україни"));

        let r3 = normalize_text("1 ТЦК");
        println!("test_abbreviation_agreement_tts r3: {:?}", r3);
        assert!(has_text(&r3, "один територіальний центр комплектування"));

        let r4 = normalize_text("1 МВС");
        println!("test_abbreviation_agreement_tts r4: {:?}", r4);
        assert!(has_text(&r4, "одне міністерство внутрішніх справ"));
    }

    #[test]
    fn test_declension_of_units_contextual() {
        let r1 = normalize_text("з 5 кг");
        println!("test_declension_of_units_contextual r1: {:?}", r1);
        assert!(has_text(&r1, "п'яти"));
        assert!(has_text(&r1, "кілограмів"));

        let r2 = normalize_text("з 1 кг");
        println!("test_declension_of_units_contextual r2: {:?}", r2);
        assert!(has_text(&r2, "одного"));
        assert!(has_text(&r2, "грама"));

        let r3 = normalize_text("при 1°");
        println!("test_declension_of_units_contextual r3: {:?}", r3);
        assert!(has_text(&r3, "одному"));
        assert!(has_text(&r3, "градусі"));

        let r4 = normalize_text("температура котла 40°");
        println!("test_declension_of_units_contextual r4: {:?}", r4);
        assert!(has_text(&r4, "сорок градусів"));

        let r5 = normalize_text("температура котла 41°");
        println!("test_declension_of_units_contextual r5: {:?}", r5);
        assert!(has_text(&r5, "сорок один градус"));

        let r6 = normalize_text("ми витратили 2 з 5 кг");
        println!("test_declension_of_units_contextual r6: {:?}", r6);
        assert!(has_text(&r6, "два з п'яти кілограмів"));
    }

    #[test]
    fn test_new_abbreviations_contextual() {
        let r1 = normalize_text("передано для ЗСУ");
        println!("test_new_abbreviations_contextual r1: {:?}", r1);
        assert!(has_text(&r1, "передано для збройних сил україни"));

        let r2 = normalize_text("повідомлення від ДСНС");
        println!("test_new_abbreviations_contextual r2: {:?}", r2);
        assert!(has_text(&r2, "повідомлення від державної служби з надзвичайних ситуацій"));

        let r3 = normalize_text("рішення ВРУ");
        println!("test_new_abbreviations_contextual r3: {:?}", r3);
        assert!(has_text(&r3, "рішення верховної ради україни"));

        let r4 = normalize_text("заява від НАБУ");
        println!("test_new_abbreviations_contextual r4: {:?}", r4);
        assert!(has_text(&r4, "заява від національного антикорупційного бюро україни"));

        let r5 = normalize_text("3 БПЛА");
        println!("test_new_abbreviations_contextual r5: {:?}", r5);
        assert!(has_text(&r5, "три безпілотні літальні апарати"));

        let r6 = normalize_text("заява від ФОП");
        println!("test_new_abbreviations_contextual r6: {:?}", r6);
        assert!(has_text(&r6, "заява від фізичної особи підприємця"));

        let r7 = normalize_text("допомога від ОП");
        println!("test_new_abbreviations_contextual r7: {:?}", r7);
        assert!(has_text(&r7, "допомога від офісу президента"));

        let r8 = normalize_text("рішення КМУ");
        println!("test_new_abbreviations_contextual r8: {:?}", r8);
        assert!(has_text(&r8, "рішення кабінету міністрів україни"));

        let r9 = normalize_text("повідомлення від ОВА");
        println!("test_new_abbreviations_contextual r9: {:?}", r9);
        assert!(has_text(&r9, "повідомлення від обласної військової адміністрації"));

        let r10 = normalize_text("звіт від НПУ");
        println!("test_new_abbreviations_contextual r10: {:?}", r10);
        assert!(has_text(&r10, "звіт від національної поліції україни"));

        let r11 = normalize_text("для НГУ");
        println!("test_new_abbreviations_contextual r11: {:?}", r11);
        assert!(has_text(&r11, "для національної гвардії україни"));

        let r12 = normalize_text("рішення ВВК");
        println!("test_new_abbreviations_contextual r12: {:?}", r12);
        assert!(has_text(&r12, "рішення військово-лікарської комісії"));

        let r13 = normalize_text("повідомлення від МЗС");
        println!("test_new_abbreviations_contextual r13: {:?}", r13);
        assert!(has_text(&r13, "повідомлення від міністерства закордонних справ"));

        let r14 = normalize_text("заява від НАТО");
        println!("test_new_abbreviations_contextual r14: {:?}", r14);
        assert!(has_text(&r14, "заява від організації північноатлантичного договору"));

        let r15 = normalize_text("сплата ПДВ");
        println!("test_new_abbreviations_contextual r15: {:?}", r15);
        assert!(has_text(&r15, "сплата податку на додану вартість"));

        let r16 = normalize_text("оплата ЄСВ");
        println!("test_new_abbreviations_contextual r16: {:?}", r16);
        assert!(has_text(&r16, "оплата єдиного соціального внеску"));

        let r17 = normalize_text("комітет МКЧХ");
        println!("test_new_abbreviations_contextual r17: {:?}", r17);
        assert!(has_text(&r17, "комітет міжнародного комітету червоного хреста"));

        let r18 = normalize_text("направлення на МРТ");
        println!("test_new_abbreviations_contextual r18: {:?}", r18);
        assert!(has_text(&r18, "направлення на магнітно-резонансну томографію"));

        let r19 = normalize_text("результати ЗНО");
        println!("test_new_abbreviations_contextual r19: {:?}", r19);
        assert!(has_text(&r19, "результати зовнішнього незалежного оцінювання"));

        let r20 = normalize_text("надіслати СМС");
        println!("test_new_abbreviations_contextual r20: {:?}", r20);
        assert!(has_text(&r20, "надіслати коротке текстове повідомлення"));
    }

    #[test]
    fn test_smart_home_scenarios() {
        // Temperature simplified decimals
        let temp1 = normalize_text("температура 36.6°C");
        println!("test_smart_home_scenarios temp1: {:?}", temp1);
        assert!(has_text(&temp1, "тридцять шість і шість градусів цельсія"));

        let temp2 = normalize_text("температура 9.6°C");
        println!("test_smart_home_scenarios temp2: {:?}", temp2);
        assert!(has_text(&temp2, "дев'ять цілих шість десятих градуса цельсія"));

        let temp3 = normalize_text("температура котла 10.5°");
        println!("test_smart_home_scenarios temp3: {:?}", temp3);
        assert!(has_text(&temp3, "десять і п'ять градусів"));

        // Time format
        let time1 = normalize_text("о 12:30");
        println!("test_smart_home_scenarios time1: {:?}", time1);
        assert!(has_text(&time1, "о дванадцятій тридцять"));

        let time2 = normalize_text("зараз 08:05");
        println!("test_smart_home_scenarios time2: {:?}", time2);
        assert!(has_text(&time2, "зараз вісім нуль п'ять"));

        let time3 = normalize_text("час 12:30:15");
        println!("test_smart_home_scenarios time3: {:?}", time3);
        assert!(has_text(&time3, "час дванадцять тридцять п'ятнадцять"));

        // Decimal quantity agreement (Genitive Singular)
        let dec1 = normalize_text("1.1 кг");
        println!("test_smart_home_scenarios dec1: {:?}", dec1);
        assert!(has_text(&dec1, "одна ціла одна десята кілограма"));

        let dec2 = normalize_text("2.5 л");
        println!("test_smart_home_scenarios dec2: {:?}", dec2);
        assert!(has_text(&dec2, "дві цілих п'ять десятих літра"));

        // HA new states
        let state1 = normalize_text("unavailable");
        println!("test_smart_home_scenarios state1: {:?}", state1);
        assert!(has_text(&state1, "недоступно"));

        let state2 = normalize_text("offline");
        println!("test_smart_home_scenarios state2: {:?}", state2);
        assert!(has_text(&state2, "поза мережею"));

        let state3 = normalize_text("playing");
        println!("test_smart_home_scenarios state3: {:?}", state3);
        assert!(has_text(&state3, "відтворюється"));

        let state4 = normalize_text("home");
        println!("test_smart_home_scenarios state4: {:?}", state4);
        assert!(has_text(&state4, "вдома"));

        // Date formats
        let date1 = normalize_text("сьогодні 28.06");
        println!("test_smart_home_scenarios date1: {:?}", date1);
        assert!(has_text(&date1, "сьогодні двадцять восьме червня"));

        let date2 = normalize_text("до 28.06");
        println!("test_smart_home_scenarios date2: {:?}", date2);
        assert!(has_text(&date2, "до двадцять восьмого червня"));

        let date3 = normalize_text("дата 28.06.2026");
        println!("test_smart_home_scenarios date3: {:?}", date3);
        assert!(has_text(&date3, "дата двадцять восьме червня дві тисячі двадцять шостого року"));
    }
}
