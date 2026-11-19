use slint::{Model, ModelRc, VecModel};

use crate::{ItemData, TopNavigationArrangement, TopNavigationMeasure, TopNavigationSlot};

pub(super) fn measure(widths: ModelRc<f32>, index: i32, width: f32) -> ModelRc<f32> {
    let Ok(index) = usize::try_from(index) else {
        return widths;
    };
    if widths.row_data(index) == Some(width) {
        return widths;
    }
    let mut data = widths.iter().collect::<Vec<_>>();
    if index >= data.len() {
        data.resize(index + 1, 0.);
    }
    data[index] = width;
    ModelRc::new(VecModel::from(data))
}

pub(super) fn arrange(
    indices: ModelRc<i32>,
    measured: ModelRc<f32>,
    measure: TopNavigationMeasure,
) -> TopNavigationArrangement {
    let width_of = |index: i32| {
        measured
            .row_data(index as usize)
            .filter(|width| *width > 0.)
            .unwrap_or(measure.minimum_item_width)
    };
    let (main, footer_indices): (Vec<_>, Vec<_>) = indices
        .iter()
        .partition(|index| measure.footer_start < 0 || *index < measure.footer_start);
    let footer_width: f32 = footer_indices.iter().copied().map(width_of).sum();
    let available = (measure.width - footer_width - measure.trailing_width).max(0.);
    let overflow_required =
        main.iter().copied().map(width_of).sum::<f32>() + measure.recovery_margin > available;
    let budget = (available
        - if overflow_required {
            measure.more_width
        } else {
            0.
        })
    .max(0.);
    // Keep the selected item on the primary bar if all desired items cannot fit.
    let reserve = if !overflow_required {
        None
    } else if main.contains(&measure.selected) {
        Some(measure.selected)
    } else {
        main.first().copied()
    };
    let reserved_width = reserve.map_or(0., |index| width_of(index).min(budget));
    let mut remaining = budget - reserved_width;
    let mut primary = Vec::new();
    let mut overflow = Vec::new();
    let mut x = 0.;
    for index in main {
        if Some(index) == reserve {
            primary.push(TopNavigationSlot {
                index,
                x,
                width: reserved_width,
            });
            x += reserved_width;
        } else {
            let width = width_of(index);
            if width <= remaining {
                primary.push(TopNavigationSlot { index, x, width });
                x += width;
                remaining -= width;
            } else {
                overflow.push(index);
                remaining = 0.;
            }
        }
    }
    let overflow_width = overflow.iter().copied().map(width_of).fold(0., f32::max);
    let mut footer_x = (measure.width - footer_width).max(0.);
    let footer = footer_indices
        .into_iter()
        .map(|index| {
            let width = width_of(index);
            let slot = TopNavigationSlot {
                index,
                x: footer_x,
                width,
            };
            footer_x += width;
            slot
        })
        .collect::<Vec<_>>();
    let indicator_x = primary
        .iter()
        .chain(footer.iter())
        .find(|slot| slot.index == measure.selected)
        .map_or(-1., |slot| slot.x + slot.width / 2.);
    TopNavigationArrangement {
        primary: ModelRc::new(VecModel::from(primary)),
        overflow: ModelRc::new(VecModel::from(overflow)),
        footer: ModelRc::new(VecModel::from(footer)),
        more_x: x,
        overflow_width,
        indicator_x,
    }
}

pub(super) fn step(
    items: ModelRc<ItemData>,
    primary: ModelRc<TopNavigationSlot>,
    footer: ModelRc<TopNavigationSlot>,
    current: i32,
    direction: i32,
) -> i32 {
    let enabled = |index: i32| {
        items
            .row_data(index as usize)
            .is_some_and(|item| !item.disabled)
    };
    let ordered = primary
        .iter()
        .chain(footer.iter())
        .map(|slot| slot.index)
        .filter(|index| enabled(*index))
        .collect::<Vec<_>>();
    let Some(index) = ordered.iter().position(|index| *index == current) else {
        return -1;
    };
    index
        .checked_add_signed(direction as isize)
        .and_then(|index| ordered.get(index))
        .copied()
        .unwrap_or(-1)
}

pub(super) fn indices(start: i32, end: i32) -> ModelRc<i32> {
    ModelRc::new(VecModel::from((start.max(0)..end).collect::<Vec<_>>()))
}

pub(super) fn slot(slots: ModelRc<TopNavigationSlot>, index: i32) -> TopNavigationSlot {
    slots
        .iter()
        .find(|slot| slot.index == index)
        .unwrap_or_else(|| TopNavigationSlot {
            index: -1,
            ..Default::default()
        })
}

pub(super) fn position(indices: ModelRc<i32>, index: i32) -> i32 {
    indices
        .iter()
        .position(|value| value == index)
        .map_or(-1, |row| row as i32)
}

pub(super) fn visible_items(items: ModelRc<ItemData>, indices: ModelRc<i32>) -> ModelRc<ItemData> {
    ModelRc::new(VecModel::from(
        indices
            .iter()
            .filter_map(|index| items.row_data(index as usize))
            .collect::<Vec<_>>(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overflow_promotes_selection_and_keeps_footer_and_source_order() {
        let sizes = ModelRc::new(VecModel::from(vec![90., 60., 90., 40.]));
        let all = indices(0, 4);
        let measure = TopNavigationMeasure {
            width: 260.,
            more_width: 40.,
            minimum_item_width: 40.,
            selected: 2,
            footer_start: 3,
            recovery_margin: 0.,
            trailing_width: 0.,
        };
        let result = arrange(all.clone(), sizes.clone(), measure.clone());
        assert_eq!(
            result
                .primary
                .iter()
                .map(|slot| slot.index)
                .collect::<Vec<_>>(),
            [0, 2]
        );
        assert_eq!(result.overflow.iter().collect::<Vec<_>>(), [1]);
        assert_eq!(result.footer.row_data(0).map(|slot| slot.x), Some(220.));
        let held = arrange(
            all.clone(),
            sizes.clone(),
            TopNavigationMeasure {
                width: 282.,
                recovery_margin: 5.,
                ..measure.clone()
            },
        );
        assert!(held.overflow.row_count() > 0);
        let wide = arrange(
            all,
            sizes,
            TopNavigationMeasure {
                width: 340.,
                selected: 3,
                ..measure
            },
        );
        assert_eq!(wide.overflow.row_count(), 0);
        assert_eq!(wide.indicator_x, 320.);
    }
}
