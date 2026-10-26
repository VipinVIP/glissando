//! Small text utilities shared by the engine and the UI.

/// Characters that can be part of a word for prediction purposes.
pub fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '\'' || c == '’' || c == '-' || c == '\u{301}'
}

/// The word fragment immediately before the cursor (`before` is the text
/// preceding the cursor). Leading apostrophes/hyphens are not part of it.
pub fn trailing_word(before: &str) -> &str {
    let mut start = before.len();
    for (i, c) in before.char_indices().rev() {
        if is_word_char(c) {
            start = i;
        } else {
            break;
        }
    }
    let w = &before[start..];
    w.trim_start_matches(['\'', '’', '-'])
}

/// The word fragment immediately after the cursor.
pub fn leading_word(after: &str) -> &str {
    let end = after.char_indices().find(|(_, c)| !is_word_char(*c)).map(|(i, _)| i).unwrap_or(after.len());
    &after[..end]
}

/// Up to `n` previous complete words before the cursor, oldest first.
/// Stops at sentence boundaries so context does not leak across sentences.
pub fn previous_words(before: &str, n: usize) -> Vec<String> {
    let mut words = Vec::new();
    let mut cur = String::new();
    for c in before.chars().rev() {
        if is_word_char(c) {
            cur.insert(0, c);
        } else {
            if !cur.is_empty() {
                words.push(std::mem::take(&mut cur));
                if words.len() == n {
                    break;
                }
            }
            if matches!(c, '.' | '!' | '?' | '\n' | '…') {
                break;
            }
        }
    }
    if !cur.is_empty() && words.len() < n {
        words.push(cur);
    }
    words.reverse();
    words
}

/// Whether the next letter should be capitalised automatically.
pub fn sentence_start(before: &str) -> bool {
    let trimmed = before.trim_end_matches([' ', '\u{a0}', '\t']);
    if trimmed.is_empty() {
        return true;
    }
    if before.len() == trimmed.len() && !before.ends_with('\n') {
        // Cursor directly after a non-space character.
        return false;
    }
    let last = trimmed.chars().last().unwrap();
    if matches!(last, '\n') {
        return true;
    }
    if matches!(last, '.' | '!' | '?' | '…' | '¿' | '¡') {
        // "e.g." / "z.B." style abbreviations: a single letter before the dot.
        let body = &trimmed[..trimmed.len() - last.len_utf8()];
        let prev_word = trailing_word(body);
        if last == '.' && prev_word.chars().count() == 1 && body.chars().rev().nth(1) == Some('.') {
            return false;
        }
        return true;
    }
    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CasePattern {
    Lower,
    /// First letter upper case.
    Capitalized,
    Upper,
}

pub fn case_pattern(word: &str) -> CasePattern {
    let mut letters = word.chars().filter(|c| c.is_alphabetic());
    let Some(first) = letters.next() else { return CasePattern::Lower };
    let rest: Vec<char> = letters.collect();
    if first.is_uppercase() {
        if !rest.is_empty() && rest.iter().all(|c| c.is_uppercase()) {
            CasePattern::Upper
        } else {
            CasePattern::Capitalized
        }
    } else {
        CasePattern::Lower
    }
}

pub fn apply_case(word: &str, pattern: CasePattern) -> String {
    match pattern {
        CasePattern::Lower => word.to_string(),
        CasePattern::Upper => word.to_uppercase(),
        CasePattern::Capitalized => capitalize(word),
    }
}

pub fn capitalize(word: &str) -> String {
    let mut c = word.chars();
    match c.next() {
        Some(f) => f.to_uppercase().chain(c).collect(),
        None => String::new(),
    }
}

/// Number of chars (not bytes) to delete to remove `n` words before the
/// cursor. Trailing whitespace belongs to the word before it; a run of
/// punctuation counts as one word.
pub fn chars_for_words_back(before: &str, n: u32) -> usize {
    let chars: Vec<char> = before.chars().collect();
    let mut i = chars.len();
    for _ in 0..n {
        while i > 0 && chars[i - 1].is_whitespace() {
            i -= 1;
        }
        if i == 0 {
            break;
        }
        if is_word_char(chars[i - 1]) {
            while i > 0 && is_word_char(chars[i - 1]) {
                i -= 1;
            }
        } else {
            while i > 0 && !is_word_char(chars[i - 1]) && !chars[i - 1].is_whitespace() {
                i -= 1;
            }
        }
    }
    chars.len() - i
}

/// Byte length of the last `n` chars of `s`.
pub fn tail_bytes(s: &str, n: usize) -> usize {
    s.chars().rev().take(n).map(|c| c.len_utf8()).sum()
}

/// Lowercase + fold a few characters that users routinely type without
/// diacritics (used for dictionary matching, never for output).
pub fn fold_char(c: char) -> char {
    let l = c.to_lowercase().next().unwrap_or(c);
    match l {
        'ё' => 'е',
        'й' => 'й',
        'à' | 'á' | 'â' | 'ã' | 'å' | 'ā' => 'a',
        'ä' => 'a',
        'ç' | 'ć' | 'č' => 'c',
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ė' | 'ę' => 'e',
        'ì' | 'í' | 'î' | 'ï' | 'ī' => 'i',
        'ñ' | 'ń' => 'n',
        'ò' | 'ó' | 'ô' | 'õ' | 'ø' | 'ō' => 'o',
        'ö' => 'o',
        'ù' | 'ú' | 'û' | 'ū' => 'u',
        'ü' => 'u',
        'ß' => 's',
        '’' => '\'',
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words() {
        assert_eq!(trailing_word("hello wor"), "wor");
        assert_eq!(trailing_word("hello "), "");
        assert_eq!(trailing_word("don't"), "don't");
        assert_eq!(trailing_word("«при"), "при");
        assert_eq!(leading_word("ld foo"), "ld");
        assert_eq!(previous_words("Hi. I am here ", 2), vec!["am", "here"]);
        assert_eq!(previous_words("Hi. I", 3), vec!["I"]);
    }

    #[test]
    fn capitalization() {
        assert!(sentence_start(""));
        assert!(sentence_start("Hello. "));
        assert!(sentence_start("Line\n"));
        assert!(!sentence_start("Hello "));
        assert!(!sentence_start("Hello."));
        assert!(!sentence_start("e.g. "));
        assert!(sentence_start("Wirklich? "));
        assert_eq!(case_pattern("Hello"), CasePattern::Capitalized);
        assert_eq!(case_pattern("NASA"), CasePattern::Upper);
        assert_eq!(apply_case("über", CasePattern::Capitalized), "Über");
        assert_eq!(apply_case("ёлка", CasePattern::Upper), "ЁЛКА");
    }

    #[test]
    fn word_deletion() {
        assert_eq!(chars_for_words_back("hello world", 1), 5);
        assert_eq!(chars_for_words_back("hello world  ", 1), 7);
        assert_eq!(chars_for_words_back("hello world", 2), 11);
        assert_eq!(chars_for_words_back("привет, мир", 2), 5);
        assert_eq!(chars_for_words_back("a", 5), 1);
        assert_eq!(tail_bytes("мир", 2), 4);
    }
}
