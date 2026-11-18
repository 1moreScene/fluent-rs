use slint::{Model, ModelExt, ModelRc};

use crate::{RadioDirection, RadioItem};

pub(super) fn target(
    items: ModelRc<RadioItem>,
    columns: i32,
    index: i32,
    direction: RadioDirection,
) -> i32 {
    let count = items.row_count();
    let enabled = |row| {
        items
            .row_data_tracked(row)
            .is_some_and(|item| !item.disabled)
    };
    if direction == RadioDirection::First {
        return result((0..count).find(|&row| enabled(row)));
    }
    let Ok(index) = usize::try_from(index) else {
        return -1;
    };
    if index >= count {
        return -1;
    }
    match direction {
        RadioDirection::First => -1,
        RadioDirection::Next => result((index + 1..count).find(|&row| enabled(row))),
        RadioDirection::Previous => result((0..index).rev().find(|&row| enabled(row))),
        RadioDirection::Left | RadioDirection::Right => {
            let columns = usize::try_from(columns).unwrap_or(1).clamp(1, count);
            result(horizontal_target(count, columns, index, direction, enabled))
        }
    }
}

fn horizontal_target(
    count: usize,
    columns: usize,
    index: usize,
    direction: RadioDirection,
    enabled: impl Fn(usize) -> bool,
) -> Option<usize> {
    let rows = count / columns;
    let longer = count % columns;
    let start = |column: usize| column * rows + column.min(longer);
    let column = (0..columns).find(|column| index < start(column + 1))?;
    let row = index - start(column);
    let closest = |candidate| {
        let start_row = start(candidate);
        (start_row..start(candidate + 1))
            .filter(|&item| enabled(item))
            .min_by_key(|&item| (item - start_row).abs_diff(row))
    };
    if direction == RadioDirection::Left {
        (0..column).rev().find_map(closest)
    } else {
        (column + 1..columns).find_map(closest)
    }
}

fn result(index: Option<usize>) -> i32 {
    index
        .and_then(|index| i32::try_from(index).ok())
        .unwrap_or(-1)
}

#[cfg(test)]
mod tests {
    use slint::{ModelRc, VecModel};

    use super::{RadioDirection as Direction, RadioItem, target};

    fn items(disabled: &[bool]) -> ModelRc<RadioItem> {
        ModelRc::new(VecModel::from(
            disabled
                .iter()
                .map(|&disabled| RadioItem {
                    disabled,
                    ..Default::default()
                })
                .collect::<Vec<_>>(),
        ))
    }

    #[test]
    fn vertical_navigation_skips_disabled_options_and_stops_at_edges() {
        let options = items(&[true, false, true, false]);
        assert_eq!(target(options.clone(), 1, -1, Direction::First), 1);
        assert_eq!(target(options.clone(), 1, 1, Direction::Next), 3);
        assert_eq!(target(options.clone(), 1, 3, Direction::Previous), 1);
        assert_eq!(target(options.clone(), 1, 1, Direction::Previous), -1);
        assert_eq!(target(options, 1, 3, Direction::Next), -1);
        assert_eq!(target(items(&[]), 1, -1, Direction::First), -1);
        assert_eq!(target(items(&[true, true]), 1, -1, Direction::First), -1);
    }

    #[test]
    fn horizontal_navigation_uses_adjacent_enabled_columns_and_nearest_row() {
        // Column-major row counts are 3, 2, 2. The middle column is disabled.
        let options = items(&[false, false, false, true, true, false, false]);
        assert_eq!(target(options.clone(), 3, 0, Direction::Right), 5);
        assert_eq!(target(options.clone(), 3, 2, Direction::Right), 6);
        assert_eq!(target(options.clone(), 3, 6, Direction::Left), 1);
        assert_eq!(target(options, 3, 6, Direction::Right), -1);
    }
}
