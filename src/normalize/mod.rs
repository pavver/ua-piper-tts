pub mod types;
pub mod helpers;
pub mod preprocessor;
pub mod lexer;
pub mod parser;
pub mod generator;

use preprocessor::{step0_fix_paragraphs, preprocess_text};
use lexer::tokenize;
use parser::parse_context;
use generator::generate_text;

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

        // Thousand multipliers
        let mult1 = normalize_text("1.5к кг");
        println!("test_smart_home_scenarios mult1: {:?}", mult1);
        assert!(has_text(&mult1, "тисяча п'ятсот кілограмів"));

        let mult2 = normalize_text("2.5 к кг");
        println!("test_smart_home_scenarios mult2: {:?}", mult2);
        assert!(has_text(&mult2, "дві тисячі п'ятсот кілограмів"));

        let mult3 = normalize_text("1 к.");
        println!("test_smart_home_scenarios mult3: {:?}", mult3);
        assert!(has_text(&mult3, "тисяча"));
    }
}
