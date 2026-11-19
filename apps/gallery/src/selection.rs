use slint::{Model, ModelExt, ModelRc, VecModel};

use crate::{ItemData, ListDirection};

pub(super) fn target(items: ModelRc<ItemData>, from: i32, direction: ListDirection) -> i32 {
    let count = i32::try_from(items.row_count()).unwrap_or(i32::MAX);
    let enabled = |index: &i32| {
        items
            .row_data_tracked(*index as usize)
            .is_some_and(|item| !item.disabled)
    };
    match direction {
        ListDirection::First => (0..count).find(enabled),
        ListDirection::Last => (0..count).rev().find(enabled),
        ListDirection::Next => (from.saturating_add(1).clamp(0, count)..count).find(enabled),
        ListDirection::Previous => (0..from.clamp(0, count)).rev().find(enabled),
    }
    .unwrap_or(-1)
}

pub(super) fn update(selected: ModelRc<i32>, index: i32, multiple: bool) -> ModelRc<i32> {
    if !multiple {
        return ModelRc::new(VecModel::from(vec![index]));
    }
    let mut indices: Vec<_> = selected.iter().collect();
    match indices.binary_search(&index) {
        Ok(position) => {
            indices.remove(position);
        }
        Err(position) => indices.insert(position, index),
    }
    ModelRc::new(VecModel::from(indices))
}

pub(super) fn contains(selected: ModelRc<i32>, index: i32) -> bool {
    let (mut start, mut end) = (0, selected.row_count());
    while start < end {
        let mid = start + (end - start) / 2;
        match selected.row_data(mid).map(|value| value.cmp(&index)) {
            Some(std::cmp::Ordering::Less) => start = mid + 1,
            Some(std::cmp::Ordering::Greater) => end = mid,
            Some(std::cmp::Ordering::Equal) => return true,
            None => return false,
        }
    }
    false
}

pub(super) fn range(
    items: ModelRc<ItemData>,
    selected: ModelRc<i32>,
    anchor: i32,
    index: i32,
) -> ModelRc<i32> {
    let count = i32::try_from(items.row_count()).unwrap_or(i32::MAX);
    let start = anchor.min(index).max(0);
    let end = anchor.max(index).min(count - 1);
    let mut indices: Vec<_> = selected.iter().collect();
    indices.extend((start..=end).filter(|&row| {
        items
            .row_data(row as usize)
            .is_some_and(|item| !item.disabled)
    }));
    indices.sort_unstable();
    indices.dedup();
    ModelRc::new(VecModel::from(indices))
}

pub(super) fn equal(left: ModelRc<i32>, right: ModelRc<i32>) -> bool {
    left.row_count() == right.row_count() && left.iter().eq(right.iter())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_skips_disabled_rows_and_handles_empty_models() {
        let items = ModelRc::new(VecModel::from(
            [true, false, true, false, true]
                .map(|disabled| ItemData {
                    disabled,
                    ..Default::default()
                })
                .to_vec(),
        ));
        assert_eq!(target(items.clone(), -1, ListDirection::First), 1);
        assert_eq!(target(items.clone(), -1, ListDirection::Last), 3);
        assert_eq!(target(items.clone(), 1, ListDirection::Next), 3);
        assert_eq!(target(items.clone(), 3, ListDirection::Previous), 1);
        assert_eq!(target(items, 3, ListDirection::Next), -1);
        assert_eq!(target(ModelRc::default(), -1, ListDirection::Last), -1);
    }

    #[test]
    fn multiple_selection_toggles_and_single_selection_replaces() {
        let one = update(ModelRc::default(), 2, true);
        let two = update(one, 4, true);
        assert_eq!(two.iter().collect::<Vec<_>>(), [2, 4]);
        let removed = update(two, 2, true);
        assert!(!contains(removed.clone(), 2));
        assert_eq!(update(removed, 7, false).iter().collect::<Vec<_>>(), [7]);
    }
}
