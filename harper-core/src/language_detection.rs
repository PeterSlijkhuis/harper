//! This module implements rudimentary, dictionary-based English language detection.

use strum::IntoEnumIterator;

use crate::spell::{Dictionary, FstDictionary};
use crate::{Document, Language, Token, TokenKind};

/// Check if the contents of the document are likely intended to represent
/// English.
pub fn is_doc_likely_english(doc: &Document, dict: &impl Dictionary) -> bool {
    is_likely_english(doc.get_tokens(), doc.get_source(), dict)
}

/// Check if given tokens are likely intended to represent English.
pub fn is_likely_english(toks: &[Token], source: &[char], dict: &impl Dictionary) -> bool {
    let mut total_words = 0;
    let mut valid_words = 0;
    let mut punctuation = 0;
    let mut unlintable = 0;

    for token in toks {
        match token.kind {
            TokenKind::Word(_) => {
                total_words += 1;

                let word_content = token.get_ch(source);
                if dict.contains_word(word_content) {
                    valid_words += 1;
                }
            }
            TokenKind::Punctuation(_) => punctuation += 1,
            TokenKind::Unlintable => unlintable += 1,
            _ => (),
        }
    }

    if total_words <= 7 && total_words - valid_words > 0 {
        return false;
    }

    if unlintable > valid_words {
        return false;
    }

    if (punctuation as f32 * 1.25) > valid_words as f32 {
        return false;
    }

    if (valid_words as f64 / total_words as f64) < 0.7 {
        return false;
    }

    true
}

const MIN_WORDS_TO_DETECT: usize = 3;

/// Fraction of words in `toks` that `dict` knows. `0.0` when there are no words.
fn valid_word_ratio(toks: &[Token], source: &[char], dict: &impl Dictionary) -> f64 {
    let (mut total, mut valid) = (0u32, 0u32);
    for token in toks.iter().filter(|t| t.kind.is_word()) {
        total += 1;
        valid += dict.contains_word(token.get_ch(source)) as u32;
    }
    if total == 0 {
        0.0
    } else {
        valid as f64 / total as f64
    }
}

/// Guess which supported language the given tokens are written in.
///
/// Every language whose dictionary passes [`is_likely_english`]'s heuristics is a candidate,
/// and the one that knows the largest share of the words wins. Returns `None` for code, other
/// languages and text too short to judge. Loads all three dictionaries on first use.
pub fn detect_language(toks: &[Token], source: &[char]) -> Option<Language> {
    // One or two words say nothing: "ja" is valid Dutch, Spanish-looking and German.
    if toks.iter().filter(|t| t.kind.is_word()).count() < MIN_WORDS_TO_DETECT {
        return None;
    }

    let mut best: Option<(Language, f64)> = None;
    for lang in Language::iter() {
        let dict = FstDictionary::curated_for(lang);
        if !is_likely_english(toks, source, &dict) {
            continue;
        }
        let ratio = valid_word_ratio(toks, source, &dict);
        // Strictly greater: on a tie the earlier language wins, so English beats the
        // others for the English loanwords they also contain.
        if best.is_none_or(|(_, r)| ratio > r) {
            best = Some((lang, ratio));
        }
    }
    best.map(|(lang, _)| lang)
}

/// [`detect_language`] for a whole document.
pub fn detect_doc_language(doc: &Document) -> Option<Language> {
    detect_language(doc.get_tokens(), doc.get_source())
}

#[cfg(test)]
mod tests {
    use super::is_doc_likely_english;
    use crate::Document;
    use crate::spell::FstDictionary;

    fn assert_not_english(source: &'static str) {
        let dict = FstDictionary::curated();
        let doc = Document::new_plain_english(source, &dict);
        let is_likely_english = is_doc_likely_english(&doc, &dict);
        dbg!(source);
        assert!(!is_likely_english);
    }

    fn assert_english(source: &'static str) {
        let dict = FstDictionary::curated();
        let doc = Document::new_plain_english(source, &dict);
        let is_likely_english = is_doc_likely_english(&doc, &dict);
        dbg!(source);
        assert!(is_likely_english);
    }

    #[test]
    fn detects_spanish() {
        assert_not_english("Esto es español. Harper no debería marcarlo como inglés.");
    }

    #[test]
    fn detects_french() {
        assert_not_english(
            "C'est du français. Il ne devrait pas être marqué comme anglais par Harper.",
        );
    }

    #[test]
    fn detects_shebang() {
        assert_not_english("#! /bin/bash");
        assert_not_english("#! /usr/bin/fish");
    }

    #[test]
    fn detects_short_english() {
        assert_english("This is English!");
    }

    #[test]
    fn detects_english() {
        assert_english("This is perfectly valid English, evn if it has a cople typos.")
    }

    #[test]
    fn detects_expressive_english() {
        assert_english("Look above! That is real English! So is this: bippity bop!")
    }

    /// Useful for detecting commented-out code.
    #[test]
    fn detects_python_fib() {
        assert_not_english(
            r"
def fibIter(n):
    if n < 2:
        return n
    fibPrev = 1
    fib = 1
    for _ in range(2, n):
        fibPrev, fib = fib, fib + fibPrev
    return fib
        ",
        );
    }

    #[test]
    fn mixed_french_english_park() {
        assert_not_english("Je voudrais promener au the park a huit heures with ma voisine");
    }

    #[test]
    fn mixed_french_english_drunk() {
        assert_not_english("Je ne suis pas drunk, je suis only ivre by you");
    }

    #[test]
    fn mixed_french_english_dress() {
        assert_not_english(
            "Je buy une robe nouveau chaque Tuesday, mais aujourd'hui, je don't have temps",
        );
    }

    #[test]
    fn english_motto() {
        assert_english("I have a simple motto in life");
    }

    #[test]
    fn detects_supported_languages() {
        use super::detect_doc_language;
        use crate::Language;

        let detect = |text: &str| {
            let dict = FstDictionary::curated();
            detect_doc_language(&Document::new_plain_english(text, &dict))
        };

        assert_eq!(
            detect("This is perfectly valid English, evn if it has a cople typos."),
            Some(Language::English)
        );
        assert_eq!(
            detect("Dit is een Nederlandse zin, en hij is helemaal correct geschreven."),
            Some(Language::Dutch)
        );
        assert_eq!(
            detect("Esto es español y debería ser detectado correctamente por Harper."),
            Some(Language::Spanish)
        );
        assert_eq!(detect("#! /bin/bash"), None);
        assert_eq!(detect("The quick brown fox"), Some(Language::English));
        assert_eq!(detect("Ja"), None);
    }
}
