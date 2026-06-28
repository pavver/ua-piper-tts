/// Нормалізація українського тексту для Piper TTS.
/// Архітектура: кожен крок — окрема функція для простого дебагу та надійності.

use num2words::{UkContext, Lang};
use crate::abbr::{
    is_abbreviation, expand_abbreviation, expand_abbr_contextual,
    get_context_noun, get_unit_form, find_unit, ABBR_MAP, is_preposition
};
use crate::stress::{apply_stress, with_stress_units, STRESS_DICT};



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

fn replace_times(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut result = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        let is_start_boundary = i == 0 || !chars[i - 1].is_ascii_digit();
        
        if is_start_boundary && i + 3 < chars.len() {
            let mut hours_len = 0;
            if chars[i].is_ascii_digit() {
                if chars[i + 1] == ':' {
                    hours_len = 1;
                } else if chars[i + 1].is_ascii_digit() && i + 2 < chars.len() && chars[i + 2] == ':' {
                    hours_len = 2;
                }
            }
            
            if hours_len > 0 {
                let mm_start = i + hours_len + 1;
                if mm_start + 1 < chars.len() 
                   && chars[mm_start].is_ascii_digit() 
                   && chars[mm_start + 1].is_ascii_digit()
                   && (mm_start + 2 == chars.len() || !chars[mm_start + 2].is_ascii_digit() && chars[mm_start + 2] != ':')
                {
                    let hours: String = chars[i..i+hours_len].iter().collect();
                    let minutes: String = chars[mm_start..mm_start+2].iter().collect();
                    
                    let minutes_val = minutes.parse::<i32>().unwrap_or(0);
                    let minutes_word = if minutes_val == 0 {
                        "".to_string()
                    } else if minutes.starts_with('0') {
                        format!("нуль {}", num_to_ua(&minutes[1..], false))
                    } else {
                        num_to_ua(&minutes, false)
                    };
                    
                    let mut time_str = hours;
                    if !minutes_word.is_empty() {
                        time_str.push(' ');
                        time_str.push_str(&minutes_word);
                    }
                    
                    result.push_str(&time_str);
                    i = mm_start + 2;
                    continue;
                } else if mm_start + 4 < chars.len()
                   && chars[mm_start].is_ascii_digit() 
                   && chars[mm_start + 1].is_ascii_digit()
                   && chars[mm_start + 2] == ':'
                   && chars[mm_start + 3].is_ascii_digit()
                   && chars[mm_start + 4].is_ascii_digit()
                   && (mm_start + 5 == chars.len() || !chars[mm_start + 5].is_ascii_digit())
                {
                    let hours: String = chars[i..i+hours_len].iter().collect();
                    let minutes: String = chars[mm_start..mm_start+2].iter().collect();
                    let seconds: String = chars[mm_start+3..mm_start+5].iter().collect();
                    
                    let minutes_val = minutes.parse::<i32>().unwrap_or(0);
                    let minutes_word = if minutes_val == 0 {
                        "".to_string()
                    } else if minutes.starts_with('0') {
                        format!("нуль {}", num_to_ua(&minutes[1..], false))
                    } else {
                        num_to_ua(&minutes, false)
                    };
                    
                    let seconds_val = seconds.parse::<i32>().unwrap_or(0);
                    let seconds_word = if seconds_val == 0 {
                        "".to_string()
                    } else if seconds.starts_with('0') {
                        format!("нуль {}", num_to_ua(&seconds[1..], false))
                    } else {
                        num_to_ua(&seconds, false)
                    };
                    
                    let mut time_str = hours;
                    if !minutes_word.is_empty() {
                        time_str.push(' ');
                        time_str.push_str(&minutes_word);
                    }
                    if !seconds_word.is_empty() {
                        time_str.push(' ');
                        time_str.push_str(&seconds_word);
                    }
                    
                    result.push_str(&time_str);
                    i = mm_start + 5;
                    continue;
                }
            }
        }
        
        result.push(chars[i]);
        i += 1;
    }
    result
}

fn replace_dates(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut result = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        let is_start_boundary = i == 0 || !chars[i - 1].is_ascii_digit();
        
        if is_start_boundary && i + 3 < chars.len() {
            let mut dd_len = 0;
            if chars[i].is_ascii_digit() {
                if chars[i+1] == '.' {
                    dd_len = 1;
                } else if chars[i+1].is_ascii_digit() && i + 2 < chars.len() && chars[i+2] == '.' {
                    dd_len = 2;
                }
            }
            
            if dd_len > 0 {
                let mm_start = i + dd_len + 1;
                if mm_start + 1 < chars.len() 
                   && chars[mm_start].is_ascii_digit() 
                   && chars[mm_start + 1].is_ascii_digit()
                {
                    let mut yyyy_len = 0;
                    if mm_start + 6 < chars.len() 
                       && chars[mm_start + 2] == '.' 
                       && chars[mm_start + 3].is_ascii_digit()
                       && chars[mm_start + 4].is_ascii_digit()
                       && chars[mm_start + 5].is_ascii_digit()
                       && chars[mm_start + 6].is_ascii_digit()
                       && (mm_start + 7 == chars.len() || !chars[mm_start + 7].is_ascii_digit())
                    {
                        yyyy_len = 4;
                    }
                    
                    let is_valid_date_end = yyyy_len == 4 || (mm_start + 2 == chars.len() || !chars[mm_start + 2].is_ascii_digit() && chars[mm_start + 2] != '.');
                    
                    if is_valid_date_end {
                        let dd_str: String = chars[i..i+dd_len].iter().collect();
                        let mm_str: String = chars[mm_start..mm_start+2].iter().collect();
                        let dd = dd_str.parse::<i32>().unwrap_or(0);
                        let mm = mm_str.parse::<i32>().unwrap_or(0);
                        
                        if dd >= 1 && dd <= 31 && mm >= 1 && mm <= 12 {
                            let month_name = match mm {
                                1 => "січня", 2 => "лютого", 3 => "березня", 4 => "квітня",
                                5 => "травня", 6 => "червня", 7 => "липня", 8 => "серпня",
                                9 => "вересня", 10 => "жовтня", 11 => "листопада", 12 => "грудня",
                                _ => "січня"
                             };
                             
                             let last_word = result.split_whitespace().last().unwrap_or("").to_lowercase();
                             let suffix = if matches!(last_word.as_str(), "до" | "з" | "від" | "після" | "для" | "коло" | "проти" | "біля") {
                                 "-го"
                             } else if matches!(last_word.as_str(), "на" | "о" | "об" | "при") {
                                 "-му"
                             } else {
                                 "-е"
                             };
                             
                             let mut date_expanded = format!("{}{}", dd_str, suffix);
                             date_expanded.push(' ');
                             date_expanded.push_str(month_name);
                             
                             if yyyy_len == 4 {
                                 let yyyy_str: String = chars[mm_start+3..mm_start+7].iter().collect();
                                 date_expanded.push_str(" ");
                                 date_expanded.push_str(&yyyy_str);
                                 date_expanded.push_str("-го року");
                             }
                             
                             result.push_str(&date_expanded);
                             i = mm_start + 2 + if yyyy_len == 4 { 5 } else { 0 };
                             continue;
                        }
                    }
                }
            }
        }
        
        result.push(chars[i]);
        i += 1;
    }
    result
}

fn preprocess_text(text: &str) -> String {
    let with_times = replace_times(text);
    let with_dates = replace_dates(&with_times);
    
    let mut result = String::new();
    let chars: Vec<char> = with_dates.chars().collect();
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

fn split_number_unit(word: &str) -> Option<(String, String)> {
    let chars: Vec<char> = word.chars().collect();
    let mut num_end = 0;
    for (i, c) in chars.iter().enumerate() {
        if c.is_ascii_digit() || (*c == '.' || *c == ',') { num_end = i + 1; } else { break; }
    }
    if num_end > 0 && num_end < chars.len() {
        let num: String = chars[..num_end].iter().collect();
        let unit: String = chars[num_end..].iter().collect();
        if unit.starts_with('-') {
            return None;
        }
        if num.parse::<f64>().is_ok() { return Some((num, unit)); }
    }
    None
}

/// Розділяє слово на префікс із розділових знаків, ядро та суфікс із розділових знаків.
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

fn is_number_word(word: &str) -> bool {
    matches!(word.to_lowercase().as_str(),
        "нуль" | "один" | "одна" | "одне" | "два" | "дві" | "три" | "чотири" | "п'ять" | "пʼять" |
        "шість" | "сім" | "вісім" | "дев'ять" | "девʼять" | "десять" | "одинадцять" | "дванадцять" |
        "тринадцять" | "чотирнадцять" | "п'ятнадцять" | "пʼятнадцять" | "шістнадцять" | "сімнадцять" |
        "вісімнадцять" | "дев'ятнадцять" | "девʼятнадцять" | "двадцять" | "тридцяти" | "тридцять" |
        "сорок" | "п'ятдесят" | "пʼятдесят" | "шістдесят" | "сімдесят" | "вісімдесят" | "дев'яносто" | "девʼяносто" |
        "сто" | "двісті" | "триста" | "чотириста" | "п'ятсот" | "пʼятсот" | "шістсот" | "сімсот" | "вісімсот" | "дев'ятсот" | "девʼятсот" |
        "тисяча" | "тисячі" | "тисяч" | "мільйон" | "мільйони" | "мільйонів" | "мільярд" | "мільярди" | "мільярдів"
    )
}

// ==================== Публічний API ====================

/// Повний пайплайн нормалізації тексту.
pub fn normalize_text(text: &str) -> String {
    let prepared = step0_fix_paragraphs(text);
    let preprocessed = preprocess_text(&prepared);

    let raw_words: Vec<&str> = preprocessed.split_whitespace().collect();
    let mut result = Vec::new();
    let mut i = 0;
    
    let split_words: Vec<(String, String, String)> = raw_words.iter()
        .map(|w| split_punctuation(w))
        .collect();
    
    while i < split_words.len() {
        let (prefix, core, suffix) = &split_words[i];
        
        if core.is_empty() {
            result.push(prefix.clone() + suffix);
            i += 1;
            continue;
        }

        let prev_core = if i > 0 { Some(split_words[i - 1].1.as_str()) } else { None };
        let next_core = if i + 1 < split_words.len() { Some(split_words[i + 1].1.as_str()) } else { None };

        let mut word_result = String::new();

        // Розділення числа та одиниці типу "220V"
        if let Some((num_str, unit)) = split_number_unit(core) {
            let mapped_unit = get_context_noun(&unit).unwrap_or("поверхів");
            let is_temp = unit.starts_with('°') || unit.to_lowercase().starts_with("град");
            let num_word = UkContext::analyze(prev_core, &num_str, Some(mapped_unit))
                .unwrap_or_else(|_| num_to_ua(&num_str, is_temp));
            word_result.push_str(&num_word);
            word_result.push(' ');
            
            if let Some((end, u)) = find_unit(&unit.to_lowercase()) {
                if end == unit.chars().count() {
                    let declined = get_unit_form(&num_word, u, prev_core);
                    word_result.push_str(&with_stress_units(&declined));
                }
            }
            result.push(format!("{}{}{}", prefix, word_result.trim(), suffix));
            i += 1;
            continue;
        }

        // Число з контекстом
        let mapped_next = if prev_core.map(|p| p.to_lowercase()).as_deref() == Some("о") 
            || prev_core.map(|p| p.to_lowercase()).as_deref() == Some("об") 
        {
            Some("годині")
        } else if let Some(n) = next_core {
            if let Some(mapped) = get_context_noun(n) {
                Some(mapped)
            } else if is_preposition(n) 
                || is_number_word(n)
                || n.chars().any(|c| c.is_ascii_digit())
                || n.chars().all(|c| !c.is_alphanumeric()) 
            {
                None
            } else {
                Some(n)
            }
        } else {
            None
        };

        if let Ok(num_word) = UkContext::analyze(prev_core, core, mapped_next) {
            let processed = num_word.replace('ʼ', "'");
            result.push(format!("{}{}{}", prefix, processed, suffix));
            i += 1;
            continue;
        }
        
        // Десяткові числа
        if core.contains('.') || core.contains(',') {
            let is_temp = next_core.map(|n| n.starts_with('°') || n.to_lowercase().starts_with("град")).unwrap_or(false);
            if core.parse::<f64>().is_ok() {
                let processed = num_to_ua(core, is_temp);
                result.push(format!("{}{}{}", prefix, processed, suffix));
                i += 1;
                continue;
            }
        }

        // Звичайне слово / одиниця виміру / абревіатура
        let lower = core.to_lowercase();
        let mut processed_as_unit = false;
        if let Some((end, u)) = find_unit(&lower) {
            if end == lower.chars().count() {
                let unit_form = if let Some(p) = prev_core {
                    if p.parse::<f64>().is_ok() {
                        let last_item = result.last().cloned().unwrap_or_default();
                        let last_word_clean: String = last_item.split_whitespace().last().unwrap_or("")
                            .chars().filter(|c| !matches!(*c, '.' | ',' | '!' | '?' | ':' | ';')).collect();
                        get_unit_form(&last_word_clean, u, if i > 1 { Some(split_words[i - 2].1.as_str()) } else { None })
                    } else {
                        u.to_string()
                    }
                } else {
                    u.to_string()
                };
                result.push(format!("{}{}{}", prefix, with_stress_units(&unit_form), suffix));
                processed_as_unit = true;
            }
        }

        if !processed_as_unit {
            let is_abbr = is_abbreviation(core) || ABBR_MAP.contains_key(lower.as_str());
            if is_abbr {
                let lower_word = core.to_lowercase();
                if ABBR_MAP.contains_key(lower_word.as_str()) {
                    let before_prev = if i > 1 { Some(split_words[i - 2].1.as_str()) } else { None };
                    let expanded = expand_abbr_contextual(core, prev_core, before_prev);
                    let stressed_parts: Vec<String> = expanded.split_whitespace()
                        .map(|w| apply_stress(w))
                        .collect();
                    result.push(format!("{}{}{}", prefix, stressed_parts.join(" "), suffix));
                } else if let Some(stressed) = STRESS_DICT.get(&lower_word) {
                    result.push(format!("{}{}{}", prefix, stressed.clone(), suffix));
                } else {
                    let expanded = expand_abbreviation(core);
                    let stressed_parts: Vec<String> = expanded.split_whitespace()
                        .map(|w| apply_stress(w))
                        .collect();
                    result.push(format!("{}{}{}", prefix, stressed_parts.join(" "), suffix));
                }
            } else {
                result.push(format!("{}{}{}", prefix, apply_stress(core), suffix));
            }
        }
        i += 1;
    }

    result.join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
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
        // "з 5 кг" -> "з п'яти кілограмів"
        let r1 = normalize_text("з 5 кг");
        println!("test_declension_of_units_contextual r1: {:?}", r1);
        assert!(has_text(&r1, "п'яти"));
        assert!(has_text(&r1, "кілограмів"));

        // "з 1 кг" -> "з одного кілограма"
        let r2 = normalize_text("з 1 кг");
        println!("test_declension_of_units_contextual r2: {:?}", r2);
        assert!(has_text(&r2, "одного"));
        assert!(has_text(&r2, "грама"));

        // "при 1 градусі" -> "при одному градусі"
        let r3 = normalize_text("при 1°");
        println!("test_declension_of_units_contextual r3: {:?}", r3);
        assert!(has_text(&r3, "одному"));
        assert!(has_text(&r3, "градусі"));

        // "температура котла 40°" -> "сорок градусів"
        let r4 = normalize_text("температура котла 40°");
        println!("test_declension_of_units_contextual r4: {:?}", r4);
        assert!(has_text(&r4, "сорок градусів"));

        // "температура котла 41°" -> "сорок один градус"
        let r5 = normalize_text("температура котла 41°");
        println!("test_declension_of_units_contextual r5: {:?}", r5);
        assert!(has_text(&r5, "сорок один градус"));

        // "ми витратили 2 з 5 кг" -> "два з п'яти кілограмів"
        let r6 = normalize_text("ми витратили 2 з 5 кг");
        println!("test_declension_of_units_contextual r6: {:?}", r6);
        assert!(has_text(&r6, "два з п'яти кілограмів"));
    }

    #[test]
    fn test_new_abbreviations_contextual() {
        // Тест для нових доданих абревіатур
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
