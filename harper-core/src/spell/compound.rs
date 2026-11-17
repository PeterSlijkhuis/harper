use super::Dictionary;

const MIN_PART: usize = 3;
const MAX_PARTS: usize = 3;
/// Dutch linking elements (tussenklanken) that may follow a non-final part.
const LINKS: [&[char]; 3] = [&['s'], &['e'], &['e', 'n']];

/// Whether `word` (lowercase) is a Dutch compound of two or three dictionary words, where
/// each non-final part may carry a linking `s`, `e` or `en`.
///
// ponytail: greedy dictionary split with no frequency weighting, so a typo that happens to
// split into valid words is accepted. Add a frequency-weighted splitter if that bites.
pub fn is_dutch_compound(dict: &impl Dictionary, word: &[char]) -> bool {
    split(dict, word, MAX_PARTS, false)
}

fn split(dict: &impl Dictionary, rest: &[char], parts_left: usize, has_prefix: bool) -> bool {
    if parts_left == 0 || rest.len() < MIN_PART {
        return false;
    }
    // The final part has to be a plain dictionary word.
    if has_prefix && dict.contains_exact_word(rest) {
        return true;
    }
    for cut in MIN_PART..=rest.len().saturating_sub(MIN_PART) {
        let (head, tail) = rest.split_at(cut);
        let head_ok = dict.contains_exact_word(head)
            || LINKS.iter().any(|link| {
                head.ends_with(link)
                    && head.len() - link.len() >= MIN_PART
                    && dict.contains_exact_word(&head[..head.len() - link.len()])
            });
        if head_ok && split(dict, tail, parts_left - 1, true) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::is_dutch_compound;
    use crate::Language;
    use crate::spell::FstDictionary;

    fn check(word: &str) -> bool {
        let chars: Vec<char> = word.chars().collect();
        is_dutch_compound(&*FstDictionary::curated_for(Language::Dutch), &chars)
    }

    #[test]
    fn splits_valid_compounds_only() {
        assert!(check("fietsenstalling"));
        assert!(check("ziekenhuisbed"));
        assert!(check("boekenkastplank"));
        assert!(!check("fietsnstalling"));
        assert!(!check("xqzvbn"));
        assert!(!check("huis")); // a single word is not a compound
    }
}
