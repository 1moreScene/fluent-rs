use slint::{Model, ModelRc, VecModel};

use crate::FrameEntry;

pub(super) fn append(
    journal: ModelRc<FrameEntry>,
    keep: i32,
    entry: FrameEntry,
    limit: i32,
) -> ModelRc<FrameEntry> {
    let keep = usize::try_from(keep)
        .unwrap_or_default()
        .min(journal.row_count());
    let skip = if limit > 0 {
        (keep + 1).saturating_sub(limit as usize)
    } else {
        0
    };
    let entries = journal
        .iter()
        .take(keep)
        .skip(skip)
        .chain([entry])
        .collect::<Vec<_>>();
    ModelRc::new(VecModel::from(entries))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NavigationTransition;

    fn entry(page: i32) -> FrameEntry {
        FrameEntry {
            page,
            transition: NavigationTransition::FromRight,
        }
    }

    #[test]
    fn navigating_from_history_discards_the_forward_branch() {
        let journal = ModelRc::new(VecModel::from(vec![entry(0), entry(2), entry(1)]));
        let next = append(journal.clone(), 2, entry(3), 0);
        assert_eq!(
            next.iter().collect::<Vec<_>>(),
            [entry(0), entry(2), entry(3)]
        );
        assert_eq!(
            journal.iter().collect::<Vec<_>>(),
            [entry(0), entry(2), entry(1)]
        );
        assert_eq!(
            append(next, 0, entry(1), 0).iter().collect::<Vec<_>>(),
            [entry(1)]
        );
    }

    #[test]
    fn a_bounded_journal_keeps_the_latest_back_entries_and_transition() {
        let journal = ModelRc::new(VecModel::from(vec![entry(0), entry(2), entry(1)]));
        assert_eq!(
            append(journal, 3, entry(3), 2).iter().collect::<Vec<_>>(),
            [entry(1), entry(3)]
        );
    }
}
