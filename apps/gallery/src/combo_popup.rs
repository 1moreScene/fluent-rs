use crate::{ComboPopupGeometry, OverlayArea};

const INITIAL_ABOVE_LIMIT: i32 = 7;
const MAX_POPUP_ROWS: i32 = 15;

pub(super) fn arrange_editable(
    viewport: OverlayArea,
    anchor: OverlayArea,
    requested_width: f32,
    row_height: f32,
    inset: f32,
    count: i32,
) -> ComboPopupGeometry {
    let width = requested_width.min(viewport.width);
    if count <= 0 || viewport.height <= 0. || row_height <= 0. || width <= 0. {
        return ComboPopupGeometry::default();
    }

    let bottom = viewport.y + viewport.height;
    let anchor_top = anchor.y.clamp(viewport.y, bottom);
    let anchor_bottom = (anchor.y + anchor.height).clamp(anchor_top, bottom);
    let above = anchor_top - viewport.y;
    let below = bottom - anchor_bottom;
    let preferred_height = count.min(MAX_POPUP_ROWS) as f32 * row_height + 2. * inset;
    let open_above = below < preferred_height
        && (above >= preferred_height || (below < row_height + 2. * inset && above > below));
    let height = preferred_height.min(if open_above { above } else { below });
    if height <= 2. * inset {
        return ComboPopupGeometry::default();
    }
    ComboPopupGeometry {
        first_row: 0,
        last_row: (((height - 2. * inset) / row_height).ceil() as i32 - 1).clamp(0, count - 1),
        bounds: OverlayArea {
            x: anchor
                .x
                .clamp(viewport.x, viewport.x + viewport.width - width),
            y: if open_above {
                anchor_top - height
            } else {
                anchor_bottom
            },
            width,
            height,
        },
    }
}

pub(super) fn arrange(
    viewport: OverlayArea,
    anchor: OverlayArea,
    requested_width: f32,
    row_height: f32,
    inset: f32,
    count: i32,
    selected: i32,
) -> ComboPopupGeometry {
    let width = requested_width.min(viewport.width);
    if count <= 0 || viewport.height <= 0. || row_height <= 0. || width <= 0. {
        return ComboPopupGeometry::default();
    }
    let left = anchor
        .x
        .clamp(viewport.x, viewport.x + viewport.width - width);

    let center_row = if (0..count).contains(&selected) {
        selected
    } else {
        count / 2
    };
    let bottom = viewport.y + viewport.height;
    let anchor_height = anchor.height.min(viewport.height);
    let center = anchor.y.clamp(viewport.y, bottom - anchor_height) + anchor_height / 2.;
    let center_height = (row_height + 2. * inset).min(viewport.height);
    let mut top_edge = (center - center_height / 2.).clamp(viewport.y, bottom - center_height);
    let mut bottom_edge = top_edge + center_height;
    let mut first = center_row;
    let mut last = center_row;
    let mut visible_count = 1;

    // GetNonPannablePopupLayout first lays out both sides of the aligned row,
    // then admits more rows until it hits the visible bounds or the 15-row cap.
    let upper_limit = (center - viewport.height / 2.).max(viewport.y);
    while first > 0 && visible_count < INITIAL_ABOVE_LIMIT && top_edge - row_height >= upper_limit {
        first -= 1;
        top_edge -= row_height;
        visible_count += 1;
    }
    let under_center = (center + viewport.height / 2.).min(bottom);
    while last < count - 1
        && visible_count < MAX_POPUP_ROWS
        && bottom_edge + row_height < under_center
    {
        last += 1;
        bottom_edge += row_height;
        visible_count += 1;
    }
    while visible_count < MAX_POPUP_ROWS {
        if first > 0 && top_edge - row_height >= viewport.y {
            first -= 1;
            top_edge -= row_height;
        } else if last < count - 1 && bottom_edge + row_height <= bottom {
            last += 1;
            bottom_edge += row_height;
        } else {
            break;
        }
        visible_count += 1;
    }

    ComboPopupGeometry {
        first_row: first,
        last_row: last,
        bounds: OverlayArea {
            x: left,
            y: top_edge,
            width,
            height: bottom_edge - top_edge,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(x: f32, y: f32, width: f32, height: f32) -> OverlayArea {
        OverlayArea {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn editable_dropdown_adjoins_the_editor_and_flips_when_it_fits_above() {
        let viewport = area(0., 0., 800., 600.);
        let below = arrange_editable(
            viewport.clone(),
            area(40., 200., 200., 32.),
            200.,
            40.,
            5.,
            5,
        );
        assert_eq!((below.bounds.y, below.bounds.height), (232., 210.));
        assert_eq!((below.first_row, below.last_row), (0, 4));
        let above = arrange_editable(
            viewport.clone(),
            area(40., 500., 200., 32.),
            200.,
            40.,
            5.,
            5,
        );
        assert_eq!((above.bounds.y, above.bounds.height), (290., 210.));
        let clipped =
            arrange_editable(viewport, area(40., 200., 200., 32.), 200., 40., 5., 100_000);
        assert_eq!((clipped.bounds.y, clipped.bounds.height), (232., 368.));
        assert_eq!((clipped.first_row, clipped.last_row), (0, 8));
    }

    #[test]
    fn centers_the_selection_or_middle_row_over_the_faceplate() {
        let viewport = area(0., 0., 800., 600.);
        let anchor = area(40., 200., 240., 32.);
        for (selection, row) in [(-1, 2), (0, 0), (3, 3), (4, 4)] {
            let layout = arrange(
                viewport.clone(),
                anchor.clone(),
                240.,
                40.,
                5.,
                5,
                selection,
            );
            assert_eq!((layout.first_row, layout.last_row), (0, 4));
            assert_eq!(layout.bounds.x, 40.);
            assert_eq!(layout.bounds.height, 210.);
            assert_eq!(layout.bounds.y + 5. + (row as f32 + 0.5) * 40., 216.);
        }
    }

    #[test]
    fn fits_edges_and_keeps_large_models_bounded() {
        let viewport = area(12., 22., 240., 360.);
        let anchor = area(200., 322., 72., 32.);
        let layout = arrange(viewport, anchor, 180., 40., 5., 100_000, 99_999);
        assert_eq!(layout.bounds.x + layout.bounds.width, 252.);
        assert!(layout.bounds.y >= 22. && layout.bounds.y + layout.bounds.height <= 382.);
        let selected_center = layout.bounds.y + 5. + (99_999 - layout.first_row) as f32 * 40. + 20.;
        assert_eq!(selected_center, 338.);
        assert!(99_999 - layout.first_row < 15);
    }
}
