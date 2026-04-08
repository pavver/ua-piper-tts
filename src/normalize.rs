/// Нормалізація українського тексту для Piper TTS.
/// Архітектура: кожен крок — окрема функція для простого дебагу.
///
/// Пайплайн:
///   Крок 1: Парсинг тексту → токени (числа, одиниці, слова, символи)
///   Крок 2: Конвертація чисел → українські слова
///   Крок 3: Заміна одиниць виміру → повні назви з відмінюванням
///   Крок 4: Додавання наголосів через словник lang-uk
///   Крок 5: Фінальне очищення

use num2words::Lang;
use std::collections::HashMap;
use std::sync::LazyLock;

// ==================== Словник наголосів (lang-uk + custom) ====================

/// Словник наголосів української мови.
/// Порядок завантаження:
///   1. custom_stress_dict.txt (вищий пріоритет — перевизначає основний)
///   2. ua_stress_dict.txt (основний словник lang-uk)
///
/// Формат: кожен рядок — слово з наголосом (U+0301 після голосної).
/// Ключ = слово БЕЗ наголосів (lowercase), значення = слово З наголосами.
static STRESS_DICT: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let mut dict = HashMap::new();

    // Спочатку основний словник
    let main_path = "data/ua_stress_dict.txt";
    if let Ok(content) = std::fs::read_to_string(main_path) {
        for line in content.lines() {
            let stressed = line.trim();
            if stressed.is_empty() { continue; }
            
            // ВАЖЛИВО: спочатку lowercase, потім видаляємо наголоси
            // Якщо навпаки — U+0301 може "з'їстися" при lowercase
            let lower_first = stressed.to_lowercase();
            let unstressed: String = lower_first.chars()
                .filter(|c| *c != '\u{0301}')
                .collect();
            if lower_first != unstressed {
                dict.insert(unstressed, lower_first);
            }
        }
        eprintln!("[stress_dict] Основний: {} слів", dict.len());
    } else {
        eprintln!("[stress_dict] Попередження: не знайдено {}", main_path);
    }

    // Потім custom — перевизначає основний
    let custom_path = "data/custom_stress_dict.txt";
    let mut custom_count = 0;
    if let Ok(content) = std::fs::read_to_string(custom_path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') { continue; }
            
            // Підтримка формату "ключ=значення" для винятків
            let (key, stressed) = if let Some(eq_pos) = trimmed.find('=') {
                // Формат: сша=се ше́ а
                let k = trimmed[..eq_pos].trim().to_lowercase();
                let v = trimmed[eq_pos+1..].trim().to_lowercase();
                (k, v)
            } else {
                // Звичайний формат: сло́во
                let lower_first = trimmed.to_lowercase();
                let unstressed: String = lower_first.chars()
                    .filter(|c| *c != '\u{0301}')
                    .collect();
                (unstressed, lower_first)
            };
            
            if !key.is_empty() {
                dict.insert(key, stressed);
                custom_count += 1;
            }
        }
        eprintln!("[stress_dict] Custom: {} слів (пріоритет вище)", custom_count);
    } else {
        eprintln!("[stress_dict] Попередження: не знайдено {} (не обов'язково)", custom_path);
    }

    eprintln!("[stress_dict] Загалом: {} слів", dict.len());
    dict
});

fn apply_stress(word: &str) -> String {
    // Абревіатури вже розширені в кроці 0, тут тільки шукаємо в словнику
    let lower = word.to_lowercase();
    if let Some(stressed) = STRESS_DICT.get(&lower) {
        return stressed.clone();
    }
    word.to_string()
}

/// Перевіряє чи слово — абревіатура (тільки слова з ВЕЛИКИМИ літерами)
fn is_abbreviation(word: &str) -> bool {
    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() || chars.len() == 1 {
        return false;
    }
    // Абревіатура = ВСІ літери великі
    let all_uppercase = chars.iter().all(|c| c.is_uppercase());
    // І довжина 2-5 символів
    all_uppercase && chars.len() <= 5
}

fn is_vowel(c: char) -> bool {
    matches!(c.to_lowercase().next(),
        Some('а') | Some('е') | Some('и') | Some('і') | Some('у') |
        Some('о') | Some('ю') | Some('я') | Some('ї') | Some('є') |
        Some('a') | Some('e') | Some('i') | Some('o') | Some('u') | Some('y')
    )
}

/// Розширює абревіатуру в назви літер
fn expand_abbreviation(word: &str) -> String {
    let mut parts = Vec::new();
    for c in word.chars() {
        let lower_c = c.to_lowercase().next().unwrap_or(c);
        let letter_name = match lower_c {
            // Українські літери
            'а' => "а", 'б' => "бе", 'в' => "ве", 'г' => "ге", 'ґ' => "ґе",
            'д' => "де", 'е' => "е", 'є' => "є", 'ж' => "же", 'з' => "зе",
            'и' => "и", 'і' => "і", 'ї' => "ї", 'й' => "йот", 'к' => "ка",
            'л' => "ел", 'м' => "ем", 'н' => "ен", 'о' => "о", 'п' => "пе",
            'р' => "ер", 'с' => "ес", 'т' => "те", 'у' => "у", 'ф' => "еф",
            'х' => "ха", 'ц' => "це", 'ч' => "че", 'ш' => "ша", 'щ' => "ща",
            'ь' => "м'який знак", 'ю' => "ю", 'я' => "я", 'ъ' => "твердий знак",
            // Латинські літери
            'a' => "ей", 'b' => "бі", 'c' => "сі", 'd' => "ді", 'e' => "і",
            'f' => "еф", 'g' => "джі", 'h' => "ейч", 'i' => "ай", 'j' => "джей",
            'k' => "кей", 'l' => "ел", 'm' => "ем", 'n' => "ен", 'o' => "оу",
            'p' => "пі", 'q' => "к'ю", 'r' => "ар", 's' => "ес", 't' => "ті",
            'u' => "ю", 'v' => "ві", 'w' => "дабл-ю", 'x' => "екс", 'y' => "вай",
            'z' => "зі",
            _ => &c.to_string(),
        };
        parts.push(letter_name.to_string());
    }
    parts.join(" ")
}

// ==================== Словник одиниць виміру ====================

static UNIT_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("v", "вольт");
    m.insert("mv", "мілівольт");
    m.insert("kv", "кіловольт");
    m.insert("a", "ампер");
    m.insert("ma", "міліампер");
    m.insert("ka", "кілоампер");
    m.insert("ua", "мікроампер");
    m.insert("w", "ват");
    m.insert("mw", "міліват");
    m.insert("kw", "кіловат");
    m.insert("wh", "ват-годин");
    m.insert("mwh", "міліват-годин");
    m.insert("kwh", "кіловат-годин");
    m.insert("ohm", "ом");
    m.insert("kohm", "кілоом");
    m.insert("mohm", "мегаом");
    m.insert("f", "фарад");
    m.insert("uf", "мікрофарад");
    m.insert("nf", "нанофарад");
    m.insert("pf", "пікофарад");
    m.insert("hz", "герц");
    m.insert("khz", "кілогерц");
    m.insert("mhz", "мегагерц");
    m.insert("ghz", "гігагерц");
    m.insert("°c", "градусів цельсія");
    m.insert("°f", "градусів фаренгейта");
    m.insert("°k", "кельвін");
    m.insert("°", "градусів");
    m.insert("%", "відсотків");
    m.insert("pa", "паскаль");
    m.insert("kpa", "кілопаскаль");
    m.insert("lux", "люкс");
    m.insert("db", "децибел");
    m.insert("mm", "міліметр");
    m.insert("cm", "сантиметр");
    m.insert("m", "метр");
    m.insert("km", "кілометр");
    m.insert("мм", "міліметр");
    m.insert("см", "сантиметр");
    m.insert("м", "метр");
    m.insert("км", "кілометр");
    m.insert("g", "грам");
    m.insert("kg", "кілограм");
    m.insert("г", "грам");
    m.insert("кг", "кілограм");
    m.insert("l", "літр");
    m.insert("ml", "мілілітр");
    m.insert("л", "літр");
    m.insert("мл", "мілілітр");
    m.insert("mm/s", "міліметрів за секунду");
    m.insert("m/s", "метрів за секунду");
    m.insert("km/h", "кілометрів на годину");
    m.insert("mmhg", "міліметрів ртутного стовпця");
    m.insert("on", "увімкнено");
    m.insert("off", "вимкнено");
    m.insert("open", "відчинено");
    m.insert("closed", "зачинено");
    m.insert("detected", "виявлено");
    m.insert("clear", "чисто");
    m.insert("true", "так");
    m.insert("false", "ні");
    m
});

fn decline_unit(number_words: &str, unit: &str) -> String {
    let last_word = number_words.split_whitespace().last().unwrap_or("");
    let group = if last_word == "один" || last_word == "одна" { 1 }
    else if last_word == "два" || last_word == "дві"
        || last_word == "три" || last_word == "чотири" { 2 }
    else if last_word == "п'ять" || last_word == "шість" || last_word == "сім"
        || last_word == "вісім" || last_word == "дев'ять"
        || last_word == "десять" || last_word == "одинадцять"
        || last_word == "дванадцять" || last_word == "тринадцять"
        || last_word == "чотирнадцять" || last_word == "п'ятнадцять"
        || last_word == "шістнадцять" || last_word == "сімнадцять"
        || last_word == "вісімнадцять" || last_word == "дев'ятнадцять"
        || last_word == "двадцять" || last_word == "тридцять"
        || last_word == "сорок" || last_word == "п'ятдесят"
        || last_word == "шістдесят" || last_word == "сімдесят"
        || last_word == "вісімдесят" || last_word == "дев'яносто"
        || last_word == "сто" || last_word == "двісті"
        || last_word == "триста" || last_word == "чотириста"
        || last_word == "п'ятсот" || last_word == "шістсот"
        || last_word == "сімсот" || last_word == "вісімсот"
        || last_word == "дев'ятсот" || last_word == "нуль"
        || last_word == "тисяч" || last_word == "мільйон"
        || last_word == "мільярд" { 5 }
    else { 0 };

    match (group, unit) {
        (1, "міліметр") => "міліметр", (1, "сантиметр") => "сантиметр",
        (1, "метр") => "метр", (1, "кілометр") => "кілометр",
        (1, "грам") => "грам", (1, "кілограм") => "кілограм",
        (1, "літр") => "літр", (1, "мілілітр") => "мілілітр",
        (1, "вольт") => "вольт", (1, "мілівольт") => "мілівольт",
        (1, "кіловольт") => "кіловольт", (1, "ампер") => "ампер",
        (1, "міліампер") => "міліампер", (1, "ват") => "ват",
        (1, "міліват") => "міліват", (1, "кіловат") => "кіловат",
        (1, "герц") => "герц", (1, "паскаль") => "паскаль",
        (2, "міліметр") => "міліметри", (2, "сантиметр") => "сантиметри",
        (2, "метр") => "метри", (2, "кілометр") => "кілометри",
        (2, "грам") => "грами", (2, "кілограм") => "кілограми",
        (2, "літр") => "літри", (2, "мілілітр") => "мілілітри",
        (2, "вольт") => "вольти", (2, "мілівольт") => "мілівольти",
        (2, "кіловольт") => "кіловольти", (2, "ампер") => "ампери",
        (2, "міліампер") => "міліампери", (2, "ват") => "вати",
        (2, "міліват") => "мілівати", (2, "кіловат") => "кіловати",
        (2, "герц") => "герци", (2, "паскаль") => "паскалі",
        (5, "міліметр") => "міліметрів", (5, "сантиметр") => "сантиметрів",
        (5, "метр") => "метрів", (5, "кілометр") => "кілометрів",
        (5, "грам") => "грамів", (5, "кілограм") => "кілограмів",
        (5, "літр") => "літрів", (5, "мілілітр") => "мілілітрів",
        (5, "вольт") => "вольт", (5, "мілівольт") => "мілівольт",
        (5, "кіловольт") => "кіловольт", (5, "ампер") => "ампер",
        (5, "міліампер") => "міліампер", (5, "ват") => "ват",
        (5, "міліват") => "міліват", (5, "кіловат") => "кіловат",
        (5, "герц") => "герц", (5, "паскаль") => "паскалів",
        _ => unit,
    }.to_string()
}

fn with_stress_units(word: &str) -> String {
    word.replace("вольт", "во\u{0301}льт")
        .replace("ампер", "ампе\u{0301}р")
        .replace("ват", "ва\u{0301}т")
        .replace("ват-годин", "ва\u{0301}т-годи\u{0301}н")
        .replace("кіловат", "кілова\u{0301}т")
        .replace("кіловат-годин", "кілова\u{0301}т-годи\u{0301}н")
        .replace("міліампер", "міліампе\u{0301}р")
        .replace("міліват", "міліва\u{0301}т")
        .replace("герц", "ге\u{0301}рц")
        .replace("фарад", "фара\u{0301}д")
        .replace("ом", "о\u{0301}м")
        .replace("паскаль", "паска\u{0301}ль")
        .replace("цельсія", "це\u{0301}льсія")
}

// ==================== Конвертація чисел ====================

fn decimal_suffix(digits: usize) -> &'static str {
    match digits {
        1 => "десятих", 2 => "сотих", 3 => "тисячних",
        4 => "десяти тисячних", 5 => "сот тисячних",
        6 => "мільйонних", _ => "десятих",
    }
}

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
    let int_words = if int_part.is_empty() || int_part == "0" {
        "нуль".to_string()
    } else {
        int_to_ua(int_part)
    };
    let dec_digits: Vec<_> = dec_part.chars().filter(|c| c.is_ascii_digit()).collect();
    let dec_clean: String = dec_digits.iter().collect();

    if is_temp && int_val > 9 && dec_digits.len() == 1 {
        let digit_word = if let Some(d) = dec_digits.first().and_then(|c| c.to_digit(10)) {
            num2words::Num2Words::new(d as f64)
                .lang(Lang::Ukrainian)
                .to_words()
                .unwrap_or(dec_clean.clone())
                .replace('ʼ', "'")
        } else { dec_clean };
        format!("{} і {}", int_words, digit_word)
    } else {
        let dec_words = if dec_clean.is_empty() { "нуль".to_string() }
        else if let Ok(n) = dec_clean.parse::<f64>() {
            num2words::Num2Words::new(n)
                .lang(Lang::Ukrainian)
                .to_words()
                .unwrap_or(dec_clean.clone())
                .replace('ʼ', "'")
        } else { dec_clean };
        format!("{} цілих {} {}", int_words, dec_words, decimal_suffix(dec_digits.len()))
    }
}

fn num_to_ua(num_str: &str) -> String {
    let (sign, unsigned) = if num_str.starts_with('-') {
        ("мінус ", &num_str[1..])
    } else { ("", num_str) };

    if let Some(dot) = unsigned.find(|c| c == '.' || c == ',') {
        let int_part = &unsigned[..dot];
        let dec_part = &unsigned[dot + 1..];
        format!("{}{}", sign, decimal_to_ua(int_part, dec_part, false))
    } else if let Ok(n) = unsigned.parse::<f64>() {
        format!("{}{}", sign,
            num2words::Num2Words::new(n)
                .lang(Lang::Ukrainian)
                .to_words()
                .unwrap_or(unsigned.to_string())
                .replace('ʼ', "'"))
    } else { format!("{}{}", sign, unsigned) }
}

fn temp_to_ua(num_str: &str) -> String {
    let (sign, unsigned) = if num_str.starts_with('-') {
        ("мінус ", &num_str[1..])
    } else { ("", num_str) };

    if let Some(dot) = unsigned.find(|c| c == '.' || c == ',') {
        let int_part = &unsigned[..dot];
        let dec_part = &unsigned[dot + 1..];
        format!("{}{}", sign, decimal_to_ua(int_part, dec_part, true))
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

/// Крок 0: Попередня обробка.
/// 1. Знаходить абревіатури (всі великі) і замінює на назви літер
/// 2. Перетворює весь текст на нижній регістр для Piper
/// 3. Подвійні переноси → крапка (пауза при озвучці)
fn step0_fix_paragraphs(text: &str) -> String {
    // Спочатку обробляємо абревіатури (до lowercase, інакше не розрізнити)
    let text = expand_abbreviations_in_text(text);
    
    // Потім — нижній регістр для всього тексту
    let text = text.to_lowercase();
    
    // Потім — подвійні переноси → крапка
    let mut result = String::with_capacity(text.len() + 32);
    let mut prev_char: Option<char> = None;
    let mut consecutive_newlines = 0;

    for c in text.chars() {
        if c == '\n' {
            consecutive_newlines += 1;
        } else if c == '\r' {
            // Пропускаємо \r
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

/// Знаходить і розширює абревіатури в тексті (ДО lowercase)
fn expand_abbreviations_in_text(text: &str) -> String {
    let mut result = String::with_capacity(text.len() * 3);
    let mut chars = text.chars().peekable();
    let mut in_word = String::new();
    
    while let Some(c) = chars.next() {
        if c.is_alphabetic() {
            in_word.push(c);
        } else {
            // Кінець слова — перевіряємо чи це абревіатура
            if is_abbreviation(&in_word) {
                // Спочатку перевіряємо custom словник (винятки як "США")
                let lower = in_word.to_lowercase();
                if let Some(stressed) = STRESS_DICT.get(&lower) {
                    result.push_str(stressed);
                } else {
                    result.push_str(&expand_abbreviation(&in_word));
                }
            } else {
                result.push_str(&in_word);
            }
            in_word.clear();
            result.push(c);
        }
    }
    
    // Останнє слово
    if is_abbreviation(&in_word) {
        let lower = in_word.to_lowercase();
        if let Some(stressed) = STRESS_DICT.get(&lower) {
            result.push_str(stressed);
        } else {
            result.push_str(&expand_abbreviation(&in_word));
        }
    } else {
        result.push_str(&in_word);
    }
    
    result
}

// ==================== КРОК 1: Парсинг тексту в токени ====================

#[derive(Debug, Clone)]
enum Token {
    NumberWithUnit { number: String, unit: String },
    Word(String),
    Symbol(String),
}

/// Крок 1: Розбиваємо текст на токени (числа+одиниці, слова, символи)
fn step1_tokenize(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        // Числа (включаючи від'ємні)
        if c.is_ascii_digit() || (c == '-' && chars.peek().map_or(false, |n| n.is_ascii_digit())) {
            let mut num = String::from(c);
            while let Some(&next) = chars.peek() {
                if next.is_ascii_digit() || next == '.' || next == ',' {
                    num.push(chars.next().unwrap());
                } else { break; }
            }

            // Пропускаємо пробіли перед одиницею
            while let Some(&next) = chars.peek() {
                if next.is_whitespace() { chars.next(); }
                else { break; }
            }

            // Збираємо потенційну одиницю (кирилиця + латиниця + спецсимволи)
            // ОБМЕЖЕННЯ: max 4 символи — всі реальні одиниці коротші
            // (V, Hz, мм, кг, kWh, mmHg). Це запобігає збору звичайних слів.
            let potential_unit: String = chars.clone()
                .take_while(|c| c.is_alphabetic() || *c == '°' || *c == '/' || *c == 'μ' || *c == '%')
                .take(4)
                .collect();

            let potential_lower = potential_unit.to_lowercase();

            // Перевіряємо чи це дійсно одиниця виміру
            let is_known_unit = UNIT_MAP.contains_key(potential_lower.as_str())
                || potential_lower.starts_with('°')
                || potential_lower == "%";

            let unit_str = if is_known_unit {
                // Споживаємо символи одиниці
                let unit_char_len = potential_unit.chars().count();
                for _ in 0..unit_char_len { chars.next(); }
                potential_unit
            } else {
                // Не одиниця — залишаємо символи для наступних ітерацій
                String::new()
            };

            let unit_lower = unit_str.to_lowercase();

            if !unit_str.is_empty() {
                // Спеціальна обробка для "мм рт. ст."
                if unit_lower == "мм" || unit_lower == "mm" {
                    if let Some(pressure) = try_consume_pressure(&mut chars) {
                        tokens.push(Token::NumberWithUnit { number: num, unit: pressure });
                        continue;
                    }
                }

                tokens.push(Token::NumberWithUnit { number: num, unit: unit_str });
            } else {
                tokens.push(Token::NumberWithUnit { number: num, unit: String::new() });
            }
        } else if c.is_alphabetic() {
            // Збираємо слово
            let word_start: String = std::iter::once(c)
                .chain(chars.clone().take_while(|c| c.is_alphabetic()))
                .collect();
            let char_count = word_start.chars().count();
            for _ in 0..char_count - 1 { chars.next(); }
            tokens.push(Token::Word(word_start));
        } else if c == '+' {
            tokens.push(Token::Symbol("+".to_string()));
        } else if c == '=' {
            tokens.push(Token::Symbol("=".to_string()));
        } else if c == '&' {
            tokens.push(Token::Symbol("&".to_string()));
        } else if c.is_whitespace() {
            // Пропускаємо пробіли (вони відновлюються при join)
        } else {
            tokens.push(Token::Symbol(c.to_string()));
        }
    }

    tokens
}

/// Споживає контекст "ртутного стовпця" після "мм"
fn try_consume_pressure(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<String> {
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() { chars.next(); }
        else { break; }
    }
    let next_word: String = chars.clone()
        .take_while(|c| c.is_alphabetic())
        .collect();
    let next_lower = next_word.to_lowercase();
    if next_lower == "рт" || next_lower.starts_with("ртутн") {
        let word_len = next_word.chars().count();
        for _ in 0..word_len { chars.next(); }
        while let Some(&c) = chars.peek() {
            if c.is_whitespace() { chars.next(); }
            else { break; }
        }
        let after: String = chars.clone()
            .take_while(|c| c.is_alphabetic())
            .collect();
        let after_lower = after.to_lowercase();
        if after_lower.starts_with("стовп") || after_lower == "ст" {
            let after_len = after.chars().count();
            for _ in 0..after_len { chars.next(); }
        }
        return Some("міліметрів ртутного стовпця".to_string());
    }
    None
}

// ==================== КРОК 2: Конвертація чисел ====================

/// Крок 2: Конвертуємо числа в українські слова + одиниці
fn step2_convert_numbers(tokens: &[Token]) -> Vec<String> {
    let mut words = Vec::new();

    for token in tokens {
        match token {
            Token::NumberWithUnit { number, unit } => {
                let unit_lower = unit.to_lowercase();
                let is_temp = unit_lower.starts_with('°');
                let num_words = if is_temp {
                    temp_to_ua(number)
                } else {
                    num_to_ua(number)
                };

                // Якщо це "міліметрів ртутного стовпця" — спеціальний випадок
                if unit == "міліметрів ртутного стовпця" {
                    words.push(num_words);
                    words.push("міліметрів ртутного стовпця".to_string());
                    continue;
                }

                if !unit.is_empty() {
                    if let Some((_len, replacement)) = find_unit(&unit_lower) {
                        let declined = decline_unit(&num_words, replacement);
                        let declined = with_stress_units(&declined);
                        words.push(num_words);
                        words.push(declined);
                    } else {
                        words.push(num_words);
                    }
                } else {
                    words.push(num_words);
                }
            }
            Token::Word(w) => {
                let word_lower = w.to_lowercase();
                // ВАЖЛИВО: для звичайних слів перевіряємо ТОЧНИЙ збіг в UNIT_MAP,
                // а не префікс. Інакше "г" → "грам" з'їдає "глава",
                // а "м" → "метр" з'їдає "мзс".
                if let Some(&replacement) = UNIT_MAP.get(word_lower.as_str()) {
                    words.push(with_stress_units(replacement));
                } else {
                    words.push(w.clone());
                }
            }
            Token::Symbol(s) => {
                match s.as_str() {
                    "+" => words.push("плюс".to_string()),
                    "=" => words.push("дорівнює".to_string()),
                    "&" => words.push("і".to_string()),
                    _ => words.push(s.clone()),
                }
            }
        }
    }

    words
}

fn find_unit(text: &str) -> Option<(usize, &'static str)> {
    let lower = text.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();
    for end in (1..=chars.len()).rev() {
        let candidate: String = chars[..end].iter().collect();
        if let Some(&replacement) = UNIT_MAP.get(candidate.as_str()) {
            return Some((end, replacement));
        }
    }
    None
}

// ==================== КРОК 3: Додавання наголосів ====================

/// Крок 3: Додаємо наголоси до кожного слова через словник
fn step3_apply_stress(words: &[String]) -> Vec<String> {
    words.iter()
        .map(|w| apply_stress(w))
        .collect()
}

// ==================== КРОК 4: Фінальне очищення ====================

/// Крок 4: Видаляємо зайві символи, нормалізуємо пробіли.
/// Текст вже lowercase з кроку 0.
fn step4_cleanup(words: &[String]) -> String {
    words.join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

// ==================== Публічний API ====================

/// Повний пайплайн нормалізації тексту.
/// Кожен крок можна викликати окремо для дебагу.
pub fn normalize_text(text: &str) -> String {
    let prepared = step0_fix_paragraphs(text);
    let tokens = step1_tokenize(&prepared);
    let words = step2_convert_numbers(&tokens);
    let stressed = step3_apply_stress(&words);
    step4_cleanup(&stressed)
}

/// Для дебагу: повертає проміжні результати всіх кроків
pub fn normalize_text_debug(text: &str) -> (String, Vec<Token>, Vec<String>, Vec<String>, String) {
    let prepared = step0_fix_paragraphs(text);
    let tokens = step1_tokenize(&prepared);
    let words = step2_convert_numbers(&tokens);
    let stressed = step3_apply_stress(&words);
    let final_text = step4_cleanup(&stressed);
    (prepared, tokens, words, stressed, final_text)
}

// ==================== Тести ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step0_paragraphs() {
        // Подвійний перенос без крапки → додається крапка
        let r = step0_fix_paragraphs("привіт світе\n\nяк справи");
        eprintln!("Подвійний \\n без крапки: '{}'", r);
        assert!(has_text(&r, "привіт світе. як справи"));

        // Подвійний перенос з крапкою → крапка НЕ додається
        let r = step0_fix_paragraphs("привіт світе.\n\nяк справи");
        eprintln!("Подвійний \\n з крапкою: '{}'", r);
        assert!(has_text(&r, "привіт світе. як справи"));

        // Потрійний перенос
        let r = step0_fix_paragraphs("привіт\n\n\nяк справи");
        eprintln!("Потрійний \\n: '{}'", r);
        assert!(has_text(&r, "привіт. як справи"));

        // Знак питання — крапка не потрібна
        let r = step0_fix_paragraphs("як справи?\n\nдобре");
        eprintln!("Знак питання: '{}'", r);
        assert!(has_text(&r, "як справи? добре"));

        // Одинарний перенос — ігнорується
        let r = step0_fix_paragraphs("рядок один\nрядок два");
        eprintln!("Одинарний \\n: '{}'", r);
        assert_eq!(r, "рядок одинрядок два");
    }

    #[test]
    fn test_step1_tokenize_simple() {
        let tokens = step1_tokenize("Вологість 65%, тиск 760 мм ртутного стовпця");
        eprintln!("=== Крок 1: Токени ===");
        for t in &tokens {
            eprintln!("  {:?}", t);
        }
        // Перевіряємо що є число 65 з одиницею %
        assert!(tokens.iter().any(|t| matches!(t, Token::NumberWithUnit { number, .. } if number == "65")));
        // Перевіряємо що є число 760 з одиницею мм
        assert!(tokens.iter().any(|t| matches!(t, Token::NumberWithUnit { number, .. } if number == "760")));
    }

    #[test]
    fn test_step2_convert_numbers() {
        let tokens = step1_tokenize("5 м");
        let words = step2_convert_numbers(&tokens);
        eprintln!("=== Крок 2: Слова ===");
        for w in &words {
            eprintln!("  '{}'", w);
        }
        assert!(words.iter().any(|w| w.contains("п'ять")));
        assert!(words.iter().any(|w| w.contains("метр")));
    }

    #[test]
    fn test_step3_apply_stress() {
        let words = vec!["вологість".to_string(), "тиск".to_string()];
        let stressed = step3_apply_stress(&words);
        eprintln!("=== Крок 3: Наголоси ===");
        for (before, after) in words.iter().zip(stressed.iter()) {
            eprintln!("  '{}' → '{}'", before, after);
        }
    }

    #[test]
    fn test_full_weather_example() {
        let text = "Вологість 65%, тиск 760 мм ртутного стовпця";
        let (_prepared, tokens, words, stressed, final_text) = normalize_text_debug(text);

        eprintln!("\n=== Повний пайплайн ===");
        eprintln!("Вхід: '{}'", text);
        eprintln!("Крок 1 (токени): {:?}", tokens);
        eprintln!("Крок 2 (слова): {:?}", words);
        eprintln!("Крок 3 (наголоси): {:?}", stressed);
        eprintln!("Крок 4 (фінал): '{}'", final_text);
        
        assert!(has_text(&final_text, "шістдесят"));
        assert!(has_text(&final_text, "відсотків"));
        assert!(has_text(&final_text, "сімсот"));
        assert!(has_text(&final_text, "міліметрів ртутного стовпця"));
    }

    /// Helper: перевіряє наявність підрядка БЕЗ урахування наголосів
    fn has_text(result: &str, expected: &str) -> bool {
        let stripped: String = result.chars().filter(|c| *c != '\u{0301}').collect();
        stripped.contains(expected)
    }

    #[test]
    fn test_decimals() {
        assert!(int_to_ua("36.6").contains("цілих"));
        let r = normalize_text("100%");
        assert!(has_text(&r, "сто"), "100% → {}", r);
    }

    #[test]
    fn test_temp_short_format() {
        assert!(temp_to_ua("36.6").contains(" і "));
        assert!(!temp_to_ua("36.6").contains("цілих"));
    }

    #[test]
    fn test_electric_units() {
        let r = normalize_text("220V");
        assert!(has_text(&r, "двісті"), "220V → {}", r);
        let r = normalize_text("500mA");
        assert!(has_text(&r, "п'ятсот"), "500mA → {}", r);
    }

    #[test]
    fn test_ha_states() {
        assert!(has_text(&normalize_text("on"), "увімкнено"));
        assert!(has_text(&normalize_text("off"), "вимкнено"));
    }

    #[test]
    fn test_voltage_variants() {
        assert!(has_text(&normalize_text("12V"), "дванадцять"));
    }

    #[test]
    fn test_current_variants() {
        assert!(has_text(&normalize_text("10A"), "десять"));
    }

    #[test]
    fn test_power_variants() {
        assert!(has_text(&normalize_text("100W"), "сто"));
    }

    #[test]
    fn test_mm_pressure_cyrillic() {
        let r = normalize_text("760 мм ртутного стовпця");
        assert!(has_text(&r, "сімсот"), "760 мм → {}", r);
        assert!(has_text(&r, "міліметрів ртутного стовпця"), "760 мм → {}", r);
    }

    #[test]
    fn test_mm_pressure_short() {
        let r = normalize_text("760 мм рт ст");
        assert!(has_text(&r, "міліметрів ртутного стовпця"), "760 мм рт ст → {}", r);
    }

    #[test]
    fn test_mm_latin() {
        let r = normalize_text("760mm");
        assert!(has_text(&r, "сімсот"), "760mm → {}", r);
        assert!(has_text(&r, "міліметрів"), "760mm → {}", r);
    }

    #[test]
    fn test_meters_cyrillic() {
        let r = normalize_text("5 м");
        assert!(has_text(&r, "п'ять"), "5 м → {}", r);
        assert!(has_text(&r, "метрів"), "5 м → {}", r);
    }

    #[test]
    fn test_centimeters_cyrillic() {
        let r = normalize_text("10 см");
        assert!(has_text(&r, "десять"), "10 см → {}", r);
        assert!(has_text(&r, "сантиметрів"), "10 см → {}", r);
    }

    #[test]
    fn test_grams_cyrillic() {
        let r = normalize_text("200 г");
        assert!(has_text(&r, "двісті"), "200 г → {}", r);
        assert!(has_text(&r, "грамів"), "200 г → {}", r);
    }

    #[test]
    fn test_kilograms_cyrillic() {
        let r = normalize_text("3 кг");
        assert!(has_text(&r, "три"), "3 кг → {}", r);
        assert!(has_text(&r, "кілограми"), "3 кг → {}", r);
    }

    #[test]
    fn test_liters_cyrillic() {
        let r = normalize_text("2 л");
        assert!(has_text(&r, "два"), "2 л → {}", r);
        assert!(has_text(&r, "літри"), "2 л → {}", r);
    }

    #[test]
    fn test_declension_singular() {
        assert!(has_text(&normalize_text("1 мм"), "один міліметр"));
        assert!(has_text(&normalize_text("1 м"), "один метр"));
        assert!(has_text(&normalize_text("1 г"), "один грам"));
        assert!(has_text(&normalize_text("1 л"), "один літр"));
    }

    #[test]
    fn test_declension_plural() {
        assert!(has_text(&normalize_text("2 мм"), "два міліметри"));
        assert!(has_text(&normalize_text("3 м"), "три метри"));
        assert!(has_text(&normalize_text("4 см"), "чотири сантиметри"));
        assert!(has_text(&normalize_text("2 кг"), "два кілограми"));
    }

    #[test]
    fn test_declension_genitive() {
        assert!(has_text(&normalize_text("5 мм"), "п'ять міліметрів"));
        assert!(has_text(&normalize_text("10 м"), "десять метрів"));
        assert!(has_text(&normalize_text("100 см"), "сто сантиметрів"));
        assert!(has_text(&normalize_text("200 г"), "двісті грамів"));
    }

    #[test]
    fn test_stress_dict_loaded() {
        assert!(!STRESS_DICT.is_empty(), "Словник наголосів порожній!");
    }

    #[test]
    fn test_abbreviation_expansion() {
        // Абревіатури розшифровуються по літерах
        assert_eq!(expand_abbreviation("РТЦК"), "ер те це ка");
        assert_eq!(expand_abbreviation("ртцк"), "ер те це ка");
        assert_eq!(expand_abbreviation("ТЦК"), "те це ка");
        assert_eq!(expand_abbreviation("СБУ"), "ес бе у");
        assert_eq!(expand_abbreviation("МВС"), "ем ве ес");
        assert_eq!(expand_abbreviation("НАТО"), "ен а те о");
        assert_eq!(expand_abbreviation("ABC"), "ей бі сі");
    }

    #[test]
    fn test_is_abbreviation() {
        // Абревіатури — ТІЛЬКИ з великими літерами
        assert!(is_abbreviation("РТЦК"));
        assert!(is_abbreviation("ТЦК"));
        assert!(is_abbreviation("СБУ"));
        assert!(is_abbreviation("МВС"));
        assert!(is_abbreviation("НАТО"));
        assert!(is_abbreviation("USA"));
        assert!(is_abbreviation("NATO"));
        // Lowercase — НЕ абревіатури
        assert!(!is_abbreviation("ртцк"));
        assert!(!is_abbreviation("мм"));
        assert!(!is_abbreviation("мзс"));
        // Звичайні слова — НЕ абревіатури
        assert!(!is_abbreviation("андрій"));
        assert!(!is_abbreviation("євтушенко"));
        assert!(!is_abbreviation("привіт"));
        assert!(!is_abbreviation("а")); // занадто коротке
    }

    #[test]
    fn test_officer_text_with_abbreviations() {
        let r = normalize_text("заступник керівника бучанського РТЦК андрій євтушенко");
        eprintln!("Офіцер: '{}'", r);
        assert!(has_text(&r, "заступник"));
        assert!(has_text(&r, "керівника"));
        assert!(has_text(&r, "бучанського"));
        assert!(has_text(&r, "ер те це ка"), "РТЦК має бути розшифровано → {}", r);
        assert!(has_text(&r, "андрій"));
        assert!(has_text(&r, "євтушенко"));
    }

    #[test]
    fn test_paragraph_breaks_in_full_pipeline() {
        let text = "заступник керівника бучанського ртцк\n\nандрій євтушенко";
        let (prepared, _tokens, _words, _stressed, final_text) = normalize_text_debug(text);
        eprintln!("\n=== Параграфи ===");
        eprintln!("Підготовлено: '{}'", prepared);
        eprintln!("Фінал: '{}'", final_text);
        // Крапка додана після "ртцк" і збережена в фіналі
        assert!(final_text.contains("."), "має бути крапка → {}", final_text);
        assert!(has_text(&final_text, "андрій"), "ім'я має бути з наголосом → {}", final_text);
    }
}


