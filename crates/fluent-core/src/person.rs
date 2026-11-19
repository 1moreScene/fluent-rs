//! Initials from the documented WinUI character groups and display-name rules.

fn standard(ch: char) -> bool {
    matches!(ch as u32, 1..=0x24f | 0x300..=0x36f | 0x400..=0x52f | 0x1e00..=0x1eff | 0x2c60..=0x2c7f | 0xa720..=0xa7ff | 0xab30..=0xab6f)
}
fn non_standard(ch: char) -> bool {
    matches!(ch as u32, 0x250..=0x2af | 0x370..=0x3ff | 0x530..=0x5ff | 0x600..=0x6ff | 0x750..=0x77f | 0x8a0..=0x8ff | 0x900..=0xeff | 0xa8e0..=0xa8ff | 0xfb50..=0xfdff | 0xfe70..=0xfeff | 0x2e80..=0x2eff | 0x3000..=0x303f | 0x31c0..=0x33ff | 0x3400..=0x4dbf | 0x4e00..=0x9fff | 0xf900..=0xfaff | 0xfe30..=0xfe4f | 0x10000..=0x10ffff)
}

fn initial(word: &str) -> String {
    let skip = |ch: char| matches!(ch as u32, 0x21..=0x2f | 0x3a..=0x40 | 0x7b..=0x7e);
    let start = word
        .char_indices()
        .find(|(_, ch)| !skip(*ch))
        .map_or(0, |(index, _)| index);
    let mut chars = word[start..].chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut upper = first.to_uppercase();
    let first_upper = upper.next().unwrap_or(first);
    let mut result = String::from(if upper.next().is_none() {
        first_upper
    } else {
        first
    });
    result.extend(chars.take_while(|ch| matches!(*ch as u32, 0x300..=0x36f)));
    result
}

/// Returns generated initials, or empty for names that should use the generic icon.
/// Explicit initials bypass this helper in PersonPicture.
pub fn initials(name: &str) -> String {
    let mut saw_standard = false;
    for ch in name
        .chars()
        .take(3)
        .take_while(|ch| *ch != '\0' && *ch != '\u{feff}')
    {
        if non_standard(ch) {
            return String::new();
        }
        saw_standard |= standard(ch);
    }
    if !saw_standard {
        return String::new();
    }
    let mut name = name;
    for (open, close) in [('{', '}'), ('(', ')'), ('[', ']')] {
        if name.ends_with(close)
            && let Some(start) = name.rfind(open)
        {
            name = &name[..start];
            break;
        }
    }
    let mut words = name.split(' ').take(25).filter(|word| !word.is_empty());
    let Some(first) = words.next() else {
        return String::new();
    };
    let mut result = initial(first);
    if let Some(last) = words.last() {
        result.push_str(&initial(last));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn names_use_first_and_last_word_and_trailing_brackets() {
        for (name, expected) in [
            ("Jane Doe", "JD"),
            ("Jordan", "J"),
            ("John von Smith (OSG)", "JS"),
            (" John  -Smith", "JS"),
            ("e\u{301}mile Zola", "E\u{301}Z"),
            ("123 Person", "1P"),
            ("Иван Петров", "ИП"),
            ("   ", ""),
        ] {
            assert_eq!(initials(name), expected, "{name}");
        }
    }
    #[test]
    fn symbolic_and_shaping_scripts_use_generic_image() {
        for name in ["张三", "Αλέξανδρος", "שלום", "محمد", "हिन्दी", "😀 Alice"]
        {
            assert_eq!(initials(name), "", "{name}");
        }
    }
}
