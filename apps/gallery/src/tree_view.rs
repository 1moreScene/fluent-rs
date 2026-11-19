use slint::{Model, ModelRc, VecModel};

use crate::{ItemData, ItemDecoration, TreeChecks, TreeNode, TreeProjection};

fn model<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(rows))
}

fn item(node: TreeNode) -> ItemData {
    ItemData {
        text: node.text,
        disabled: node.disabled,
    }
}

pub(super) fn large_demo() -> ModelRc<TreeNode> {
    let mut nodes = Vec::with_capacity(4_000);
    for folder in 0..1_000 {
        let parent_index = nodes.len() as i32;
        nodes.push(TreeNode {
            text: format!("Folder {}", folder + 1).into(),
            parent_index: -1,
            ..Default::default()
        });
        for file in ["Notes.md", "Plan.md", "Readme.md"] {
            nodes.push(TreeNode {
                text: file.into(),
                parent_index,
                ..Default::default()
            });
        }
    }
    model(nodes)
}

pub(super) fn items(nodes: ModelRc<TreeNode>, _: i32) -> ModelRc<ItemData> {
    model(nodes.iter().map(item).collect())
}

#[derive(Default)]
struct TreeLinks {
    parent: Option<usize>,
    depth: usize,
    end: usize,
}

fn tree_links(nodes: &[TreeNode]) -> Vec<TreeLinks> {
    let mut path = Vec::new();
    let mut links = Vec::with_capacity(nodes.len());
    for (index, node) in nodes.iter().enumerate() {
        let parent = usize::try_from(node.parent_index)
            .ok()
            .filter(|&parent| parent < index);
        while !path.is_empty() && path.last().copied() != parent {
            path.pop();
        }
        links.push(TreeLinks {
            parent: path.last().copied(),
            depth: path.len(),
            end: index + 1,
        });
        path.push(index);
    }

    for index in (0..links.len()).rev() {
        if let Some(parent) = links[index].parent {
            links[parent].end = links[index].end.max(links[parent].end);
        }
    }
    links
}

pub(super) fn project(nodes: ModelRc<TreeNode>, expanded: ModelRc<i32>, _: i32) -> TreeProjection {
    let nodes: Vec<_> = nodes.iter().collect();
    let links = tree_links(&nodes);
    let mut opened = vec![false; nodes.len()];
    for index in expanded.iter() {
        if let Some(open) = opened.get_mut(index as usize) {
            *open = true;
        }
    }
    let mut row_by_node = vec![None; nodes.len()];
    let mut indices = Vec::new();
    let mut items = Vec::new();
    let mut decorations = Vec::new();
    let mut first_children = Vec::new();
    let mut index = 0;
    while let Some(node) = nodes.get(index) {
        let info = &links[index];
        let row = indices.len() as i32;
        if let Some(parent_row) = info.parent.and_then(|parent| row_by_node[parent]) {
            let child: &mut i32 = &mut first_children[parent_row];
            if *child < 0 && !node.disabled {
                *child = row;
            }
        }
        row_by_node[index] = Some(row as usize);
        indices.push(index as i32);
        items.push(item(node.clone()));
        first_children.push(-1);
        decorations.push(ItemDecoration {
            icon: node.icon.clone(),
            indent: info.depth as i32,
            has_children: info.end > index + 1,
            expanded: opened[index] && info.end > index + 1,
            ..Default::default()
        });
        index = if opened[index] { index + 1 } else { info.end };
    }
    TreeProjection {
        items: model(items),
        indices: model(indices),
        decorations: model(decorations),
        first_children: model(first_children),
    }
}

pub(super) fn visible_selection(visible: ModelRc<i32>, selected: ModelRc<i32>) -> ModelRc<i32> {
    // Both models use ascending source indices, so selection mapping is a merge.
    let mut selected = selected.iter().peekable();
    model(
        visible
            .iter()
            .enumerate()
            .filter_map(|(row, node)| {
                while selected.peek().is_some_and(|&target| target < node) {
                    selected.next();
                }
                selected
                    .peek()
                    .filter(|&&target| target == node)
                    .map(|_| row as i32)
            })
            .collect(),
    )
}

pub(super) fn visible_ancestor(
    nodes: ModelRc<TreeNode>,
    visible: ModelRc<i32>,
    mut node: i32,
) -> i32 {
    let visible: Vec<_> = visible.iter().collect();
    while node >= 0 {
        let Some(data) = nodes.row_data(node as usize) else {
            break;
        };
        if !data.disabled
            && let Ok(row) = visible.binary_search(&node)
        {
            return row as i32;
        }
        if data.parent_index >= node {
            break;
        }
        node = data.parent_index;
    }
    -1
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NodeCheck {
    NoEligibleChildren,
    Unchecked,
    Checked,
    Mixed,
}

impl NodeCheck {
    fn merge(self, child: Self) -> Self {
        match (self, child) {
            (Self::NoEligibleChildren, state) | (state, Self::NoEligibleChildren) => state,
            (a, b) if a == b => a,
            _ => Self::Mixed,
        }
    }
}

pub(super) fn toggle_check(
    nodes: ModelRc<TreeNode>,
    selected: ModelRc<i32>,
    index: i32,
) -> TreeChecks {
    let nodes: Vec<_> = nodes.iter().collect();
    let links = tree_links(&nodes);
    let mut checked = vec![false; nodes.len()];
    for index in selected.iter() {
        if let Some(value) = checked.get_mut(index as usize) {
            *value = true;
        }
    }
    let index = index as usize;
    if nodes.get(index).is_some_and(|node| !node.disabled) {
        let next = !checked[index];
        checked[index..links[index].end].fill(next);
    }

    let mut states = vec![NodeCheck::NoEligibleChildren; nodes.len()];
    for (index, node) in nodes.iter().enumerate().rev() {
        let child_state = states[index];
        if child_state == NodeCheck::NoEligibleChildren && !node.disabled {
            states[index] = if checked[index] {
                NodeCheck::Checked
            } else {
                NodeCheck::Unchecked
            };
        }
        if let Some(parent) = links[index].parent {
            states[parent] = states[parent].merge(states[index]);
        }
    }

    let mut result = Vec::new();
    let mut mixed = Vec::new();
    for (index, (node, state)) in nodes.iter().zip(states).enumerate() {
        if !node.disabled {
            match state {
                NodeCheck::Checked => result.push(index as i32),
                NodeCheck::Mixed => mixed.push(index as i32),
                _ => {}
            }
        }
    }
    TreeChecks {
        checked: model(result),
        mixed: model(mixed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nodes() -> ModelRc<TreeNode> {
        model(
            [-1, 0, 1, 0, -1]
                .into_iter()
                .enumerate()
                .map(|(index, parent_index)| TreeNode {
                    text: index.to_string().into(),
                    parent_index,
                    ..Default::default()
                })
                .collect(),
        )
    }

    #[test]
    fn projects_visible_branches_and_restores_focus_to_ancestor() {
        let projection = project(nodes(), model(vec![0]), 5);
        assert_eq!(projection.indices.iter().collect::<Vec<_>>(), [0, 1, 3, 4]);
        assert_eq!(
            projection.first_children.iter().collect::<Vec<_>>(),
            [1, -1, -1, -1]
        );
        assert_eq!(visible_ancestor(nodes(), projection.indices, 2), 1);
    }

    #[test]
    fn checks_subtrees_and_merges_parent_states_even_when_collapsed() {
        let leaf = toggle_check(nodes(), ModelRc::default(), 2);
        assert_eq!(leaf.checked.iter().collect::<Vec<_>>(), [1, 2]);
        assert_eq!(leaf.mixed.iter().collect::<Vec<_>>(), [0]);
        let full = toggle_check(nodes(), leaf.checked, 0);
        assert_eq!(full.checked.iter().collect::<Vec<_>>(), [0, 1, 2, 3]);
        let empty = toggle_check(nodes(), full.checked, 0);
        assert_eq!(empty.checked.row_count(), 0);
    }

    #[test]
    fn deep_tree_projection_is_iterative() {
        let count = 10_000;
        let nodes = model(
            (0..count)
                .map(|index| TreeNode {
                    parent_index: index - 1,
                    ..Default::default()
                })
                .collect(),
        );
        let projection = project(nodes, model((0..count).collect()), count);
        assert_eq!(projection.indices.row_count(), count as usize);
        assert_eq!(
            projection
                .decorations
                .row_data(count as usize - 1)
                .map(|row| row.indent),
            Some(count - 1)
        );
    }
}
