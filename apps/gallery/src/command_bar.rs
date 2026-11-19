use std::collections::BTreeMap;

use slint::{Model, ModelRc, VecModel};

use crate::{
    CommandBarItem, CommandBarLayout, CommandBarMeasure, CommandBarMenu, ItemData, ItemDecoration,
};

pub(super) fn arrange(
    commands: ModelRc<CommandBarItem>,
    widths: ModelRc<f32>,
    secondary: i32,
    measure: CommandBarMeasure,
) -> CommandBarLayout {
    let count = commands.row_count();
    let command_width = |index| {
        widths
            .row_data(index)
            .unwrap_or_default()
            .max(measure.button_width)
    };
    let mut used: f32 = (0..count).map(command_width).sum();
    let more = secondary > 0 || measure.force_more || (measure.dynamic && used > measure.width);
    let available = if more {
        (measure.width - measure.more_width).max(0.)
    } else {
        measure.width
    };
    let mut moved = vec![false; count];

    if measure.dynamic && used > available {
        let mut ordered = BTreeMap::<i32, Vec<usize>>::new();
        let mut remaining = Vec::new();
        for (index, command) in commands.iter().enumerate() {
            if command.overflow_order > 0 {
                ordered
                    .entry(command.overflow_order)
                    .or_default()
                    .push(index);
            } else {
                remaining.push(index);
            }
        }
        for group in ordered.values() {
            if used <= available {
                break;
            }
            for &index in group {
                moved[index] = true;
                used -= command_width(index);
            }
        }
        for index in remaining.into_iter().rev() {
            if used <= available {
                break;
            }
            moved[index] = true;
            used -= command_width(index);
        }
    }

    let (overflow, primary): (Vec<_>, Vec<_>) =
        (0..count as i32).partition(|&index| moved[index as usize]);
    let overflow = overflow
        .into_iter()
        .chain(count as i32..count as i32 + secondary.max(0))
        .collect::<Vec<_>>();
    CommandBarLayout {
        primary: ModelRc::new(VecModel::from(primary)),
        overflow: ModelRc::new(VecModel::from(overflow)),
        more,
    }
}

pub(super) fn menu(
    primary: ModelRc<CommandBarItem>,
    secondary: ModelRc<CommandBarItem>,
    indices: ModelRc<i32>,
) -> CommandBarMenu {
    let boundary = primary.row_count();
    let rows: Vec<_> = indices
        .iter()
        .filter_map(|index| {
            let index = usize::try_from(index).ok()?;
            if index < boundary {
                primary.row_data(index).map(|cmd| (false, cmd))
            } else {
                secondary.row_data(index - boundary).map(|cmd| (true, cmd))
            }
        })
        .collect();
    let icons = rows
        .iter()
        .any(|(_, command)| command.icon.size().width > 0);
    let toggles = rows.iter().any(|(_, command)| command.toggle);
    let mut items = Vec::with_capacity(rows.len());
    let mut decorations = Vec::with_capacity(rows.len());
    let mut previous_primary = false;
    for (secondary, command) in rows {
        items.push(ItemData {
            text: command.text,
            disabled: command.disabled,
        });
        decorations.push(ItemDecoration {
            icon: command.icon,
            icon_space: icons,
            command_check_space: toggles,
            command_toggle: command.toggle,
            command_checked: command.toggle && command.checked,
            separator_before: secondary && previous_primary,
            ..Default::default()
        });
        previous_primary = !secondary;
    }
    CommandBarMenu {
        items: ModelRc::new(VecModel::from(items)),
        decorations: ModelRc::new(VecModel::from(decorations)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moves_ordered_groups_together_and_restores_source_order() {
        let commands = ModelRc::new(VecModel::from(
            [0, 2, 1, 2]
                .map(|order| CommandBarItem {
                    overflow_order: order,
                    ..Default::default()
                })
                .to_vec(),
        ));
        let measure = CommandBarMeasure {
            width: 220.,
            more_width: 48.,
            button_width: 68.,
            dynamic: true,
            force_more: false,
        };
        let narrow = arrange(commands.clone(), ModelRc::default(), 2, measure.clone());
        assert_eq!(narrow.primary.iter().collect::<Vec<_>>(), [0]);
        assert_eq!(narrow.overflow.iter().collect::<Vec<_>>(), [1, 2, 3, 4, 5]);
        let wide = arrange(
            commands,
            ModelRc::default(),
            2,
            CommandBarMeasure {
                width: 400.,
                ..measure
            },
        );
        assert_eq!(wide.primary.iter().collect::<Vec<_>>(), [0, 1, 2, 3]);
        assert_eq!(wide.overflow.iter().collect::<Vec<_>>(), [4, 5]);
    }

    #[test]
    fn reserves_more_only_when_it_is_visible_and_moves_from_the_right() {
        let commands = ModelRc::new(VecModel::from(vec![CommandBarItem::default(); 3]));
        let measure = CommandBarMeasure {
            width: 158.,
            more_width: 48.,
            button_width: 68.,
            dynamic: true,
            force_more: false,
        };
        let narrow = arrange(commands.clone(), ModelRc::default(), 0, measure.clone());
        assert_eq!(narrow.primary.iter().collect::<Vec<_>>(), [0]);
        assert_eq!(narrow.overflow.iter().collect::<Vec<_>>(), [1, 2]);
        let wide = arrange(
            commands,
            ModelRc::default(),
            0,
            CommandBarMeasure {
                width: 204.,
                ..measure
            },
        );
        assert!(!wide.more);
        assert_eq!(wide.primary.row_count(), 3);
    }
}
