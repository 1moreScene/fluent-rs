use slint::{Model, ModelRc, VecModel};

use crate::{NavigationItemDetails, NavigationItemKind, NavigationRow};

pub(super) fn details(
    entries: ModelRc<NavigationItemDetails>,
    index: i32,
) -> NavigationItemDetails {
    entry(&entries, index)
}

pub(super) fn has_children(entries: ModelRc<NavigationItemDetails>, index: i32) -> bool {
    entries.iter().any(|item| item.parent_index == index)
}

fn entry(entries: &ModelRc<NavigationItemDetails>, index: i32) -> NavigationItemDetails {
    entries
        .iter()
        .find(|item| item.index == index)
        .unwrap_or_else(|| NavigationItemDetails {
            index,
            parent_index: -1,
            selects_on_invoked: true,
            ..Default::default()
        })
}

pub(super) fn visible(
    entries: ModelRc<NavigationItemDetails>,
    expanded: ModelRc<i32>,
    start: i32,
    end: i32,
    ancestor: i32,
    compact: bool,
) -> ModelRc<i32> {
    let indices = (start.max(0)..end)
        .filter(|index| {
            let item = entry(&entries, *index);
            if compact {
                return item.parent_index == ancestor && item.kind != NavigationItemKind::Header;
            }
            // A parent must precede its descendants in the source model.
            let mut child = *index;
            let mut parent = item.parent_index;
            while parent != ancestor {
                if parent < 0 || parent >= child || !expanded.iter().any(|value| value == parent) {
                    return false;
                }
                child = parent;
                parent = entry(&entries, parent).parent_index;
            }
            true
        })
        .collect::<Vec<_>>();
    ModelRc::new(VecModel::from(indices))
}

pub(super) fn ancestor(
    entries: ModelRc<NavigationItemDetails>,
    indices: ModelRc<i32>,
    index: i32,
) -> i32 {
    let mut candidate = index;
    while candidate >= 0 {
        if indices.iter().any(|index| index == candidate) {
            return candidate;
        }
        let parent = entry(&entries, candidate).parent_index;
        if parent >= candidate {
            break;
        }
        candidate = parent;
    }
    -1
}

pub(super) fn rows(
    entries: ModelRc<NavigationItemDetails>,
    indices: ModelRc<i32>,
    row_height: f32,
    header_height: f32,
    separator_height: f32,
    root_parent: i32,
) -> ModelRc<NavigationRow> {
    let mut offset = 0.;
    let rows = indices
        .iter()
        .map(|index| {
            let details = entry(&entries, index);
            let extent = match details.kind {
                NavigationItemKind::Destination => row_height,
                NavigationItemKind::Header => header_height,
                NavigationItemKind::Separator => separator_height,
            };
            let mut depth = 0;
            let mut child = index;
            let mut parent = details.parent_index;
            while parent >= 0 && parent != root_parent && parent < child {
                depth += 1;
                child = parent;
                parent = entry(&entries, parent).parent_index;
            }
            let row = NavigationRow {
                index,
                offset,
                extent,
                depth,
                has_children: entries.iter().any(|item| item.parent_index == index),
                details,
            };
            offset += extent;
            row
        })
        .collect::<Vec<_>>();
    ModelRc::new(VecModel::from(rows))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapsed_nodes_hide_descendants_but_keep_the_selected_ancestor() {
        let details = ModelRc::new(VecModel::from(vec![
            NavigationItemDetails {
                index: 1,
                parent_index: 0,
                ..Default::default()
            },
            NavigationItemDetails {
                index: 2,
                parent_index: 1,
                ..Default::default()
            },
        ]));
        let expanded = ModelRc::new(VecModel::from(vec![0]));
        let shown = visible(details.clone(), expanded, 0, 4, -1, false);
        assert_eq!(shown.iter().collect::<Vec<_>>(), [0, 1, 3]);
        assert_eq!(ancestor(details, shown, 2), 1);
    }
}
