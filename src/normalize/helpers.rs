use num2words::Lang;

pub fn int_to_ua(unsigned: &str) -> String {
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

pub fn decimal_to_ua(int_part: &str, dec_part: &str, is_temp: bool) -> String {
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

pub fn num_to_ua(num_str: &str, is_temp: bool) -> String {
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
