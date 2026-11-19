use crate::ItemData;
use slint::ModelRc;

pub(super) fn byte_length(text: slint::SharedString) -> i32 {
    text.len().min(i32::MAX as usize) as i32
}

/// Filtering belongs to this example. The control accepts any caller-owned model.
pub(super) fn filter(query: slint::SharedString) -> ModelRc<ItemData> {
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    let items = [
        "AutoSuggestBox",
        "Button",
        "CheckBox",
        "ComboBox",
        "ContentDialog",
        "ContextMenu",
        "Flyout",
        "InfoBar",
        "ListBox",
        "NavigationView",
        "ProgressBar",
        "ProgressRing",
        "RadioButton",
        "Slider",
        "TextBox",
        "ToggleButton",
        "Tooltip",
    ]
    .into_iter()
    .filter(|name| {
        let lower = name.to_lowercase();
        words.iter().all(|word| lower.contains(word))
    })
    .map(|name| ItemData {
        text: name.into(),
        disabled: false,
    })
    .collect::<Vec<_>>();
    ModelRc::new(slint::VecModel::from(items))
}
