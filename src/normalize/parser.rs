use super::types::{RawToken, Token, NumberValue};
use crate::abbr::{
    is_abbreviation, get_context_noun, find_unit, is_preposition, ABBR_MAP, Gender
};

pub fn parse_context(raw_tokens: Vec<RawToken>) -> Vec<Token> {
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
            RawToken::Time { hours, minutes, seconds } => {
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
            RawToken::Date { day, month, year } => {
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

pub fn find_preceding_word(tokens: &[Token], start_idx: usize) -> Option<String> {
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

pub fn find_following_word(tokens: &[Token], start_idx: usize) -> Option<(usize, String)> {
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
