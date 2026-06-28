use crate::abbr::Gender;

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
