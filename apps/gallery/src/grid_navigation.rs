use slint::{Model, ModelExt, ModelRc};

use crate::ItemData;

// WinUI advances one source item along the wrapping direction and one whole
// strip across it. Keep the latter stride when skipping disabled cells.
pub(super) fn step(items: ModelRc<ItemData>, from: i32, stride: i32) -> i32 {
    let count = i32::try_from(items.row_count()).unwrap_or(i32::MAX);
    if stride == 0 {
        return -1;
    }
    let mut index = from;
    while let Some(next) = index
        .checked_add(stride)
        .filter(|next| (0..count).contains(next))
    {
        if items
            .row_data_tracked(next as usize)
            .is_some_and(|item| !item.disabled)
        {
            return next;
        }
        index = next;
    }
    -1
}

#[cfg(test)]
mod tests {
    use super::*;
    use slint::VecModel;

    #[test]
    fn steps_in_source_order_or_in_the_same_lane_and_stops_at_edges() {
        let items = ModelRc::new(VecModel::from(
            [false, true, false, false, true, false, false]
                .map(|disabled| ItemData {
                    disabled,
                    ..Default::default()
                })
                .to_vec(),
        ));
        assert_eq!(step(items.clone(), 0, 1), 2);
        assert_eq!(step(items.clone(), 2, 1), 3);
        assert_eq!(step(items.clone(), 2, 3), 5);
        assert_eq!(step(items.clone(), 5, 3), -1);
        assert_eq!(step(items.clone(), 6, -3), 3);
        assert_eq!(step(items, 0, -1), -1);
    }
}
