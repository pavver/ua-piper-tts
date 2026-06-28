use super::types::{Token, NumberValue};
use super::helpers::{num_to_ua, decimal_to_ua};
use num2words::UkContext;
use crate::abbr::{
    expand_abbreviation, expand_abbr_contextual, get_unit_form, find_unit, ABBR_MAP
};
use crate::stress::{apply_stress, with_stress_units, STRESS_DICT};

pub fn generate_text(tokens: Vec<Token>) -> String {
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

pub fn find_preceding_words(tokens: &[Token], start_idx: usize) -> (Option<String>, Option<String>) {
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
