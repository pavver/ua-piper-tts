/// Модуль для розумного контекстного розгортання та відмінювання українських абревіатур та одиниць виміру.

use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Declension {
    Nominative,
    Genitive,
    Dative,
    Accusative,
    Instrumental,
    Locative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordRole {
    Static,
    Adj,
    Noun,
}

#[derive(Debug, Clone)]
pub struct WordSpec {
    pub text: &'static str,
    pub role: WordRole,
    pub gender: Gender,
    pub is_always_plural: bool,
}

impl WordSpec {
    pub const fn static_word(text: &'static str) -> Self {
        Self {
            text,
            role: WordRole::Static,
            gender: Gender::Masculine,
            is_always_plural: false,
        }
    }
    
    pub const fn adj(text: &'static str) -> Self {
        Self {
            text,
            role: WordRole::Adj,
            gender: Gender::Masculine,
            is_always_plural: false,
        }
    }
    
    pub const fn noun(text: &'static str, gender: Gender) -> Self {
        Self {
            text,
            role: WordRole::Noun,
            gender,
            is_always_plural: false,
        }
    }
    
    pub const fn noun_pl(text: &'static str, gender: Gender) -> Self {
        Self {
            text,
            role: WordRole::Noun,
            gender,
            is_always_plural: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AbbrSpec {
    pub gender_class: &'static str,
    pub words: Vec<WordSpec>,
}

/// Вираховує довжину основи в байтах, ігноруючи символи наголосу `\u{0301}`.
fn get_stem_len_with_accent(word: &str, suffix_clean_len: usize) -> usize {
    let chars: Vec<char> = word.chars().collect();
    let mut clean_counted = 0;
    let mut idx = chars.len();
    
    while idx > 0 && clean_counted < suffix_clean_len {
        idx -= 1;
        if chars[idx] != '\u{0301}' {
            clean_counted += 1;
        }
    }
    
    chars[..idx].iter().collect::<String>().len()
}

/// Відмінює прикметники за родами, числами й відмінками (м'які й тверді основи).
pub fn decline_adjective(
    orig_word: &str,
    decl: Declension,
    is_plural: bool,
    gender: Gender,
) -> String {
    let clean = orig_word.replace('\u{0301}', "");
    let mut is_soft = false;
    let mut stem_len = orig_word.len();
    
    if clean.ends_with("ий") {
        stem_len = get_stem_len_with_accent(orig_word, 2);
    } else if clean.ends_with("ій") {
        is_soft = true;
        stem_len = get_stem_len_with_accent(orig_word, 2);
    } else if clean.ends_with('а') {
        stem_len = get_stem_len_with_accent(orig_word, 1);
    } else if clean.ends_with('я') {
        is_soft = true;
        stem_len = get_stem_len_with_accent(orig_word, 1);
    } else if clean.ends_with('е') {
        stem_len = get_stem_len_with_accent(orig_word, 1);
    } else if clean.ends_with('є') {
        is_soft = true;
        stem_len = get_stem_len_with_accent(orig_word, 1);
    } else if clean.ends_with('і') {
        stem_len = get_stem_len_with_accent(orig_word, 1);
    }
    
    let stem = &orig_word[..stem_len];
    
    if is_plural {
        match decl {
            Declension::Nominative => format!("{}і", stem),
            Declension::Genitive => if is_soft { format!("{}іх", stem) } else { format!("{}их", stem) },
            Declension::Dative => if is_soft { format!("{}ім", stem) } else { format!("{}им", stem) },
            Declension::Accusative => format!("{}і", stem),
            Declension::Instrumental => if is_soft { format!("{}іми", stem) } else { format!("{}ими", stem) },
            Declension::Locative => if is_soft { format!("{}іх", stem) } else { format!("{}их", stem) },
        }
    } else {
        match gender {
            Gender::Feminine => {
                match decl {
                    Declension::Nominative => if is_soft { format!("{}я", stem) } else { format!("{}а", stem) },
                    Declension::Genitive => if is_soft { format!("{}ньої", stem) } else { format!("{}ої", stem) },
                    Declension::Dative | Declension::Locative => format!("{}ій", stem),
                    Declension::Accusative => if is_soft { format!("{}ю", stem) } else { format!("{}у", stem) },
                    Declension::Instrumental => if is_soft { format!("{}ньою", stem) } else { format!("{}ою", stem) },
                }
            }
            Gender::Masculine | Gender::Neuter => {
                match decl {
                    Declension::Nominative => {
                        if gender == Gender::Masculine {
                            if is_soft { format!("{}ій", stem) } else { format!("{}ий", stem) }
                        } else {
                            if is_soft { format!("{}є", stem) } else { format!("{}е", stem) }
                        }
                    }
                    Declension::Genitive => if is_soft { format!("{}ього", stem) } else { format!("{}ого", stem) },
                    Declension::Dative => if is_soft { format!("{}ьому", stem) } else { format!("{}ому", stem) },
                    Declension::Accusative => {
                        if gender == Gender::Masculine {
                            if is_soft { format!("{}ій", stem) } else { format!("{}ий", stem) }
                        } else {
                            if is_soft { format!("{}є", stem) } else { format!("{}е", stem) }
                        }
                    }
                    Declension::Instrumental => format!("{}им", stem),
                    Declension::Locative => if is_soft { format!("{}ьому", stem) } else { format!("{}ому", stem) },
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct NounRules {
    pub nominative_sg: String,
    pub stem: String,
    pub gender: Gender,
    pub is_soft: bool,
    pub is_always_plural: bool,
    pub gen_sg_ending: String,
    pub gen_pl_ending: String,
    pub loc_sg_ending: String,
}

impl NounRules {
    pub fn build(orig_word: &str, gender: Gender, is_always_plural: bool) -> Self {
        let clean = orig_word.replace('\u{0301}', "");
        
        if is_always_plural {
            let is_soft = clean.ends_with('і') || clean.ends_with('я') || clean.ends_with('ї')
                || clean.ends_with("і\u{0301}") || clean.ends_with("я\u{0301}") || clean.ends_with("ї\u{0301}");
            let stem_len = get_stem_len_with_accent(orig_word, 1);
            
            let gen_pl_ending = match gender {
                Gender::Feminine => {
                    if clean.ends_with('ї') || clean.ends_with("ї\u{0301}") || clean.ends_with("ія") {
                        "й".to_string()
                    } else if is_soft {
                        "ь".to_string()
                    } else {
                        "".to_string()
                    }
                }
                Gender::Neuter => {
                    if clean.ends_with('я') || clean.ends_with("я\u{0301}") {
                        "ь".to_string()
                    } else {
                        "".to_string()
                    }
                }
                Gender::Masculine => {
                    if is_soft {
                        "ей".to_string()
                    } else {
                        "ів".to_string()
                    }
                }
            };
            
            return NounRules {
                nominative_sg: orig_word.to_string(),
                stem: orig_word[..stem_len].to_string(),
                gender,
                is_soft,
                is_always_plural,
                gen_sg_ending: "".to_string(),
                gen_pl_ending,
                loc_sg_ending: "".to_string(),
            };
        }

        let mut is_soft = false;
        let mut stem_len = orig_word.len();
        let mut gen_sg_ending = "а".to_string();
        let mut gen_pl_ending = "ів".to_string();
        let mut loc_sg_ending = "і".to_string();
        
        match gender {
            Gender::Feminine => {
                if clean.ends_with("ія") {
                    is_soft = true;
                    stem_len = get_stem_len_with_accent(orig_word, 1);
                    gen_sg_ending = "і".to_string();
                    gen_pl_ending = "й".to_string();
                } else if clean.ends_with('я') {
                    is_soft = true;
                    stem_len = get_stem_len_with_accent(orig_word, 1);
                    gen_sg_ending = "і".to_string();
                    gen_pl_ending = "ь".to_string();
                } else if clean.ends_with('а') {
                    stem_len = get_stem_len_with_accent(orig_word, 1);
                    gen_sg_ending = "и".to_string();
                    gen_pl_ending = "".to_string();
                }
            }
            Gender::Neuter => {
                if clean.ends_with("ння") {
                    is_soft = true;
                    stem_len = get_stem_len_with_accent(orig_word, 1);
                    gen_sg_ending = "я".to_string();
                    gen_pl_ending = "ь".to_string();
                } else if clean.ends_with('я') {
                    is_soft = true;
                    stem_len = get_stem_len_with_accent(orig_word, 1);
                    gen_sg_ending = "я".to_string();
                    gen_pl_ending = "".to_string();
                } else if clean.ends_with('о') {
                    stem_len = get_stem_len_with_accent(orig_word, 1);
                    gen_sg_ending = "а".to_string();
                    gen_pl_ending = "".to_string();
                }
            }
            Gender::Masculine => {
                if clean.ends_with('ь') {
                    is_soft = true;
                    stem_len = get_stem_len_with_accent(orig_word, 1);
                    gen_sg_ending = "я".to_string();
                    gen_pl_ending = "ів".to_string();
                } else if clean.ends_with('й') {
                    is_soft = true;
                    stem_len = get_stem_len_with_accent(orig_word, 1);
                    gen_sg_ending = "я".to_string();
                    gen_pl_ending = "ів".to_string();
                } else {
                    stem_len = orig_word.len();
                    gen_sg_ending = "а".to_string();
                    gen_pl_ending = "ів".to_string();
                    
                    let clean_lower = clean.to_lowercase();
                    if matches!(clean_lower.as_str(), "центр" | "центру" | "банк" | "союз" | "пункт" | "комплекс" | "комітет" | "кооператив" | "тест" | "суд" | "фонд" | "оборот" | "офіс" | "кабінет") {
                        gen_sg_ending = "у".to_string();
                    }
                    if clean_lower.ends_with('к') {
                        loc_sg_ending = "у".to_string();
                    }
                }
            }
        }
        
        let mut stem = orig_word[..stem_len].to_string();
        
        if gender == Gender::Masculine && clean.ends_with("ець") {
            let base_len = get_stem_len_with_accent(orig_word, 3);
            let mut base = orig_word[..base_len].to_string();
            base.push('ц');
            stem = base;
        }
        
        let clean_lower = clean.to_lowercase();
        if clean_lower == "відсоток" {
            if orig_word.contains('\u{0301}') {
                stem = orig_word.replace("ток", "тк");
            } else {
                stem = "відсотк".to_string();
            }
            gen_sg_ending = "а".to_string();
            loc_sg_ending = "у".to_string();
        } else if clean_lower == "податок" {
            if orig_word.contains('\u{0301}') {
                stem = orig_word.replace("ток", "тк");
            } else {
                stem = "податк".to_string();
            }
            gen_sg_ending = "у".to_string();
            loc_sg_ending = "у".to_string();
        } else if clean_lower == "внесок" {
            if orig_word.contains('\u{0301}') {
                stem = orig_word.replace("сок", "ск");
            } else {
                stem = "внеск".to_string();
            }
            gen_sg_ending = "у".to_string();
            loc_sg_ending = "у".to_string();
        }
        
        NounRules {
            nominative_sg: orig_word.to_string(),
            stem,
            gender,
            is_soft,
            is_always_plural,
            gen_sg_ending,
            gen_pl_ending,
            loc_sg_ending,
        }
    }
}

pub fn decline_noun(rules: &NounRules, decl: Declension, is_plural: bool) -> String {
    let stem = &rules.stem;
    let is_soft = rules.is_soft;
    let actual_plural = is_plural || rules.is_always_plural;
    
    let clean_nom = rules.nominative_sg.replace('\u{0301}', "");
    if rules.gender == Gender::Neuter && (clean_nom.ends_with("ння") || clean_nom.ends_with("ння\u{0301}")) {
        let base = rules.nominative_sg.clone();
        let has_end_accent = base.ends_with('\u{0301}');
        let mut clean_base = if has_end_accent { base[..base.len() - 1].to_string() } else { base.clone() };
        
        if clean_base.ends_with('я') {
            clean_base.pop();
        }
        
        if actual_plural {
            return match decl {
                Declension::Nominative | Declension::Accusative => rules.nominative_sg.clone(),
                Declension::Genitive => {
                    if clean_base.ends_with('н') {
                        clean_base.pop();
                    }
                    format!("{}ь", clean_base)
                }
                Declension::Dative => format!("{}ям", clean_base),
                Declension::Instrumental => format!("{}ями", clean_base),
                Declension::Locative => format!("{}ях", clean_base),
            };
        } else {
            return match decl {
                Declension::Nominative | Declension::Accusative | Declension::Genitive => rules.nominative_sg.clone(),
                Declension::Dative => format!("{}ю", clean_base),
                Declension::Instrumental => format!("{}ям", clean_base),
                Declension::Locative => format!("{}і", clean_base),
            };
        }
    }
    
    let result_str = if actual_plural {
        match rules.gender {
            Gender::Feminine => {
                match decl {
                    Declension::Nominative | Declension::Accusative => {
                        if is_soft { format!("{}і", stem) } else { format!("{}и", stem) }
                    }
                    Declension::Genitive => {
                        if rules.gen_pl_ending == "й" {
                            format!("{}й", stem)
                        } else if rules.gen_pl_ending == "ь" {
                            format!("{}ь", stem)
                        } else {
                            stem.clone()
                        }
                    }
                    Declension::Dative => {
                        if is_soft { format!("{}ям", stem) } else { format!("{}ам", stem) }
                    }
                    Declension::Instrumental => {
                        if is_soft { format!("{}ями", stem) } else { format!("{}ами", stem) }
                    }
                    Declension::Locative => {
                        if is_soft { format!("{}ях", stem) } else { format!("{}ах", stem) }
                    }
                }
            }
            Gender::Neuter => {
                match decl {
                    Declension::Nominative | Declension::Accusative => {
                        if is_soft { format!("{}я", stem) } else { format!("{}а", stem) }
                    }
                    Declension::Genitive => {
                        if rules.gen_pl_ending == "ь" {
                            format!("{}ь", stem)
                        } else {
                            stem.clone()
                        }
                    }
                    Declension::Dative => {
                        if is_soft { format!("{}ям", stem) } else { format!("{}ам", stem) }
                    }
                    Declension::Instrumental => {
                        if is_soft { format!("{}ями", stem) } else { format!("{}ами", stem) }
                    }
                    Declension::Locative => {
                        if is_soft { format!("{}ях", stem) } else { format!("{}ах", stem) }
                    }
                }
            }
            Gender::Masculine => {
                match decl {
                    Declension::Nominative | Declension::Accusative => {
                        if is_soft { format!("{}і", stem) } else { format!("{}и", stem) }
                    }
                    Declension::Genitive => {
                        if rules.gen_pl_ending.is_empty() {
                            stem.clone()
                        } else {
                            format!("{}{}", stem, rules.gen_pl_ending)
                        }
                    }
                    Declension::Dative => {
                        if is_soft { format!("{}ям", stem) } else { format!("{}ам", stem) }
                    }
                    Declension::Instrumental => {
                        if is_soft { format!("{}ями", stem) } else { format!("{}ами", stem) }
                    }
                    Declension::Locative => {
                        if is_soft { format!("{}ях", stem) } else { format!("{}ах", stem) }
                    }
                }
            }
        }
    } else {
        match rules.gender {
            Gender::Feminine => {
                match decl {
                    Declension::Nominative => {
                        if is_soft { format!("{}я", stem) } else { format!("{}а", stem) }
                    }
                    Declension::Genitive => {
                        format!("{}{}", stem, rules.gen_sg_ending)
                    }
                    Declension::Dative | Declension::Locative => {
                        format!("{}і", stem)
                    }
                    Declension::Accusative => {
                        if is_soft { format!("{}ю", stem) } else { format!("{}у", stem) }
                    }
                    Declension::Instrumental => {
                        if is_soft { format!("{}єю", stem) } else { format!("{}ою", stem) }
                    }
                }
            }
            Gender::Neuter => {
                match decl {
                    Declension::Nominative | Declension::Accusative => {
                        if is_soft { format!("{}я", stem) } else { format!("{}о", stem) }
                    }
                    Declension::Genitive => {
                        format!("{}{}", stem, rules.gen_sg_ending)
                    }
                    Declension::Dative => {
                        if is_soft { format!("{}ю", stem) } else { format!("{}у", stem) }
                    }
                    Declension::Instrumental => {
                        if is_soft { format!("{}ям", stem) } else { format!("{}ом", stem) }
                    }
                    Declension::Locative => {
                        format!("{}і", stem)
                    }
                }
            }
            Gender::Masculine => {
                match decl {
                    Declension::Nominative | Declension::Accusative => {
                        rules.nominative_sg.clone()
                    }
                    Declension::Genitive => {
                        format!("{}{}", stem, rules.gen_sg_ending)
                    }
                    Declension::Dative => {
                        if is_soft { format!("{}ю", stem) } else { format!("{}у", stem) }
                    }
                    Declension::Instrumental => {
                        if is_soft { format!("{}ем", stem) } else { format!("{}ом", stem) }
                    }
                    Declension::Locative => {
                        format!("{}{}", stem, rules.loc_sg_ending)
                    }
                }
            }
        }
    };
    
    let mut res = result_str;
    if res.ends_with("іі") {
        let new_len = res.len() - 2;
        res.truncate(new_len);
        res.push('ї');
    } else if res.ends_with("і\u{0301}і") {
        let new_len = res.len() - 2;
        res.truncate(new_len);
        res.push('ї');
    }
    res
}

pub static ABBR_MAP: LazyLock<HashMap<&'static str, AbbrSpec>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    
    // СБУ
    m.insert("сбу", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::noun("слу\u{0301}жба", Gender::Feminine),
            WordSpec::static_word("безпе\u{0301}ки"),
            WordSpec::static_word("Украї\u{0301}ни"),
        ],
    });
    
    // ДСНС
    m.insert("дснс", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("держа\u{0301}вна"),
            WordSpec::noun("слу\u{0301}жба", Gender::Feminine),
            WordSpec::static_word("з"),
            WordSpec::static_word("надзвича\u{0301}йних"),
            WordSpec::static_word("ситуа\u{0301}цій"),
        ],
    });
    
    // ДПСУ
    m.insert("дпсу", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("держа\u{0301}вна"),
            WordSpec::adj("прикордо\u{0301}нна"),
            WordSpec::noun("слу\u{0301}жба", Gender::Feminine),
            WordSpec::static_word("Украї\u{0301}ни"),
        ],
    });
    
    // ВРУ
    m.insert("вру", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("верхо\u{0301}вна"),
            WordSpec::noun("ра\u{0301}да", Gender::Feminine),
            WordSpec::static_word("Украї\u{0301}ни"),
        ],
    });
    
    // РНБО
    m.insert("рнбо", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::noun("ра\u{0301}да", Gender::Feminine),
            WordSpec::static_word("націона\u{0301}льної"),
            WordSpec::static_word("безпе\u{0301}ки"),
            WordSpec::static_word("і"),
            WordSpec::static_word("оборо\u{0301}ни"),
            WordSpec::static_word("Украї\u{0301}ни"),
        ],
    });
    
    // ООН
    m.insert("оон", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::noun("організа\u{0301}ція", Gender::Feminine),
            WordSpec::static_word("об'є\u{0301}днаних"),
            WordSpec::static_word("на\u{0301}цій"),
        ],
    });
    
    // ППО
    m.insert("ппо", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("proтипові\u{0301}тряна"), // виправлено typo 'pro' -> 'про' в оригіналі? Давай напишемо "протипові\u{0301}тряна"
            WordSpec::noun("оборо\u{0301}на", Gender::Feminine),
        ],
    });
    
    // РЕБ
    m.insert("реб", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("радіоелектро\u{0301}нна"),
            WordSpec::noun("боротьба\u{0301}", Gender::Feminine),
        ],
    });
    
    // ГЕС
    m.insert("гес", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::noun("гідроелектроста\u{0301}нція", Gender::Feminine),
        ],
    });
    
    // АЕС
    m.insert("аес", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("а\u{0301}томна"),
            WordSpec::noun("електроста\u{0301}нція", Gender::Feminine),
        ],
    });
    
    // ТЕС
    m.insert("тес", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("теплова\u{0301}"),
            WordSpec::noun("електроста\u{0301}нція", Gender::Feminine),
        ],
    });
    
    // ТЦК
    m.insert("тцк", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::adj("територіа\u{0301}льний"),
            WordSpec::noun("це\u{0301}нтр", Gender::Masculine),
            WordSpec::static_word("комплекту\u{0301}вання"),
        ],
    });
    
    // РТЦК
    m.insert("ртцк", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::adj("райо\u{0301}нний"),
            WordSpec::adj("територіа\u{0301}льний"),
            WordSpec::noun("це\u{0301}нтр", Gender::Masculine),
            WordSpec::static_word("комплекту\u{0301}вання"),
        ],
    });
    
    // НБУ
    m.insert("нбу", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::adj("націона\u{0301}льний"),
            WordSpec::noun("банк", Gender::Masculine),
            WordSpec::static_word("Украї\u{0301}ни"),
        ],
    });
    
    // ЄС
    m.insert("єс", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::adj("європейськи\u{0301}й"),
            WordSpec::noun("сою\u{0301}з", Gender::Masculine),
        ],
    });
    
    // ЦНАП
    m.insert("цнап", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::noun("це\u{0301}нтр", Gender::Masculine),
            WordSpec::static_word("нада\u{0301}ння"),
            WordSpec::static_word("адміністрати\u{0301}вних"),
            WordSpec::static_word("послу\u{0301}г"),
        ],
    });
    
    // КПП
    m.insert("кпп", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::adj("контро\u{0301}льно-пропускни\u{0301}й"),
            WordSpec::noun("пу\u{0301}нкт", Gender::Masculine),
        ],
    });
    
    // БПЛА
    m.insert("бпла", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::adj("безпіло\u{0301}тний"),
            WordSpec::adj("літа\u{0301}льний"),
            WordSpec::noun("апара\u{0301}т", Gender::Masculine),
        ],
    });
    
    // ЗРК
    m.insert("зрк", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::adj("зені\u{0301}тно-раке\u{0301}тний"),
            WordSpec::noun("ко\u{0301}мплекс", Gender::Masculine),
        ],
    });
    
    // МВС
    m.insert("мвс", AbbrSpec {
        gender_class: "імен",
        words: vec![
            WordSpec::noun("міністе\u{0301}рство", Gender::Neuter),
            WordSpec::static_word("вну\u{0301}трішніх"),
            WordSpec::static_word("справ"),
        ],
    });
    
    // ГУР
    m.insert("гур", AbbrSpec {
        gender_class: "імен",
        words: vec![
            WordSpec::adj("голо\u{0301}вне"),
            WordSpec::noun("управлі\u{0301}ння", Gender::Neuter),
            WordSpec::static_word("ро\u{0301}звідки"),
        ],
    });
    
    // МОЗ
    m.insert("моз", AbbrSpec {
        gender_class: "імен",
        words: vec![
            WordSpec::noun("міністе\u{0301}рство", Gender::Neuter),
            WordSpec::static_word("охоро\u{0301}ни"),
            WordSpec::static_word("здоро\u{0301}в'я"),
            WordSpec::static_word("Украї\u{0301}ни"),
        ],
    });
    
    // МОН
    m.insert("мон", AbbrSpec {
        gender_class: "імен",
        words: vec![
            WordSpec::noun("міністе\u{0301}рство", Gender::Neuter),
            WordSpec::static_word("осві\u{0301}ти"),
            WordSpec::static_word("і"),
            WordSpec::static_word("нау\u{0301}ки"),
            WordSpec::static_word("Украї\u{0301}ни"),
        ],
    });
    
    // НАБУ
    m.insert("набу", AbbrSpec {
        gender_class: "імен",
        words: vec![
            WordSpec::adj("націона\u{0301}льне"),
            WordSpec::adj("антикорупці\u{0301}йне"),
            WordSpec::static_word("бюро\u{0301}"),
            WordSpec::static_word("Украї\u{0301}ни"),
        ],
    });
    
    // ДБР
    m.insert("дбр", AbbrSpec {
        gender_class: "імен",
        words: vec![
            WordSpec::adj("держа\u{0301}вне"),
            WordSpec::static_word("бюро\u{0301}"),
            WordSpec::static_word("розслі\u{0301}дувань"),
        ],
    });
    
    // ЗСУ
    m.insert("зсу", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("збро\u{0301}йні"),
            WordSpec::noun_pl("си\u{0301}ли", Gender::Feminine),
            WordSpec::static_word("Украї\u{0301}ни"),
        ],
    });
    
    // США
    m.insert("сша", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::adj("сполу\u{0301}чені"),
            WordSpec::noun_pl("шта\u{0301}ти", Gender::Masculine),
            WordSpec::static_word("аме\u{0301}рики"),
        ],
    });
    
    // ЗМІ
    m.insert("змі", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::noun_pl("за\u{0301}соби", Gender::Masculine),
            WordSpec::static_word("ма\u{0301}сової"),
            WordSpec::static_word("інформа\u{0301}ції"),
        ],
    });
    
    // КМУ
    m.insert("кму", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::noun("Кабіне\u{0301}т", Gender::Masculine),
            WordSpec::static_word("Міні\u{0301}стрів"),
            WordSpec::static_word("Украї\u{0301}ни"),
        ],
    });

    // ОП
    m.insert("оп", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::noun("О\u{0301}фіс", Gender::Masculine),
            WordSpec::static_word("Президе\u{0301}нта"),
        ],
    });

    // ОВА
    m.insert("ова", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("обласна\u{0301}"),
            WordSpec::adj("військо\u{0301}ва"),
            WordSpec::noun("адміністра\u{0301}ція", Gender::Feminine),
        ],
    });

    // РВА
    m.insert("рва", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("районна\u{0301}"),
            WordSpec::adj("військо\u{0301}ва"),
            WordSpec::noun("адміністра\u{0301}ція", Gender::Feminine),
        ],
    });

    // МВА
    m.insert("мва", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("міська\u{0301}"),
            WordSpec::adj("військо\u{0301}ва"),
            WordSpec::noun("адміністра\u{0301}ція", Gender::Feminine),
        ],
    });

    // КМДА
    m.insert("кмда", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("Ки\u{0301}ївська"),
            WordSpec::adj("міська\u{0301}"),
            WordSpec::adj("держа\u{0301}вна"),
            WordSpec::noun("адміністра\u{0301}ція", Gender::Feminine),
        ],
    });

    // КМВА
    m.insert("кмва", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("Ки\u{0301}ївська"),
            WordSpec::adj("міська\u{0301}"),
            WordSpec::adj("військо\u{0301}ва"),
            WordSpec::noun("адміністра\u{0301}ція", Gender::Feminine),
        ],
    });

    // ГШ
    m.insert("гш", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::adj("Генера\u{0301}льний"),
            WordSpec::noun("штаб", Gender::Masculine),
        ],
    });

    // НПУ
    m.insert("нпу", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("Націона\u{0301}льна"),
            WordSpec::noun("полі\u{0301}ція", Gender::Feminine),
            WordSpec::static_word("Украї\u{0301}ни"),
        ],
    });

    // КП
    m.insert("кп", AbbrSpec {
        gender_class: "яблук",
        words: vec![
            WordSpec::adj("комуна\u{0301}льне"),
            WordSpec::noun("підпри\u{0301}ємство", Gender::Neuter),
        ],
    });

    // ОТГ
    m.insert("отг", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("об'єдна\u{0301}на"),
            WordSpec::adj("територіа\u{0301}льна"),
            WordSpec::noun("грома\u{0301}да", Gender::Feminine),
        ],
    });

    // ОСББ
    m.insert("осбб", AbbrSpec {
        gender_class: "яблук",
        words: vec![
            WordSpec::noun("об'є\u{0301}днання", Gender::Neuter),
            WordSpec::static_word("співвла\u{0301}сників"),
            WordSpec::static_word("багатокварти\u{0301}рного"),
            WordSpec::static_word("буди\u{0301}нку"),
        ],
    });

    // ФОП
    m.insert("фоп", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("фізи\u{0301}чна"),
            WordSpec::noun("осо\u{0301}ба", Gender::Feminine),
            WordSpec::noun("підприє\u{0301}мець", Gender::Masculine),
        ],
    });

    // ТОВ
    m.insert("тов", AbbrSpec {
        gender_class: "яблук",
        words: vec![
            WordSpec::noun("товари\u{0301}ство", Gender::Neuter),
            WordSpec::static_word("з"),
            WordSpec::static_word("обме\u{0301}женою"),
            WordSpec::static_word("відповіда\u{0301}льністю"),
        ],
    });

    // НЕК
    m.insert("нек", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("націона\u{0301}льна"),
            WordSpec::adj("енергети\u{0301}чна"),
            WordSpec::noun("компа\u{0301}нія", Gender::Feminine),
        ],
    });

    // ЦВК
    m.insert("цвк", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("Центра\u{0301}льна"),
            WordSpec::adj("ви\u{0301}борча"),
            WordSpec::noun("комі\u{0301}сія", Gender::Feminine),
        ],
    });

    // ВПО
    m.insert("впо", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::static_word("внутрі\u{0301}шньо"),
            WordSpec::adj("перемі\u{0301}щена"),
            WordSpec::noun("осо\u{0301}ба", Gender::Feminine),
        ],
    });

    // ЗВО
    m.insert("зво", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::noun("за\u{0301}клад", Gender::Masculine),
            WordSpec::static_word("ви\u{0301}щої"),
            WordSpec::static_word("осві\u{0301}ти"),
        ],
    });

    // ВНЗ
    m.insert("внз", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::adj("ви\u{0301}щий"),
            WordSpec::adj("навча\u{0301}льний"),
            WordSpec::noun("за\u{0301}клад", Gender::Masculine),
        ],
    });

    // НАНУ
    m.insert("нану", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("Націона\u{0301}льна"),
            WordSpec::noun("акаде\u{0301}мія", Gender::Feminine),
            WordSpec::static_word("нау\u{0301}к"),
            WordSpec::static_word("Украї\u{0301}ни"),
        ],
    });

    // МВФ
    m.insert("мвф", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::adj("Міжнаро\u{0301}дний"),
            WordSpec::adj("валю\u{0301}тний"),
            WordSpec::noun("фонд", Gender::Masculine),
        ],
    });

    // ОБСЄ
    m.insert("обсє", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::noun("організа\u{0301}ція", Gender::Feminine),
            WordSpec::static_word("з"),
            WordSpec::static_word("безпе\u{0301}ки"),
            WordSpec::static_word("і"),
            WordSpec::static_word("співробі\u{0301}тництва"),
            WordSpec::static_word("в"),
            WordSpec::static_word("Євро\u{0301}пі"),
        ],
    });

    // ВМС
    m.insert("вмс", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("Військо\u{0301}во-морські\u{0301}"),
            WordSpec::noun_pl("си\u{0301}ли", Gender::Feminine),
        ],
    });

    // ПС
    m.insert("пс", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("Пові\u{0301}тряні"),
            WordSpec::noun_pl("си\u{0301}ли", Gender::Feminine),
        ],
    });

    // ССО
    m.insert("ссо", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::noun_pl("си\u{0301}ли", Gender::Feminine),
            WordSpec::static_word("спеціа\u{0301}льних"),
            WordSpec::static_word("опера\u{0301}цій"),
        ],
    });

    // ТрО
    m.insert("тро", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("територіа\u{0301}льна"),
            WordSpec::noun("оборо\u{0301}на", Gender::Feminine),
        ],
    });

    // ПТРК
    m.insert("птрк", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::adj("протита\u{0301}нковий"),
            WordSpec::adj("раке\u{0301}тний"),
            WordSpec::noun("ко\u{0301}мплекс", Gender::Masculine),
        ],
    });

    // ПЗРК
    m.insert("пзрк", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::adj("переносни\u{0301}й"),
            WordSpec::adj("зені\u{0301}тно-раке\u{0301}тний"),
            WordSpec::noun("ко\u{0301}мплекс", Gender::Masculine),
        ],
    });

    // РСЗВ
    m.insert("рсзв", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("реакти\u{0301}вна"),
            WordSpec::noun("систе\u{0301}ма", Gender::Feminine),
            WordSpec::static_word("за\u{0301}лпового"),
            WordSpec::static_word("вогню\u{0301}"),
        ],
    });

    // БТР
    m.insert("бтр", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::noun("бронетранспорте\u{0301}р", Gender::Masculine),
        ],
    });

    // БМП
    m.insert("бмп", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("бойова\u{0301}"),
            WordSpec::noun("маши\u{0301}на", Gender::Feminine),
        ],
    });

    // ДШВ
    m.insert("дшв", AbbrSpec {
        gender_class: "яблук",
        words: vec![
            WordSpec::adj("деса\u{0301}нтно-штурмові\u{0301}"),
            WordSpec::noun_pl("війська\u{0301}", Gender::Neuter),
        ],
    });

    // ЧАЕС
    m.insert("чаес", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("Чорно\u{0301}бильська"),
            WordSpec::adj("а\u{0301}томна"),
            WordSpec::noun("електроста\u{0301}нція", Gender::Feminine),
        ],
    });

    // ЗАЕС
    m.insert("заес", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("Запорі\u{0301}зька"),
            WordSpec::adj("а\u{0301}томна"),
            WordSpec::noun("електроста\u{0301}нція", Gender::Feminine),
        ],
    });

    // ХАЕС
    m.insert("хаес", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("Хмельни\u{0301}цька"),
            WordSpec::adj("а\u{0301}томна"),
            WordSpec::noun("електроста\u{0301}нція", Gender::Feminine),
        ],
    });

    // РАЕС
    m.insert("раес", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("Рі\u{0301}вненська"),
            WordSpec::adj("а\u{0301}томна"),
            WordSpec::noun("електроста\u{0301}нція", Gender::Feminine),
        ],
    });

    // УЗ
    m.insert("уз", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::noun("Укрзалізни\u{0301}ця", Gender::Feminine),
        ],
    });

    // ПЦУ
    m.insert("пцу", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("Правосла\u{0301}вна"),
            WordSpec::noun("це\u{0301}рква", Gender::Feminine),
            WordSpec::static_word("Украї\u{0301}ни"),
        ],
    });

    // ДТП
    m.insert("дтп", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("доро\u{0301}жньо-тра\u{0301}нспортна"),
            WordSpec::noun("приго\u{0301}да", Gender::Feminine),
        ],
    });

    // ГРВІ
    m.insert("грві", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("го\u{0301}стра"),
            WordSpec::adj("респірато\u{0301}рна"),
            WordSpec::adj("ві\u{0301}русна"),
            WordSpec::noun("інфе\u{0301}кція", Gender::Feminine),
        ],
    });

    // ІТ
    m.insert("іт", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("інформаці\u{0301}йні"),
            WordSpec::noun_pl("техноло\u{0301}гії", Gender::Feminine),
        ],
    });

    // СНІД
    m.insert("снід", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::noun("синдро\u{0301}м", Gender::Masculine),
            WordSpec::static_word("набу\u{0301}того"),
            WordSpec::static_word("імунодефіци\u{0301}ту"),
        ],
    });

    // ВІЛ
    m.insert("віл", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::noun("ві\u{0301}рус", Gender::Masculine),
            WordSpec::static_word("імунодефіци\u{0301}ту"),
            WordSpec::static_word("люди\u{0301}ни"),
        ],
    });

    // НАТО
    m.insert("нато", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::noun("організа\u{0301}ція", Gender::Feminine),
            WordSpec::static_word("Північноатланти\u{0301}чного"),
            WordSpec::static_word("догово\u{0301}ру"),
        ],
    });

    // НГУ
    m.insert("нгу", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("Націона\u{0301}льна"),
            WordSpec::noun("гва\u{0301}рдія", Gender::Feminine),
            WordSpec::static_word("Украї\u{0301}ни"),
        ],
    });

    // САУ
    m.insert("сау", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("самохі\u{0301}дна"),
            WordSpec::adj("артилері\u{0301}йська"),
            WordSpec::noun("устано\u{0301}вка", Gender::Feminine),
        ],
    });

    // ББМ
    m.insert("ббм", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("бойова\u{0301}"),
            WordSpec::adj("броньо\u{0301}вана"),
            WordSpec::noun("маши\u{0301}на", Gender::Feminine),
        ],
    });

    // МЗС
    m.insert("мзс", AbbrSpec {
        gender_class: "імен",
        words: vec![
            WordSpec::noun("міністе\u{0301}рство", Gender::Neuter),
            WordSpec::static_word("закордо\u{0301}нних"),
            WordSpec::static_word("справ"),
        ],
    });

    // ДПС
    m.insert("дпс", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("держа\u{0301}вна"),
            WordSpec::adj("податко\u{0301}ва"),
            WordSpec::noun("слу\u{0301}жба", Gender::Feminine),
        ],
    });

    // ДМС
    m.insert("дмс", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("держа\u{0301}вна"),
            WordSpec::adj("міграці\u{0301}йна"),
            WordSpec::noun("слу\u{0301}жба", Gender::Feminine),
        ],
    });

    // НАЗК
    m.insert("назк", AbbrSpec {
        gender_class: "імен",
        words: vec![
            WordSpec::adj("націона\u{0301}льне"),
            WordSpec::noun("аге\u{0301}нтство", Gender::Neuter),
            WordSpec::static_word("з"),
            WordSpec::static_word("пита\u{0301}нь"),
            WordSpec::static_word("запобіга\u{0301}ння"),
            WordSpec::static_word("кору\u{0301}пції"),
        ],
    });

    // АРМА
    m.insert("арма", AbbrSpec {
        gender_class: "імен",
        words: vec![
            WordSpec::noun("аге\u{0301}нтство", Gender::Neuter),
            WordSpec::static_word("з"),
            WordSpec::static_word("ро\u{0301}зшуку"),
            WordSpec::static_word("та"),
            WordSpec::static_word("мене\u{0301}джменту"),
            WordSpec::static_word("акти\u{0301}вів"),
        ],
    });

    // САП
    m.insert("сап", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("спеціалізо\u{0301}вана"),
            WordSpec::adj("антикорупці\u{0301}йна"),
            WordSpec::noun("прокурату\u{0301}ра", Gender::Feminine),
        ],
    });

    // ОГП
    m.insert("огп", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::noun("о\u{0301}фіс", Gender::Masculine),
            WordSpec::static_word("Генера\u{0301}льного"),
            WordSpec::static_word("прокуро\u{0301}ра"),
        ],
    });

    // УДО
    m.insert("удо", AbbrSpec {
        gender_class: "імен",
        words: vec![
            WordSpec::noun("управлі\u{0301}ння", Gender::Neuter),
            WordSpec::static_word("держа\u{0301}вної"),
            WordSpec::static_word("охоро\u{0301}ни"),
        ],
    });

    // АМКУ
    m.insert("амку", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::adj("антимонопо\u{0301}льний"),
            WordSpec::noun("коміте\u{0301}т", Gender::Masculine),
            WordSpec::static_word("Украї\u{0301}ни"),
        ],
    });

    // ПФУ
    m.insert("пфу", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::adj("пенсі\u{0301}йний"),
            WordSpec::noun("фонд", Gender::Masculine),
            WordSpec::static_word("Украї\u{0301}ни"),
        ],
    });

    // ФДМУ
    m.insert("фдму", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::noun("фонд", Gender::Masculine),
            WordSpec::static_word("держа\u{0301}вного"),
            WordSpec::static_word("ма\u{0301}йна"),
            WordSpec::static_word("Украї\u{0301}ни"),
        ],
    });

    // ПрАТ
    m.insert("прат", AbbrSpec {
        gender_class: "імен",
        words: vec![
            WordSpec::adj("прива\u{0301}тне"),
            WordSpec::adj("акціоне\u{0301}рне"),
            WordSpec::noun("товари\u{0301}ство", Gender::Neuter),
        ],
    });

    // АТ
    m.insert("ат", AbbrSpec {
        gender_class: "імен",
        words: vec![
            WordSpec::adj("акціоне\u{0301}рне"),
            WordSpec::noun("товари\u{0301}ство", Gender::Neuter),
        ],
    });

    // ПАТ
    m.insert("пат", AbbrSpec {
        gender_class: "імен",
        words: vec![
            WordSpec::adj("публі\u{0301}чне"),
            WordSpec::adj("акціоне\u{0301}рне"),
            WordSpec::noun("товари\u{0301}ство", Gender::Neuter),
        ],
    });

    // ЖБК
    m.insert("жбк", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::adj("житло\u{0301}во-будіве\u{0301}льний"),
            WordSpec::noun("кооперати\u{0301}в", Gender::Masculine),
        ],
    });

    // КВЕД
    m.insert("квед", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::noun("класифіка\u{0301}тор", Gender::Masculine),
            WordSpec::static_word("ви\u{0301}дів"),
            WordSpec::static_word("економі\u{0301}чної"),
            WordSpec::static_word("дія\u{0301}льності"),
        ],
    });

    // ПДВ
    m.insert("пдв", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::noun("пода\u{0301}ток", Gender::Masculine),
            WordSpec::static_word("на"),
            WordSpec::static_word("до\u{0301}дану"),
            WordSpec::static_word("ва\u{0301}ртість"),
        ],
    });

    // ЄСВ
    m.insert("єсв", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::adj("єди\u{0301}ний"),
            WordSpec::adj("соціа\u{0301}льний"),
            WordSpec::noun("вне\u{0301}сок", Gender::Masculine),
        ],
    });

    // ПДФО
    m.insert("пдфо", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::noun("пода\u{0301}ток", Gender::Masculine),
            WordSpec::static_word("на"),
            WordSpec::static_word("дохо\u{0301}ди"),
            WordSpec::static_word("фізи\u{0301}чних"),
            WordSpec::static_word("осі\u{0301}б"),
        ],
    });

    // РРО
    m.insert("рро", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::noun("реєстра\u{0301}тор", Gender::Masculine),
            WordSpec::static_word("розрахунко\u{0301}вих"),
            WordSpec::static_word("опера\u{0301}цій"),
        ],
    });

    // ПРРО
    m.insert("прро", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::adj("програ\u{0301}мний"),
            WordSpec::noun("реєстра\u{0301}тор", Gender::Masculine),
            WordSpec::static_word("розрахунко\u{0301}вих"),
            WordSpec::static_word("опера\u{0301}цій"),
        ],
    });

    // МФО
    m.insert("мфо", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::adj("міжфілі\u{0301}йний"),
            WordSpec::noun("оборо\u{0301}т", Gender::Masculine),
        ],
    });

    // РАЦС
    m.insert("рацс", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::noun("реєстра\u{0301}ція", Gender::Feminine),
            WordSpec::static_word("акти\u{0301}в"),
            WordSpec::static_word("циві\u{0301}льного"),
            WordSpec::static_word("ста\u{0301}ну"),
        ],
    });

    // ВООЗ
    m.insert("вооз", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("Всесві\u{0301}тня"),
            WordSpec::noun("організа\u{0301}ція", Gender::Feminine),
            WordSpec::static_word("охоро\u{0301}ни"),
            WordSpec::static_word("здоро\u{0301}в'я"),
        ],
    });

    // МАГАТЕ
    m.insert("магате", AbbrSpec {
        gender_class: "імен",
        words: vec![
            WordSpec::adj("Міжнаро\u{0301}дне"),
            WordSpec::noun("аге\u{0301}нтство", Gender::Neuter),
            WordSpec::static_word("з"),
            WordSpec::static_word("а\u{0301}томної"),
            WordSpec::static_word("енерге\u{0301}тики"),
        ],
    });

    // ЮНЕСКО
    m.insert("юнеско", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::noun("організа\u{0301}ція", Gender::Feminine),
            WordSpec::static_word("Об'є\u{0301}днаних"),
            WordSpec::static_word("На\u{0301}цій"),
            WordSpec::static_word("з"),
            WordSpec::static_word("пита\u{0301}нь"),
            WordSpec::static_word("осві\u{0301}ти,"),
            WordSpec::static_word("нау\u{0301}ки"),
            WordSpec::static_word("і"),
            WordSpec::static_word("культу\u{0301}ри"),
        ],
    });

    // МКЧХ
    m.insert("мкчх", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::adj("Міжнаро\u{0301}дний"),
            WordSpec::noun("коміте\u{0301}т", Gender::Masculine),
            WordSpec::static_word("Черво\u{0301}ного"),
            WordSpec::static_word("Хреста\u{0301}"),
        ],
    });

    // МКС
    m.insert("мкс", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::adj("Міжнаро\u{0301}дний"),
            WordSpec::adj("криміна\u{0301}льний"),
            WordSpec::noun("суд", Gender::Masculine),
        ],
    });

    // ШМД
    m.insert("шмд", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("швидка\u{0301}"),
            WordSpec::adj("меди\u{0301}чна"),
            WordSpec::noun("допомо\u{0301}га", Gender::Feminine),
        ],
    });

    // МСЕК
    m.insert("мсек", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("меди\u{0301}ко-соціа\u{0301}льна"),
            WordSpec::adj("експе\u{0301}ртна"),
            WordSpec::noun("комі\u{0301}сія", Gender::Feminine),
        ],
    });

    // ВВК
    m.insert("ввк", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("військо\u{0301}во-лі\u{0301}карська"),
            WordSpec::noun("комі\u{0301}сія", Gender::Feminine),
        ],
    });

    // НСЗУ
    m.insert("нсзу", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("Націона\u{0301}льна"),
            WordSpec::noun("слу\u{0301}жба", Gender::Feminine),
            WordSpec::static_word("здоро\u{0301}в'я"),
            WordSpec::static_word("Украї\u{0301}ни"),
        ],
    });

    // ГРЗ
    m.insert("грз", AbbrSpec {
        gender_class: "імен",
        words: vec![
            WordSpec::adj("го\u{0301}стре"),
            WordSpec::adj("респірато\u{0301}рне"),
            WordSpec::noun("захво\u{0301}рювання", Gender::Neuter),
        ],
    });

    // ЗПСШ
    m.insert("зпсш", AbbrSpec {
        gender_class: "імен",
        words: vec![
            WordSpec::noun_pl("захво\u{0301}рювання", Gender::Neuter),
            WordSpec::static_word("що"),
            WordSpec::static_word("передаю\u{0301}ться"),
            WordSpec::static_word("стате\u{0301}вим"),
            WordSpec::static_word("шля\u{0301}хом"),
        ],
    });

    // КТ
    m.insert("кт", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("комп'ю\u{0301}терна"),
            WordSpec::noun("томогра\u{0301}фія", Gender::Feminine),
        ],
    });

    // МРТ
    m.insert("мрт", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("магні\u{0301}тно-резона\u{0301}нсна"),
            WordSpec::noun("томогра\u{0301}фія", Gender::Feminine),
        ],
    });

    // УЗД
    m.insert("узд", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("ультразвукова\u{0301}"),
            WordSpec::noun("діагно\u{0301}стика", Gender::Feminine),
        ],
    });

    // ЕКГ
    m.insert("екг", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::noun("електрокардіогра\u{0301}ма", Gender::Feminine),
        ],
    });

    // ЗНО
    m.insert("зно", AbbrSpec {
        gender_class: "імен",
        words: vec![
            WordSpec::adj("зо\u{0301}внішнє"),
            WordSpec::adj("незале\u{0301}жне"),
            WordSpec::noun("оціню\u{0301}вання", Gender::Neuter),
        ],
    });

    // НМТ
    m.insert("нмт", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::adj("націона\u{0301}льний"),
            WordSpec::adj("мультипредме\u{0301}тний"),
            WordSpec::noun("тест", Gender::Masculine),
        ],
    });

    // НУШ
    m.insert("нуш", AbbrSpec {
        gender_class: "гривень",
        words: vec![
            WordSpec::adj("Но\u{0301}ва"),
            WordSpec::adj("украї\u{0301}нська"),
            WordSpec::noun("шко\u{0301}ла", Gender::Feminine),
        ],
    });

    // ЖКГ
    m.insert("жкг", AbbrSpec {
        gender_class: "імен",
        words: vec![
            WordSpec::adj("житло\u{0301}во-комуна\u{0301}льне"),
            WordSpec::noun("господа\u{0301}рство", Gender::Neuter),
        ],
    });

    // ЖК
    m.insert("жк", AbbrSpec {
        gender_class: "поверхів",
        words: vec![
            WordSpec::adj("житлови\u{0301}й"),
            WordSpec::noun("ко\u{0301}мплекс", Gender::Masculine),
        ],
    });

    // СМС
    m.insert("смс", AbbrSpec {
        gender_class: "імен",
        words: vec![
            WordSpec::adj("коро\u{0301}тке"),
            WordSpec::adj("тексто\u{0301}ве"),
            WordSpec::noun("повідо\u{0301}млення", Gender::Neuter),
        ],
    });

    // ПІБ
    m.insert("піб", AbbrSpec {
        gender_class: "імен",
        words: vec![
            WordSpec::noun("прі\u{0301}звище", Gender::Neuter),
            WordSpec::static_word("ім'я\u{0301}"),
            WordSpec::static_word("по"),
            WordSpec::static_word("ба\u{0301}тькові"),
        ],
    });

    m
});

/// Перевіряє, чи є слово прийменником.
pub fn is_preposition(word: &str) -> bool {
    matches!(word.to_lowercase().as_str(), 
        "о" | "об" | "на" | "при" |
        "до" | "від" | "після" | "без" | "біля" | "понад" | "з" | "зі" | "із" |
        "для" | "коло" | "поблизу" | "серед" | "навколо" | "проти" |
        "у" | "в" | "за" | "через" | "про" | "під" |
        "над" | "перед" | "між" | "разом" |
        "назустріч" | "всупереч" | "завдяки" | "відповідно"
    )
}

/// Повертає відмінок за прийменником.
pub fn get_declension(prep: Option<&str>) -> Declension {
    let p = match prep {
        Some(val) => val.to_lowercase(),
        None => return Declension::Nominative,
    };
    match p.as_str() {
        "о" | "об" | "на" | "при" => Declension::Locative,
        "до" | "від" | "після" | "без" | "біля" | "понад" | "з" | "зі" | "із"
        | "для" | "коло" | "поблизу" | "серед" | "навколо" | "проти" => Declension::Genitive,
        "у" | "в" | "за" | "через" | "про" | "під" => Declension::Accusative,
        "над" | "перед" | "між" | "разом" => Declension::Instrumental,
        "назустріч" | "всупереч" | "завдяки" | "відповідно" => Declension::Dative,
        _ => Declension::Nominative,
    }
}

/// Визначає відмінок та число на основі контексту (попереднього слова та прийменника перед ним).
pub fn deduce_declension_and_number(
    prev_word: Option<&str>,
    before_prev: Option<&str>,
) -> (Declension, bool) {
    let prev_lower = prev_word.map(|w| w.to_lowercase()).unwrap_or_default();
    
    if let Ok(num) = prev_lower.parse::<i64>() {
        let before_prev_lower = before_prev.map(|w| w.to_lowercase());
        let prep = before_prev_lower.as_deref();
        let decl = get_declension(prep);
        
        let last_digit = num % 10;
        let last_two = num % 100;
        
        if last_two >= 11 && last_two <= 19 {
            let final_decl = if prep.is_some() { decl } else { Declension::Genitive };
            return (final_decl, true);
        } else if last_digit == 1 {
            return (decl, false);
        } else if last_digit >= 2 && last_digit <= 4 {
            let final_decl = if prep.is_some() { decl } else { Declension::Nominative };
            return (final_decl, true);
        } else {
            let final_decl = if prep.is_some() { decl } else { Declension::Genitive };
            return (final_decl, true);
        }
    }
    
    if matches!(prev_lower.as_str(), 
        "один" | "одна" | "одне" | "одного" | "однієї" | "одному" | "одній" | "одним" | "однією" |
        "два" | "дві" | "двох" | "двом" | "двома" |
        "три" | "трьох" | "трьом" | "трьома" |
        "чотири" | "чотирьох" | "чотирьом" | "чотирма" |
        "п'ять" | "пʼять" | "п'яти" | "пʼяти" | "п'ятьма" | "пʼятьма" |
        "шість" | "шести" | "сім" | "семи" | "вісім" | "восьми" | "дев'ять" | "девʼять" | "дев'яти" | "девʼяти" |
        "десять" | "десяти" | "одинадцять" | "одинадцяти" | "дванадцять" | "дванадцяти" |
        "двадцять" | "двадцяти" | "тридцять" | "тридцяти" | "сорок" | "сорока" |
        "п'ятдесят" | "пʼятдесят" | "п'ятдесяти" | "пʼятдесяти" | "шістдесят" | "шістдесяти" |
        "сімдесят" | "сімдесяти" | "вісімдесят" | "вісімдесяти" | "дев'яносто" | "девʼяносто" | "дев'яноста" | "девʼяноста" |
        "сто" | "ста" | "двісті" | "двохсот" | "триста" | "трьохсот" | "чотириста" | "чотирьохсот" |
        "п'ятсот" | "пʼятсот" | "п'ятисот" | "пʼятисот" |
        "тисяча" | "тисячі" | "тисяч" | "мільйон" | "мільйони" | "мільйонів" | "мільярд" | "мільярди" | "мільярдів"
    ) {
        let before_prev_lower = before_prev.map(|w| w.to_lowercase());
        let prep = before_prev_lower.as_deref();
        let decl = if prep.is_some() { get_declension(prep) } else { Declension::Genitive };
        
        let is_plural = !matches!(prev_lower.as_str(), "один" | "одна" | "одне" | "одного" | "однієї" | "одному" | "одній" | "одним" | "однією");
        return (decl, is_plural);
    }
    
    if is_preposition(&prev_lower) {
        let mut decl = get_declension(Some(&prev_lower));
        if prev_lower == "на" {
            if let Some(bp) = before_prev {
                let bp_lower = bp.to_lowercase();
                if bp_lower.ends_with("ння") || bp_lower.ends_with("ня") || matches!(bp_lower.as_str(), "скарга" | "план" | "напад" | "вплив" | "перевірка" | "дослідження" | "відповідь" | "посилання") {
                    decl = Declension::Accusative;
                }
            }
        }
        return (decl, false);
    }
    
    if matches!(prev_lower.as_str(),
        "рішення" | "заява" | "указ" | "постанова" | "наказ" | "лист" | "рапорт" |
        "закон" | "код" | "стаття" | "вимога" | "орган" | "центр" | "офіс" | "служба" | "рада" |
        "заступник" | "начальник" | "керівник" | "працівник" | "радник" | "директор" | "інспектор" |
        "прокурор" | "президент" | "голова" | "депутат" | "секретар" | "міністр" | "агент" |
        "бійці" | "сили" | "підрозділи" | "командування" | "керівництво" | "представник" |
        "співробітник" | "співробітники" | "представники" | "воїни" | "солдати" | "офіцери" |
        "підрозділ" | "агентство" | "установа" | "організація" | "комісія" | "комітет" | "фонд" | "товариство" | "гвардія" |
        "сплата" | "оплата" | "результати"
    ) || prev_lower.ends_with("ння") || prev_lower.ends_with("ство") || prev_lower.ends_with("ник") {
        return (Declension::Genitive, false);
    }
    
    let mut decl = Declension::Nominative;
    let mut is_plural = false;
    
    if !prev_lower.is_empty() {
        if prev_lower.ends_with("ого") {
            decl = Declension::Genitive;
        } else if prev_lower.ends_with("ої") || prev_lower.ends_with("оі") {
            decl = Declension::Genitive;
        } else if prev_lower.ends_with("ому") {
            decl = Declension::Dative;
        } else if prev_lower.ends_with("ій") || prev_lower.ends_with("іі") {
            decl = Declension::Locative;
        } else if prev_lower.ends_with("им") {
            decl = Declension::Instrumental;
        } else if prev_lower.ends_with("ою") || prev_lower.ends_with("ею") || prev_lower.ends_with("єю") {
            decl = Declension::Instrumental;
        } else if prev_lower.ends_with("их") {
            decl = Declension::Genitive;
            is_plural = true;
        } else if prev_lower.ends_with("ими") {
            decl = Declension::Instrumental;
            is_plural = true;
        }
    }
    
    (decl, is_plural)
}

/// Контекстно розгортає відому абревіатуру з урахуванням відмінка та числа.
pub fn expand_abbr_contextual(
    abbr: &str,
    prev_word: Option<&str>,
    before_prev: Option<&str>,
) -> String {
    let lower = abbr.to_lowercase();
    let Some(spec) = ABBR_MAP.get(lower.as_str()) else {
        return abbr.to_string();
    };

    let (decl, is_plural) = deduce_declension_and_number(prev_word, before_prev);

    let main_noun = spec.words.iter().find(|w| w.role == WordRole::Noun);
    let (noun_gender, noun_plural) = match main_noun {
        Some(n) => (n.gender, is_plural || n.is_always_plural),
        None => (Gender::Masculine, is_plural),
    };

    let mut result_words = Vec::new();

    for w in &spec.words {
        match w.role {
            WordRole::Static => {
                result_words.push(w.text.to_string());
            }
            WordRole::Adj => {
                let declined = decline_adjective(w.text, decl, noun_plural, noun_gender);
                result_words.push(declined);
            }
            WordRole::Noun => {
                let rules = NounRules::build(w.text, w.gender, w.is_always_plural);
                let declined = decline_noun(&rules, decl, is_plural);
                result_words.push(declined);
            }
        }
    }

    result_words.join(" ")
}

/// Повертає базовий іменник для узгодження числівників у UkContext::analyze.
pub fn get_context_noun(next_word: &str) -> Option<&'static str> {
    let lower = next_word.to_lowercase();
    
    if let Some(spec) = ABBR_MAP.get(lower.as_str()) {
        return Some(spec.gender_class);
    }

    match lower.as_str() {
        "кг" | "kg" | "г" | "g" | "м" | "m" | "см" | "cm" | "мм" | "mm" | "км" | "km" | 
        "л" | "l" | "мл" | "ml" | "v" | "w" | "a" | "hz" | "°" | "°c" | "°f" | "°k" | "%" |
        "вт" | "вт-год" | "квт" | "квт-год" | "гц" | "кгц" | "мгц" | "ом" | "ком" => Some("поверхів"),
        _ => None,
    }
}

// ==================== Одиниці виміру (збережено для сумісності) ====================

pub static UNIT_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
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
    m.insert("°c", "градус цельсія");
    m.insert("°f", "градус фаренгейта");
    m.insert("°k", "кельвін");
    m.insert("°", "градус");
    m.insert("%", "відсоток");
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
    m.insert("unavailable", "недоступно");
    m.insert("unknown", "невідомо");
    m.insert("online", "у мережі");
    m.insert("offline", "поза мережею");
    m.insert("playing", "відтворюється");
    m.insert("paused", "призупинено");
    m.insert("idle", "очікування");
    m.insert("home", "вдома");
    m.insert("not_home", "не вдома");
    m.insert("away", "відсутній");
    m
});

pub fn find_unit(text: &str) -> Option<(usize, &'static str)> {
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

pub fn get_unit_form(last_word: &str, unit: &str, prep: Option<&str>) -> String {
    let last_clean = last_word.to_lowercase().replace('\u{0301}', "");
    let last_word_only = last_clean.split_whitespace().last().unwrap_or("").to_string();
    let prep_lower = prep.map(|p| p.to_lowercase());
    
    let mut decl = Declension::Nominative;
    if let Some(p) = &prep_lower {
        decl = get_declension(Some(p));
    } else {
        if matches!(last_word_only.as_str(), 
            "одного" | "однієї" | "двох" | "трьох" | "чотирьох" | "п'яти" | "пʼяти" | "шести" | "семи" | "восьми" | "дев'яти" | "девʼяти" | "десяти" |
            "одинадцяти" | "дванадцяти" | "двадцяти" | "тридцяти" | "сорока" | "п'ятдесяти" | "пʼятдесяти" | "шістдесяти" | "сімдесяти" |
            "вісімдесяти" | "дев'яноста" | "девʼяноста" | "ста" | "двохсот" | "трьохсот" | "чотирьохсот" | "п'ятисот" | "пʼятисот" |
            "тисячі" | "тисяч" | "мільйонів" | "мільярдів"
        ) {
            decl = Declension::Genitive;
        } else if matches!(last_word_only.as_str(),
            "одному" | "одній" | "двом" | "трьом" | "чотирьом" | "п'ятим" | "пʼятим"
        ) {
            decl = Declension::Dative;
        } else if matches!(last_word_only.as_str(),
            "одним" | "однією" | "двома" | "трьома" | "чотирма"
        ) {
            decl = Declension::Instrumental;
        }
    }

    let is_decimal = last_word_only.ends_with("десятих") 
        || last_word_only.ends_with("сотих") 
        || last_word_only.ends_with("тисячних") 
        || last_word_only.ends_with("мільйонних")
        || last_word_only.ends_with("десята") 
        || last_word_only.ends_with("сота") 
        || last_word_only.ends_with("тисячна") 
        || last_word_only.ends_with("мільйонна");

    let group = if matches!(last_word_only.as_str(), "один" | "одна" | "одне" | "одного" | "одній" | "одному" | "одним" | "однією") {
        1
    } else if matches!(last_word_only.as_str(), 
        "два" | "дві" | "три" | "чотири" | "двох" | "трьох" | "чотирьох" | "двом" | "трьом" | "чотирьом" | "двома" | "трьома" | "чотирма"
    ) {
        2
    } else {
        5
    };

    let (noun_text, gen_pl_ending, suffix_text) = if unit.starts_with("градус") {
        let suffix = if unit.contains("цельсія") {
            " цельсія"
        } else if unit.contains("фаренгейта") {
            " фаренгейта"
        } else {
            ""
        };
        ("градус", "ів", suffix)
    } else {
        let matched = match unit {
            "міліметр" => ("міліметр", "ів"),
            "сантиметр" => ("сантиметр", "ів"),
            "метр" => ("метр", "ів"),
            "кілометр" => ("кілометр", "ів"),
            "грам" => ("грам", "ів"),
            "кілограм" => ("кілограм", "ів"),
            "літр" => ("літр", "ів"),
            "мілілітр" => ("мілілітр", "ів"),
            "вольт" => ("вольт", ""),
            "мілівольт" => ("мілівольт", ""),
            "кіловольт" => ("кіловольт", ""),
            "ампер" => ("ампер", ""),
            "міліампер" => ("міліампер", ""),
            "ват" => ("ват", ""),
            "міліват" => ("міліват", ""),
            "кіловат" => ("кіловат", ""),
            "герц" => ("герц", ""),
            "паскаль" => ("паскаль", "ів"),
            "градус" => ("градус", "ів"),
            "відсоток" => ("відсоток", "ів"),
            _ => return unit.to_string(),
        };
        (matched.0, matched.1, "")
    };

    let rules = NounRules::build(noun_text, Gender::Masculine, false);
    
    let mut custom_rules = rules.clone();
    custom_rules.gen_pl_ending = gen_pl_ending.to_string();

    let declined = if is_decimal {
        decline_noun(&custom_rules, Declension::Genitive, false)
    } else if group == 1 {
        decline_noun(&custom_rules, decl, false)
    } else if group == 2 {
        let actual_decl = if decl == Declension::Nominative { Declension::Nominative } else { decl };
        decline_noun(&custom_rules, actual_decl, true)
    } else {
        let actual_decl = if decl == Declension::Nominative { Declension::Genitive } else { decl };
        decline_noun(&custom_rules, actual_decl, true)
    };

    if suffix_text.is_empty() {
        declined
    } else {
        format!("{}{}", declined, suffix_text)
    }
}

/// Перевіряє, чи є слово абревіатурою (всі великі літери, довжина від 2 до 5 символів).
pub fn is_abbreviation(word: &str) -> bool {
    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() || chars.len() == 1 {
        return false;
    }
    let all_uppercase = chars.iter().all(|c| c.is_uppercase());
    all_uppercase && chars.len() <= 5
}

/// Розширює абревіатуру в назви літер.
pub fn expand_abbreviation(word: &str) -> String {
    let mut parts = Vec::new();
    for c in word.chars() {
        let lower_c = c.to_lowercase().next().unwrap_or(c);
        let letter_name = match lower_c {
            'а' => "а", 'б' => "бе", 'в' => "ве", 'г' => "ге", 'ґ' => "ґе",
            'д' => "де", 'е' => "е", 'є' => "є", 'ж' => "же", 'з' => "зе",
            'и' => "и", 'і' => "і", 'ї' => "ї", 'й' => "йот", 'к' => "ка",
            'л' => "ел", 'м' => "ем", 'н' => "ен", 'о' => "о", 'п' => "пе",
            'р' => "ер", 'с' => "ес", 'т' => "те", 'у' => "у", 'ф' => "еф",
            'х' => "ха", 'ц' => "це", 'ч' => "че", 'ш' => "ша", 'щ' => "ща",
            'ь' => "м'який знак", 'ю' => "ю", 'я' => "я", 'ъ' => "твердий знак",
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
