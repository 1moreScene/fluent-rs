use slint::{Model, ModelRc, SharedString};

use crate::{ComboTextMatch, ItemData};

pub(super) fn find(
    items: ModelRc<ItemData>,
    input: SharedString,
    cursor: i32,
    complete: bool,
) -> ComboTextMatch {
    let input = input.as_str();
    let autocomplete = complete && cursor as usize == input.len();
    let missing = ComboTextMatch {
        index: -1,
        ..Default::default()
    };
    if input.trim().is_empty() {
        return missing;
    }

    let folded: String = input.chars().flat_map(char::to_lowercase).collect();
    for (index, item) in items.iter().enumerate().filter(|(_, item)| !item.disabled) {
        let text = item.text.as_str().trim_start_matches(' ');
        if let Some(prefix_bytes) = prefix_end(text, &folded)
            && (autocomplete || prefix_bytes == text.len())
        {
            let leading = item.text.len() - text.len();
            return ComboTextMatch {
                index: index as i32,
                prefix_bytes: (leading + prefix_bytes) as i32,
                item_bytes: item.text.len() as i32,
            };
        }
    }
    missing
}

fn prefix_end(text: &str, folded: &str) -> Option<usize> {
    let mut remaining = folded;
    for (offset, character) in text.char_indices() {
        for lower in character.to_lowercase() {
            remaining = remaining.strip_prefix(lower)?;
        }
        if remaining.is_empty() {
            return Some(offset + character.len_utf8());
        }
    }
    None
}

pub(super) fn contains_text(text: SharedString) -> bool {
    !text.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use slint::VecModel;

    #[test]
    fn completion_uses_utf8_boundaries_and_skips_disabled_options() {
        let items = ModelRc::new(VecModel::from(vec![
            ItemData {
                text: "中文".into(),
                disabled: true,
            },
            ItemData {
                text: "中国".into(),
                disabled: false,
            },
            ItemData {
                text: "Studio".into(),
                disabled: false,
            },
        ]));
        let chinese = find(items.clone(), "中".into(), 3, true);
        assert_eq!(
            (chinese.index, chinese.prefix_bytes, chinese.item_bytes),
            (1, 3, 6)
        );
        assert_eq!(find(items.clone(), "ST".into(), 2, true).index, 2);
        assert_eq!(find(items.clone(), "中".into(), 3, false).index, -1);
        assert_eq!(find(items.clone(), "中".into(), 0, true).index, -1);
        assert_eq!(find(items, "studio".into(), 0, false).index, 2);
    }
}
